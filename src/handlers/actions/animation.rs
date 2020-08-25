use super::*;

///Adds a userstate record with animation state to userstate records map.  
///Fires wipe history command for animation state.
pub async fn start(bot_message: Box<dyn BotMessage>) {
    let source = "START_ANIMATION";
    let info = util::logger::info_logger(source);

    info("animation initiated");
    info(&format!("record added for id {}", bot_message.get_id()));
    // Arc cloneable message
    let arc_message = Arc::new(bot_message);
    // And fire off wipe history
    set_timed_state(Arc::clone(&arc_message), UserState::Animation).await;
    arc_message
        .send_message(responses::load("animation-start").into())
        .await;
}

///Finishes animation fetching.  
///Fires immediate purge history command for animation state.
pub async fn resume(bot_message: Box<dyn BotMessage>, processed_text: String) {
    let source = "CONTINUE_ANIMATION";
    let info = util::logger::info_logger(source);
    info("Animation response");
    // Arc cloneable message
    let arc_message = Arc::new(bot_message);
    // Purge state history
    cancel_matching_state(Arc::clone(&arc_message), UserState::Animation).await;

    arc_message
        .send_message(
            match gfycat_service::get_by_keyword(&processed_text).await {
                // If retrieving gif succeeds
                Ok(url) => MsgCount::SingleMsg(Msg::File(url)),
                // If retrieving fails
                _ => responses::load("animation-fail").into(),
            },
        )
        .await;
}
