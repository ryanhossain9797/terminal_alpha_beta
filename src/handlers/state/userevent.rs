use super::*;
use std::fmt;

///A user state record holds an individual user's state.  
///Last holds when it was last updated.
#[derive(Clone)]
pub struct UserEventData {
    event: UserEvent,
    message: Arc<Box<dyn BotMessage>>,
}

impl UserEventData {
    pub fn new(event: UserEvent, message: Arc<Box<dyn BotMessage>>) -> Self {
        Self { event, message }
    }

    pub fn event_and_message(self) -> (UserEvent, Arc<Box<dyn BotMessage>>) {
        (self.event, self.message)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum UserEvent {
    Search,
    Identify,
    Animation,
    Notes(Vec<String>),
    Unknown,
    SearchCompleted,
    IdentifyCompleted,
    AnimationCompleted,
    // NotesCompleted,
    // UnknownCompleted,
    Reset,
}

impl fmt::Display for UserEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserEvent::Search => write!(f, "Search"),
            UserEvent::Identify => write!(f, "Identify"),
            UserEvent::Animation => write!(f, "Animation"),
            UserEvent::Notes(_) => write!(f, "Notes"),
            UserEvent::Unknown => write!(f, "Unknown"),
            UserEvent::SearchCompleted => write!(f, "SearchCompleted"),
            UserEvent::IdentifyCompleted => write!(f, "IdentifyCompleted"),
            UserEvent::AnimationCompleted => write!(f, "AnimationCompleted"),
            // UserEvent::NotesCompleted => write!(f, "NotesCompleted"),
            // UserEvent::UnknownCompleted => write!(f, "UnknownCompleted"),
            UserEvent::Reset => write!(f, "Reset"),
        }
    }
}

pub async fn handle_event(event_data: UserEventData) -> anyhow::Result<()> {
    let (event, message) = event_data.event_and_message();
    match (
        retrieve_state(message.get_id().as_str()).await.state(),
        event,
    ) {
        (UserState::Initial, UserEvent::Search) => {
            userstate::set_timed_state(message, UserState::Search).await
        }
        (UserState::Initial, UserEvent::Identify) => {
            userstate::set_timed_state(message, UserState::Identify).await
        }
        (UserState::Initial, UserEvent::Animation) => {
            userstate::set_timed_state(message, UserState::Animation).await
        }
        (UserState::Initial, UserEvent::Notes(notes)) => {
            userstate::set_timed_state(message, UserState::Notes(notes)).await
        }
        (UserState::Notes(_), UserEvent::Notes(notes)) => {
            userstate::set_timed_state(message, UserState::Notes(notes)).await
        }
        (UserState::Initial, UserEvent::Unknown) => {
            userstate::set_timed_state(message, UserState::Unknown).await
        }
        (UserState::Search, UserEvent::SearchCompleted) => {
            userstate::delete_state(message).await;
        }
        (UserState::Identify, UserEvent::IdentifyCompleted) => {
            userstate::delete_state(message).await;
        }
        (UserState::Animation, UserEvent::AnimationCompleted) => {
            userstate::delete_state(message).await;
        }
        // (UserState::Notes(_), UserEvent::NotesCompleted) => {
        //     userstate::delete_state(message).await;
        // }
        // (UserState::Unknown, UserEvent::UnknownCompleted) => {
        //     userstate::delete_state(message).await;
        // }
        (_, UserEvent::Reset) => userstate::purge_state(message).await,
        _ => return Err(anyhow::anyhow!("Event not valid for this state")),
    }
    Ok(())
}
