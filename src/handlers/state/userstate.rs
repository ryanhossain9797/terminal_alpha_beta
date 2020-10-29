use super::*;
use async_std::task;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::fmt;
use std::time::Instant;
pub use user_state_model::{UserState, UserStateRecord};

static RECORDS: Lazy<DashMap<String, UserStateRecord>> = Lazy::new(DashMap::new);

pub fn initialize_state() {
    Lazy::force(&RECORDS);
}

mod user_state_model {
    use super::*;
    ///A user state record holds an individual user's state.  
    ///Last holds when it was last updated.
    #[derive(Clone)]
    pub struct UserStateRecord {
        state: UserState,
        last: Instant,
    }

    impl UserStateRecord {
        pub fn new(state: UserState, last: Instant) -> Self {
            Self { state, last }
        }

        pub fn state(&self) -> &UserState {
            &self.state
        }

        pub fn last(&self) -> &Instant {
            &self.last
        }
    }

    #[derive(PartialEq, Eq, Clone)]
    pub enum UserState {
        // Chat,
        Initial,
        Search,
        Identify,
        Animation,
        Notes(Vec<String>),
        Unknown,
    }

    impl fmt::Display for UserState {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                UserState::Initial => write!(f, "Initial"),
                UserState::Search => write!(f, "Search"),
                UserState::Identify => write!(f, "Identify"),
                UserState::Animation => write!(f, "Animation"),
                UserState::Notes(_) => write!(f, "Notes"),
                UserState::Unknown => write!(f, "Unknown"),
            }
        }
    }
}

///Removes current state with a cancellation message.  
///Doesn't care about state.  
///Used with the cancel last command.
pub async fn purge_state(bot_message: Box<dyn BotMessage>) {
    delete_state(bot_message.get_id().as_str()).await;
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
    let info = util::logger::info(source);

    if state == UserState::Initial {
        return;
    }

    //---Insert the intent
    set_state(bot_message.get_id(), state.clone()).await;

    let _ = task::spawn(async move {
        //Wait a specified amount of time before deleting user state
        task::sleep(Duration::from_secs(WAITTIME)).await;
        let record = get_state(bot_message.get_id().as_str()).await;
        //If the current state matches pending deletion state
        if format!("{}", record.state()) == format!("{}", state) {
            //If the current state is older than threshold wait time
            if record.last().elapsed() > Duration::from_secs(WAITTIME) {
                delete_state(bot_message.get_id().as_str()).await;
                info(format!("deleted state record '{}'", state).as_str());
                bot_message
                    .send_message(responses::load("delay-notice").into())
                    .await;

            //If the current state is not older than threshold wait time
            } else {
                info("aborted record delete due to recency");
            }
        //If the current state doesn't match pending deletion state
        } else {
            info(
                format!(
                    "aborted record delete for '{}' because current state is '{}'",
                    state,
                    record.state()
                )
                .as_str(),
            );
        }
    });
}

///Immediately cancel's the state IF provided state matches current state.  
///Used to remove state after state action is completed.  
///No notice provided.
pub async fn cancel_matching_state(bot_message: Arc<Box<dyn BotMessage>>, state: UserState) {
    let source = "PURGE_HISTORY";
    let info = util::logger::info(source);

    if state == *get_state(bot_message.get_id().as_str()).await.state() {
        delete_state(bot_message.get_id().as_str()).await;
        info(format!("deleted state record for {}", state).as_str());
    }
}

///Public API of fetching user's state
pub async fn retrieve_state(id: &str) -> UserStateRecord {
    get_state(id).await
}

///Sets the Provided user's state to the Provided state
async fn set_state(id: String, state: UserState) {
    if state == UserState::Initial {
        return;
    }

    RECORDS.insert(id, UserStateRecord::new(state, Instant::now()));
}
///Returns the state of the Provided user
async fn get_state(id: &str) -> UserStateRecord {
    match RECORDS.get(id) {
        Some(record) => record.value().clone(),
        None => UserStateRecord::new(UserState::Initial, Instant::now()),
    }
}
///Remove the Provided user's state
async fn delete_state(id: &str) {
    RECORDS.remove(id);
}
