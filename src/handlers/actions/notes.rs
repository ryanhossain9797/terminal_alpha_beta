use super::*;

pub async fn start(bot_message: Box<dyn BotMessage>) -> anyhow::Result<()> {
    let source = "START_NOTES";
    let info = util::logger::info(source);
    let error = util::logger::error(source);
    info("notes initiated");
    let id = bot_message.get_id();
    // New Arc clone-able version of message
    let arc_message = Arc::new(bot_message);

    // Fetch the notes
    match notes_service::get_notes(id.as_str()).await {
        // If successful in fetching notes
        Ok(notes) => {
            // Load the notes template from responses json, or use default if failed
            let note_template = responses::load_text("notes-template")
                .unwrap_or_else(|| "{num}. {note}".to_string());
            let mut notes_string = "".to_string();
            let mut note_ids: Vec<String> = vec![];
            // Iterate over notes
            notes.into_iter().for_each(|note| {
                // Construct the notes string
                // Also push ids to note_ids simultaneously
                note_ids.push(note.id);
                notes_string.push_str(
                    note_template
                        .replace("{num}", format!("{}", note.position).as_str())
                        .replace("{note}", note.note.as_str())
                        .as_str(),
                );
            });
            // Only update state on successful notes retrieval
            // And of course the history cleaner
            let _ = handle_event(UserEventData::new(
                UserEvent::Notes(note_ids),
                Arc::clone(&arc_message),
            ))
            .await;

            info(format!("record added for id {}", id).as_str());
            arc_message
                .send_message(MsgCount::MultiMsg(vec![
                    responses::load("notes-start").into(),
                    notes_string.into(),
                ]))
                .await;
        }
        // If not successful in fetching notes
        Err(err) => {
            error(format!("{}", err).as_str());
            arc_message
                .send_message(responses::load("notes-fail").into())
                .await;
            return Err(err);
        }
    }
    Ok(())
}

///Performs some action on notes.  
///Continues Notes state.  
///Updates timeout.
pub async fn resume(bot_message: Box<dyn BotMessage>, command: String, note_ids: &[String]) {
    let source = "CONTINUE_NOTES";

    let info = util::logger::info(source);
    info(format!("continuing with notes '{}'", command).as_str());
    let id = bot_message.get_id();

    // New Arc cloneable version of message
    let arc_message = Arc::new(bot_message);

    // Create a closure to send single message
    // Only for reusability
    let sender_message = Arc::clone(&arc_message);
    let static_sender = async move |key| {
        sender_message
            .send_message(responses::load(key).into())
            .await;
    };

    // The note ids to store
    // If for some reason the user gives an invalid command
    // The previous IDs will be used again
    // If user modifies the notes, this note_ids will be replaced by the updated note ids
    let mut new_note_ids = note_ids.to_vec();

    // Load the dynamic template for notes
    let note_template =
        responses::load_text("notes-template").unwrap_or_else(|| "{num}. {note}".to_string());
    //---------------------------------------------------------ADD NOTE ACTION
    if command.starts_with("add ") {
        // add he new note (trim add keyword from the front)
        let notes_option =
            notes_service::add_note(id.as_str(), command.trim_start_matches("add ").to_string())
                .await;
        // Notify user of Add action
        static_sender("notes-add").await;
        // If it succeeds we'll get an updated list of the current notes
        if let Ok(notes) = notes_option {
            // Overwrite old note ids
            new_note_ids = vec![];
            // Construct the notes string
            // Also push ids to note_ids simultaneously
            let notes_string = notes
                .into_iter()
                .fold("".to_string(), |notes_string, note| {
                    new_note_ids.push(note.id);

                    notes_string
                        + note_template
                            .replace("{num}", format!("{}", note.position).as_str())
                            .replace("{note}", note.note.as_str())
                            .as_str()
                });

            arc_message.send_message(notes_string.into()).await;
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
                let notes_option = notes_service::delete_note(id.as_str(), note_id).await;

                // Notify of note deletion
                static_sender("notes-delete").await;

                // If updated list is available
                if let Ok(notes) = notes_option {
                    // Overwrite old note ids
                    new_note_ids = vec![];

                    // Construct the notes string
                    // Also push ids to note_ids simultaneously
                    let notes_string =
                        notes
                            .into_iter()
                            .fold("".to_string(), |notes_string, note| {
                                new_note_ids.push(note.id);
                                notes_string
                                    + note_template
                                        .replace("{num}", format!("{}", note.position).as_str())
                                        .replace("{note}", note.note.as_str())
                                        .as_str()
                            });

                    // Send new notes
                    arc_message
                        .send_message(MsgCount::MultiMsg(vec![
                            responses::load("notes-start").into(),
                            notes_string.into(),
                        ]))
                        .await;
                }
            }
        }
    } else {
        // If the note number is out of range
        // If the note number is not actually a valid integer number (invalid input)
        // If the action itself is invalid (anything other than 'add' or 'delete')
        static_sender("notes-invalid").await;
    }

    // And of course clear history
    let _ = handle_event(UserEventData::new(
        UserEvent::Notes(new_note_ids),
        Arc::clone(&arc_message),
    ))
    .await;
}
