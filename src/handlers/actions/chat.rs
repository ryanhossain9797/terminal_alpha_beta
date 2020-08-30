use super::*;

///Start chat intent, Only provides a single response without any state
pub async fn start(bot_message: Box<dyn BotMessage>) {
    let source = "START_CHAT";
    let info = util::logger::info(source);
    info("Chat initiated");

    extra::custom_response(bot_message, "chat-start").await;
}

///Continues chat.  
///Updates userstate record map with Chat intent and New time.  
///Fires wipe history command for Chat state.
pub async fn resume(bot_message: Box<dyn BotMessage>, _processed_text: String, intent: &str) {
    use extra::custom_response as response;
    let source = "CONTINUE_CHAT";
    let info = util::logger::info(source);

    info(format!("starting {}", intent).as_str());
    match intent {
        "greet" => response(bot_message, "chat-greet").await,
        "about" => response(bot_message, "chat-about").await,
        "technology" => response(bot_message, "chat-technology").await,
        "functions" => response(bot_message, "chat-functions").await,
        "creator" => response(bot_message, "chat-creator").await,
        _ => extra::unsupported_notice(bot_message).await,
    }
}
