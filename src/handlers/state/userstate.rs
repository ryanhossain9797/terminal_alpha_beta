use super::*;

use futures_delay_queue::{delay_queue, DelayQueue};
use futures_intrusive::buffer::GrowingHeapBuf;
use once_cell::sync::{Lazy, OnceCell};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;

static RECORDS: Lazy<TokioMutex<HashMap<String, UserStateRecord>>> =
    Lazy::new(|| TokioMutex::new(HashMap::new()));

type Cleaner<T> = OnceCell<DelayQueue<T, GrowingHeapBuf<T>>>;
static CLEANER: Cleaner<Box<dyn BotMessage>> = OnceCell::new();

pub fn initialize_state() {
    Lazy::force(&RECORDS);
    // tokio::spawn(async { state_cleaner().await });
}

#[allow(dead_code)]
async fn state_cleaner() {
    let local = tokio::task::LocalSet::new();

    let bind = local.run_until(async move {
        let (clean_request, clean_queue) = delay_queue();
        {
            if CLEANER.set(clean_request).is_err() {
                panic!("couldn't start cleaning queue");
            }
        }
        while let Some(message) = clean_queue.receive().await {
            message
                .send_message("hello from the other thread".to_string().into())
                .await;
        }
    });
    bind.await;
}

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
