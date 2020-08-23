use super::*;

use async_std::sync::Mutex;
use async_std::task;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

static RECORDS: Lazy<Mutex<HashMap<String, UserStateRecord>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn initialize_state() {
    Lazy::force(&RECORDS);
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

///Removes current state with a cancellation message.  
///Doesn't care about state.  
///Used with the cancel last command.
pub async fn purge_state(bot_message: Box<dyn BotMessage>) {
    delete_state(&bot_message.get_id()).await;
    bot_message
        .send_message(responses::load("cancel-state").into())
        .await;
}

///Sets the user's state to the provided state
///Removes state after 30 seconds, unless it's updated with a new time    
///or the recorded state doesn't match provided state.  
///Notice Message is provided to user.
pub async fn set_timed_state(bot_message: Arc<Box<dyn BotMessage>>, state: UserState) {
    let source = "SET_TIMED_STATE";
    let info = util::logger::make_info(source);

    //---Insert the intent
    set_state(bot_message.get_id(), state.clone()).await;

    let _ = task::spawn(async move {
        //Wait a specified amount of time before deleting user state
        task::sleep(Duration::from_secs(WAITTIME)).await;
        if let Some(record) = get_state(&bot_message.get_id()).await {
            //If the current state matches pending deletion state
            if format!("{}", record.state) == format!("{}", state) {
                //If the current state is older than threshold wait time
                if record.last.elapsed() > Duration::from_secs(WAITTIME) {
                    delete_state(&bot_message.get_id()).await;
                    info(&format!("deleted state record '{}'", state));
                    bot_message
                        .send_message(responses::load("delay-notice").into())
                        .await;
                //If the current state is not older than threshold wait time
                } else {
                    info("aborted record delete due to recency");
                }
            //If the current state doesn't match pending deletion state
            } else {
                info(&format!(
                    "aborted record delete for '{}' because current state is '{}'",
                    state, record.state
                ));
            }
        //If user has no pending state
        } else {
            info(&format!(
                "aborted record delete for '{}', there is no recorded state for '{}'",
                state,
                bot_message.get_id()
            ))
        }
    });
}

///Immediately cancel's the state IF provided state matches current state.  
///Used to remove state after state action is completed.  
///No notice provided.
pub async fn cancel_matching_state(bot_message: Arc<Box<dyn BotMessage>>, state: UserState) {
    let source = "PURGE_HISTORY";
    let info = util::logger::make_info(source);

    if let Some(r) = get_state(&bot_message.get_id()).await {
        if r.state == state {
            delete_state(&bot_message.get_id()).await;
            info(&format!("deleted state record for {}", state));
        }
    }
}

///Public API of fetching user's state
pub async fn retrieve_state(id: &str) -> Option<UserStateRecord> {
    get_state(id).await
}

///Sets the Provided user's state to the Provided state
async fn set_state(id: String, state: UserState) {
    let mut map = RECORDS.lock().await;
    map.insert(
        id,
        UserStateRecord {
            last: Instant::now(),
            state,
        },
    );
}
///Returns the state of the Provided user
async fn get_state(id: &str) -> Option<UserStateRecord> {
    let map = RECORDS.lock().await;
    match map.get(id) {
        Some(record) => Some(record.clone()),
        None => None,
    }
}
///Remove the Provided user's state
async fn delete_state(id: &str) {
    let mut map = RECORDS.lock().await;
    map.remove(id);
}
