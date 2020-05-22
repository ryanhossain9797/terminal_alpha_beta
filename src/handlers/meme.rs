use crate::handlers::*;
use std::mem::drop;
use std::time::Instant;

//---adds a userstate record with meme state to userstate records map
//---fires wipe history command for meme state
pub async fn start_meme(m: Box<dyn root::BotMessage + Send + Sync>) {
    println!("START_MEME: meme initiated");

    let mut map = root::RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: (*m).get_name(),
            last: Instant::now(),
            state: root::UserState::Meme,
        });
    drop(map);
    println!("START_MEME: record added for id {}", id);
    root::wipe_history(m.clone(), root::UserState::Meme);

    (*m).send_msg(root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("meme-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}

//---finishes meme fetching
//---fires immediate purge history command for meme state
#[allow(unused_variables)]
pub async fn continue_meme(m: Box<dyn root::BotMessage + Send + Sync>, processesed_text: String) {
    println!("CONTINUE_MEME: meme response");
    root::immediate_purge_history(m.clone(), root::UserState::Meme);
    (*m).send_msg(root::MsgCount::SingleMsg(root::Msg::File(
        "files/dp.jpg".to_string(),
    )));
}
