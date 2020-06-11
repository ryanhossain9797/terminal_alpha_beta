use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

pub fn initialize_state() {
    lazy_static::initialize(&RECORDS);
}

lazy_static! {
    ///Records is a map holding all users state record info
    static ref RECORDS: tokio::sync::Mutex<HashMap<String, UserStateRecord>> =
    tokio::sync::Mutex::new(HashMap::new());
}

///A user state record holds an individual user's state.  
///Last holds when it was last updated.
#[derive(Copy, Clone)]
pub struct UserStateRecord {
    pub state: UserState,
    pub last: Instant,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum UserState {
    // Chat,
    Search,
    Identify,
    Animation,
    Notes,
    Unknown,
}

impl fmt::Display for UserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserState::Search => write!(f, "Search"),
            UserState::Identify => write!(f, "Identify"),
            UserState::Animation => write!(f, "Animation"),
            UserState::Notes => write!(f, "Notes"),
            UserState::Unknown => write!(f, "Unknown"),
        }
    }
}

pub async fn get_state(id: &str) -> Option<UserStateRecord> {
    let map = RECORDS.lock().await;
    match map.get(id) {
        Some(record) => Some(record.clone()),
        None => None,
    }
}
pub async fn set_state(id: String, state: UserState) {
    let mut map = RECORDS.lock().await;
    map.insert(
        id,
        UserStateRecord {
            last: Instant::now(),
            state,
        },
    );
}
pub async fn remove_state(id: &str) {
    let mut map = RECORDS.lock().await;
    map.remove(id);
}
