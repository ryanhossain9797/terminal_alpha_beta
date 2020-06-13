//---For CGO
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde_json::Value;

///THE FOLLOWING IS USED TO INTERACT WITH THE 'golibs' STUFF
///DEPENDENT ON GOLANG LIBS
///RECOMMENDED MOVE TO SEPARATE CRATE
extern "C" {
    fn GetInfo(title: GoString, pass: GoString) -> *const c_char;
    fn GoogleSearch(search: GoString) -> *const c_char;
// fn GetNotes(id: GoString) -> *const c_char;
}

///Representation of GO String in C Format
#[repr(C)]
#[derive(Clone, Copy)]
struct GoString {
    a: *const c_char,
    b: isize,
}

pub struct SearchResult {
    pub description: String,
    pub link: String,
}

///WARNING!! unsafe calls made here
///Google searches using GoLang lib
///Returns list of SearchResult structs
pub fn google_search(search: String) -> Option<Vec<SearchResult>> {
    println!("GO GETTING SEARCH RESULTS: {}", search);
    let c_search = CString::new(search).expect("CString::new failed");
    let ptr = c_search.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_search.as_bytes().len() as isize,
    };
    let result = unsafe { GoogleSearch(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str
        .to_str()
        .expect("Error translating search data from library");
    if let Some(json) = serde_json::from_str(&string.to_string()).ok() {
        println!("GET_INFO: search json fetched successfully");
        match json {
            Value::Object(map) => match &map.get("results") {
                Some(Value::Array(results)) => {
                    let mut result_msgs: Vec<SearchResult> = vec![];

                    for result in results {
                        match (result.get("description"), result.get("link")) {
                            (Some(Value::String(description)), Some(Value::String(link))) => {
                                result_msgs.push(SearchResult {
                                    description: description.clone(),
                                    link: link.clone(),
                                });
                            }
                            _ => (),
                        }
                    }
                    return Some(result_msgs);
                }
                _ => (),
            },
            _ => (),
        }
    }
    return None;
}

pub fn get_info(title: String, pass: String) -> Option<String> {
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
        if let Value::Object(map) = json {
            if let Some(Value::String(response)) = &map.get("info") {
                return Some(response.to_string().replace("\\n", "\n"));
            }
        }
    }
    return None;
}

#[allow(dead_code)]
fn get_c_string(string: &str) -> Option<GoString> {
    if let Ok(c_string) = CString::new(string) {
        let ptr = c_string.as_ptr();
        Some(GoString {
            a: ptr,
            b: c_string.as_bytes().len() as isize,
        })
    } else {
        None
    }
}
#[allow(dead_code)]
fn get_rust_string(c_pointer: *const c_char) -> Option<String> {
    let c_str = unsafe { CStr::from_ptr(c_pointer) };
    if let Ok(string) = c_str.to_str() {
        Some(string.to_string())
    } else {
        None
    }
}
