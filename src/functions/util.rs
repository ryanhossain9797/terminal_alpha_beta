use colored::*;
pub fn log_info(source: &str, msg: &str) {
    println!("{}: {}", source.green(), msg.blue())
}

pub fn log_warning(source: &str, msg: &str) {
    println!("{}: {}", source.green(), msg.yellow())
}

pub fn log_error(source: &str, msg: &str) {
    println!("{}: {}", source.green(), msg.red())
}
