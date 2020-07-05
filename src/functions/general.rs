use crate::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, oid, Bson};
use serde_json::Value;

// use reqwest::blocking::Client;
// use reqwest::header::USER_AGENT;

// use select::document::Document;
// use select::predicate::*;

///Logs the provided text to the action_log.txt file.  
///Used for when a message is unknown.
pub fn log_message(processed_text: &str) {
    //Open/Create the action_log.txt file with read, append, create options
    if let Ok(mut file) = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("action_log.txt")
    {
        //Attempt to write to file
        if file
            .write((&(format!("{}{}", processed_text, "\n"))).as_bytes())
            .is_ok()
        {
            println!("MESSAGE_LOGGER: successfully logged unknown action")
        } else {
            println!("MESSAGE_LOGGER: failed to log unknown action")
        }
    } else {
        //If file opening fails
        println!("MESSAGE_LOGGER: failed to open file for logging unknown action")
    }
}

///Retrieves the title and pass for the info intent.  
///Parses the intent JSON.
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
                        //If slotName is title
                        if entity == &String::from("title") {
                            //Then use rawValue as title
                            title = (*value).clone();
                        //If slotName is pass
                        } else if entity == &String::from("pass") {
                            //Then use rawValue as pass
                            pass = (*value).clone();
                        }
                    }
                }
            }
        }
    }
    (title, pass)
}

///Makes a simple get request to the provided url.  
///Return an Option<serde_json::Value>
pub async fn get_request_json(url: &str) -> Option<serde_json::Value> {
    let req_result = reqwest::get(url).await;
    match req_result {
        //If Request succesful
        Ok(result) => match result.text().await {
            //If body text is available
            Ok(body) => {
                println!("Fetched JSON successfully");
                return serde_json::from_str(&body).ok();
            }
            //If request body fails
            Err(error) => {
                println!("{}", error);
            }
        },
        //If request fails
        Err(error) => {
            println!("{}", error);
        }
    }
    None
}

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
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("notes");
        match notes
            .insert_one(doc! {"id":&user_id, "note": &note}, None)
            .await
        {
            Ok(_) => functions::util::log_info(source, "successful insertion"),
            Err(error) => {
                functions::util::log_error(source, &format!("{}", error));
            }
        }
    }
    get_notes(user_id.to_string()).await
}

pub async fn delete_note(user_id: &str, note_id: &str) -> Option<Vec<Note>> {
    let source = "NOTE_DELETE";
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("notes");
        if let Ok(object_id) = oid::ObjectId::with_string(note_id) {
            match notes.delete_one(doc! {"_id": object_id}, None).await {
                Ok(_) => functions::util::log_info(source, "successful delete"),
                Err(error) => {
                    functions::util::log_error(source, &format!("{}", error));
                }
            }
        } else {
            functions::util::log_error(source, &"invalid note id".to_string());
        }
    }
    get_notes(user_id.to_string()).await
}
pub struct Person {
    pub name: String,
    pub description: String,
}

pub async fn get_person(name: String) -> Option<Person> {
    println!("GETTING PERSON: {}", name);
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("people");
        let person_result = notes
            .find_one(
                doc! {
                    "name": &name
                },
                None,
            )
            .await;
        if let Ok(person) = person_result {
            if let Some(document) = person {
                if let Some(description) = document.get("description").and_then(Bson::as_str) {
                    return Some(Person {
                        name,
                        description: description.to_string(),
                    });
                }
            }
        }
    }

    None
}

pub async fn get_people() -> Option<Vec<Person>> {
    if let Some(db) = database::get_mongo().await {
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

    None
}

pub async fn get_info(title: String, pass: String) -> Option<String> {
    println!("GETTING INFO: {}", title);
    if let Some(db) = database::get_mongo().await {
        let info = db.collection("info");
        let info_result = info
            .find_one(
                doc! {
                    "title": &title,
                    "pass": &pass,
                },
                None,
            )
            .await;
        if let Ok(info) = info_result {
            if let Some(document) = info {
                if let Some(info) = document.get("info").and_then(Bson::as_str) {
                    return Some(info.to_string().replace("\\n", "\n"));
                }
            }
        }
    }
    None
}

// pub struct SearchResult {
//     pub title: String,
//     pub description: String,
//     pub link: String,
// }

// const AGENT_STRING: &str =
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:34.0) Gecko/20100101 Firefox/34.0";

//Google searches
//Returns list of SearchResult structs
// pub fn google_search(search: String) -> Option<Vec<SearchResult>> {
//     let request_string = format!(
//         "https://www.google.com/search?q={}&gws_rd=ssl&num={}",
//         search, 5
//     );
//     if let Ok(body) = Client::new()
//         .get(request_string.as_str())
//         .header(USER_AGENT, AGENT_STRING)
//         .send()
//     {
//         if let Ok(text) = body.text() {
//             let document = Document::from(text.as_str());
//             let mut results: Vec<SearchResult> = Vec::new();
//             for node in document.find(
//                 Attr("id", "search")
//                     .descendant(Attr("id", "rso"))
//                     .descendant(Class("g"))
//                     .descendant(Class("rc")),
//             ) {
//                 let mut link = String::new();
//                 if let Some(a) = node.find(Class("r").child(Name("a"))).into_iter().next() {
//                     if let Some(l) = a.attr("href") {
//                         link = l.to_string();
//                     }
//                 }
//                 let mut description = String::new();
//                 if let Some(desc) = node
//                     .find(Class("s").descendant(Name("span").and(Class("st"))))
//                     .into_iter()
//                     .next()
//                 {
//                     for child in desc.children() {
//                         let frag = scraper::Html::parse_fragment(&child.html());
//                         for node in frag.tree {
//                             if let scraper::node::Node::Text(text) = node {
//                                 &description.push_str(&(format!("{}", text.text)));
//                             }
//                         }
//                     }
//                 }
//                 for new_node in node.find(Class("LC20lb")) {
//                     results.push(SearchResult {
//                         title: new_node.text(),
//                         link: link.clone(),
//                         description: description.clone(),
//                     });
//                 }
//             }
//             return Some(results);
//         }
//     }
//     return None;
// }
