use super::*;

//Tests any unknown intent
pub async fn start_unknown(m: Box<dyn BotMessage>) {
    println!("START_UNKNOWN: unknown state initiated");

    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    map.insert(
        format!("{}", id),
        UserStateRecord {
            last: Instant::now(),
            state: UserState::Unknown,
        },
    );
    drop(map);
    println!("START_UNKNOWN: record added for id {}", id);
    wipe_history(m.clone(), UserState::Unknown);
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("intentional-unknownstate") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )))
    .await;
}
