use colored::*;
///Logs the message with green text
// pub fn log_info(source: &str, msg: &str) {
//     println!("{}: {}", source.green(), msg.blue())
// }
///Logs the message with yellow text
// pub fn log_warning(source: &str, msg: &str) {
//     println!("{}: {}", source.green(), msg.yellow())
// }
///Logs the message with red text
// pub fn log_error(source: &str, msg: &str) {
//     println!("{}: {}", source.green(), msg.red())
// }

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
///Returns a closure that logs the message with purple on white text
pub fn make_status<'a>() -> impl Fn(&str) + 'a {
    move |msg: &str| println!("{}", msg.on_white().purple())
}
