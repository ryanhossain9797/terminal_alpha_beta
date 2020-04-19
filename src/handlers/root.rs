use crate::handlers::chat;
use crate::handlers::responses;
use crate::handlers::search;

const LONGWAIT: u64 = 30;
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

lazy_static! {}

lazy_static! {}

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
    if let Some(record) = entry_option {
        //---"cancel last will shut off the conversation"
        let handler_assignment = if processesed_text == "cancel last" {
            drop(map);
            cancel_history(message.clone()).await
        }
        //---"if state is chat"
        else if record.state == "chat".to_string() {
            drop(map);
            println!("continuing chat");
            chat::continue_chat(API.clone(), message.clone(), processesed_text.clone()).await
        }
        //---"if state is search"
        else if record.state == "search".to_string() {
            drop(map);
            println!("continuing search");
            search::continue_search(API.clone(), message.clone(), processesed_text.clone()).await
        }
        //---"if state is unknown"
        else {
            drop(map);
            println!("some unknown state");
            responses::unknown_state_notice(API.clone(), message.chat.clone()).await
        };
        match handler_assignment {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
    //---if record from user doesn't exist, but is either IN A PRIVATE CHAT or MENTIONED IN A GROUP CHAT
    //---will start processing new info
    else if will_respond {
        drop(map);
        //---cancel last does nothing as ther's nothing to cancel
        if processesed_text == "cancel last" {
        }
        //---starts a chat
        else if processesed_text.starts_with("chat") {
            println!("starting chat");
            let start_chat = chat::start_chat(API.clone(), message.clone()).await;
            match start_chat {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
        //---starts a search
        else if processesed_text.starts_with("search") {
            println!("starting search");
            let start_search = search::start_search(API.clone(), message.clone()).await;
            match start_search {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
        //---if nothing matches directly
        //---hand over to the natural understanding system for advanced matching
        else {
            let handler_assignment = natural_understanding(message.clone(), processesed_text).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
    }
    Ok(())
}

pub async fn natural_understanding(message: Message, processed_text: String) -> Result<(), Error> {
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
    if let Some(intent) = result.intent.intent_name {
        println!(
            "{} with confidence {}",
            intent, result.intent.confidence_score
        );
        //---tries to match against existing intents like chat, search etc
        //---only valid if confidence greater than 0.5
        if result.intent.confidence_score > 0.5 {
            let response_result = if intent == "chat" {
                println!("starting chat");
                chat::start_chat(API.clone(), message.clone()).await
            } else if intent == "search" {
                println!("starting search");
                search::start_search(API.clone(), message.clone()).await
            } else {
                responses::unsupported_notice(API.clone(), message.chat.clone()).await
            };
            match response_result {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
        //---unknown intent if cannot match to any intent confidently
        else {
            println!("unknown intent");
            let handler_assignment =
                responses::unsupported_notice(API.clone(), message.chat.clone()).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
    }
    //---unknown intent if can't match intent at all
    else {
        println!("could not understand intent");
        let handler_assignment =
            responses::unsupported_notice(API.clone(), message.chat.clone()).await;
        match handler_assignment {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
    Ok(())
}

//---removes current history with a cancellation message
//---doesn't care about state
//---used with the cancel last command
pub async fn cancel_history(message: Message) -> Result<(), Error> {
    let mut map = RECORDS.lock().await;
    map.remove(&message.from.id);
    drop(map);
    let notice_result = API
        .send(
            message
                .chat
                .text(format!("understood. we will not prolong this conversation")),
        )
        .await;
    match notice_result {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

//---removes history after 30 seconds if it's not updated with a new time
//---AND the history state matches the provided state
//---message is provided to user
pub async fn wipe_history(message: Message, state: String) -> Result<(), Error> {
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
    Ok(())
}

//---immediately purges history IF provided state matches history state
//---used to remove history after state action is completed
//---no notice provided
pub async fn imeediate_purge_history(user: User, state: String) -> Result<(), Error> {
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
    Ok(())
}
