use crate::handlers::*;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde_json::Value;

pub async fn start_info(m: Box<dyn root::BotMessage + Send + Sync>, json: String) {
    //println!("ACTION_PICKER: intent json is {}", json);
    let title_pass = util::title_pass_retriever(json);
    println!(
        "ACTION_PICKER: info title pass is {}, {}",
        title_pass.0, title_pass.1
    );
    if let Some(info) = get_info_go(title_pass.0, title_pass.1) {
        (*m).send_message(root::MsgCount::SingleMsg(root::Msg::Text(info)));
    } else {
        responses::unsupported_notice(m);
    }
}

//--------------THE FOLLOWING IS USED TO INTERACT WITH THE 'golibs' STUFF
//--------------DEPENDENT ON GOLANG LIBS
//--------------RECOMMENDED MOVE TO SEPARATE CRATE
extern "C" {
    fn GetInfo(title: GoString, pass: GoString) -> *const c_char;
}

#[repr(C)]
struct GoString {
    a: *const c_char,
    b: isize,
}

pub fn get_info_go(title: String, pass: String) -> Option<String> {
    println!("GO GETTING INFO: {}", title);
    let c_title = CString::new(title).expect("CString::new failed");
    let t_ptr = c_title.as_ptr();
    let go_title = GoString {
        a: t_ptr,
        b: c_title.as_bytes().len() as isize,
    };

    let c_pass = CString::new(pass).expect("CString::new failed");
    let p_ptr = c_pass.as_ptr();
    let go_pass = GoString {
        a: p_ptr,
        b: c_pass.as_bytes().len() as isize,
    };
    let result = unsafe { GetInfo(go_title, go_pass) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating info data from library");
    println!("GET_INFO: got stuff from golang libs");
    if let Some(json) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_INFO: valid json");
        match json {
            Value::Object(map) => match &map.get("info") {
                Some(Value::String(response)) => Some(response.to_string().replace("\\n", "\n")),
                _ => None,
            },
            // Value::String(response) =>
            _ => None,
        }
    } else {
        None
    }
}
