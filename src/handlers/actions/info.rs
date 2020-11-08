use super::*;
use serde_json::Value;

pub async fn start(
    bot_message: Box<dyn BotMessage>,
    json: serde_json::Value,
) -> anyhow::Result<()> {
    let source = "START_INFO";
    let info = util::logger::info(source);

    match title_pass_retriever(&json).ok_or_else(|| anyhow::anyhow!("title and pass not found")) {
        Ok((title, pass)) => {
            info(format!("Info title pass is {}, {}", title, pass).as_str());
            match info_service::get_info(title, pass).await {
                Ok(Some(info)) => bot_message.send_message(info.into()).await,
                Ok(None) => extra::unsupported_notice(bot_message).await,
                Err(err) => {
                    bot_message
                        .send_message(responses::load("info-fail").into())
                        .await;
                    return Err(err);
                }
            }
        }
        Err(err) => {
            bot_message
                .send_message(responses::load("info-fail").into())
                .await;
            return Err(err);
        }
    }
    Ok(())
}

///Retrieves the title and pass for the info intent.  
///Parses the intent JSON.
fn title_pass_retriever(json: &serde_json::Value) -> Option<(String, String)> {
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
    None
}
