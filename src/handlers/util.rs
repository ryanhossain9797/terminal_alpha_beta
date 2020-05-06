use std::fs::OpenOptions;
use std::io::prelude::*;

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
