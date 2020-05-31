//--------DISCORD CODE
use super::*;
use std::env;
use std::time::Duration;

use serenity::{
    async_trait,
    model::{channel::Message as DMessage, gateway::Ready},
    prelude::*,
};

pub async fn run_discord() {
    discord_main().await;
}

async fn discord_main() {
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
        if message.author.name != "Terminal Alpha & Beta" {
            sender(&message, ctx, message.content.clone(), true).await;
        }
    }

    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

//---Sender handles forwarding the message, receiving response and sending it to the user
async fn sender(
    message: &DMessage,
    ctx: Context,
    processed_text: String,
    start_conversation: bool,
) {
    let disc_msg = Box::new(DiscordMessage {
        message: message.clone(),
        ctx,
        start_conversation,
    }) as Box<dyn handlers::BotMessage + Send + Sync>;
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
    fn clone_bot_message(&self) -> Box<dyn handlers::BotMessage + Send + Sync> {
        Box::new(self.clone())
    }
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
            }
            _ => {}
        }
    }
}

//--------DISCORD CODE END
