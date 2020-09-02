use super::*;

pub async fn start(bot_message: Box<dyn BotMessage>, json: String) {
    let source = "START_REMINDER";
    let info = util::logger::info(source);
    info(json.as_str());
    bot_message
        .send_message("Starting reminder".to_string().into())
        .await;
}
