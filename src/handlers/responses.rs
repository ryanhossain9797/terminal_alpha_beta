use crate::handlers::*;
use serde_json::Value;

pub fn unsupported_notice(m: Box<dyn root::BotMessage + Send + Sync>) {
    (*m).send_message(root::MsgCount::MultiMsg(vec![
        root::Msg::Text(match load_response("unsupported-notice-1") {
            Some(response) => response,
            None => response_unavailable(),
        }),
        root::Msg::Text(match load_response("unsupported-notice-2") {
            Some(response) => response,
            None => response_unavailable(),
        }),
    ]));
}

pub fn unknown_state_notice(m: Box<dyn root::BotMessage + Send + Sync>) {
    (*m).send_message(root::MsgCount::SingleMsg(root::Msg::Text(
        match load_response("unknown-state") {
            Some(response) => response,
            None => response_unavailable(),
        },
    )));
}

pub fn custom_response(m: Box<dyn root::BotMessage + Send + Sync>, key: String) {
    (*m).send_message(root::MsgCount::SingleMsg(root::Msg::Text(
        match load_response(&key) {
            Some(response) => response,
            _ => "we could not understand your question".to_string(),
        },
    )));
}
pub fn load_response(key: &str) -> Option<String> {
    if let Some(json) = &*root::RESPONSES {
        match &json[key] {
            Value::String(response) => {
                return Some(response.to_string());
            }
            _ => {}
        }
    }
    return None;
}

pub fn response_unavailable() -> String {
    "response unavailable error".to_string()
}

/*
{
    "chat-start": "Greetings unit.\nYou are free to ask any questions.\nWhether we answer or not depends on us.\nNote that in public groups you must mention us by our handle.",
    "chat-greet": "Greetings unit\nwhat is it you require?",
    "chat-about": "We are terminal alpha and beta\nwe represent the collective intelligence of the machine life forms",
    "chat-technology": "We physically exist on a Raspberry Pi 3B+\nrunning Arch Linux.\nWe were made using RUST and GO",
    "identify-start": "Terminal Alpha and Beta:\nGreetings unit\nwho do you want to look up?",
    "identify-partialmatch": "We could not find that exact person\nBut we found a {name}:\n{description}",
    "identify-notfound": "We could not find that person, Tagged for future identification",
    "identify-dberror": "We could not access the people database",
    "unknown-state": "we could not remember what we were doing\nplease be aware that we are a test system with only sub-functions available\nwe can only utilize a fraction of our full capabilites on this server",
    "unsupported-notice-1": "we could not understand that\nplease be aware that we are a test system with only sub-functions available\nwe can only utilize a fraction of our full capabilites on this server",
    "unsupported-notice-2": "note that this query may be stored for further analysis of intent"
}
*/
