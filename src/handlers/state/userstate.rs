// use super::*;
// use crate::clients::{discord::*, telegram::*};
// use futures_delay_queue::{delay_queue, DelayQueue};
// use futures_intrusive::buffer::GrowingHeapBuf;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;

static RECORDS: Lazy<TokioMutex<HashMap<String, UserStateRecord>>> =
    Lazy::new(|| TokioMutex::new(HashMap::new()));

// type Cleaner<T> = Lazy<tokio::sync::Mutex<Option<DelayQueue<T, GrowingHeapBuf<T>>>>>;
// static CLEANER: Cleaner<impl BotMessage> = Lazy::new(|| TokioMutex::new(None));

pub fn initialize_state() {
    Lazy::force(&RECORDS);
    // tokio::spawn(async move { state_cleaner().await });
}

// async fn state_cleaner() {
//     let (clean_request, clean_queue) = delay_queue();

//     let clean_request_opt = Some(clean_request);
//     *CLEANER.lock().await = clean_request_opt;

//     if let Some(cleaner) = &*CLEANER.lock().await {
//         cleaner.insert(1, Duration::from_secs(2));
//         cleaner.insert(5, Duration::from_secs(1));
//         cleaner.insert(4, Duration::from_secs(5));
//     }
//     while let Some(val) = clean_queue.receive().await {
//         tokio::time::delay_for(Duration::from_secs(2)).await;
//         println!("val is {}", val);
//     }
// }

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
