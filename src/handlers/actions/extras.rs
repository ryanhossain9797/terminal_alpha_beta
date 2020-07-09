use super::*;

//Tests any unknown intent
pub async fn start_unknown(bot_message: impl BotMessage + 'static) {
    println!("START_UNKNOWN: unknown state initiated");
    let id = bot_message.get_id();
    set_state(id.clone(), UserState::Unknown).await;
    println!("START_UNKNOWN: record added for id {}", id);
    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Unknown);
    arc_message
        .send_message(
            responses::load_named("intentional-unknownstate")
                .unwrap_or_else(responses::unavailable),
        )
        .await;
}
