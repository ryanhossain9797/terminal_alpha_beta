use crate::handlers::*;
use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;

//---adds a userstate record with meme state to userstate records map
//---fires wipe history command for meme state
pub async fn start_meme(message: Message) -> root::MsgCount {
    println!("START_MEME: meme initiated");

    let mut map = root::RECORDS.lock().await;
    let id: i64 = message.from.id.into();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: root::UserState::Meme,
        });
    drop(map);
    println!("START_MEME: record added for id {}", id);
    root::wipe_history(message.clone(), root::UserState::Meme);

    root::MsgCount::SingleMsg(root::Msg::Text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
        \nyou want to find a so called \"MEME\"?\
        \nvery well, name one",
        &message.from.first_name
    )))
}

//---finishes meme fetching
//---fires immediate purge history command for meme state
#[allow(unused_variables)]
pub async fn continue_meme(message: Message, processesed_text: String) -> root::MsgCount {
    println!("CONTINUE_MEME: meme response");
    root::immediate_purge_history(message.from.clone(), root::UserState::Meme);
    root::MsgCount::SingleMsg(root::Msg::File("files/dp.jpg".to_string()))
}
