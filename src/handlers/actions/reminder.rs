use super::*;
use cached::proc_macro::cached;
use serde_json::Value;

pub async fn start(bot_message: Box<dyn BotMessage>, json: String) {
    let source = "START_REMINDER";
    let info = util::logger::info(source);
    let maybe_title_pass = duration_reminder_retriever(json);
    if let Some((reminder, duration)) = maybe_title_pass {
        info(format!("{} {}", reminder, duration).as_str());
        bot_message
            .send_message(format!("reminder: {} for duration: {}", reminder, duration).into())
            .await;
    } else {
        bot_message
            .send_message(responses::load("reminder-fail").into())
            .await
    }
}

///Retrieves the title and pass for the info intent.  
///Parses the intent JSON.
#[cached]
fn duration_reminder_retriever(json_string: String) -> Option<(String, String)> {
    let json_result: Result<Value, _> = serde_json::from_str(json_string.as_str());
    if let Ok(json) = json_result {
        let mut reminder: Option<&str> = None;
        let mut duration: Option<&str> = None;
        let val = &json["slots"];
        if let Value::Array(list) = val {
            for slot in list {
                if let Value::String(entity) = &slot["slotName"] {
                    if let Value::String(value) = &slot["rawValue"] {
                        //If slotName is title
                        if entity == &String::from("reminder") {
                            reminder = Some(&value);
                        //If slotName is pass
                        } else if entity == &String::from("duration") {
                            duration = Some(&value);
                        }
                    }
                }
            }
        }
        if let (Some(title), Some(pass)) = (reminder, duration) {
            return Some((title.to_string(), pass.to_string()));
        }
    }
    None
}
