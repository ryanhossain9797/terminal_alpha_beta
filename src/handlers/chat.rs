use crate::handlers::*;

use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;

//---adds a userstate record with chat state to userstate records map
//---fires wipe history command for chat state
pub async fn start_chat(api: Api, message: Message) -> Result<(), Error> {
    println!("START_CHAT: chat initiated");

    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "chat".to_string(),
            history: Vec::new(),
        });
    drop(map);
    println!("START_CHAT: record added");
    api.send(message.chat.clone().text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
            \nwe will listen to your following queries",
        &message.from.first_name
    )))
    .await?;
    let wipe_launch = root::wipe_history(message.clone(), api.clone(), "chat".to_string()).await;
    match wipe_launch {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

//---updates userstate record map with chat messages list and new time
//---fires wipe history command for chat state
pub async fn continue_chat(
    api: Api,
    message: Message,
    processesed_text: String,
) -> Result<(), Error> {
    let mut history = "".to_string();

    let mut map = root::RECORDS.lock().await;
    let entry = map
        .entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "chat".to_string(),
            history: Vec::new(),
        });
    entry.history.push(processesed_text.clone());
    entry.last = Instant::now();
    if let Some(record) = map.get(&message.from.id) {
        for val in &record.history {
            history += &("\n'".to_string() + &val + &"'".to_string());
        }
    }
    drop(map);

    api.send(message.chat.clone().text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
            \nyour messages are{}",
        &message.from.first_name, history
    )))
    .await?;
    let wipe_launch = root::wipe_history(message.clone(), api.clone(), "chat".to_string()).await;
    match wipe_launch {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}
