//--------DISCORD CODE
use super::*;
use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message as DMessage, gateway::Ready},
    prelude::*,
};
use std::env;
use std::time::Duration;

///Main Starting point for the Discord api.
pub(crate) async fn discord_main(sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>) {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new instance of the Client, logging in as a bot. This will
    let mut client = Client::new(&token)
        .event_handler(Handler { sender })
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        util::logger::show_status(&format!("Client error: {:?}", why));
    }
}

struct Handler {
    sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>,
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    async fn message(&self, ctx: Context, message: DMessage) {
        util::logger::show_status(&format!(
            "DISCORD: <{}>: {}",
            message.author.name, message.content
        ));
        if !message.author.bot {
            if let Some((msg, start_conversation)) = filter(&message, &ctx).await {
                self.sender
                    .send((
                        Arc::new(Box::new(DiscordMessage {
                            message,
                            ctx,
                            start_conversation,
                        })),
                        msg,
                    ))
                    .await;
            }
        }
    }

    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, _: Ready) {
        util::logger::show_status("Discord is connected!\n");
    }
}

///Filter basically does some spring cleaning.
/// - checks whether the update is actually a message or some other type.
/// - trims leading and trailing spaces ("   /hellow    @machinelifeformbot   world  " becomes "/hellow    @machinelifeformbot   world").
/// - removes / from start if it's there ("/hellow    @machinelifeformbot   world" becomes "hellow    @machinelifeformbot   world").
/// - removes mentions of the bot from the message ("hellow    @machinelifeformbot   world" becomes "hellow      world").
/// - replaces redundant spaces with single spaces using regex ("hellow      world" becomes "hellow world").
async fn filter(message: &DMessage, ctx: &Context) -> Option<(String, bool)> {
    let source = "DISCORD";
    let error = util::logger::error(source);
    if let Ok(info) = ctx.http.get_current_application_info().await {
        let id: i64 = info.id.into();
        //-----------------------remove self mention from message
        let handle = format!("<@{}>", &id);
        let mut msg: &str = &message.content.replace(&handle, "");
        msg = msg.trim().trim_start_matches('/').trim();
        let msg: &str = &msg.to_lowercase();
        let space_trimmer = Regex::new(r"\s+").unwrap();

        let msg: String = space_trimmer.replace_all(msg, " ").into();
        //-----------------------check if message is from a group chat.......
        if message.is_private() {
            //---if not in group chat mentions aren't necessary and any message will be replied by the bot
            return Some((msg, true));
        } else {
            //-----------------------......and check if handle is present if message IS from group chat
            if message.content.contains(&handle) {
                //---true means message is to be processed even if no conversation is in progress
                //---if bot is mentioned new convo can start
                return Some((msg, true));
            } else {
                //---false means message won't start a new conversation
                //---required because ongoing conversation will continue regardless of true or false
                return Some((msg, false));
            }
        };
    } else {
        error("Problem occurred while fetching self ID");
    }
    None
}

#[derive(Clone)]
struct DiscordMessage {
    message: DMessage,
    ctx: Context,
    start_conversation: bool,
}

#[async_trait]
impl handlers::BotMessage for DiscordMessage {
    // fn dynamic_clone(&self) -> Box<dyn handlers::BotMessage> {
    //     Box::new(self.clone())
    // }
    fn get_name(&self) -> &str {
        &self.message.author.name
    }
    fn get_id(&self) -> String {
        let id: i64 = self.message.author.id.into();
        format!("{}", id)
    }
    fn start_conversation(&self) -> bool {
        self.start_conversation
    }
    fn dyn_clone(&self) -> Box<dyn handlers::BotMessage> {
        Box::new(self.clone())
    }
    async fn send_message(&self, message: handlers::MsgCount) {
        let source = "DISCORD_SEND";
        let error = util::logger::error(source);
        match message {
            handlers::MsgCount::SingleMsg(msg) => match msg {
                handlers::Msg::Text(text) => {
                    if let Err(why) = &self.message.channel_id.say(&self.ctx.http, text).await {
                        error(&format!("Error sending message: {:?}", why));
                    }
                }
                handlers::Msg::File(url) => {
                    if let Err(why) = &self.message.channel_id.say(&self.ctx.http, url).await {
                        error(&format!("Error sending message: {:?}", why));
                    }
                }
            },
            handlers::MsgCount::MultiMsg(msg_list) => {
                for msg in msg_list {
                    //---Need send here because spawn would send messages out of order
                    match msg {
                        handlers::Msg::Text(text) => {
                            if let Err(why) =
                                &self.message.channel_id.say(&self.ctx.http, text).await
                            {
                                error(&format!("Error sending message: {:?}", why));
                            }
                        }
                        handlers::Msg::File(url) => {
                            if let Err(why) =
                                &self.message.channel_id.say(&self.ctx.http, url).await
                            {
                                error(&format!("Error sending message: {:?}", why));
                            }
                        }
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            } // _ => {}
        }
    }
}

//--------DISCORD CODE END
