use crate::handlers::util;
use crate::handlers::*;
use telegram_bot::*;
//------Chat will not be a state any more.
//------Rather any unknown message will be handled by chat in default
//use std::mem::drop;
//use std::time::Instant;

//
extern crate snips_nlu_lib;
use snips_nlu_lib::SnipsNluEngine;
//

//---Snips NLU is used to pick actions when they don't match directly
lazy_static! {
    pub static ref ENGINE: SnipsNluEngine = {
        println!("\nLoading the nlu engine...");
        SnipsNluEngine::from_path("chatengine/").unwrap()
    };
}

//---FIX LEVEL: Returns Strings (y)
//---adds a userstate record with chat state to userstate records map
//---fires wipe history command for chat state
pub async fn start_chat(message: Message) -> String {
    println!("START_CHAT: chat initiated");
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "chat".to_string(),
            history: Vec::new(),
        });
    drop(map);
    root::wipe_history(message.clone(), "chat".to_string());
    println!("START_CHAT: record added");
    */
    println!("START_CHAT: responding to chat intent");

    format!(
        "Greetings unit {}.\
        \nYou are free to ask any questions.\
        \nWhether we answer or not depends on us.\
        \nNote that in public groups you must mention us by our handle.",
        &message.from.first_name
    )
}

//---FIX LEVEL: Works with strings
//---updated to implement RETURN STRINGS
//---updates userstate record map with chat messages list and new time
//---fires wipe history command for chat state
pub async fn continue_chat(/*message: Message,*/ processed_text: String) -> String {
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    let mut map = root::RECORDS.lock().await;
    let entry = map
        .entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "chat".to_string(),
            history: Vec::new(),
        });
    entry.history.push(processed_text.clone());
    entry.last = Instant::now();
    drop(map);
    */

    let intents_alternatives = 1;
    let slots_alternatives = 1;

    let result = ENGINE
        .parse_with_alternatives(
            &processed_text,
            None,
            None,
            intents_alternatives,
            slots_alternatives,
        )
        .unwrap();
    let response = if let Some(intent) = result.intent.intent_name {
        println!(
            "{} with confidence {}",
            intent, result.intent.confidence_score
        );
        //---tries to match against existing intents like chat, search etc
        //---only valid if confidence greater than 0.5
        if result.intent.confidence_score > 0.5 {
            if intent == "greet" {
                println!("starting greet");
                responses::custom_response("greet".to_string())
            } else if intent == "about" {
                println!("starting about");
                responses::custom_response("about".to_string())
            } else if intent == "technology" {
                println!("starting technology");
                responses::custom_response("technology".to_string())
            } else {
                responses::unsupported_notice()
            }
        }
        //---unknown intent if cannot match to any intent confidently
        else {
            println!("unsure intent");
            util::log_message(processed_text.clone());
            responses::unsupported_notice()
        }
    }
    //---unknown intent if can't match intent at all
    else {
        println!("unknown intent");
        util::log_message(processed_text.clone());
        responses::unsupported_notice()
    };
    //------Chat will not be a state any more.
    //------Rather any unknown message will be handled by chat in default
    /*
    root::wipe_history(message.clone(), "chat".to_string());
    */
    response
}
