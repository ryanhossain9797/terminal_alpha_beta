use super::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, oid, Bson};

///A single note
/// - `id` is the id in the database
/// - `position` is it's position in the chain
/// - `note` is the actual content
pub struct Note {
    pub id: String,
    pub position: usize,
    pub note: String,
}

impl Note {
    fn new(id: impl Into<String>, position: usize, note: impl Into<String>) -> Self {
        Note {
            id: id.into(),
            position,
            note: note.into(),
        }
    }
}

///Returns all notes for the user.
pub async fn get_notes(user_id: &str) -> Option<Vec<Note>> {
    if let Some(db) = database::get_mongo().await {
        if let Ok(my_notes) = db
            .collection("notes")
            .find(
                //Searching the 'notes' collection with the specific id
                doc! {
                    "id": user_id
                },
                None,
            )
            .await
        {
            //If db search is successful
            return Some(
                //Using fold to convert the cursor into a vector of Note objects
                my_notes
                    .fold((vec![], 1), |(mut notes_list, position), note| async move {
                        if let Ok(document) = note {
                            if let (Some(id), Some(note)) = (
                                document.get("_id").and_then(Bson::as_object_id),
                                document.get("note").and_then(Bson::as_str),
                            ) {
                                notes_list.push(Note::new(id.to_hex(), position, note));
                            }
                        }
                        (notes_list, position + 1)
                    })
                    .await
                    .0, //Only the vector is needed, position not required for result
            );
        }
    }
    None
}

///Adds a new note for the provided note string.  
///Returns an updated all notes for the user including the new one.
pub async fn add_note(user_id: &str, note: String) -> Option<Vec<Note>> {
    let source = "NOTE_ADD";
    let info = util_service::make_info(source);
    let error = util_service::make_error(source);
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("notes");
        match notes
            .insert_one(doc! {"id":user_id, "note": &note}, None)
            .await
        {
            Ok(_) => info("successful insertion"),
            Err(err) => {
                error(&format!("{}", err));
            }
        }
    }
    get_notes(user_id).await
}

///Removes the note for the provided user and the provided note id.  
///Returns an updated all notes for the user excluding the deleted one.
pub async fn delete_note(user_id: &str, note_id: &str) -> Option<Vec<Note>> {
    let source = "NOTE_DELETE";
    let info = util_service::make_info(source);
    let error = util_service::make_error(source);
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("notes");
        if let Ok(object_id) = oid::ObjectId::with_string(note_id) {
            match notes.delete_one(doc! {"_id": object_id}, None).await {
                Ok(_) => info("successful delete"),
                Err(err) => {
                    error(&format!("{}", err));
                }
            }
        } else {
            error("invalid note id");
        }
    }
    get_notes(user_id).await
}
