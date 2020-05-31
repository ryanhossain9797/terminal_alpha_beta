//--------TELGRAM CODE
extern crate openssl_probe;
use super::*;

use async_trait::async_trait;
use futures::StreamExt;
use regex::Regex;
use std::env;
use std::time::Duration;

use telegram_bot::Message as TMessage;
use telegram_bot::*;

lazy_static! {
    //---General API for telegram
    pub static ref API: Api = {
        let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
        let api = Api::new(token);
        api
    };
}

pub async fn run_telegram() {
    telegram_main().await;
}
async fn telegram_main() {
    let mut stream = API.stream();
    // Fetch new updates via long poll method
    while let Some(update_result) = stream.next().await {
        match update_result {
            // If the received update contains a new message...
            Ok(update) => {
                if let UpdateKind::Message(message) = update.kind {
                    if let MessageKind::Text { ref data, .. } = message.kind {
                        // Print received text message to stdout.
                        println!("TELEGRAM: <{}>: {}", &message.from.first_name, data);
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

//---Filter basically does some spring cleaning
//--- => checks whether the update is actually a message or some other type
//--- => trims leading and trailing spaces ("   /hellow    @machinelifeformbot   world  " becomes "/hellow    @machinelifeformbot   world")
//--- => removes / from start if it's there ("/hellow    @machinelifeformbot   world" becomes "hellow    @machinelifeformbot   world")
//--- => removes mentions of the bot from the message ("hellow    @machinelifeformbot   world" becomes "hellow      world")
//--- => replaces redundant spaces with single spaces using regex ("hellow      world" becomes "hellow world")
async fn filter(message: TMessage) {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let myname_result = API.send(GetMe).await;
        if let Ok(myname) = myname_result {
            if let Some(name) = myname.username {
                //-----------------------remove self mention from message
                let handle = format!("@{}", &name);
                let mut msg = data.replace(&handle, "");
                msg = msg.trim().to_string();
                msg = msg.trim_start_matches("/").to_string();
                msg = msg.trim().to_string();
                msg = msg.to_lowercase();
                let space_trimmer = Regex::new(r"\s+").unwrap();

                let msg_str: &str = &msg[..];
                msg = space_trimmer.replace_all(msg_str, " ").to_string();
                //-----------------------check if message is from a group chat.......
                if let MessageChat::Group(group) = &message.chat {
                    println!("{:?}", group);
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

//---Sender handles forwarding the message, receiving response and sending it to the user
async fn sender(message: &TMessage, processed_text: String, start_conversation: bool) {
    let tele_msg = Box::new(TelegramMessage {
        message: message.clone(),
        start_conversation,
    }) as Box<dyn handlers::BotMessage + Send + Sync>;
    handlers::distributor(tele_msg, processed_text);
}

//---These will be used to generalize telegram messages with other platforms

#[derive(Clone)]
struct TelegramMessage {
    message: TMessage,
    start_conversation: bool,
}

#[async_trait]
impl handlers::BotMessage for TelegramMessage {
    fn clone_bot_message(&self) -> Box<dyn handlers::BotMessage + Send + Sync> {
        Box::new(self.clone())
    }
    fn get_name(&self) -> String {
        self.message.from.first_name.clone()
    }
    fn get_id(&self) -> String {
        let id: i64 = self.message.from.id.into();
        format!("{}", id)
    }
    fn start_conversation(&self) -> bool {
        self.start_conversation
    }
    async fn send_message(&self, msg: handlers::MsgCount) {
        match msg {
            handlers::MsgCount::SingleMsg(msg) => match msg {
                handlers::Msg::Text(text) => {
                    API.spawn(self.message.chat.text(text));
                }
                handlers::Msg::File(url) => {
                    // API.spawn(
                    //     self.message
                    //         .chat
                    //         .photo(InputFileUpload::with_path("files/dp.jpg")),
                    // );
                    API.spawn(self.message.chat.text(url));
                }
            },
            handlers::MsgCount::MultiMsg(msg_list) => {
                for msg in msg_list {
                    //---Need send here because spawn would send messages out of order
                    match msg {
                        handlers::Msg::Text(text) => {
                            API.spawn(self.message.chat.text(text));
                        }
                        handlers::Msg::File(url) => {
                            // API.spawn(
                            //     self.message
                            //         .chat
                            //         .photo(InputFileUpload::with_path("files/dp.jpg")),
                            // );
                            API.spawn(self.message.chat.text(url));
                        }
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
            _ => {}
        }
    }
}
//------------------------------------------------------------------
//--------TELGRAM CODE END