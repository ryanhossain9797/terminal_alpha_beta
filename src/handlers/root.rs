use crate::handlers::chat;
use crate::handlers::identify;
use crate::handlers::responses;
use crate::handlers::search;
use crate::handlers::util;
const LONGWAIT: u64 = 30;

#[allow(dead_code)]
const SHORTWAIT: u64 = 10;
const WAITTIME: u64 = LONGWAIT;

use std::collections::HashMap;
use std::env;

use std::mem::drop;
use std::time::{Duration, Instant};
use telegram_bot::*;

//
extern crate snips_nlu_lib;
use snips_nlu_lib::SnipsNluEngine;
//

lazy_static! {
    //---Global API access
    pub static ref API: Api = {
        let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
        Api::new(token)
    };
    //---Record is a map holding all users state record info
    pub static ref RECORDS: tokio::sync::Mutex<HashMap<UserId, UserStateRecord>> =
        { tokio::sync::Mutex::new(HashMap::new()) };
    //---Snips NLU is used to pick actions when they don't match directly
    pub static ref ENGINE: SnipsNluEngine = {
        println!("\nLoading the nlu engine...");
        SnipsNluEngine::from_path("actionengine/").unwrap()
    };


}

//---A user state record holds an individual user's state
//---Last holds when it was last updated
//---History is just a vector of strings to hold misc info (ex: messages in chat state)
pub struct UserStateRecord {
    pub username: String,
    pub state: String,
    pub last: Instant,
    pub chat: ChatId,
    pub history: Vec<String>,
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
    let handler_assignment = if let Some(record) = entry_option {
        //---"cancel last will shut off the conversation"
        if processesed_text == "cancel last" {
            drop(map);
            cancel_history(message.clone()).await
        }
        //---"if state is chat"
        else if record.state == "chat".to_string() {
            drop(map);
            println!("continuing chat");
            chat::continue_chat(message.clone(), processesed_text.clone()).await
        }
        //---"if state is search"
        else if record.state == "search".to_string() {
            drop(map);
            println!("continuing search");
            search::continue_search(message.clone(), processesed_text.clone()).await
        }
        //---"if state is identify"
        else if record.state == "identify".to_string() {
            drop(map);
            println!("continuing identify");
            identify::continue_identify(message.clone(), processesed_text.clone()).await
        }
        //---"if state is unknown"
        else {
            drop(map);
            println!("some unknown state");
            responses::unknown_state_notice()
        }
    }
    //---if record from user doesn't exist, but is either IN A PRIVATE CHAT or MENTIONED IN A GROUP CHAT
    //---will start processing new info
    else if will_respond {
        drop(map);
        //---cancel last does nothing as there's nothing to cancel
        if processesed_text == "cancel last" {
            "nothing to cancel".to_string()
        }
        //---hand over to the natural understanding system for advanced matching
        else {
            natural_understanding(message.clone(), processesed_text).await
        }
    } else {
        "".to_string()
    };
    let assisgnment_result = API.send(message.chat.text(handler_assignment)).await;
    match assisgnment_result {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

//---FIX LEVEL: Works with strings
pub async fn natural_understanding(message: Message, processed_text: String) -> String {
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
    let response = if let Some(intent) = result.intent.intent_name {
        println!(
            "{} with confidence {}",
            intent, result.intent.confidence_score
        );
        //---tries to match against existing intents like chat, search etc
        //---only valid if confidence greater than 0.5
        if result.intent.confidence_score > 0.5 {
            if intent == "chat" {
                println!("ACTION_PICKER: starting chat");
                chat::start_chat(message.clone()).await
            } else if intent == "search" {
                println!("ACTION_PICKER: starting search");
                search::start_search(message.clone()).await
            } else if intent == "identify" {
                println!("ACTION_PICKER: starting identify");
                identify::start_identify(message.clone()).await
            } else {
                //---This one is only for unimplemented but known intents
                //---DOn't put stuff related to unknown intents here
                println!("ACTION_PICKER: unknown intent");
                util::log_message(processed_text);
                responses::unsupported_notice()
            }
        }
        //---unknown intent if cannot match to any intent confidently
        else {
            util::log_message(processed_text.clone());
            chat::continue_chat(message.clone(), processed_text).await
        }
    }
    //---unknown intent if can't match intent at all
    else {
        util::log_message(processed_text.clone());
        chat::continue_chat(message.clone(), processed_text).await
    };
    response
}

//---removes current history with a cancellation message
//---doesn't care about state
//---used with the cancel last command
pub async fn cancel_history(message: Message) -> String {
    let mut map = RECORDS.lock().await;
    map.remove(&message.from.id);
    drop(map);
    format!("understood. we will not prolong this conversation")
}

//---removes history after 30 seconds if it's not updated with a new time
//---AND the history state matches the provided state
//---message is provided to user
pub fn wipe_history(message: Message, state: String) {
    tokio::spawn(async move {
        tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&message.from.id) {
            if r.last.elapsed() > Duration::from_secs(WAITTIME) && r.state == state {
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
            }
        }
    });
}

//---immediately purges history IF provided state matches history state
//---used to remove history after state action is completed
//---no notice provided
pub fn immediate_purge_history(user: User, state: String) {
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
