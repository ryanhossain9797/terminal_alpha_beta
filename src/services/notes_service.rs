use super::*;
use repositories::notes_repository;
use repositories::notes_repository::Note;

///Returns all notes for the user.
pub async fn get_notes(user_id: &str) -> anyhow::Result<Vec<Note>> {
    notes_repository::get_by_user(user_id).await
}

///Adds a new note for the provided note string.  
///Returns an updated all notes for the user including the new one.
pub async fn add_note(user_id: &str, note: String) -> anyhow::Result<Vec<Note>> {
    notes_repository::add(user_id, note).await?;
    notes_repository::get_by_user(user_id).await
}

///Removes the note for the provided user and the provided note id.  
///Returns an updated all notes for the user excluding the deleted one.
pub async fn delete_note(user_id: &str, note_id: &str) -> anyhow::Result<Vec<Note>> {
    notes_repository::delete_note(user_id, note_id).await?;
    notes_repository::get_by_user(user_id).await
}
