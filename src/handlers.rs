mod actions;
mod intent;
mod responses;
mod state;

use super::*;
use actions::*;
use intent::{Intent, NLUENGINE};
use responses::*;
use state::*;

use std::{fs::*, sync::Arc, time::Duration};

use async_std::task;
use async_trait::async_trait;
use flume::{Receiver, Sender};
use once_cell::sync::Lazy;

///Long wait time, Used in runing system
const LONGWAIT: u64 = 30;
#[allow(dead_code)]
///Short wait time, Used usually for testing
const SHORTWAIT: u64 = 10;

///Currently set waitime
const WAITTIME: u64 = LONGWAIT;

///HTTP client for..... HTTP things
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    util::logger::show_status("\nLoading Api Client");
    reqwest::Client::new()
});

pub async fn reminder_service() -> anyhow::Result<!> {
    actions::reminder::service().await
}

///Initializes a variety of things
///- State management system
///- NLU engine
///- Responses JSON
pub async fn initialize() {
    initialize_state();
    Lazy::force(&NLUENGINE);
    Lazy::force(&CLIENT);
    responses::initialize().await;
}

///ENUM, Represents Message count
///- `SingleMsg` - Contains a Msg Enum
///- `MultiMsg` - Contains a Vector of Msg Enums
///- `NoMsg` - Represnts an empty response
pub enum MsgCount {
    SingleMsg(Msg),
    MultiMsg(Vec<Msg>),
    // NoMsg,
}

//When passed an String
//Uses the value as a MsgCount::SingleMsg(Msg::Text)
impl From<String> for MsgCount {
    fn from(s: String) -> Self {
        MsgCount::SingleMsg(Msg::Text(s))
    }
}

//When passed an Option<String>
//Uses the Some value as a MsgCount::SingleMsg(Msg::Text)
//Uses the 'response unavailable...' message in case of None as MsgCount::SingleMsg(Msg::Text)
impl From<Option<String>> for MsgCount {
    fn from(s: Option<String>) -> Self {
        match s {
            Some(msg) => MsgCount::SingleMsg(Msg::Text(msg)),
            None => MsgCount::SingleMsg(Msg::Text(
                "ForgiVE uS... We SEEM t0 B3... hAVInG i55UEs".to_string(),
            )),
        }
    }
}

//When passed an Vec<String>
//Turns into MsgCount::MultiMsg(Vec<Msg::Text()>)
impl From<Vec<String>> for MsgCount {
    fn from(s: Vec<String>) -> Self {
        MsgCount::MultiMsg(s.into_iter().map(|s| s.into()).collect())
    }
}

//When passed an Vec<Msg>
//Turns into MsgCount::MultiMsg(Vec<Msg>)
impl From<Vec<Msg>> for MsgCount {
    fn from(s: Vec<Msg>) -> Self {
        MsgCount::MultiMsg(s)
    }
}

//ENUM, Represents Message type
//- Text - Contains String text
//- File - Contains String url for file
pub enum Msg {
    Text(String),
    File(String),
}

//When passed an Option<String>
//Uses the Some value as a Msg::Text
//Uses the 'response unavailable...' message in case of None as Msg::Text
impl From<Option<String>> for Msg {
    fn from(s: Option<String>) -> Self {
        match s {
            Some(msg) => Msg::Text(msg),
            None => Msg::Text("ForgiVE uS... We SEEM t0 B3... hAVInG i55UEs".to_string()),
        }
    }
}

//When passed an String
//Uses the value as a Msg::Text
impl From<String> for Msg {
    fn from(s: String) -> Self {
        Msg::Text(s)
    }
}

///## Used to generalize Message Updates for various platforms
///All clients sending message updates must implement this
///## functions
///- `fn get_name() -> String` Return user readable name
///- `fn get_id() -> String` Return unique id for user
///- `async fn send_message(message: MsgCount)` Sends message to user
///- `fn start_conversation() -> bool` Returns bool indicating whether to start a new conversation
///- `fn dyn_clone() -> Box<dyn BotMessage>` Returns a `Box<dyn >` clone of self
#[async_trait]
pub trait BotMessage: Send + Sync {
    ///Returns the user's user readable name. Not the same as id.
    fn get_name(&self) -> &str;

    ///Returns the user's unique id. This is needed to uniquely identify users.
    fn get_id(&self) -> String;

    ///Used to send messages to the sender (user) of this message.
    async fn send_message(&self, message: MsgCount);

    ///Used to check whether a new conversation should be started.  
    ///Sometimes if the user is in a state, Bot will always respond.  
    ///However if not in a state, bot needs to know when it should or should not respond.  
    ///Ex. Won't respond if message is in a group and bot wasn't mentioned.
    fn start_conversation(&self) -> bool;

    ///Returns a `Box<dyn BotMessage>` clone of self
    fn dyn_clone(&self) -> Box<dyn BotMessage>;
}

///Returns a sender and receiver channel of `Box<dyn BotMessage>`
pub async fn init_sender() -> (
    Sender<(Arc<Box<dyn BotMessage>>, String)>,
    Receiver<(Arc<Box<dyn BotMessage>>, String)>,
) {
    let (sender, receiver) = flume::bounded::<(Arc<Box<dyn BotMessage>>, String)>(10);
    (sender, receiver)
}

///Distributes incoming requests to separate threads
pub async fn receiver(r: Receiver<(Arc<Box<dyn BotMessage>>, String)>) -> anyhow::Result<!> {
    let source = "DISTRIBUTOR";
    let info = util::logger::info(source);
    while let Ok((message, text)) = r.recv_async().await {
        //Spawn a new task to handle the message
        let _ = task::spawn(async move { handler(message.dyn_clone(), text).await });
        info("Handler Thread Spawned");
    }
    Err(anyhow::anyhow!("Message receiver failed"))
}

///First place to handle messages after distribution
async fn handler(bot_message: Box<dyn BotMessage>, processed_text: String) {
    let source = "HANDLER";
    let info = util::logger::info(source);
    info(format!("Processed text is {}", processed_text).as_str());

    //If record from user exists (A Some(record)), some conversation is ongoing
    //So will be replied regardless of groups or mentions and stuff ('will_respond' is ignored)
    let record = retrieve_state(&bot_message.get_id()).await;

    //"cancel last" will shut off the conversation
    if "cancel last" == processed_text.as_str() && *record.state() != UserState::Initial {
        purge_state(bot_message).await;
    } else {
        use UserState::{Animation, Identify, Initial, Notes, Search, Unknown};
        info(format!("Saved state is {}", record.state()).as_str());
        match record.state() {
            Initial => initiate::start_interaction(bot_message, processed_text).await,
            Search => search::resume(bot_message, processed_text).await,
            Identify => identify::resume(bot_message, processed_text).await,
            Animation => animation::resume(bot_message, processed_text).await,
            Notes(data) => notes::resume(bot_message, processed_text, &data).await,
            Unknown => extra::unknown_state_notice(bot_message).await,
        }
    }
}
