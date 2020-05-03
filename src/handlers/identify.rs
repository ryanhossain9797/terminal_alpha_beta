use crate::handlers::*;
use bson::{bson, doc, Bson};
use mongodb::{options::ClientOptions, options::FindOptions, Client, Database};
use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;
//---adds a userstate record with identify state to userstate records map
//---fires wipe history command for identify state
pub async fn start_identify(message: Message) -> String {
    println!("START_IDENTIFY: identify initiated");

    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "identify".to_string(),
            history: Vec::new(),
        });
    drop(map);
    println!("START_IDENTIFY: record added");
    root::wipe_history(message.clone(), "identify".to_string());

    format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
        \nwho do you want to look up?",
        &message.from.first_name
    )
}

//---finishes identify
//---fires immediate purge history command for identify state
#[allow(unused_variables)]
pub async fn continue_identify(message: Message, processesed_text: String) -> String {
    root::immediate_purge_history(message.from.clone(), "identify".to_string());
    println!("beginning identification");
    if let Some(db) = &*root::DATABASE {
        format!(
            "Terminal Alpha and Beta:\
                \nWe cannot identify people yet"
        )
    } else {
        format!(
            "Terminal Alpha and Beta:\
                \nWe cannot identify people yet"
        )
    }
}
