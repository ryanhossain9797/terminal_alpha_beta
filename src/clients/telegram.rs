//--------TELEGRAM CODE
use super::*;
use async_std::task;
use async_trait::async_trait;
use futures::StreamExt;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use telegram_bot::Message as TMessage;
use telegram_bot::{
    Api, CanSendDocument, CanSendMessage, GetMe, InputFileRef, MessageChat, MessageKind, UpdateKind,
};

//--- Waiting time for failed connections
const WAITTIME: u64 = 10;

pub static API: OnceCell<Api> = OnceCell::new();

///Main Starting point for the telegram api.
pub async fn main(
    sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>,
) -> anyhow::Result<!> {
    let source = "TELEGRAM_CLIENT";
    let error = util::logger::error(source);
    let token = env::var("TELEGRAM_TOKEN")?;

    #[allow(clippy::map_err_ignore)]
    API.set(Api::new(token))
        .map_err(|_| anyhow::anyhow!("Telegram API already initialized"))?;

    if let Some(api) = API.get() {
        let mut stream = api.stream();
        util::logger::show_status("Telegram is connected!\n");
        //Fetch new updates via long poll method
        while let Some(update_result) = stream.next().await {
            match update_result {
                //If the received update contains a new message...
                Ok(update) => {
                    if let UpdateKind::Message(message) = update.kind {
                        if let MessageKind::Text { ref data, .. } = message.kind {
                            // Print received text message to stdout.
                            util::logger::show_status(
                                format!(
                                    "TELEGRAM: <{}>: {}",
                                    message.from.first_name.as_str(),
                                    data
                                )
                                .as_str(),
                            );

                            if let Some((msg, start_conversation)) = filter(&message).await {
                                let _ = sender
                                    .send_async((
                                        Arc::new(Box::new(TelegramMessage {
                                            message,
                                            start_conversation,
                                        })),
                                        msg,
                                    ))
                                    .await;
                            }
                        }
                    }
                }
                Err(err) => {
                    error(
                        format!(
                            "Hit problems fetching updates, stopping for {} seconds. error is {}",
                            WAITTIME, err
                        )
                        .as_str(),
                    );
                    task::sleep(Duration::from_secs(WAITTIME)).await;
                    error("Resuming");
                }
            }
        }
    }
    Err(anyhow::anyhow!("Telegram failed"))
}

///Filter basically does some spring cleaning.
/// - checks whether the update is actually a message or some other type.
/// - trims leading and trailing spaces ("   /hellow    @machinelifeformbot   world  " becomes "/hellow    @machinelifeformbot   world").
/// - removes / from start if it's there ("/hellow    @machinelifeformbot   world" becomes "hellow    @machinelifeformbot   world").
/// - removes mentions of the bot from the message ("hellow    @machinelifeformbot   world" becomes "hellow      world").
/// - replaces redundant spaces with single spaces using regex ("hellow      world" becomes "hellow world").
async fn filter(message: &TMessage) -> Option<(String, bool)> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let myname_result = API.get()?.send(GetMe).await;
        if let Ok(myname) = myname_result {
            if let Some(name) = myname.username {
                //-----------------------remove self mention from message
                let handle = format!("@{}", name.as_str());
                let mut msg: &str = &data.replace(handle.as_str(), "");
                msg = msg.trim().trim_start_matches('/').trim();
                let msg: &str = &msg.to_lowercase();
                let space_trimmer = Regex::new(r"\s+").unwrap();

                let msg: String = space_trimmer.replace_all(msg, " ").into();
                //-----------------------check if message is from a group chat.......
                if let MessageChat::Group(_) = &message.chat {
                    //-----------------------......and check if handle is present if message IS from group chat
                    if data.contains(handle.as_str()) {
                        //---true means message is to be processed even if no conversation is in progress
                        //---if bot is mentioned new convo can start
                        return Some((msg, true));
                    } else {
                        //---false means message won't start a new conversation
                        //---required because ongoing conversation will continue regardless of true or false
                        return Some((msg, false));
                    }
                } else {
                    //---if not in group chat mentions aren't necessary and any message will be replied by the bot
                    return Some((msg, true));
                };
            }
        }
    }
    None
}

#[derive(Clone)]
struct TelegramMessage {
    message: TMessage,
    start_conversation: bool,
}

#[async_trait]
impl handlers::BotMessage for TelegramMessage {
    fn get_name(&self) -> &str {
        self.message.from.first_name.as_str()
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
        if let Some(api) = API.get() {
            match message {
                handlers::MsgCount::SingleMsg(msg) => match msg {
                    handlers::Msg::Text(text) => {
                        let _ = api.send(self.message.chat.text(text)).await;
                    }
                    handlers::Msg::File(url) => {
                        if api
                            .send(self.message.chat.document(InputFileRef::new(url.clone())))
                            .await
                            .is_err()
                        {
                            let _ = api.send(self.message.chat.text(url)).await;
                        }
                    }
                },
                handlers::MsgCount::MultiMsg(msg_list) => {
                    for msg in msg_list {
                        //---Need send here because spawn would send messages out of order
                        match msg {
                            handlers::Msg::Text(text) => {
                                let _ = api.send(self.message.chat.text(text)).await;
                            }
                            handlers::Msg::File(url) => {
                                if api
                                    .send(
                                        self.message.chat.document(InputFileRef::new(url.clone())),
                                    )
                                    .await
                                    .is_err()
                                {
                                    let _ = api.send(self.message.chat.text(url)).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

//--------TELGRAM CODE END
