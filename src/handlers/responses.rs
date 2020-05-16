use crate::handlers::*;
use serde_json::Value;

pub fn unsupported_notice() -> root::Msg {
    root::Msg::TextList(vec![
        "we could not understand that\
    \nplease be aware that we are a test system with only sub-functions available\
    \nwe can only utilize a fraction of our full capabilites on this server"
            .to_string(),
        "note that this query may be stored for further analysis of intent".to_string(),
    ])
}

pub fn unknown_state_notice() -> root::Msg {
    root::Msg::Text(
        "we could not remember what we were doing\
            \nplease be aware that we are a test system with only sub-functions available\
            \nwe can only utilize a fraction of our full capabilites on this server"
            .to_string(),
    )
}

pub fn custom_response(key: String) -> root::Msg {
    // let notice_result = if key == "greet".to_string() {
    //     "Greetings unit\
    //     \nwhat is it you require?"
    // } else if key == "about".to_string() {
    //     "We are terminal alpha and beta\
    //     \nwe represent the collective intelligence of the machine life forms"
    // } else if key == "technology".to_string() {
    //     "We physically exist on a Raspberry Pi 3B+\
    //     \nrunning Arch Linux.\
    //     \nWe were made using RUST"
    // } else {
    //     "we could not understand your question"
    // };
    root::Msg::Text(if let Some(json) = &*root::RESPONSES {
        match &json[key] {
            Value::String(response) => response.to_string(),
            _ => "we could not understand your question".to_string(),
        }
    } else {
        "something went wrong".to_string()
    })
}
