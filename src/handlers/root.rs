use crate::handlers::chat;
use crate::handlers::identify;
use crate::handlers::info;
use crate::handlers::responses;
use crate::handlers::search;
use crate::handlers::util;
const LONGWAIT: u64 = 30;

#[allow(dead_code)]
const SHORTWAIT: u64 = 10;
const WAITTIME: u64 = LONGWAIT;

use std::collections::HashMap;
use std::env;

use std::fmt;

use std::fs::*;

use std::mem::drop;
use std::time::{Duration, Instant};
use telegram_bot::*;

use serde_json;

//
extern crate snips_nlu_lib;
use snips_nlu_lib::SnipsNluEngine;
//

lazy_static! {
    //---Global API access
    pub static ref API: Api = {
        let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
        let api = Api::new(token);
        api
    };
    //---Record is a map holding all users state record info
    pub static ref RECORDS: tokio::sync::Mutex<HashMap<UserId, UserStateRecord>> =
        tokio::sync::Mutex::new(HashMap::new()) ;
    //---Snips NLU is used to pick actions when they don't match directly
    pub static ref ENGINE: SnipsNluEngine = {
        println!("\nLoading the nlu engine...");
        SnipsNluEngine::from_path("actionengine/").unwrap()
    };
    pub static ref RESPONSES: Option<serde_json::Value> = {
        println!("\nLoading JSON responses");
        match read_to_string("responses.json"){
            Ok(json) => serde_json::from_str(&json).ok(),
            Err(_) => None,
        }
    };
}

#[derive(PartialEq, Eq)]
pub enum UserState {
    Search,
    Identify,
}
impl fmt::Display for UserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserState::Search => write!(f, "Search"),
            UserState::Identify => write!(f, "Identify"),
        }
    }
}

pub enum Msg {
    Text(String),
    TextList(Vec<String>),
    NoMsg,
}

//---A user state record holds an individual user's state
//---Last holds when it was last updated
//---History is just a vector of strings to hold misc info (ex: messages in chat state)
pub struct UserStateRecord {
    pub username: String,
    pub state: UserState,
    pub last: Instant,
    pub chat: ChatId,
}

//----------First place to handler messages after initial filtering
pub async fn handler(
    message: &Message,
    processesed_text: String,
    will_respond: bool,
) -> Result<(), Error> {
    println!("processed text is '{}'", processesed_text);
    let map = RECORDS.lock().await;
    let entry_option = map.get(&message.from.id);
    //---If record from user exists (A Some(record)), some conversation is ongoing
    //---So will be replied regardless of groups or mentions and stuff ('will_respond' is ignored)
    let received_message = if let Some(record) = entry_option {
        //---"cancel last will shut off the conversation"
        if processesed_text == "cancel last" {
            drop(map);
            cancel_history(message.clone()).await
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
            search::continue_search(message.clone(), processesed_text.clone()).await
        }
        //---"if state is identify"
        else if record.state == UserState::Identify {
            drop(map);
            println!("continuing identify");
            identify::continue_identify(message.clone(), processesed_text.clone()).await
        }
        //---"if state is unknown"
        else {
            println!("some unknown state");
            drop(map);
            responses::unknown_state_notice()
        }
    }
    //---if record from user doesn't exist, but is either IN A PRIVATE CHAT or MENTIONED IN A GROUP CHAT
    //---will start processing new info
    else if will_respond {
        drop(map);
        //---cancel last does nothing as there's nothing to cancel
        if processesed_text == "cancel last" {
            Msg::Text("nothing to cancel".to_string())
        }
        //---hand over to the natural understanding system for advanced matching
        else {
            natural_understanding(message.clone(), processesed_text).await
        }
    } else {
        Msg::NoMsg
    };
    match received_message {
        Msg::Text(text) => {
            API.spawn(message.chat.text(text));
        }
        Msg::TextList(text_list) => {
            for text in text_list {
                //---Need send here because spawn would send messages out of order
                let _ = API.send(message.chat.text(text)).await;
            }
        }
        _ => {}
    }

    Ok(())
}

pub async fn natural_understanding(message: Message, processed_text: String) -> Msg {
    let intents_alternatives = 1;
    let slots_alternatives = 1;

    let result = ENGINE
        .parse_with_alternatives(
            &processed_text,
            None,
            None,
            intents_alternatives,
            slots_alternatives,
        )
        .unwrap();
    let response = if let Some(intent) = result.intent.intent_name.clone() {
        println!(
            "{} with confidence {}",
            intent, result.intent.confidence_score
        );
        //---tries to match against existing intents like chat, search etc
        //---only valid if confidence greater than 0.5
        if result.intent.confidence_score > 0.5 {
            //---Convert result to json string
            if let Ok(json) = serde_json::to_string(&result) {
                println!("ACTION_PICKER: intent json is {}", &json);
                match &*intent {
                    "chat" => {
                        println!("ACTION_PICKER: starting chat");
                        chat::start_chat(message.clone()).await
                    }
                    "search" => {
                        println!("ACTION_PICKER: starting search");
                        search::start_search(message.clone()).await
                    }
                    "identify" => {
                        println!("ACTION_PICKER: starting identify");
                        identify::start_identify(message.clone()).await
                    }
                    "info" => {
                        println!("ACTION_PICKER: starting info");
                        info::start_info(json)
                    }
                    _ => {
                        //---This one is only for unimplemented but known intents
                        //---Don't put stuff related to unknown intents here
                        println!("ACTION_PICKER: unimplemented intent");
                        util::log_message(processed_text);
                        responses::unsupported_notice()
                    }
                }
            } else {
                println!("ACTION_PICKER: couldn't convert intent to json");
                util::log_message(processed_text);
                responses::unsupported_notice()
            }
        }
        //---unknown intent if cannot match to any intent confidently
        else {
            chat::continue_chat(processed_text).await
        }
    }
    //---unknown intent if can't match intent at all
    else {
        chat::continue_chat(processed_text).await
    };
    response
}

//---removes current history with a cancellation message
//---doesn't care about state
//---used with the cancel last command
pub async fn cancel_history(message: Message) -> Msg {
    let mut map = RECORDS.lock().await;
    map.remove(&message.from.id);
    drop(map);
    Msg::Text(format!("understood. we will not prolong this conversation"))
}

//---removes history after 30 seconds if it's not updated with a new time
//---AND the history state matches the provided state
//---message is provided to user
pub fn wipe_history(message: Message, state: UserState) {
    tokio::spawn(async move {
        tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&message.from.id) {
            if r.state == state {
                if r.last.elapsed() > Duration::from_secs(WAITTIME) {
                    map.remove(&message.from.id);
                    drop(map);
                    println!("deleted state record for {}", state);
                    let notice_result = API
                        .send(message.chat.text(format!(
                            "you have been silent for too long\nwe cannot wait for you any longer"
                        )))
                        .await;
                    match notice_result {
                        Err(e) => println!("{:?}", e),
                        _ => (),
                    }
                } else {
                    println!("aborted record delete due to recency");
                    drop(map);
                }
            } else {
                println!(
                    "aborted record delete for {} because current state is {}",
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
pub fn immediate_purge_history(user: User, state: UserState) {
    tokio::spawn(async move {
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&user.id) {
            if r.state == state {
                map.remove(&user.id);
                drop(map);
                println!("deleted state record for {}", state);
            }
        }
    });
}
