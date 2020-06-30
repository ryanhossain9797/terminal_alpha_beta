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
    // New Arc cloneable version of message
    let arc_message = Arc::new(bot_message);

    // Fetch the notes
    match general::get_notes(id.clone()).await {
        // If successful in fetching notes
        Some(notes) => {
            // Load the notes template from responses json, or use default if failed
            let note_template =
                responses::load_text("notes-template").unwrap_or("{num}. {note}".to_string());
            let mut notes_string = "".to_string();
            let mut note_ids: Vec<String> = vec![];
            // Iterate over notes
            for note in notes {
                // Construct the notes string
                // Also push ids to note_ids simultaneously
                note_ids.push(note.id);
                notes_string.push_str(
                    &(note_template
                        .replace("{num}", &format!("{}", note.position))
                        .replace("{note}", &note.note.to_string())),
                );
            }

            // Only update state on successful notes retrieval
            set_state(id.clone(), UserState::Notes(note_ids)).await;
            // And of course the history cleaner
            wipe_history(Arc::clone(&arc_message), UserState::Notes(vec![]));
            util::log_info(source, &format!("record added for id {}", id));

            arc_message
                .send_message(MsgCount::MultiMsg(vec![
                    Msg::Text(
                        responses::load_named("notes-start").unwrap_or_else(responses::unavailable),
                    ),
                    Msg::Text(notes_string),
                ]))
                .await;
        }
        // If not successful in fetching notes
        None => {
            arc_message
                .send_message(MsgCount::SingleMsg(Msg::Text(
                    responses::load_named("notes-fail").unwrap_or_else(responses::unavailable),
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
    note_ids: Vec<String>,
) {
    let source = "CONTINUE_NOTES";
    util::log_info(source, &format!("continuing with notes '{}'", command));
    let id = bot_message.get_id();

    // New Arc cloneable version of message
    let arc_message = Arc::new(bot_message);

    // The note ids to store
    // If fire some reason the user gives an invalid command
    // The previous IDs will be used again
    // If user modifies the notes, this note_ids will be replaced by the updated note ids
    let mut new_note_ids = note_ids.clone();

    // Load the dynamic template for notes
    let note_template =
        responses::load_text("notes-template").unwrap_or("{num}. {note}".to_string());
    //---------------------------------------------------------ADD NOTE ACTION
    if command.starts_with("add ") {
        // add he new note (trim add keyword from the front)
        let notes_option =
            general::add_note(&id, command.trim_start_matches("add ").to_string()).await;
        // Notify user of Add action
        arc_message
            .send_message(MsgCount::SingleMsg(Msg::Text(
                responses::load_named("notes-add").unwrap_or_else(responses::unavailable),
            )))
            .await;
        // If it succeeds we'll get an updated list of the current notes
        if let Some(notes) = notes_option {
            let mut notes_string = "".to_string();
            // First get rid of old note_ids as that's outdated info now
            new_note_ids = vec![];
            // Construct the notes string
            // Also push ids to note_ids simultaneously
            for note in notes {
                new_note_ids.push(note.id);
                notes_string.push_str(
                    &(note_template
                        .replace("{num}", &format!("{}", note.position))
                        .replace("{note}", &note.note.to_string())),
                );
            }

            arc_message
                .send_message(MsgCount::SingleMsg(Msg::Text(notes_string)))
                .await;
        }
    //---------------------------------------------------------DELETE NOTE ACTION
    } else if command.starts_with("delete ") {
        // Try to convert the note number to delete to an integer
        if let Ok(number) = command
            .trim_start_matches("delete ")
            .to_string()
            .parse::<usize>()
        {
            // If successfully retrieved note id from previous state's note_ids
            // (may fail if note number is higher than number of notes i.e Index out of bound)
            if let Some(note_id) = note_ids.get(number - 1) {
                // Deleting will return updated list of notes
                let notes_option = general::delete_note(&id, note_id).await;
                // Notify of note deletion
                arc_message
                    .send_message(MsgCount::SingleMsg(Msg::Text(
                        responses::load_named("notes-delete")
                            .unwrap_or_else(responses::unavailable),
                    )))
                    .await;
                // If updated list is avaialable
                if let Some(notes) = notes_option {
                    let mut notes_string = "".to_string();
                    // Overwrite old note ids
                    new_note_ids = vec![];
                    // Construct the notes string
                    // Also push ids to note_ids simultaneously
                    for note in notes {
                        new_note_ids.push(note.id);
                        notes_string.push_str(
                            &(note_template
                                .replace("{num}", &format!("{}", note.position))
                                .replace("{note}", &note.note.to_string())),
                        );
                    }
                    // Send new notes
                    arc_message
                        .send_message(MsgCount::MultiMsg(vec![
                            Msg::Text(
                                responses::load_named("notes-start")
                                    .unwrap_or_else(responses::unavailable),
                            ),
                            Msg::Text(notes_string),
                        ]))
                        .await;
                }
            }
        // If the note number is not actually a valid integer number (invalid input)
        } else {
            arc_message
                .send_message(MsgCount::SingleMsg(Msg::Text(
                    responses::load_named("notes-invalid").unwrap_or_else(responses::unavailable),
                )))
                .await;
        }
    // If the action itself is invalid (anything other than 'add' or 'delete')
    } else {
        arc_message
            .send_message(MsgCount::SingleMsg(Msg::Text(
                responses::load_named("notes-invalid").unwrap_or_else(responses::unavailable),
            )))
            .await;
    }
    // Update the state, if this action was a failure, with same old note ids
    // Else the new note ids
    set_state(id, UserState::Notes(new_note_ids)).await;
    // And of course clean history
    wipe_history(Arc::clone(&arc_message), UserState::Notes(vec![]));
}
