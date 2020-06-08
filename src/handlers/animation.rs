use super::*;
use serde_json::Value;

//---adds a userstate record with animation state to userstate records map
//---fires wipe history command for animation state
pub async fn start_gif(bot_message: impl BotMessage + 'static) {
    println!("START_ANIMATION: aniamtion initiated");
    let id = bot_message.get_id();
    set_state(id.clone(), UserState::Animation).await;
    println!("START_ANIMATION: record added for id {}", id);
    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Animation);
    arc_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match responses::load_response("animation-start") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            },
        )))
        .await;
}

//---finishes animation fetching
//---fires immediate purge history command for animation state
pub async fn continue_gif(bot_message: impl BotMessage + 'static, processed_text: String) {
    println!("CONTINUE_ANIMATION: animation response");
    let arc_message = Arc::new(bot_message);
    immediate_purge_history(Arc::clone(&arc_message), UserState::Animation);
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
                            arc_message
                                .send_message(MsgCount::SingleMsg(Msg::File(url.to_string())))
                                .await;
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
    arc_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match responses::load_response("animation-fail") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            },
        )))
        .await;
}
