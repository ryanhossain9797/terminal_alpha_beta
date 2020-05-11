use std::fs::OpenOptions;
use std::io::prelude::*;

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
                if let Value::String(entity) = &slot["entity"] {
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
