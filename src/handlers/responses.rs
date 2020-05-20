use crate::handlers::*;
use serde_json::Value;

pub fn unsupported_notice() -> root::MsgCount {
    root::MsgCount::MultiMsg(vec![
        root::Msg::Text(match load_response("unsupported-notice-1") {
            Some(response) => response,
            None => "response unavailable error".to_string(),
        }),
        root::Msg::Text(match load_response("unsupported-notice-2") {
            Some(response) => response,
            None => "response unavailable error".to_string(),
        }),
    ])
}

pub fn unknown_state_notice() -> root::MsgCount {
    root::MsgCount::SingleMsg(root::Msg::Text(match load_response("unknown-state") {
        Some(response) => response,
        None => "response unavailable error".to_string(),
    }))
}

pub fn custom_response(key: String) -> root::MsgCount {
    root::MsgCount::SingleMsg(root::Msg::Text(match load_response(&key) {
        Some(response) => response,
        _ => "we could not understand your question".to_string(),
    }))
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
