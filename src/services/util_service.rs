use colored::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

use serde_json::Value;

///Returns a closure that logs the message with blue text
pub fn make_info<'a>(source: &'a str) -> impl Fn(&str) + 'a {
    move |msg: &str| println!("{}: {}", source.green(), msg.blue())
}
///Returns a closure that logs the message with yellow text
pub fn make_warning<'a>(source: &'a str) -> impl Fn(&str) + 'a {
    move |msg: &str| println!("{}: {}", source.green(), msg.yellow())
}
///Returns a closure that logs the message with red text
pub fn make_error<'a>(source: &'a str) -> impl Fn(&str) + 'a {
    move |msg: &str| println!("{}: {}", source.green(), msg.red())
}
///Returns a closure that logs the message with white on purple text
pub fn make_status() -> impl Fn(&str) {
    move |msg: &str| show_status(msg)
}
///Logs the message with white on purple text
pub fn show_status(msg: &str) {
    println!("{}", msg.on_white().black());
}
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
pub fn title_pass_retriever(json_string: &str) -> (String, String) {
    let json_result: Result<Value, _> = serde_json::from_str(json_string);
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
