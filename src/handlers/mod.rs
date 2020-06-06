//----------------------mod.rs name is mandatory as it is the main file for the handlers module
//----------------------'handlers/mod.rs' means handlers module
mod animation;
mod chat;
mod corona;
mod extras;
mod identify;
mod info;
mod notes;
mod responses;
mod search;

use crate::functions::*;

const LONGWAIT: u64 = 30;

#[allow(dead_code)]
const SHORTWAIT: u64 = 10;
const WAITTIME: u64 = LONGWAIT;

use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs::*;
use std::mem::drop;
use std::time::{Duration, Instant};

//
extern crate snips_nlu_lib;
use snips_nlu_lib::SnipsNluEngine;
//

lazy_static! {
    //---Record is a map holding all users state record info
    pub static ref RECORDS: tokio::sync::Mutex<HashMap<String, UserStateRecord>> =
        tokio::sync::Mutex::new(HashMap::new()) ;
    //---Snips NLU is used to pick actions when they don't match directly
    pub static ref ROOTENGINE: SnipsNluEngine = {
        println!("\nLoading the action nlu engine...");
        SnipsNluEngine::from_path("nlu/rootengine/").unwrap()
    };
    pub static ref RESPONSES: Option<serde_json::Value> = {
        println!("\nLoading JSON responses");
        match read_to_string("data/responses.json"){
            Ok(json) => serde_json::from_str(&json).ok(),
            Err(_) => None,
        }
    };
    pub static ref CLIENT: reqwest::Client = {
        println!("\nLoading Api Client");
        reqwest::Client::new()
    };
}

pub fn initialize() {
    lazy_static::initialize(&RECORDS);
    lazy_static::initialize(&ROOTENGINE);
    lazy_static::initialize(&RESPONSES);
}

#[derive(PartialEq, Eq)]
enum UserState {
    // Chat,
    Search,
    Identify,
    Animation,
    Notes,
    Unknown,
}

impl fmt::Display for UserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserState::Search => write!(f, "Search"),
            UserState::Identify => write!(f, "Identify"),
            UserState::Animation => write!(f, "Animation"),
            UserState::Notes => write!(f, "Notes"),
            UserState::Unknown => write!(f, "Unknown"),
        }
    }
}

///ENUM, Represents Message count
///- SingleMsg - Contains a Msg Enum
///- MultiMsg - Contains a Vector of Msg Enums
///- NoMsg - Represnts an empty response
pub enum MsgCount {
    SingleMsg(Msg),
    MultiMsg(Vec<Msg>),
    NoMsg,
}

///ENUM, Represents Message type
///- Text - Contains String text
///- File - Contains String url for file
pub enum Msg {
    Text(String),
    File(String),
}

///A user state record holds an individual user's state.
///Last holds when it was last updated.
///History is just a vector of strings to hold misc info (ex: messages in chat state).
pub struct UserStateRecord {
    state: UserState,
    last: Instant,
}

///Used to generalize Message Updates for various platforms
#[async_trait]
pub trait BotMessage: Send + Sync {
    // this is used to make cloneable box< send + sync> version of itself
    fn dynamic_clone(&self) -> Box<dyn BotMessage>;
    fn get_name(&self) -> String;
    fn get_id(&self) -> String;
    async fn send_message(&self, message: MsgCount);
    fn start_conversation(&self) -> bool;
}

///Implment clone for this trait
impl Clone for Box<dyn BotMessage> {
    fn clone(&self) -> Self {
        self.dynamic_clone()
    }
}

///Distributes incoming requests to separate threads
pub fn distributor(bot_message: impl BotMessage + 'static, processesed_text: String) {
    tokio::spawn(async move { handler(bot_message, processesed_text).await });
    println!("DISTRIBUTOR: Handler Thread Spawned");
}

///First place to handle messages after distribution
async fn handler(bot_message: impl BotMessage + 'static, processesed_text: String) {
    let m = bot_message.dynamic_clone();
    println!("processed text is '{}'", processesed_text);
    let map = RECORDS.lock().await;
    let entry_option = map.get({
        let id = (*m).get_id();
        &format!("{}", id)
    });
    //---If record from user exists (A Some(record)), some conversation is ongoing
    //---So will be replied regardless of groups or mentions and stuff ('will_respond' is ignored)
    if let Some(record) = entry_option {
        //---"cancel last will shut off the conversation"
        if processesed_text == "cancel last" {
            drop(map);
            cancel_history(m).await;
        }
        //---"if state is chat"
        //------Chat will not be a state any more.
        //------Rather any unknown message will be handled by chat in default
        /*
        else if record.state == "chat".to_string() {
            drop(map);
            println!("continuing chat");
            chat::continue_chat(message.clone(), processesed_text.clone()).await
        }
        */
        //---"if state is search"
        else if record.state == UserState::Search {
            drop(map);
            println!("continuing search");
            search::continue_search(bot_message, processesed_text.clone()).await;
        }
        //---"if state is identify"
        else if record.state == UserState::Identify {
            drop(map);
            println!("continuing identify");
            identify::continue_identify(bot_message, processesed_text.clone()).await;
        }
        //---"if state is animatios"
        else if record.state == UserState::Animation {
            drop(map);
            println!("continuing animation");
            animation::continue_gif(bot_message, processesed_text.clone()).await;
        }
        //---"if state is animatios"
        else if record.state == UserState::Notes {
            drop(map);
            println!("continuing notes");
            notes::continue_notes(bot_message, processesed_text.clone()).await;
        }
        //---"if state is unknown"
        else {
            println!("some unknown state");
            drop(map);
            responses::unknown_state_notice(m).await;
        }
    }
    //---if record from user doesn't exist, but is either IN A PRIVATE CHAT or MENTIONED IN A GROUP CHAT
    //---will start processing new info
    else if (*m).start_conversation() {
        drop(map);
        //---cancel last does nothing as there's nothing to cancel
        if processesed_text == "cancel last" {
            (*m).send_message(MsgCount::SingleMsg(Msg::Text(
                match responses::load_response("cancel-nothing") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                },
            )))
            .await;
        }
        //---hand over to the natural understanding system for advanced matching
        else {
            natural_understanding(bot_message, processesed_text).await;
        }
    }
}

