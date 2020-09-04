use super::*;
use cached::proc_macro::cached;
use serde_json::Value;

pub async fn start(bot_message: Box<dyn BotMessage>, json: String) {
    let source = "START_INFO";
    let info = util::logger::info(source);
    match title_pass_retriever(json) {
        Some((title, pass)) => {
            info(format!("Info title pass is {}, {}", title, pass).as_str());
            match info_service::get_info(title, pass).await {
                Ok(Some(info)) => bot_message.send_message(info.into()).await,
                Ok(None) => extra::unsupported_notice(bot_message).await,
                _ => {
                    bot_message
                        .send_message(responses::load("info-fail").into())
                        .await
                }
            }
        }
        None => {
            bot_message
                .send_message(responses::load("info-fail").into())
                .await
        }
    }
}

///Retrieves the title and pass for the info intent.  
///Parses the intent JSON.
#[cached]
fn title_pass_retriever(json_string: String) -> Option<(String, String)> {
    let json_result: Result<Value, _> = serde_json::from_str(json_string.as_str());
    if let Ok(json) = json_result {
        let mut title: Option<&str> = None;
        let mut pass: Option<&str> = None;
        let val = &json["slots"];
        if let Value::Array(list) = val {
            for slot in list {
                if let Value::String(entity) = &slot["slotName"] {
                    if let Value::String(value) = &slot["rawValue"] {
                        //If slotName is title
                        if entity == &String::from("title") {
                            title = Some(&value);
                        //If slotName is pass
                        } else if entity == &String::from("pass") {
                            pass = Some(&value);
                        }
                    }
                }
            }
        }
        if let (Some(title), Some(pass)) = (title, pass) {
            return Some((title.to_string(), pass.to_string()));
        }
    }
    None
}
