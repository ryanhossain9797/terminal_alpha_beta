use crate::handlers::*;

use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;
//---adds a userstate record with identify state to userstate records map
//---fires wipe history command for identify state

//---For CGO
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub async fn start_identify(message: Message) -> root::Msg {
    println!("START_IDENTIFY: identify initiated");

    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: root::UserState::Identify,
        });
    drop(map);
    println!("START_IDENTIFY: record added");
    root::wipe_history(message.clone(), root::UserState::Identify);

    root::Msg::Text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
        \nwho do you want to look up?",
        &message.from.first_name
    ))
}

//---finishes identify
//---fires immediate purge history command for identify state
#[allow(unused_variables)]
pub async fn continue_identify(message: Message, processesed_text: String) -> root::Msg {
    root::immediate_purge_history(message.from.clone(), root::UserState::Identify);
    println!("IDENTIFY: beginning identification");
    get_person_go(&processesed_text)
}

//--------------THE FOLLOWING IS USED TO INTERACT WITH THE 'golibs' STUFF
//--------------DEPENDENT ON GOLANG LIBS
//--------------RECOMMENDED MOVE TO SEPARATE CRATE
extern "C" {
    fn GetPerson(name: GoString) -> *const c_char;
}

#[repr(C)]
struct GoString {
    a: *const c_char,
    b: isize,
}

fn get_person_go(name: &str) -> root::Msg {
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
    root::Msg::Text(match string.is_empty() || string.starts_with("Error") {
        true => format!(
            "Terminal Alpha and Beta:\
                    \nWe cannot identify people yet"
        ),
        false => string.to_string(),
    })
}
