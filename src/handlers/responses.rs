use crate::handlers::*;
use telegram_bot::*;

//---------------------Will eventually replace unsupported notice
pub fn unsupported_notice() -> String {
    "we could not understand that\
    \nplease be aware that we are a test system with only sub-functions available\
    \nwe can only utilize a fraction of our full capabilites on this server"
        .to_string()
}

pub fn unknown_state_notice() -> String {
    format!(
        "we could not remember what we were doing\
            \nplease be aware that we are a test system with only sub-functions available\
            \nwe can only utilize a fraction of our full capabilites on this server"
    )
}

pub fn custom_response(key: String) -> String {
    let notice_result = if key == "about".to_string() {
        "we are terminal alpha and beta\
                \nwe represent the collective intelligence of the machine life forms"
    } else if key == "technology".to_string() {
        "we are running on a raspberry pi 3 b+\
                    \nwe were made using RUST"
    } else {
        "we could not understand your question"
    };
    notice_result.to_string()
}
