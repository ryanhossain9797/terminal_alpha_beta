use super::*;
use serde_json::Value;

pub async fn unsupported_notice(m: impl BotMessage) {
    m.send_message(MsgCount::MultiMsg(vec![
        Msg::Text(match load_response("unsupported-notice-1") {
            Some(response) => response,
            None => response_unavailable(),
        }),
        Msg::Text(match load_response("unsupported-notice-2") {
            Some(response) => response,
            None => response_unavailable(),
        }),
    ]))
    .await;
}

pub async fn unknown_state_notice(bot_message: impl BotMessage + 'static) {
    bot_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match load_response("unknown-state") {
                Some(response) => response,
                None => response_unavailable(),
            },
        )))
        .await;
}

pub async fn custom_response(m: impl BotMessage, key: String) {
    m.send_message(MsgCount::SingleMsg(Msg::Text(match load_response(&key) {
        Some(response) => response,
        _ => "we could not understand your question".to_string(),
    })))
    .await;
}
pub fn load_response(key: &str) -> Option<String> {
    if let Some(json) = &*RESPONSES {
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
