use super::*;

pub async fn start_notes(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_NOTES: notes initiated");
    let id = (*m).get_id();
    match golib::get_notes(id.clone()) {
        Some(notes) => {
            let mut notes_string = "".to_string();
            for note in notes {
                notes_string.push_str(&format!("{}. {}\n", note.position, note.note));
            }
            {
                //---Only update state on successful notes retrieval
                let mut map = RECORDS.lock().await;
                let entry = map
                    .entry(format!("{}", &id))
                    .or_insert_with(|| UserStateRecord {
                        last: Instant::now(),
                        state: UserState::Notes,
                    });
                entry.last = Instant::now();
                drop(map);
                println!("START_NOTES: record added for id {}", id);
                wipe_history(m.clone(), UserState::Notes);
            }
            (*m).send_message(MsgCount::MultiMsg(vec![
                Msg::Text(match responses::load_response("notes-start") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                }),
                Msg::Text(notes_string),
            ]))
            .await;
        }
        None => {
            (*m).send_message(MsgCount::SingleMsg(Msg::Text(
                match responses::load_response("notes-fail") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                },
            )))
            .await;
        }
    }
}

//---finishes identify
//---fires immediate purge history command for identify state
pub async fn continue_notes(m: Box<dyn BotMessage + Send + Sync>, command: String) {
    println!("NOTES: continuing with notes '{}'", command);
    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    let entry = map
        .entry(format!("{}", &id))
        .or_insert_with(|| UserStateRecord {
            last: Instant::now(),
            state: UserState::Notes,
        });
    entry.last = Instant::now();
    drop(map);
    if command.starts_with("add ") {
        let _notes = golib::add_note(id, command.trim_start_matches("add ").to_string());
    }
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("notes-add") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )))
    .await;
    wipe_history(m.clone(), UserState::Notes);
}
