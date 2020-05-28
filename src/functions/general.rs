use std::fs::OpenOptions;
use std::io::prelude::*;

use serde_json::Value;

//---For CGO
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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

//--------------THE FOLLOWING IS USED TO INTERACT WITH THE 'golibs' STUFF
//--------------DEPENDENT ON GOLANG LIBS
//--------------RECOMMENDED MOVE TO SEPARATE CRATE
extern "C" {
    fn GetPerson(name: GoString) -> *const c_char;
    fn GetPeople() -> *const c_char;
    fn GoogleSearch(search: GoString) -> *const c_char;
    fn GetNotes(id: GoString) -> *const c_char;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GoString {
    a: *const c_char,
    b: isize,
}

pub struct Person {
    pub name: String,
    pub description: String,
}

pub fn get_person(name: String) -> Option<Person> {
    println!("GO GETTING PERSON: {}", name);
    let c_name = CString::new(name).expect("CString::new failed");
    let ptr = c_name.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_name.as_bytes().len() as isize,
    };
    let result = unsafe { GetPerson(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating person data from library");
    if let Some(json) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_INFO: person json is {} ", json);
        match json {
            Value::Object(map) => match &map.get("person") {
                Some(Value::Object(map)) => match (&map.get("name"), &map.get("description")) {
                    (Some(Value::String(name)), Some(Value::String(description))) => Some(Person {
                        name: name.clone(),
                        description: description.clone(),
                    }),
                    _ => None,
                },
                _ => None,
            },
            // Value::String(response) =>
            _ => None,
        }
    } else {
        None
    }
}

pub fn get_people() -> Option<Vec<Person>> {
    let result = unsafe { GetPeople() };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating people data from library");
    if let Some(json) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_INFO: people json is valid");
        match json {
            Value::Object(map) => match &map.get("people") {
                Some(Value::Array(people_values)) => {
                    let mut people: Vec<Person> = vec![];
                    for person in people_values {
                        match (&person["name"], &person["description"]) {
                            (Value::String(name), Value::String(description)) => {
                                people.push(Person {
                                    name: name.clone(),
                                    description: description.clone(),
                                });
                            }
                            _ => (),
                        }
                    }
                    Some(people)
                }
                _ => None,
            },
            // Value::String(response) =>
            _ => None,
        }
    } else {
        None
    }
}

pub struct SearchResult {
    pub description: String,
    pub link: String,
}

//WARNING!! unsafe calls made here
//Google searches using GoLang lib
//Returns list of SearchResult structs
pub fn google_search(search: String) -> Option<Vec<SearchResult>> {
    println!("GO GETTING SEARCH RESULTS: {}", search);
    let c_search = CString::new(search).expect("CString::new failed");
    let ptr = c_search.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_search.as_bytes().len() as isize,
    };
    let result = unsafe { GoogleSearch(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating search data from library");
    if let Some(json) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_INFO: search json fetched successfully");
        match json {
            Value::Object(map) => match &map.get("results") {
                Some(Value::Array(results)) => {
                    let mut result_msgs: Vec<SearchResult> = vec![];

                    for result in results {
                        match (result.get("description"), result.get("link")) {
                            (Some(Value::String(description)), Some(Value::String(link))) => {
                                result_msgs.push(SearchResult {
                                    description: description.clone(),
                                    link: link.clone(),
                                });
                            }
                            _ => (),
                        }
                    }
                    return Some(result_msgs);
                }
                _ => (),
            },
            _ => (),
        }
    }
    return None;
}

pub struct Note {
    pub position: usize,
    pub note: String,
}

//WARNING!! unsafe calls made here
//Fetches notes using GoLang lib
//Returns list of Note structs
pub fn get_notes(user_id: String) -> Option<Vec<Note>> {
    println!("GO GETTING NOTES: id: {}", user_id);
    let c_note = CString::new(user_id).expect("CString::new failed");
    let ptr = c_note.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_note.as_bytes().len() as isize,
    };
    let result = unsafe { GetNotes(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating notes data from library");
    if let Some(Value::Array(notes)) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_NOTES: notes json fetched successfully {}", string);
        let mut notes_list: Vec<Note> = vec![];
        for (position, note_val) in notes.iter().enumerate() {
            if let Some(Value::String(note_str)) = &note_val.get("note") {
                let note = note_str.clone();
                notes_list.push(Note { position, note });
            }
        }
        return Some(notes_list);
    }
    return None;
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
