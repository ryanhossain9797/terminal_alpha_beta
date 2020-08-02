use super::*;

///Start chat intent, Only provides a single response without any state
pub async fn start_chat(bot_message: impl BotMessage) {
    let source = "START_CHAT";
    let info = util::logger::make_info(source);
    info("Chat initiated");

    let dyn_clone = bot_message.dyn_clone();

    extra::custom_response(bot_message, "chat-start").await;

    state::userstate::send_msg(dyn_clone).await;
}

///Continues chat.  
///Updates userstate record map with Chat intent and New time.  
///Fires wipe history command for Chat state.
pub async fn continue_chat(bot_message: impl BotMessage, _processed_text: String, intent: &str) {
    let source = "CONTINUE_CHAT";
    let info = util::logger::make_info(source);

    use extra::custom_response as response;

    let response = match intent {
        "greet" => response(bot_message, "chat-greet").await,
        "about" => response(bot_message, "chat-about").await,
        "technology" => response(bot_message, "chat-technology").await,
        "functions" => response(bot_message, "chat-functions").await,
        "creator" => response(bot_message, "chat-creator").await,
        _ => extra::unsupported_notice(bot_message).await,
    };
    info(&format!("starting {}", intent));
    response
}
