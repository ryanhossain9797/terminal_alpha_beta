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
        Self {
            id: id.into(),
            position,
            note: note.into(),
        }
    }
}

///Returns all notes for the user.
pub async fn get_by_user(user_id: &str) -> anyhow::Result<Vec<Note>> {
    Ok(
        //Using fold to convert the cursor into a vector of Note objects
        database::mongo::get()
            .await
            .ok_or_else(|| anyhow::anyhow!("Couldn't fetch db connection"))?
            //If db connection is successful
            .collection("notes")
            .find(
                //Searching the 'notes' collection with the specific id
                doc! {
                    "id": user_id
                },
                None,
            )
            .await?
            //If db search is successful
            .fold(
                (vec![], 1),
                |(mut notes_list, position), note_result| async move {
                    if let Ok(document) = note_result {
                        if let (Some(id), Some(note)) = (
                            document.get("_id").and_then(Bson::as_object_id),
                            document.get("note").and_then(Bson::as_str),
                        ) {
                            notes_list.push(Note::new(id.to_hex(), position, note));
                        }
                    }
                    (notes_list, position + 1)
                },
            )
            .await
            .0, //Only the vector is needed, position not required for result
    )
}

///Adds a new note for the provided note string.
pub async fn add(user_id: &str, note: String) -> anyhow::Result<()> {
    let source = "NOTE_ADD";
    let info = util::logger::info(source);
    let error = util::logger::error(source);

    let notes = database::mongo::get()
        .await
        .ok_or_else(|| anyhow::anyhow!("Couldn't fetch db connection"))?
        .collection("notes");
    match notes
        .insert_one(doc! {"id":user_id, "note": &note}, None)
        .await
    {
        Ok(_) => {
            info("successful insertion");
            Ok(())
        }
        Err(err) => {
            error(&format!("{}", err));
            Err(err.into())
        }
    }
}

///Removes the note for the provided user and the provided note id.
pub async fn delete_note(user_id: &str, note_id: &str) -> anyhow::Result<()> {
    let source = "NOTE_DELETE";
    let info = util::logger::info(source);
    let error = util::logger::error(source);

    let notes = database::mongo::get()
        .await
        .ok_or_else(|| anyhow::anyhow!("Couldn't fetch db connection"))?
        .collection("notes");

    match notes
        .delete_one(
            doc! {"_id": oid::ObjectId::with_string(note_id).map_err(|_| anyhow::anyhow!("Invalid note id"))?, "id":user_id},
            None,
        )
        .await
    {
        Ok(_) => {
            info("successful delete");
            Ok(())
        }
        Err(err) => {
            error(&format!("{}", err));
            Err(err.into())
        }
    }
}
