//--------TELGRAM CODE
use super::*;
use async_trait::async_trait;
use futures::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use std::time::Duration;
use telegram_bot::Message as TMessage;
use telegram_bot::{Api, CanSendMessage, GetMe, MessageChat, MessageKind, UpdateKind};

//--- Waiting time for failed connections
const WAITTIME: u64 = 10;

pub static API: Lazy<Api> = Lazy::new(|| {
    let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    Api::new(token)
});

///Main Starting point for the telegram api.
pub(crate) async fn telegram_main() {
    let mut stream = API.stream();
    //Fetch new updates via long poll method
    while let Some(update_result) = stream.next().await {
        match update_result {
            //If the received update contains a new message...
            Ok(update) => {
                if let UpdateKind::Message(message) = update.kind {
                    if let MessageKind::Text { ref data, .. } = message.kind {
                        // Print received text message to stdout.
                        util::logger::show_status(&format!(
                            "TELEGRAM: <{}>: {}",
                            &message.from.first_name, data
                        ));
                        // Spawn a handler for the message.

                        filter(message).await;
                    }
                }
            }
            Err(error) => {
                println!("ALPHA BETA MAIN: Hit problems fetching updates, stopping for {} seconds. error is {}", WAITTIME, error);
                tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
                println!("ALPHA BETA MAIN: Resuming")
            }
        }
    }
}

///Filter basically does some spring cleaning.
/// - checks whether the update is actually a message or some other type.
/// - trims leading and trailing spaces ("   /hellow    @machinelifeformbot   world  " becomes "/hellow    @machinelifeformbot   world").
/// - removes / from start if it's there ("/hellow    @machinelifeformbot   world" becomes "hellow    @machinelifeformbot   world").
/// - removes mentions of the bot from the message ("hellow    @machinelifeformbot   world" becomes "hellow      world").
/// - replaces redundant spaces with single spaces using regex ("hellow      world" becomes "hellow world").
async fn filter(message: TMessage) {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let myname_result = API.send(GetMe).await;
        if let Ok(myname) = myname_result {
            if let Some(name) = myname.username {
                //-----------------------remove self mention from message
                let handle = format!("@{}", &name);
                let mut msg: &str = &data.replace(&handle, "");
                msg = msg.trim().trim_start_matches('/').trim();
                let msg: &str = &msg.to_lowercase();
                let space_trimmer = Regex::new(r"\s+").unwrap();

                let msg: String = space_trimmer.replace_all(msg, " ").into();
                //-----------------------check if message is from a group chat.......
                if let MessageChat::Group(_) = &message.chat {
                    //-----------------------......and check if handle is present if message IS from group chat
                    if data.contains(&handle) {
                        //---true means message is to be processed even if no conversation is in progress
                        //---if bot is mentioned new convo can start
                        sender(&message, msg, true).await
                    } else {
                        //---false means message won't start a new conversation
                        //---required because ongoing conversation will continue regardless of true or false
                        sender(&message, msg, false).await
                    }
                } else {
                    //---if not in group chat mentions aren't necessary and any message will be replied by the bot
                    sender(&message, msg, true).await
                };
            }
        }
    }
}

///Sender handles forwarding the message.
async fn sender(message: &TMessage, processed_text: String, start_conversation: bool) {
    //---Create a TelegramMessage object, which implements the BotMessage trait.
    let tele_msg = TelegramMessage {
        message: message.clone(),
        start_conversation,
    };
    handlers::distributor(tele_msg, processed_text);
}

#[derive(Clone)]
struct TelegramMessage {
    message: TMessage,
    start_conversation: bool,
}

#[async_trait]
impl handlers::BotMessage for TelegramMessage {
    // fn dynamic_clone(&self) -> Box<dyn handlers::BotMessage> {
    //     Box::new(self.clone())
    // }
    fn get_name(&self) -> &str {
        &self.message.from.first_name
    }
    fn get_id(&self) -> String {
        let id: i64 = self.message.from.id.into();
        format!("{}", id)
    }
    fn start_conversation(&self) -> bool {
        self.start_conversation
    }
    fn dyn_clone(&self) -> Box<dyn handlers::BotMessage> {
        Box::new(self.clone())
    }
    async fn send_message(&self, message: handlers::MsgCount) {
        match message {
            handlers::MsgCount::SingleMsg(msg) => match msg {
                handlers::Msg::Text(text) => {
                    let _ = API.send(self.message.chat.text(text)).await;
                }
                handlers::Msg::File(url) => {
                    // API.spawn(
                    //     self.message
                    //         .chat
                    //         .photo(InputFileUpload::with_path("files/dp.jpg")),
                    // );
                    let _ = API.send(self.message.chat.text(url)).await;
                }
            },
            handlers::MsgCount::MultiMsg(msg_list) => {
                for msg in msg_list {
                    //---Need send here because spawn would send messages out of order
                    match msg {
                        handlers::Msg::Text(text) => {
                            let _ = API.send(self.message.chat.text(text)).await;
                        }
                        handlers::Msg::File(url) => {
                            // API.spawn(
                            //     self.message
                            //         .chat
                            //         .photo(InputFileUpload::with_path("files/dp.jpg")),
                            // );
                            let _ = API.send(self.message.chat.text(url)).await;
                        }
                    }
                }
            }
        }
    }
}

//--------TELGRAM CODE END
