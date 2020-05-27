use super::*;

pub async fn start_notes(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_NOTES: notes initiated");
    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| UserStateRecord {
            last: Instant::now(),
            state: UserState::Notes,
        });
    drop(map);
    println!("START_NOTES: record added for id {}", id);
    wipe_history(m.clone(), UserState::Notes);
    (*m).send_message(MsgCount::MultiMsg(vec![
        Msg::Text(match responses::load_response("notes-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        }),
        Msg::Text("1. Note 1\n2. Note 2\n3. Note 3".to_string()),
    ]));
}

//---finishes identify
//---fires immediate purge history command for identify state
pub async fn continue_notes(m: Box<dyn BotMessage + Send + Sync>, command: String) {
    println!("NOTES: continuing with notes '{}'", command);
    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    let entry = map
        .entry(format!("{}", id))
        .or_insert_with(|| UserStateRecord {
            last: Instant::now(),
            state: UserState::Notes,
        });
    entry.last = Instant::now();
    drop(map);
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("notes-add") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
    wipe_history(m.clone(), UserState::Notes);
}
