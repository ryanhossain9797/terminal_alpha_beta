use super::*;
//------Chat will not be a state any more.
//------Rather any unknown message will be handled by chat in default
//use std::mem::drop;
//use std::time::Instant;

///Start chat intent, Only provides a single response without any state
pub async fn start_chat(bot_message: impl BotMessage) {
    println!("START_CHAT: chat initiated");

    println!("START_CHAT: responding to chat intent");

    responses::custom_response(bot_message, "chat-start".to_string()).await
}

///Continues chat.  
///Updates userstate record map with Chat intent and New time.  
///Fires wipe history command for Chat state.
pub async fn continue_chat(bot_message: impl BotMessage, _processed_text: String, intent: &str) {
    let source = "CONTINUE_CHAT";
    let info = util::make_info(source);
    if intent == "greet" {
        info("starting greet");
        responses::custom_response(bot_message, "chat-greet".to_string()).await
    } else if intent == "about" {
        info("starting about");
        responses::custom_response(bot_message, "chat-about".to_string()).await
    } else if intent == "technology" {
        info("starting technology");
        responses::custom_response(bot_message, "chat-technology".to_string()).await
    } else if intent == "functions" {
        info("starting functions");
        responses::custom_response(bot_message, "chat-functions".to_string()).await
    } else if intent == "creator" {
        info("starting creator");
        responses::custom_response(bot_message, "chat-creator".to_string()).await
    } else {
        info("unsupported");
        responses::unsupported_notice(bot_message).await
    }
}
