use super::*;

pub async fn start_info(bot_message: impl BotMessage, json: String) {
    let title_pass = general::title_pass_retriever(&json);
    println!(
        "ACTION_PICKER: info title pass is {}, {}",
        title_pass.0, title_pass.1
    );
    if let Some(info) = general::get_info(title_pass.0, title_pass.1).await {
        bot_message
            .send_message(MsgCount::SingleMsg(Msg::Text(info)))
            .await;
    } else {
        responses::unsupported_notice(bot_message).await;
    }
}
