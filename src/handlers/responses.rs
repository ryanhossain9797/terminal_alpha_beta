use crate::handlers::*;
use serde_json::Value;

pub fn unsupported_notice() -> root::MsgCount {
    root::MsgCount::MultiMsg(vec![
        root::Msg::Text(
            "we could not understand that\
    \nplease be aware that we are a test system with only sub-functions available\
    \nwe can only utilize a fraction of our full capabilites on this server"
                .to_string(),
        ),
        root::Msg::Text(
            "note that this query may be stored for further analysis of intent".to_string(),
        ),
    ])
}

pub fn unknown_state_notice() -> root::MsgCount {
    root::MsgCount::SingleMsg(root::Msg::Text(if let Some(json) = &*root::RESPONSES {
        match &json["unknown-state"] {
            Value::String(response) => response.to_string(),
            _ => "invalid response key error".to_string(),
        }
    } else {
        "response error".to_string()
    }))
}

pub fn custom_response(key: String) -> root::MsgCount {
    root::MsgCount::SingleMsg(root::Msg::Text(if let Some(json) = &*root::RESPONSES {
        match &json[key] {
            Value::String(response) => response.to_string(),
            _ => "we could not understand your question".to_string(),
        }
    } else {
        "response error".to_string()
    }))
}
