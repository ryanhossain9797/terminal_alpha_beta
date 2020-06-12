use super::*;
//------Chat will not be a state any more.
//------Rather any unknown message will be handled by chat in default
//use std::mem::drop;
//use std::time::Instant;

///Start chat intent, Only provides a single response without any state
pub async fn start_chat(bot_message: impl BotMessage) {
    println!("START_CHAT: chat initiated");
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    let mut map = RECORDS.lock().await;
    let id = m.get_id();
    map.insert(
        format!("{}", id),
        UserStateRecord {
            last: Instant::now(),
            state: UserState::Chat,
        },
    );
    drop(map);
    wipe_history(message.clone(), UserState::Chat);
    println!("START_CHAT: record added");
    */
    println!("START_CHAT: responding to chat intent");

    responses::custom_response(bot_message, "chat-start".to_string()).await
}

///Continues chat.  
///Updates userstate record map with Chat intent and New time.  
///Fires wipe history command for Chat state.
pub async fn continue_chat(bot_message: impl BotMessage, _processed_text: String, intent: &str) {
    let source = "CONTINUE_CHAT";
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default

    // let mut map = RECORDS.lock().await;
    // let id = m.get_id();
    // map.insert(
    //     format!("{}", id),
    //     UserStateRecord {
    //         last: Instant::now(),
    //         state: UserState::Chat,
    //     },
    // );
    // drop(map);

    if intent == "greet" {
        util::log_info(source, "starting greet");
        responses::custom_response(bot_message, "chat-greet".to_string()).await
    } else if intent == "about" {
        util::log_info(source, "starting about");
        responses::custom_response(bot_message, "chat-about".to_string()).await
    } else if intent == "technology" {
        util::log_info(source, "starting technology");
        responses::custom_response(bot_message, "chat-technology".to_string()).await
    } else if intent == "functions" {
        util::log_info(source, "starting functions");
        responses::custom_response(bot_message, "chat-functions".to_string()).await
    } else if intent == "creator" {
        util::log_info(source, "starting creator");
        responses::custom_response(bot_message, "chat-creator".to_string()).await
    } else {
        util::log_info(source, "unsupported");
        responses::unsupported_notice(bot_message).await
    }
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    wipe_history(message.clone(), "chat".to_string());
    */
}
