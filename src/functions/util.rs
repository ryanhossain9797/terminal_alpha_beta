use colored::*;
///Logs the message with green text
pub fn log_info(source: &str, msg: &str) {
    println!("{}: {}", source.green(), msg.blue())
}
///Logs the message with yellow text
pub fn log_warning(source: &str, msg: &str) {
    println!("{}: {}", source.green(), msg.yellow())
}
///Logs the message with red text
pub fn log_error(source: &str, msg: &str) {
    println!("{}: {}", source.green(), msg.red())
}