///Uses natural understanding to determine intent if no state is found
async fn natural_understanding(bot_message: impl BotMessage + 'static, processed_text: String) {
    let m = bot_message.dynamic_clone();
    let intents_alternatives = 1;
    let slots_alternatives = 1;

    let result = ROOTENGINE
        .parse_with_alternatives(
            &processed_text,
            None,
            None,
            intents_alternatives,
            slots_alternatives,
        )
        .unwrap();
    if let Some(intent) = result.intent.intent_name.clone() {
        println!(
            "{} with confidence {}",
            intent, result.intent.confidence_score
        );
        //---tries to match against existing intents like chat, search etc
        //---only valid if confidence greater than 0.5
        if result.intent.confidence_score > 0.5 {
            //---Convert result to json string
            if let Ok(json) = serde_json::to_string(&result) {
                println!("ACTION_PICKER: intent json is valid" /*, &json*/);
                match &*intent {
                    "chat" => {
                        println!("ACTION_PICKER: starting chat");
                        chat::start_chat(m).await
                    }
                    "search" => {
                        println!("ACTION_PICKER: starting search");
                        search::start_search(m).await
                    }
                    "identify" => {
                        println!("ACTION_PICKER: starting identify");
                        identify::start_identify(m).await
                    }
                    "animation" => {
                        println!("ACTION_PICKER: starting animation");
                        animation::start_gif(m).await
                    }
                    "info" => {
                        println!("ACTION_PICKER: starting info");
                        info::start_info(m, json).await
                    }
                    "notes" => {
                        println!("ACTION_PICKER: starting notes");
                        notes::start_notes(m).await
                    }
                    "corona" => {
                        println!("ACTION_PICKER: starting corona");
                        corona::start_corona(m).await
                    }
                    "unknown" => {
                        println!("ACTION_PICKER: starting unknown state test");
                        extras::start_unknown(m).await
                    }
                    _ => {
                        //forward to chat for more intents
                        println!("ACTION_PICKER: forwarding to chat");
                        chat::continue_chat(m, processed_text, &intent).await;
                    }
                }
            } else {
                println!("ACTION_PICKER: couldn't convert intent to json");
                general::log_message(processed_text);
                responses::unsupported_notice(m).await
            }
        }
        //---unknown intent if cannot match to any intent confidently
        else {
            println!("unsure intent");
            general::log_message(processed_text.clone());
            responses::unsupported_notice(m).await
        }
    }
    //---unknown intent if can't match intent at all
    else {
        println!("unknown intent");
        general::log_message(processed_text.clone());
        responses::unsupported_notice(m).await
    };
}

///Removes current history with a cancellation message.
///Doesn't care about state.
///Used with the cancel last command.
async fn cancel_history(m: Box<dyn BotMessage>) {
    let mut map = RECORDS.lock().await;
    map.remove({
        let id = (*m).get_id();
        &format!("{}", id)
    });
    drop(map);
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("cancel-state") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )))
    .await;
}

///Removes history after 30 seconds if it's not updated with a new time.
///AND the history state matches the provided state.
///Message is provided to user.
fn wipe_history(m: Box<dyn BotMessage>, state: UserState) {
    tokio::spawn(async move {
        tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&format!("{}", (*m).get_id())) {
            if r.state == state {
                if r.last.elapsed() > Duration::from_secs(WAITTIME) {
                    map.remove(&format!("{}", (*m).get_id()));
                    drop(map);
                    println!("WIPE_HISTORY: deleted state record for {}", state);
                    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
                        match responses::load_response("delay-notice") {
                            Some(response) => response,
                            _ => responses::response_unavailable(),
                        },
                    )))
                    .await;
                } else {
                    drop(map);
                    util::log_info("WIPE_HISTORY: aborted record delete due to recency");
                }
            } else {
                util::log_info(&format!(
                    "WIPE_HISTORY: aborted record delete for {} because current state is {}",
                    state, r.state
                ));
                drop(map);
            }
        } else {
            drop(map);
            util::log_info(&format!(
                "WIPE_HISTORY: aborted record delete for {}, there is no recorded state for {}",
                state,
                (*m).get_id()
            ))
        }
    });
}
/**
Immediately purges history IF provided state matches history state
used to remove history after state action is completed
no notice provided
*/
fn immediate_purge_history(m: Box<dyn BotMessage>, state: UserState) {
    tokio::spawn(async move {
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&format!("{}", (*m).get_id())) {
            if r.state == state {
                map.remove(&format!("{}", (*m).get_id()));
                drop(map);
                println!("deleted state record for {}", state);
            }
        }
    });
}
