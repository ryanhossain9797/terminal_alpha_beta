use super::*;

pub async fn start_info(bot_message: impl BotMessage, json: String) {
    let title_pass = util_service::title_pass_retriever(&json);
    println!(
        "ACTION_PICKER: info title pass is {}, {}",
        title_pass.0, title_pass.1
    );
    if let Some(info) = info_service::get_info(title_pass.0, title_pass.1).await {
        bot_message.send_message(info.into()).await;
    } else {
        extra::unsupported_notice(bot_message).await;
    }
}
