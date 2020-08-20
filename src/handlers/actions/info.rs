use super::*;
use serde_json::Value;

pub async fn start_info(bot_message: impl BotMessage, json: String) {
    let source = "START_INFO";
    let info = util::logger::make_info(source);
    let title_pass = title_pass_retriever(&json);

    info(&format!(
        "Info title pass is {}, {}",
        title_pass.0, title_pass.1
    ));

    match info_service::get_info(title_pass.0, title_pass.1).await {
        Ok(Some(info)) => bot_message.send_message(info.into()).await,
        Ok(None) => extra::unsupported_notice(bot_message).await,
        _ => {
            bot_message
                .send_message(responses::load("info-fail").into())
                .await
        }
    }
}

///Retrieves the title and pass for the info intent.  
///Parses the intent JSON.
fn title_pass_retriever(json_string: &str) -> (String, String) {
    let json_result: Result<Value, _> = serde_json::from_str(json_string);
    let mut title: String = String::new();
    let mut pass: String = String::new();
    if let Ok(json) = json_result {
        let val = &json["slots"];
        if let Value::Array(list) = val {
            for slot in list {
                if let Value::String(entity) = &slot["slotName"] {
                    if let Value::String(value) = &slot["rawValue"] {
                        //If slotName is title
                        if entity == &String::from("title") {
                            //Then use rawValue as title
                            title = (*value).clone();
                        //If slotName is pass
                        } else if entity == &String::from("pass") {
                            //Then use rawValue as pass
                            pass = (*value).clone();
                        }
                    }
                }
            }
        }
    }
    (title, pass)
}
