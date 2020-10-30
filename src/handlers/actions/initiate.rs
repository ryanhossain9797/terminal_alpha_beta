use super::*;

pub async fn start_interaction(bot_message: Box<dyn BotMessage>, processed_text: String) {
    if bot_message.start_conversation() {
        match processed_text.as_str() {
            "cancel last" => {
                bot_message
                    .send_message(responses::load("cancel-nothing").into())
                    .await //---cancel last does nothing as there's nothing to cancel
            }
            _ => {
                //---hand over to the natural understanding system for advanced matching
                let _ = natural_understanding(bot_message, processed_text).await;
            }
        }
    }
}

///Uses natural understanding to determine intent if no state is found
async fn natural_understanding(
    bot_message: Box<dyn BotMessage>,
    processed_text: String,
) -> anyhow::Result<()> {
    let source = "NATURAL_ACTION_PICKER";

    let info = util::logger::info(source);
    //---Stuff required to run the NLU engine to get an intent
    if let Ok(Some(intent)) = intent::detect(processed_text.as_str()).await {
        use Intent::{Animation, Chat, Corona, Identify, Info, Notes, Reminder, Search, Unknown};
        match intent {
            Chat => chat::start(bot_message).await,
            Search => search::start(bot_message).await,
            Identify => identify::start(bot_message).await,
            Animation => animation::start(bot_message).await,
            Info { json } => info::start(bot_message, json).await,
            Notes => notes::start(bot_message).await,
            Corona => corona::start(bot_message).await,
            Reminder { json } => reminder::start(bot_message, json).await,
            Unknown => extra::start(bot_message).await,
            _ => {
                //Forward to chat for more intents
                info("forwarding to chat");
                chat::resume(bot_message, intent).await;
            }
        }
    } else {
        util::logger::log_message(processed_text.as_str()).await?;
        extra::unsupported_notice(bot_message).await;
    }
    Ok(())
}
