use colored::*;

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
