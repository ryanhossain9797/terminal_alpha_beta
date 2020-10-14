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
pub async fn resume(bot_message: Box<dyn BotMessage>, intent: Intent) {
    use extra::custom_response as response;
    use Intent::{About, Creator, Functions, Greet, Technology};
    match intent {
        Greet => response(bot_message, "chat-greet").await,
        About => response(bot_message, "chat-about").await,
        Technology => response(bot_message, "chat-technology").await,
        Functions => response(bot_message, "chat-functions").await,
        Creator => response(bot_message, "chat-creator").await,
        _ => extra::unsupported_notice(bot_message).await,
    }
}
