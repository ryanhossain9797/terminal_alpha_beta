use super::*;
//------Chat will not be a state any more.
//------Rather any unknown message will be handled by chat in default
//use std::mem::drop;
//use std::time::Instant;

pub async fn start_chat(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_CHAT: chat initiated");
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
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

    responses::custom_response(m, "chat-start".to_string()).await
}

///FIX LEVEL: Works with strings
///updated to implement RETURN STRINGS
///updates userstate record map with chat messages list and new time
///fires wipe history command for chat state
pub async fn continue_chat(
    m: Box<dyn BotMessage + Send + Sync>,
    _processed_text: String,
    intent: &str,
) {
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default

    // let mut map = RECORDS.lock().await;
    // let id = (*m).get_id();
    // map.insert(
    //     format!("{}", id),
    //     UserStateRecord {
    //         last: Instant::now(),
    //         state: UserState::Chat,
    //     },
    // );
    // drop(map);

    if intent == "greet" {
        println!("starting greet");
        responses::custom_response(m, "chat-greet".to_string()).await
    } else if intent == "about" {
        println!("starting about");
        responses::custom_response(m, "chat-about".to_string()).await
    } else if intent == "technology" {
        println!("starting technology");
        responses::custom_response(m, "chat-technology".to_string()).await
    } else if intent == "functions" {
        println!("starting functions");
        responses::custom_response(m, "chat-functions".to_string()).await
    } else if intent == "creator" {
        println!("starting creator");
        responses::custom_response(m, "chat-creator".to_string()).await
    } else {
        responses::unsupported_notice(m).await
    }
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    wipe_history(message.clone(), "chat".to_string());
    */
}
