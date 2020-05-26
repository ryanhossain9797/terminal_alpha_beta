use crate::handlers::*;
use serde_json::Value;
use std::mem::drop;
use std::time::Instant;
//---adds a userstate record with animation state to userstate records map
//---fires wipe history command for animation state
pub async fn start_gif(m: Box<dyn root::BotMessage + Send + Sync>) {
    println!("START_ANIMATION: aniamtion initiated");

    let mut map = root::RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: (*m).get_name(),
            last: Instant::now(),
            state: root::UserState::Animation,
        });
    drop(map);
    println!("START_ANIMATION: record added for id {}", id);
    root::wipe_history(m.clone(), root::UserState::Animation);

    (*m).send_message(root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("animation-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}

//---finishes animation fetching
//---fires immediate purge history command for animation state
pub async fn continue_gif(m: Box<dyn root::BotMessage + Send + Sync>, processed_text: String) {
    println!("CONTINUE_ANIMATION: animation response");
    root::immediate_purge_history(m.clone(), root::UserState::Animation);

    let url = format!(
        "https://api.gfycat.com/v1/gfycats/search?search_text={}&count=1",
        processed_text
    );
    match util::get_request_json(&url).await {
        Some(json_string) => match serde_json::from_str(&json_string).ok() {
            Some(json) => match json {
                Value::Object(map) => match map.get("gfycats") {
                    Some(Value::Array(gfycats)) => {
                        for gif in gfycats {
                            match gif.get("max2mbGif") {
                                Some(Value::String(url)) => {
                                    println!("gif url is {}", url);
                                    (*m).send_message(root::MsgCount::SingleMsg(root::Msg::File(
                                        url.to_string(),
                                    )));
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
    (*m).send_message(root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("animation-fail") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}
