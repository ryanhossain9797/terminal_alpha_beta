use super::*;
use serde_json::Value;
use std::mem::drop;
use std::time::Instant;
//---adds a userstate record with animation state to userstate records map
//---fires wipe history command for animation state
pub async fn start_gif(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_ANIMATION: aniamtion initiated");

    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| UserStateRecord {
            last: Instant::now(),
            state: UserState::Animation,
        });
    drop(map);
    println!("START_ANIMATION: record added for id {}", id);
    wipe_history(m.clone(), UserState::Animation);

    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("animation-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}

//---finishes animation fetching
//---fires immediate purge history command for animation state
pub async fn continue_gif(m: Box<dyn BotMessage + Send + Sync>, processed_text: String) {
    println!("CONTINUE_ANIMATION: animation response");
    immediate_purge_history(m.clone(), UserState::Animation);

    let url = format!(
        "https://api.gfycat.com/v1/gfycats/search?search_text={}&count=1",
        processed_text
    );
    match general::get_request_json(&url).await {
        Some(Value::Object(map)) => match map.get("gfycats") {
            Some(Value::Array(gfycats)) => {
                for gif in gfycats {
                    match gif.get("max2mbGif") {
                        Some(Value::String(url)) => {
                            println!("gif url is {}", url);
                            (*m).send_message(MsgCount::SingleMsg(Msg::File(url.to_string())));
                            return;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("animation-fail") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}
