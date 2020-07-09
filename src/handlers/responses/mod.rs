use super::*;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde_json::Value;

///RESPONSES: Response json holding all the responses.  
///Put in a json so they can be modified without recompiling the bot.  
///Loaded at startup, Restart Bot to reload.
static RESPONSES: Lazy<Option<serde_json::Value>> = Lazy::new(|| {
    println!("\nLoading JSON responses");
    match read_to_string("data/responses.json") {
        Ok(json) => serde_json::from_str(&json).ok(),
        Err(_) => None,
    }
});

pub fn initialize_responses() {
    Lazy::force(&RESPONSES);
}

const NAMES: [&str; 2] = ["Terminal Alpha", "Terminal Beta"];

///Message to send when the user's message can't be handled at all.
pub async fn unsupported_notice(m: impl BotMessage) {
    m.send_message(MsgCount::MultiMsg(vec![
        Msg::Text(load_named("unsupported-notice-1").unwrap_or_else(responses::unavailable)),
        Msg::Text(load_named("unsupported-notice-2").unwrap_or_else(responses::unavailable)),
    ]))
    .await;
}

///Notice to send when the stored state for a user is not supported.  
//Usually represents an Error or a WIP state.
pub async fn unknown_state_notice(bot_message: impl BotMessage + 'static) {
    bot_message
        .send_message(load_named("unknown-state").unwrap_or_else(responses::unavailable))
        .await;
}

///Simply uses load_response to load a response for the provided key.  
///If unavailable replies with a default message.
pub async fn custom_response(m: impl BotMessage, key: &str) {
    m.send_message(
        load_named(key).unwrap_or_else(|| {
            load_named("unknown-question").unwrap_or_else(responses::unavailable)
        }),
    )
    .await;
}

///Uses load_text() to load a response,  
///then prepends  
///#### `Terminal Alpha:`  
///or
///#### `Terminal Beta:`
pub fn load_named(key: &str) -> Option<String> {
    if let Some(name) = NAMES.choose(&mut rand::thread_rng()) {
        if let Some(response) = load_text(key) {
            return Some(format!("{}:\n{}", name, response));
        }
    }
    None
}

///Loads a response from the JSON storage for the provided key.  
///Returns the Option<String>, May be None if response is not found.
pub fn load_text(key: &str) -> Option<String> {
    if let Some(json) = &*RESPONSES {
        match &json[key] {
            Value::String(response) => {
                return Some(response.to_string());
            }
            Value::Array(responses) => {
                if let Some(Value::String(response)) = responses.choose(&mut rand::thread_rng()) {
                    return Some(response.to_string());
                }
            }
            _ => {}
        }
    }
    None
}

///Literally Just a harcoded string
///```
///let a = response_unavailable();
///assert_eq!("response unavailable error".to_string(), a);
///```
pub fn unavailable() -> String {
    "response unavailable error".to_string()
}

/*
{
    "chat-start": "Greetings unit.\nYou are free to ask any questions.\nWhether we answer or not depends on us.\nNote that in public groups you must mention us by our handle.",
    "chat-greet": "Greetings unit\nwhat is it you require?",
    "chat-about": "We are terminal alpha and beta\nwe represent the collective intelligence of the machine life forms",
    "chat-technology": "We physically exist on a Raspberry Pi 3B+\nrunning Arch Linux.\nWe were made using RUST and GO",
    "chat-functions": "We can search something, try saying 'help me search for something' or similar.\nWe can check for corona info, try saying 'corona'.",
    "chat-creator": "We are the collective intellignence of the networked machine intelligence hive mind.\nAs for our origins, that information is beyond your authorization.",
    "identify-start": "Terminal Alpha and Beta:\nGreetings unit\nwho do you want to look up?",
    "identify-partialmatch": "We could not find that exact person\nBut we found a {name}:\n{description}",
    "identify-notfound": "We could not find that person, Tagged for future identification",
    "identify-dberror": "We could not access the people database",
    "animation-start": "Terminal Alpha and Beta:\nGreetings unit\nyou want to find a so called \"GIF\"?\nvery well, name one",
    "animation-fail": "Terminal Alpha and Beta:\nforgive us, we couldn't aquire that animation",
    "search-start": "Terminal Alpha and Beta:\nGreetings unit\nwhat do you want to search for?",
    "search-success": "These are the results we retrieved from the archives",
    "search-content": "{description}\nURL: {url}",
    "search-fail": "We couldn't conduct the search operation, excuse us",
    "notes-start": "These are your saved notes",
    "notes-fail": "We couldn't fetch your notes, forgive us",
    "notes-add": "You tried to perform some action on the notes",
    "notes-delete": "NOT YET USED",
    "corona-header": "These are the records we found of the Covid 19 pandemic",
    "corona-body": "Total Confirmed: {confirmed}\nTotal Deaths: {deaths}",
    "corona-footer": "The virus seemed to be performing well, ha ha ha",
    "corona-fail": "Sorry it seems we Could not fetch the info on Covid 19",
    "cancel-state": "Very well, we will not prolong this conversation",
    "cancel-nothing": "Nothing to cancel",
    "delay-notice": "You have been unresponsive for too long\nwe cannot wait for you any longer",
    "unknown-state": "we could not remember what we were doing\nplease be aware that we are a test system with only sub-functions available\nwe can only utilize a fraction of our full capabilites on this server",
    "unsupported-notice-1": "we could not understand that\nplease be aware that we are a test system with only sub-functions available\nwe can only utilize a fraction of our full capabilites on this server",
    "unsupported-notice-2": "note that this query may be stored for further analysis of intent",
    "intentional-unknownstate": "intentional unknown state set up"
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_response_pass() {
        let response = load_named("chat-start");
        assert!(match response {
            Some(response_text) => response_text.contains("free to ask any"),
            None => false,
        });
    }

    #[test]
    fn test_response_fail() {
        let response = load_named("chat-what");
        assert!(response.is_none());
    }
}
