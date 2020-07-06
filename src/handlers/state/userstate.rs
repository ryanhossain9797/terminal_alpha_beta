use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

pub fn initialize_state() {
    Lazy::force(&RECORDS);
}

static RECORDS: Lazy<tokio::sync::Mutex<HashMap<String, UserStateRecord>>> =
    Lazy::new(|| tokio::sync::Mutex::new(HashMap::new()));

///A user state record holds an individual user's state.  
///Last holds when it was last updated.
#[derive(Clone)]
pub struct UserStateRecord {
    pub state: UserState,
    pub last: Instant,
}

#[derive(PartialEq, Eq, Clone)]
pub enum UserState {
    // Chat,
    Search,
    Identify,
    Animation,
    Notes(Vec<String>),
    Unknown,
}

impl fmt::Display for UserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserState::Search => write!(f, "Search"),
            UserState::Identify => write!(f, "Identify"),
            UserState::Animation => write!(f, "Animation"),
            UserState::Notes(_) => write!(f, "Notes"),
            UserState::Unknown => write!(f, "Unknown"),
        }
    }
}

///Returns the state of the Provided user
pub async fn get_state(id: &str) -> Option<UserStateRecord> {
    let map = RECORDS.lock().await;
    match map.get(id) {
        Some(record) => Some(record.clone()),
        None => None,
    }
}

///Sets the Provided user's state to the Provided state
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
///Remove the Provided user's state
pub async fn remove_state(id: &str) {
    let mut map = RECORDS.lock().await;
    map.remove(id);
}
