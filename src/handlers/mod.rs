mod actions;
mod responses;
mod state;

use super::*;
use actions::*;
use responses::*;
use state::userstate::*;

use std::{fs::*, sync::Arc, time::Duration};

use async_std::{
    sync::{channel, Receiver, Sender},
    task,
};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use snips_nlu_lib::SnipsNluEngine;

///Long wait time, Used in runing system
const LONGWAIT: u64 = 30;
#[allow(dead_code)]
///Short wait time, Used usually for testing
const SHORTWAIT: u64 = 10;

///Currently set waitime
const WAITTIME: u64 = LONGWAIT;

///NLUENGINE: Snips NLU is used to pick actions when they don't match directly
static NLUENGINE: Lazy<Option<SnipsNluEngine>> = Lazy::new(|| {
    util::logger::show_status("\nLoading the nlu engine...");
    SnipsNluEngine::from_path("data/rootengine/").ok()
});

///HTTP client for..... HTTP things
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    util::logger::show_status("\nLoading Api Client");
    reqwest::Client::new()
});

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

//Implementation of `BotMessage` for `Box<dyn Botmessage>`
//Mainly to use it where `impl BotMessage` is required but `Box<dyn Botmessage>`  is available
#[async_trait]
impl BotMessage for Box<dyn BotMessage> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }
    fn get_id(&self) -> String {
        (**self).get_id()
    }
    async fn send_message(&self, message: MsgCount) {
        (**self).send_message(message).await;
    }
    fn start_conversation(&self) -> bool {
        (**self).start_conversation()
    }
    fn dyn_clone(&self) -> Box<dyn BotMessage> {
        (**self).dyn_clone()
    }
}

///Returns a sender and receiver channel of `Box<dyn BotMessage>`
pub async fn init_sender() -> (
    Sender<(Arc<Box<dyn BotMessage>>, String)>,
    Receiver<(Arc<Box<dyn BotMessage>>, String)>,
) {
    let (sender, receiver) = channel(10);
    (sender, receiver)
}

///Distributes incoming requests to separate threads
pub async fn receiver(r: Receiver<(Arc<Box<dyn BotMessage>>, String)>) {
    let source = "DISTRIBUTOR";
    let info = util::logger::info(source);
    while let Ok((message, text)) = r.recv().await {
        //Spawn a new task to handle the message
        let _ = task::spawn(async move { handler(message.dyn_clone(), text).await });
        info("Handler Thread Spawned");
    }
}

///First place to handle messages after distribution
async fn handler(bot_message: Box<dyn BotMessage>, processed_text: String) {
    let source = "HANDLER";
    let info = util::logger::info(source);
    info(&format!("Processed text is {}", processed_text));

    //If record from user exists (A Some(record)), some conversation is ongoing
    //So will be replied regardless of groups or mentions and stuff ('will_respond' is ignored)
    if let Some(stored_record) = retrieve_state(&bot_message.get_id()).await {
        let record = stored_record.clone();

        //"cancel last" will shut off the conversation
        if processed_text == "cancel last" {
            purge_state(bot_message).await;
        } else {
            use UserState::{Animation, Identify, Notes, Search, Unknown};
            info(&format!("Saved state is {}", record.state));
            match record.state {
                Search => search::resume(bot_message, processed_text.clone()).await,
                Identify => identify::resume(bot_message, processed_text.clone()).await,
                Animation => animation::resume(bot_message, processed_text.clone()).await,
                Notes(data) => notes::resume(bot_message, processed_text.clone(), data).await,
                Unknown => extra::unknown_state_notice(bot_message).await,
            }
        }
    }
    //---if record from user doesn't exist, but is either IN A PRIVATE CHAT or MENTIONED IN A GROUP CHAT
    //---will start processing new info
    else if bot_message.start_conversation() {
        //---cancel last does nothing as there's nothing to cancel
        if processed_text == "cancel last" {
            bot_message
                .send_message(responses::load("cancel-nothing").into())
                .await;
        }
        //---hand over to the natural understanding system for advanced matching
        else {
            natural_understanding(bot_message, processed_text).await;
        }
    }
}

///Uses natural understanding to determine intent if no state is found
async fn natural_understanding(bot_message: Box<dyn BotMessage>, processed_text: String) {
    let source = "NATURAL_ACTION_PICKER";

    let info = util::logger::info(source);
    let warning = util::logger::warning(source);
    let error = util::logger::error(source);
    //---Stuff required to run the NLU engine to get an intent
    if let Some(engine) = &*NLUENGINE {
        let intents_alternatives = 1;
        let slots_alternatives = 1;
        let result = engine
            .parse_with_alternatives(
                &processed_text,
                None,
                None,
                intents_alternatives,
                slots_alternatives,
            )
            .unwrap();

        if let Some(intent) = result.intent.intent_name.clone() {
            info(&format!(
                "{} with confidence {}",
                intent, result.intent.confidence_score
            ));
            //Tries to match against existing intents like chat, search etc
            //Only valid if confidence greater than 0.5
            if result.intent.confidence_score > 0.5 {
                //---Convert result to json string
                if let Ok(json) = serde_json::to_string(&result) {
                    info("ACTION_PICKER: intent json is valid");
                    let intent_str: &str = &intent;
                    info(&format!("intent is {}", intent_str));
                    match intent_str {
                        "chat" => chat::start(bot_message).await,
                        "search" => search::start(bot_message).await,
                        "identify" => identify::start(bot_message).await,
                        "animation" => animation::start(bot_message).await,
                        "info" => info::start(bot_message, json).await,
                        "notes" => notes::start(bot_message).await,
                        "corona" => corona::start(bot_message).await,
                        "unknown" => extra::start(bot_message).await,
                        _ => {
                            //Forward to chat for more intents
                            info("forwarding to chat");
                            chat::resume(bot_message, processed_text, &intent).await;
                        }
                    }
                }
                //If failed to parse the intent result as json
                else {
                    error("couldn't convert intent data to JSON");
                    let _ = util::logger::log_message(&processed_text)
                        .await
                        .map_err(|err| {
                            error(&format!("{}", err));
                        });
                    extra::unsupported_notice(bot_message).await
                }
            }
            //Unsure intent if cannot match to any intent confidently
            else {
                warning("couldn't match an intent confidently");
                let _ = util::logger::log_message(&processed_text)
                    .await
                    .map_err(|err| {
                        error(&format!("{}", err));
                    });
                extra::unsupported_notice(bot_message).await
            }
        }
        //Unknown intent if can't match intent at all
        else {
            warning("unknown intent");
            let _ = util::logger::log_message(&processed_text)
                .await
                .map_err(|err| {
                    error(&format!("{}", err));
                });
            extra::unsupported_notice(bot_message).await
        };
    } else {
        error("NLU engine load failed");
        extra::unsupported_notice(bot_message).await
    }
}
