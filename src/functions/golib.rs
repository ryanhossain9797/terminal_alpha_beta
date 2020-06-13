use super::util;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Bson},
    options::ClientOptions,
    // options::FindOptions,
    Client,
};
use once_cell::sync::OnceCell;
use serde_json::Value;
use std::env;
//---For CGO
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

static MONGO: OnceCell<Client> = OnceCell::new();
static MONGO_INITIALIZED: OnceCell<tokio::sync::Mutex<bool>> = OnceCell::new();

pub async fn get_mongo() -> Option<&'static Client> {
    let source = "MONGO_INIT";
    // this is racy, but that's OK: it's just a fast case
    let client_option = MONGO.get();
    if let Some(_) = client_option {
        util::log_info(source, "Already initialized");
        return client_option;
    }
    // it hasn't been initialized yet, so let's grab the lock & try to
    // initialize it
    let initializing_mutex = MONGO_INITIALIZED.get_or_init(|| tokio::sync::Mutex::new(false));
    // if initialized is true, then someone else initialized it already.
    let mut initialized = initializing_mutex.lock().await;
    if !*initialized {
        util::log_warning(source, "Not yet initialized");
        // no one else has initialized it yet, so
        if let Ok(token) = env::var("MONGO_AUTH") {
            if let Ok(client_options) = ClientOptions::parse(&token).await {
                if let Ok(client) = Client::with_options(client_options) {
                    if let Ok(_) = MONGO.set(client) {
                        *initialized = true;
                    }
                }
            }
        }
    }
    drop(initialized);
    MONGO.get()
}

///THE FOLLOWING IS USED TO INTERACT WITH THE 'golibs' STUFF
///DEPENDENT ON GOLANG LIBS
///RECOMMENDED MOVE TO SEPARATE CRATE
extern "C" {
    fn GetPerson(name: GoString) -> *const c_char;
    fn GetPeople() -> *const c_char;
    fn GetInfo(title: GoString, pass: GoString) -> *const c_char;
    fn GoogleSearch(search: GoString) -> *const c_char;
// fn GetNotes(id: GoString) -> *const c_char;
}

///Representation of GO String in C Format
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
        if let Value::Object(map) = json {
            if let Some(Value::Array(people_values)) = &map.get("people") {
                let mut people: Vec<Person> = vec![];
                for person in people_values {
                    if let (Value::String(name), Value::String(description)) =
                        (&person["name"], &person["description"])
                    {
                        people.push(Person {
                            name: name.clone(),
                            description: description.clone(),
                        });
                    }
                }
                return Some(people);
            }
        }
    }
    return None;
}

pub struct SearchResult {
    pub description: String,
    pub link: String,
}

///WARNING!! unsafe calls made here
///Google searches using GoLang lib
///Returns list of SearchResult structs
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

pub async fn get_notes(user_id: String) -> Option<Vec<Note>> {
    if let Some(client) = get_mongo().await {
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

#[allow(dead_code, unused_variables)]
pub fn add_note(user_id: String, note: String) -> Option<Vec<Note>> {
    None
}

pub fn get_info(title: String, pass: String) -> Option<String> {
    println!("GO GETTING INFO: {}", title);
    let c_title = CString::new(title).expect("CString::new failed");
    let t_ptr = c_title.as_ptr();
    let go_title = GoString {
        a: t_ptr,
        b: c_title.as_bytes().len() as isize,
    };

    let c_pass = CString::new(pass).expect("CString::new failed");
    let p_ptr = c_pass.as_ptr();
    let go_pass = GoString {
        a: p_ptr,
        b: c_pass.as_bytes().len() as isize,
    };
    let result = unsafe { GetInfo(go_title, go_pass) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating info data from library");
    println!("GET_INFO: got stuff from golang libs");
    if let Some(json) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_INFO: valid json");
        if let Value::Object(map) = json {
            if let Some(Value::String(response)) = &map.get("info") {
                return Some(response.to_string().replace("\\n", "\n"));
            }
        }
    }
    return None;
}

#[allow(dead_code)]
fn get_c_string(string: &str) -> Option<GoString> {
    if let Ok(c_string) = CString::new(string) {
        let ptr = c_string.as_ptr();
        Some(GoString {
            a: ptr,
            b: c_string.as_bytes().len() as isize,
        })
    } else {
        None
    }
}
#[allow(dead_code)]
fn get_rust_string(c_pointer: *const c_char) -> Option<String> {
    let c_str = unsafe { CStr::from_ptr(c_pointer) };
    if let Ok(string) = c_str.to_str() {
        Some(string.to_string())
    } else {
        None
    }
}
