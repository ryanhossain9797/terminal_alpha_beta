use std::process::Command;
use std::str;
static CARGOENV: &str = "cargo:rustc-env=";
fn main() {
    let path = "./lib";
    let lib = "people";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
    let time_c = Command::new("date").args(&["+%Y%m%d"]).output();
    match time_c {
        Ok(t) => {
            let time;
            unsafe {
                time = str::from_utf8_unchecked(&t.stdout);
            }
            println!("{}COMPILED_AT={}", CARGOENV, time);
        }
        Err(_) => {
            println!("{}COMPILED_AT={}", CARGOENV, "Date time fetch failed");
        }
    }
}
