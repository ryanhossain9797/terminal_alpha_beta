use crate::handlers::responses;
use crate::handlers::root;
use std::fs::OpenOptions;
use std::io::prelude::*;

use std::time::Instant;
use telegram_bot::*;

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

pub async fn start_unknown(message: Message) -> root::MsgCount {
    println!("START_UNKNOWN: unknown state initiated");

    let mut map = root::RECORDS.lock().await;
    let id: i64 = message.from.id.into();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: root::UserState::Unknown,
        });
    drop(map);
    println!("START_UNKNOWN: record added for id {}", id);
    root::wipe_history(message.clone(), root::UserState::Unknown);

    root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("intentional-unknownstate") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    ))
}
