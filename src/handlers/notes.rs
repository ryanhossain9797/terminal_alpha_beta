use super::*;

pub async fn start_notes(bot_message: impl BotMessage + 'static) {
    println!("START_NOTES: notes initiated");
    let id = bot_message.get_id();
    let arc_message = Arc::new(bot_message);
    match golib::get_notes(id.clone()) {
        Some(notes) => {
            let mut notes_string = "".to_string();
            for note in notes {
                notes_string.push_str(&format!("{}. {}\n", note.position, note.note));
            }

            //---Only update state on successful notes retrieval
            set_state(id.clone(), UserState::Notes).await;
            println!("START_NOTES: record added for id {}", id);
            wipe_history(Arc::clone(&arc_message), UserState::Notes);

            arc_message
                .send_message(MsgCount::MultiMsg(vec![
                    Msg::Text(match responses::load_response("notes-start") {
                        Some(response) => response,
                        _ => responses::response_unavailable(),
                    }),
                    Msg::Text(notes_string),
                ]))
                .await;
        }
        None => {
            arc_message
                .send_message(MsgCount::SingleMsg(Msg::Text(
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
pub async fn continue_notes(bot_message: impl BotMessage + 'static, command: String) {
    println!("NOTES: continuing with notes '{}'", command);
    let id = bot_message.get_id();
    set_state(id.clone(), UserState::Notes).await;
    if command.starts_with("add ") {
        let _notes = golib::add_note(id, command.trim_start_matches("add ").to_string());
    }
    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Notes);
    arc_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match responses::load_response("notes-add") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            },
        )))
        .await;
}
