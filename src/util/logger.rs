use colored::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

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
    let source = "LOG_MESSAGE";
    let info = make_info(source);
    let error = make_error(source);

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
            info("Successfully logged unknown action");
        } else {
            error("Failed to log unknown action");
        }
    } else {
        //If file opening fails
        error("Failed to open file for logging unknown action");
    }
}
