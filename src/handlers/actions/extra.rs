use super::*;

///Tests any unknown intent
pub async fn start(bot_message: Box<dyn BotMessage>) -> anyhow::Result<()> {
    let source = "START_UNKNOWN";
    let info = util::logger::info(source);
    info("Unknown state initiated");
    let arc_message = Arc::new(bot_message);

    arc_message
        .send_message(responses::load("intentional-unknownstate").into())
        .await;
    handle_event(UserEventData::new(UserEvent::Unknown, arc_message)).await
}

///Simply uses `load_response` to load a response for the provided key.  
///If unavailable replies with a default message.
pub async fn custom_response(bot_message: Box<dyn BotMessage>, key: &str) {
    bot_message
        .send_message(load(key).map_or_else(|| load("unknown-question").into(), |msg| msg.into()))
        .await;
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
///Usually represents an Error or a WIP state.
pub async fn unknown_state_notice(bot_message: Box<dyn BotMessage>) {
    bot_message.send_message(load("unknown-state").into()).await;
}
