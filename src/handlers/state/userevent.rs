use std::fmt;

mod user_state_model {
    use super::*;
    #[derive(PartialEq, Eq, Clone)]
    pub enum UserEvent {
        // Chat,
        Initial,
        Search,
        Identify,
        Animation,
        Notes(Vec<String>),
        Unknown,
    }

    impl fmt::Display for UserEvent {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                UserEvent::Initial => write!(f, "Initial"),
                UserEvent::Search => write!(f, "Search"),
                UserEvent::Identify => write!(f, "Identify"),
                UserEvent::Animation => write!(f, "Animation"),
                UserEvent::Notes(_) => write!(f, "Notes"),
                UserEvent::Unknown => write!(f, "Unknown"),
            }
        }
    }
}
