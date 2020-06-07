use super::*;

//Tests any unknown intent
pub async fn start_unknown(bot_message: impl BotMessage + 'static) {
    println!("START_UNKNOWN: unknown state initiated");

    let mut map = RECORDS.lock().await;
    let id = bot_message.get_id();
    map.insert(
        format!("{}", id),
        UserStateRecord {
            last: Instant::now(),
            state: UserState::Unknown,
        },
    );
    drop(map);
    println!("START_UNKNOWN: record added for id {}", id);
    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Unknown);
    arc_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match responses::load_response("intentional-unknownstate") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            },
        )))
        .await;
}
