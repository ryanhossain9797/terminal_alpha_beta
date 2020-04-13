use crate::handlers::chat;
use crate::handlers::responses;
const LONGWAIT: u64 = 30;
const SHORTWAIT: u64 = 10;
const WAITTIME: u64 = LONGWAIT;
use std::collections::HashMap;
use std::mem::drop;
use std::time::{Duration, Instant};
use telegram_bot::*;

//
extern crate snips_nlu_lib;
use snips_nlu_lib::SnipsNluEngine;
//
lazy_static! {
    pub static ref RECORDS: tokio::sync::Mutex<HashMap<UserId, UserStateRecord>> =
        { tokio::sync::Mutex::new(HashMap::new()) };
}

lazy_static! {
    pub static ref ENGINE: SnipsNluEngine = {
        println!("\nLoading the nlu engine...");
        SnipsNluEngine::from_path("actionengine/").unwrap()
    };
}

pub struct UserStateRecord {
    pub username: String,
    pub state: String,
    pub last: Instant,
    pub chat: ChatId,
    pub history: Vec<String>,
}

pub async fn handler(
    api: &Api,
    message: &Message,
    processesed_text: String,
    will_respond: bool,
) -> Result<(), Error> {
    println!("processed text is '{}'", processesed_text);
    let map = RECORDS.lock().await;
    let entry_option = map.get(&message.from.id);
    if let Some(record) = entry_option {
        if processesed_text == "cancel last" {
            drop(map);
            let handler_assignment = cancel_history(api.clone(), message.clone()).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        } else if record.state == "chat".to_string() {
            drop(map);
            println!("continuing chat");
            let handler_assignment =
                chat::continue_chat(api.clone(), message.clone(), processesed_text.clone()).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        } else {
            drop(map);
            let notice_result = api
                .send(message.chat.text(format!("we cannot search yet")))
                .await;
            match notice_result {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
    } else if will_respond {
        drop(map);
        if processesed_text == "cancel last" {
        } else if processesed_text.starts_with("chat") {
            println!("starting chat");
            let start_chat = chat::start_chat(api.clone(), message.clone()).await;
            match start_chat {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        } else {
            let handler_assignment =
                natural_understanding(api.clone(), message.clone(), processesed_text).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
    }
    Ok(())
}

pub async fn natural_understanding(
    api: Api,
    message: Message,
    processed_text: String,
) -> Result<(), Error> {
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
        if result.intent.confidence_score > 0.5 {
            let response_result = if intent == "chat" {
                println!("starting chat");
                chat::start_chat(api.clone(), message.clone()).await
            } else if intent == "search" {
                println!("starting search");
                start_search(api.clone(), message.clone()).await
            } else {
                responses::unsupported_notice(api.clone(), message.clone()).await
            };
            match response_result {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        } else {
            println!("unknown intent");
            let handler_assignment =
                responses::unsupported_notice(api.clone(), message.clone()).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
    } else {
        println!("could not understand intent");
        let handler_assignment = responses::unsupported_notice(api.clone(), message.clone()).await;
        match handler_assignment {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
    Ok(())
}

pub async fn cancel_history(api: Api, message: Message) -> Result<(), Error> {
    let mut map = RECORDS.lock().await;
    map.remove(&message.from.id);
    drop(map);
    let notice_result = api
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

pub async fn wipe_history(message: Message, api: Api, state: String) -> Result<(), Error> {
    tokio::spawn(async move {
        tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
        let mut map = RECORDS.lock().await;
        if let Some(r) = map.get(&message.from.id) {
            if r.last.elapsed() > Duration::from_secs(WAITTIME) && r.state == state {
                map.remove(&message.from.id);
                drop(map);
                println!("deleted chat record for {}", state);
                let notice_result = api
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

pub async fn start_search(api: Api, message: Message) -> Result<(), Error> {
    println!("START_SEARCH: search initiated");

    let mut map = RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "search".to_string(),
            history: Vec::new(),
        });
    drop(map);
    println!("START_SEARCH: record added");
    api.send(message.chat.clone().text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
            \nwhat do you want to search for?",
        &message.from.first_name
    )))
    .await?;
    let wipe_launch = wipe_history(message.clone(), api.clone(), "search".to_string()).await;
    match wipe_launch {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}
