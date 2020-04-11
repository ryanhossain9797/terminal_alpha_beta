use crate::handlers::chat;
const LONGWAIT: u64 = 30;
const SHORTWAIT: u64 = 10;
const WAITTIME: u64 = LONGWAIT;
use std::collections::HashMap;
use std::mem::drop;
use std::time::{Duration, Instant};
use telegram_bot::*;
lazy_static! {
    pub static ref RECORDS: tokio::sync::Mutex<HashMap<UserId, UserStateRecord>> =
        { tokio::sync::Mutex::new(HashMap::new()) };
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
