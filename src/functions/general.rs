use crate::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, Bson};
use serde_json::Value;

pub fn log_message(processed_text: String) {
    if let Ok(mut file) = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("action_log.txt")
    {
        if let Ok(_) = file.write((&(format!("{}{}", processed_text, "\n"))).as_bytes()) {
            println!("MESSAGE_LOGGER: successfully logged unknown action")
        } else {
            println!("MESSAGE_LOGGER: failed to log unknown action")
        }
    } else {
        println!("MESSAGE_LOGGER: failed to open file for logging unknown action")
    }
}

pub fn title_pass_retriever(json_string: String) -> (String, String) {
    let json_result: Result<Value, _> = serde_json::from_str(&json_string);
    let mut title: String = String::new();
    let mut pass: String = String::new();
    if let Ok(json) = json_result {
        let val = &json["slots"];
        if let Value::Array(list) = val {
            for slot in list {
                if let Value::String(entity) = &slot["slotName"] {
                    if let Value::String(value) = &slot["rawValue"] {
                        if entity == &String::from("title") {
                            title = (*value).clone();
                        } else if entity == &String::from("pass") {
                            pass = (*value).clone();
                        }
                    }
                }
            }
        }
    }
    (title, pass)
}

//Makes a simple get request to the provided url
//Return an Option<serde_json::Value>
pub async fn get_request_json(url: &str) -> Option<serde_json::Value> {
    let req_result = reqwest::get(url).await;
    match req_result {
        Ok(result) => match result.text().await {
            Ok(body) => {
                println!("Fetched JSON successfully");
                return serde_json::from_str(&body).ok();
            }
            Err(error) => {
                println!("{}", error);
            }
        },
        Err(error) => {
            println!("{}", error);
        }
    }
    return None;
}

pub struct Note {
    pub position: usize,
    pub note: String,
}

pub async fn get_notes(user_id: String) -> Option<Vec<Note>> {
    if let Some(client) = database::get_mongo().await {
        let db = client.database("terminal");
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
                match result {
                    Ok(document) => {
                        if let Some(note) = document.get("note").and_then(Bson::as_str) {
                            notes_list.push(Note {
                                position,
                                note: note.to_string(),
                            });
                            position += 1;
                        }
                    }
                    _ => {}
                }
            }
            return Some(notes_list);
        }
    }
    return None;
}

pub async fn add_note(user_id: String, note: String) -> Option<Vec<Note>> {
    let source = "NOTE_ADD";
    if let Some(client) = database::get_mongo().await {
        let db = client.database("terminal");
        let notes = db.collection("notes");
        match notes
            .insert_one(doc! {"id":&user_id, "note": &note}, None)
            .await
        {
            Ok(insertion) => functions::util::log_info(source, "successful insertion"),
            Err(error) => {
                functions::util::log_error(source, &format!("{}", error));
            }
        }
    }
    return get_notes(user_id).await;
}

#[allow(dead_code, unused_variables)]
pub fn delete_note(user_id: String, number: u32) -> Option<Vec<Note>> {
    None
}
pub struct Person {
    pub name: String,
    pub description: String,
}

pub async fn get_person(name: String) -> Option<Person> {
    println!("GO GETTING PERSON: {}", name);
    if let Some(client) = database::get_mongo().await {
        let db = client.database("terminal");
        let notes = db.collection("people");
        let my_notes_result = notes
            .find(
                doc! {
                    "name": &name
                },
                None,
            )
            .await;
        if let Ok(mut my_notes) = my_notes_result {
            if let Some(result) = my_notes.next().await {
                if let Ok(document) = result {
                    if let Some(description) = document.get("description").and_then(Bson::as_str) {
                        return Some(Person {
                            name,
                            description: description.to_string(),
                        });
                    }
                }
            }
        }
    }

    return None;
}

pub async fn get_people() -> Option<Vec<Person>> {
    if let Some(client) = database::get_mongo().await {
        let db = client.database("terminal");
        let people = db.collection("people");
        let people_result = people.find(None, None).await;
        if let Ok(mut my_notes) = people_result {
            let mut people_list: Vec<Person> = vec![];

            while let Some(result) = my_notes.next().await {
                if let Ok(document) = result {
                    if let (Some(name), Some(description)) = (
                        document.get("name").and_then(Bson::as_str),
                        document.get("description").and_then(Bson::as_str),
                    ) {
                        people_list.push(Person {
                            name: name.to_string(),
                            description: description.to_string(),
                        });
                    }
                }
            }
            return Some(people_list);
        }
    }

    return None;
}
