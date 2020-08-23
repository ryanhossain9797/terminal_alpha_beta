use super::*;

//Tests any unknown intent
pub async fn start_unknown(bot_message: Box<dyn BotMessage>) {
    let source = "START_UNKNOWN";
    let info = util::logger::make_info(source);
    info("Unknown state initiated");
    let arc_message = Arc::new(bot_message);

    set_timed_state(Arc::clone(&arc_message), UserState::Unknown).await;

    arc_message
        .send_message(responses::load("intentional-unknownstate").into())
        .await;
}

///Simply uses load_response to load a response for the provided key.  
///If unavailable replies with a default message.
pub async fn custom_response(bot_message: Box<dyn BotMessage>, key: &str) {
    match load(key) {
        Some(msg) => bot_message.send_message(msg.into()).await,
        _ => {
            bot_message
                .send_message(load("unknown-question").into())
                .await
        }
    }
}

///Message to send when the user's message can't be handled at all.
pub async fn unsupported_notice(bot_message: Box<dyn BotMessage>) {
    bot_message
        .send_message(MsgCount::MultiMsg(vec![
            load("unsupported-notice-1").into(),
            load("unsupported-notice-2").into(),
        ]))
        .await;
}

///Notice to send when the stored state for a user is not supported.  
//Usually represents an Error or a WIP state.
pub async fn unknown_state_notice(bot_message: Box<dyn BotMessage>) {
    bot_message.send_message(load("unknown-state").into()).await;
}
