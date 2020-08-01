use super::*;
use serde_json::Value;

///Adds a userstate record with animation state to userstate records map.  
///Fires wipe history command for animation state.
pub async fn start_gif(bot_message: impl BotMessage + 'static) {
    let source = "START_ANIMATION";
    let info = util_service::make_info(source);

    info("animation initiated");
    let id = bot_message.get_id();
    // Set the state
    set_state(id.clone(), UserState::Animation).await;
    info(&format!("record added for id {}", id));
    // Arc cloneable message
    let arc_message = Arc::new(bot_message);
    // And fire off wipe history
    wipe_history(Arc::clone(&arc_message), UserState::Animation);
    arc_message
        .send_message(responses::load("animation-start").into())
        .await;
}

///Finishes animation fetching.  
///Fires immediate purge history command for animation state.
pub async fn continue_gif(bot_message: impl BotMessage + 'static, processed_text: String) {
    let source = "CONTINUE_ANIMATION";
    let info = util_service::make_info(source);
    info("Animation response");
    // Arc cloneable message
    let arc_message = Arc::new(bot_message);
    // Purge state history
    immediate_purge_history(Arc::clone(&arc_message), UserState::Animation).await;
    let url = format!(
        "https://api.gfycat.com/v1/gfycats/search?search_text={}&count=1",
        processed_text
    );
    // Get json value from request
    if let Some(Value::Object(map)) = util_service::get_request_json(&url).await {
        // Get desired stuff from json
        if let Some(Value::Array(gfycats)) = map.get("gfycats") {
            for gif in gfycats {
                if let Some(Value::String(url)) = gif.get("max2mbGif") {
                    info(&format!("gif url is {}", url));
                    arc_message
                        .send_message(MsgCount::SingleMsg(Msg::File(url.to_string())))
                        .await;
                    return;
                }
            }
        }
    }
    // If something fails
    arc_message
        .send_message(responses::load("animation-fail").into())
        .await;
}
