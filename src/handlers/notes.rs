use super::*;

// pub enum NoteState{
//     ActionState,
//     AddState,
//     DeleteState,
// }

pub async fn start_notes(bot_message: impl BotMessage + 'static) {
    let source = "START_NOTES";
    util::log_info(source, "notes initiated");
    let id = bot_message.get_id();
    let arc_message = Arc::new(bot_message);
    match general::get_notes(id.clone()).await {
        Some(notes) => {
            let mut notes_string = "".to_string();
            let mut note_ids: Vec<String> = vec![];
            for note in notes {
                note_ids.push(note.id);
                notes_string.push_str(&format!("{}. {}\n", note.position, note.note));
            }

            //---Only update state on successful notes retrieval
            set_state(id.clone(), UserState::Notes(note_ids)).await;
            util::log_info(source, &format!("record added for id {}", id));
            wipe_history(Arc::clone(&arc_message), UserState::Notes(vec![]));
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

///Performs some action on notes.  
///Continues Notes state.  
///Updates timeout.
pub async fn continue_notes(
    bot_message: impl BotMessage + 'static,
    command: String,
    data: Vec<String>,
) {
    let source = "CONTINUE_NOTES";
    util::log_info(source, &format!("continuing with notes '{}'", command));
    let id = bot_message.get_id();

    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Notes(vec![]));
    if command.starts_with("add ") {
        let notes_option =
            general::add_note(&id, command.trim_start_matches("add ").to_string()).await;
        arc_message
            .send_message(MsgCount::SingleMsg(Msg::Text(
                match responses::load_response("notes-add") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                },
            )))
            .await;
        if let Some(notes) = notes_option {
            let mut notes_string = "".to_string();
            let mut note_ids: Vec<String> = vec![];
            for note in notes {
                note_ids.push(note.id);
                notes_string.push_str(&format!("{}. {}\n", note.position, note.note));
            }
            //---Only update state on successful notes retrieval
            arc_message
                .send_message(MsgCount::MultiMsg(vec![
                    Msg::Text(match responses::load_response("notes-start") {
                        Some(response) => response,
                        _ => responses::response_unavailable(),
                    }),
                    Msg::Text(notes_string),
                ]))
                .await;
            set_state(id.clone(), UserState::Notes(note_ids)).await;
            return;
        }
    } else if command.starts_with("delete ") {
        if let Ok(number) = command
            .trim_start_matches("delete ")
            .to_string()
            .parse::<usize>()
        {
            if let Some(note_id) = data.get(number - 1) {
                let notes_option = general::delete_note(&id, note_id).await;
                arc_message
                    .send_message(MsgCount::SingleMsg(Msg::Text(
                        match responses::load_response("notes-delete") {
                            Some(response) => response,
                            _ => responses::response_unavailable(),
                        },
                    )))
                    .await;
                if let Some(notes) = notes_option {
                    let mut notes_string = "".to_string();
                    let mut note_ids: Vec<String> = vec![];
                    for note in notes {
                        note_ids.push(note.id);
                        notes_string.push_str(&format!("{}. {}\n", note.position, note.note));
                    }
                    //---Only update state on successful notes retrieval
                    arc_message
                        .send_message(MsgCount::MultiMsg(vec![
                            Msg::Text(match responses::load_response("notes-start") {
                                Some(response) => response,
                                _ => responses::response_unavailable(),
                            }),
                            Msg::Text(notes_string),
                        ]))
                        .await;
                    set_state(id.clone(), UserState::Notes(note_ids)).await;
                    return;
                }
            }
        } else {
            arc_message
                .send_message(MsgCount::SingleMsg(Msg::Text(
                    match responses::load_response("notes-invalid") {
                        Some(response) => response,
                        _ => responses::response_unavailable(),
                    },
                )))
                .await;
        }
    } else {
        arc_message
            .send_message(MsgCount::SingleMsg(Msg::Text(
                match responses::load_response("notes-invalid") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                },
            )))
            .await;
    }
    set_state(id.clone(), UserState::Notes(data.clone())).await;
}
