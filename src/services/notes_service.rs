use super::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, oid, Bson};

pub struct Note {
    pub id: String,
    pub position: usize,
    pub note: String,
}

pub async fn get_notes(user_id: String) -> Option<Vec<Note>> {
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("notes");
        let my_notes_result = notes
            .find(
                doc! {
                    "id": &user_id
                },
                None,
            )
            .await;
        if let Ok(mut my_notes) = my_notes_result {
            let mut notes_list: Vec<Note> = vec![];
            let mut position = 1;
            while let Some(result) = my_notes.next().await {
                if let Ok(document) = result {
                    if let (Some(id), Some(note)) = (
                        document.get("_id").and_then(Bson::as_object_id),
                        document.get("note").and_then(Bson::as_str),
                    ) {
                        notes_list.push(Note {
                            id: id.to_hex(),
                            position,
                            note: note.to_string(),
                        });
                        position += 1;
                    }
                }
            }
            return Some(notes_list);
        }
    }
    None
}

pub async fn add_note(user_id: &str, note: String) -> Option<Vec<Note>> {
    let source = "NOTE_ADD";
    let info = util_service::make_info(source);
    let error = util_service::make_error(source);
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("notes");
        match notes
            .insert_one(doc! {"id":&user_id, "note": &note}, None)
            .await
        {
            Ok(_) => info("successful insertion"),
            Err(err) => {
                error(&format!("{}", err));
            }
        }
    }
    get_notes(user_id.to_string()).await
}

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
            error(&"invalid note id".to_string());
        }
    }
    get_notes(user_id.to_string()).await
}
