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
pub(crate) async fn discord_main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new instance of the Client, logging in as a bot. This will
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    async fn message(&self, ctx: Context, message: DMessage) {
        println!("DISCORD: <{}>: {}", message.author.name, message.content);
        if !message.author.bot {
            filter(message, ctx).await;
        }
    }

    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

///Filter basically does some spring cleaning.
/// - checks whether the update is actually a message or some other type.
/// - trims leading and trailing spaces ("   /hellow    @machinelifeformbot   world  " becomes "/hellow    @machinelifeformbot   world").
/// - removes / from start if it's there ("/hellow    @machinelifeformbot   world" becomes "hellow    @machinelifeformbot   world").
/// - removes mentions of the bot from the message ("hellow    @machinelifeformbot   world" becomes "hellow      world").
/// - replaces redundant spaces with single spaces using regex ("hellow      world" becomes "hellow world").
async fn filter(message: DMessage, ctx: Context) {
    if let Ok(info) = ctx.http.get_current_application_info().await {
        let id: i64 = info.id.into();
        let handle = format!("<@{}>", &id);
        //-----------------------remove self mention from message
        let mut msg = message.content.replace(&handle, "");
        msg = msg.trim().to_string();
        msg = msg.trim_start_matches('/').to_string();
        msg = msg.trim().to_string();
        msg = msg.to_lowercase();
        let space_trimmer = Regex::new(r"\s+").unwrap();

        let msg_str: &str = &msg[..];
        msg = space_trimmer.replace_all(msg_str, " ").to_string();
        //-----------------------check if message is from a group chat.......
        if !message.is_private() {
            //-----------------------......and check if handle is present if message IS from group chat
            if message.content.contains(&handle) {
                //---true means message is to be processed even if no conversation is in progress
                //---if bot is mentioned new convo can start
                sender(message, ctx, msg, true).await
            } else {
                //---false means message won't start a new conversation
                //---required because ongoing conversation will continue regardless of true or false
                sender(message, ctx, msg, false).await
            }
        } else {
            //---if not in group chat mentions aren't necessary and any message will be replied by the bot
            sender(message, ctx, msg, true).await
        };
    } else {
        println!("DISCORD: Error occured while fetching self ID")
    }
}

///Sender handles forwarding the message.
async fn sender(message: DMessage, ctx: Context, processed_text: String, start_conversation: bool) {
    let disc_msg = DiscordMessage {
        message,
        ctx,
        start_conversation,
    };
    handlers::distributor(disc_msg, processed_text);
}

//---These will be used to generalize telegram messages with other platforms

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
    fn get_name(&self) -> String {
        self.message.author.name.clone()
    }
    fn get_id(&self) -> String {
        let id: i64 = self.message.author.id.into();
        format!("{}", id)
    }
    fn start_conversation(&self) -> bool {
        self.start_conversation
    }
    async fn send_message(&self, msg: handlers::MsgCount) {
        match msg {
            handlers::MsgCount::SingleMsg(msg) => match msg {
                handlers::Msg::Text(text) => {
                    if let Err(why) = &self.message.channel_id.say(&self.ctx.http, text).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
                handlers::Msg::File(url) => {
                    if let Err(why) = &self.message.channel_id.say(&self.ctx.http, url).await {
                        println!("Error sending message: {:?}", why);
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
                                println!("Error sending message: {:?}", why);
                            }
                        }
                        handlers::Msg::File(url) => {
                            if let Err(why) =
                                &self.message.channel_id.say(&self.ctx.http, url).await
                            {
                                println!("Error sending message: {:?}", why);
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
