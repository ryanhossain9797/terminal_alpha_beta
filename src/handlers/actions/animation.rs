use super::*;

///Adds a userstate record with animation state to userstate records map.  
///Fires wipe history command for animation state.
pub async fn start(bot_message: Box<dyn BotMessage>) -> anyhow::Result<()> {
    let source = "START_ANIMATION";
    let info = util::logger::info(source);

    info("animation initiated");
    // Arc cloneable message
    let arc_message = Arc::new(bot_message);

    arc_message
        .send_message(responses::load("animation-start").into())
        .await;

    // And fire off wipe history
    handle_event(UserEventData::new(UserEvent::Animation, arc_message)).await
}

///Finishes animation fetching.  
///Fires immediate purge history command for animation state.
pub async fn resume(bot_message: Box<dyn BotMessage>, processed_text: String) {
    let source = "CONTINUE_ANIMATION";
    let info = util::logger::info(source);
    info("Animation response");
    // Arc cloneable message
    let arc_message = Arc::new(bot_message);
    // Purge state history
    let _ = handle_event(UserEventData::new(
        UserEvent::AnimationCompleted,
        Arc::clone(&arc_message),
    ))
    .await;

    arc_message
        .send_message(
            match gfycat_service::get_by_keyword(processed_text.as_str()).await {
                // If retrieving gif succeeds
                Ok(url) => MsgCount::SingleMsg(Msg::File(url)),
                // If retrieving fails
                Err(_) => responses::load("animation-fail").into(),
            },
        )
        .await;
}
