use crate::handlers::animation;
use crate::handlers::chat;
use crate::handlers::corona;
use crate::handlers::identify;
use crate::handlers::info;
use crate::handlers::responses;
use crate::handlers::search;
use crate::handlers::util;
const LONGWAIT: u64 = 30;

#[allow(dead_code)]
const SHORTWAIT: u64 = 10;
const WAITTIME: u64 = LONGWAIT;

// use async_trait::async_trait;
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
    pub static ref ACTIONENGINE: SnipsNluEngine = {
        println!("\nLoading the action nlu engine...");
        SnipsNluEngine::from_path("actionengine/").unwrap()
    };
    pub static ref RESPONSES: Option<serde_json::Value> = {
        println!("\nLoading JSON responses");
        match read_to_string("responses.json"){
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
    lazy_static::initialize(&ACTIONENGINE);
    lazy_static::initialize(&chat::CHATENGINE);
    lazy_static::initialize(&RESPONSES);
}

#[derive(PartialEq, Eq)]
pub enum UserState {
    Search,
    Identify,
    Animation,
    Unknown,
}
impl fmt::Display for UserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserState::Search => write!(f, "Search"),
            UserState::Identify => write!(f, "Identify"),
            UserState::Animation => write!(f, "Animation"),
            UserState::Unknown => write!(f, "Unknown"),
        }
    }
}

pub enum MsgCount {
    SingleMsg(Msg),
    MultiMsg(Vec<Msg>),
    NoMsg,
}

pub enum Msg {
    Text(String),
    File(String),
}

//---A user state record holds an individual user's state
//---Last holds when it was last updated
//---History is just a vector of strings to hold misc info (ex: messages in chat state)
pub struct UserStateRecord {
    pub username: String,
    pub state: UserState,
    pub last: Instant,
}
//---Will be used in the future to generalize bot for other platforms in future versions

pub trait BotMessage {
    // this is used to make cloneable box< send + sync> version of itself
    fn clone_bot_message(&self) -> Box<dyn BotMessage + Send + Sync>;
    fn get_name(&self) -> String;
    fn get_id(&self) -> String;
    fn send_message(&self, message: MsgCount);
    fn start_conversation(&self) -> bool;
}

// Implment clone for this trait
impl Clone for Box<dyn BotMessage + Send + Sync> {
    fn clone(&self) -> Self {
        self.clone_bot_message()
    }
}

///Distributes incoming requests to separate threads
pub fn distributor(m: Box<dyn BotMessage + Send + Sync>, processesed_text: String) {
    tokio::spawn(async move { handler(m, processesed_text).await });
}

///First place to handle messages after distribution
pub async fn handler(m: Box<dyn BotMessage + Send + Sync>, processesed_text: String) {
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
            search::continue_search(m, processesed_text.clone()).await;
        }
        //---"if state is identify"
        else if record.state == UserState::Identify {
            drop(map);
            println!("continuing identify");
            identify::continue_identify(m, processesed_text.clone()).await;
        }
        //---"if state is animatios"
        else if record.state == UserState::Animation {
            drop(map);
            println!("continuing animation");
            animation::continue_gif(m, processesed_text.clone()).await;
        }
        //---"if state is unknown"
        else {
            println!("some unknown state");
            drop(map);
            responses::unknown_state_notice(m);
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
            )));
        }
        //---hand over to the natural understanding system for advanced matching
        else {
            natural_understanding(m, processesed_text).await;
        }
    }
}

///Uses natural understanding to determine intent if no state is found
pub async fn natural_understanding(m: Box<dyn BotMessage + Send + Sync>, processed_text: String) {
    let intents_alternatives = 1;
    let slots_alternatives = 1;

    let result = ACTIONENGINE
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
                    "corona" => {
                        println!("ACTION_PICKER: starting corona");
                        corona::start_corona(m).await
                    }
                    "unknown" => {
                        println!("ACTION_PICKER: starting unknown state test");
                        util::start_unknown(m).await
                    }
                    _ => {
                        //---This one is only for unimplemented but known intents
                        //---Don't put stuff related to unknown intents here
                        println!("ACTION_PICKER: unimplemented intent");
                        util::log_message(processed_text);
                        responses::unsupported_notice(m)
                    }
                }
            } else {
                println!("ACTION_PICKER: couldn't convert intent to json");
                util::log_message(processed_text);
                responses::unsupported_notice(m)
            }
        }
        //---unknown intent if cannot match to any intent confidently
        else {
            chat::continue_chat(m, processed_text).await
        }
    }
    //---unknown intent if can't match intent at all
    else {
        chat::continue_chat(m, processed_text).await
    };
}

//---removes current history with a cancellation message
//---doesn't care about state
//---used with the cancel last command
pub async fn cancel_history(m: Box<dyn BotMessage + Send + Sync>) {
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
    )));
}

//---removes history after 30 seconds if it's not updated with a new time
//---AND the history state matches the provided state
//---message is provided to user
pub fn wipe_history(m: Box<dyn BotMessage + Send + Sync>, state: UserState) {
    tokio::spawn(async move {
        tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&format!("{}", (*m).get_id())) {
            if r.state == state {
                if r.last.elapsed() > Duration::from_secs(WAITTIME) {
                    map.remove(&format!("{}", (*m).get_id()));
                    drop(map);
                    println!("deleted state record for {}", state);
                    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
                        match responses::load_response("delay-notice") {
                            Some(response) => response,
                            _ => responses::response_unavailable(),
                        },
                    )));
                } else {
                    println!("WIPE_HISTORY: aborted record delete due to recency");
                    drop(map);
                }
            } else {
                println!(
                    "WIPE_HISTORY: aborted record delete for {} because current state is {}",
                    state, r.state
                );
                drop(map);
            }
        }
    });
}

//---immediately purges history IF provided state matches history state
//---used to remove history after state action is completed
//---no notice provided
pub fn immediate_purge_history(m: Box<dyn BotMessage + Send + Sync>, state: UserState) {
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
