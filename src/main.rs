#[macro_use]
extern crate lazy_static;
extern crate snips_nlu_lib;
// extern crate discord;
mod functions;
mod handlers;

extern crate openssl_probe;
use dotenv::dotenv;
use futures::StreamExt;
use regex::Regex;
use std::env;
use std::time::Duration;
use tokio::prelude::*;

// use discord::model::Event;
// use discord::Discord;

use telegram_bot::Message as TMessage;
use telegram_bot::*;

use serenity::{
    async_trait,
    model::{channel::Message as DMessage, gateway::Ready},
    prelude::*,
};

const WAITTIME: u64 = 10;

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Starting up Terminal Alpha Beta, compiled at");
    println!("-----Starting TELEGRAM and DISCORD-----\n");
    //---Prints the Date of compilation, added at compile time
    if let Some(date) = option_env!("COMPILED_AT") {
        println!("Compile date {}", date);
    }
    println!("Initializing everything");
    lazy_static::initialize(&API);
    handlers::initialize();
    println!("\nInitialized Everything\n");
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let tasks = vec![
                tokio::task::spawn_local(async move {
                    run_telegram().await;
                }),
                tokio::task::spawn_local(async move {
                    run_discord().await;
                }),
            ];
            futures::future::join_all(tasks).await;
        })
        .await;
}

//--------DISCORD CODE

async fn run_discord() {
    discord_main().await;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    async fn message(&self, ctx: Context, msg: DMessage) {
        println!("DISCORD: <{}>: {}", msg.author.name, msg.content);
        if msg.author.name != "Terminal Alpha & Beta" {
            if let Err(why) = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Hey {}, You said {}", msg.author.name, msg.content),
                )
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
#[allow(dead_code)]
async fn discord_main() {
    //     let discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token"))
    //     .expect("login failed");
    //     // Establish and use a websocket connection
    //     let (mut connection, _) = discord.connect().expect("connect failed");
    //     println!("Ready.");
    //     loop {
    //         match connection.recv_event() {
    //         Ok(Event::MessageCreate(message)) => {
    //             println!("{} says: {}", message.author.name, message.content);
    //             if message.content == "!test" {
    //                 let _ = discord.send_message(
    //                     message.channel_id,
    //                     "This is a reply to the test.",
    //                     "",
    //                     false,
    //                 );
    //             } else if message.content == "!quit" {
    //                 println!("Quitting.");
    //                 break;
    //             }
    //         }
    //         Ok(_) => {}
    //         Err(discord::Error::Closed(code, body)) => {
    //             println!("Gateway closed on us with code {:?}: {}", code, body);
    //             break;
    //         }
    //         Err(err) => println!("Receive error: {:?}", err),
    //     }
    // }

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new instance of the Client, logging in as a bot. This will
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

//--------DISCORD CODE END

//--------TELGRAM CODE
lazy_static! {
    //---Global API access
    pub static ref API: Api = {
        let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
        let api = Api::new(token);
        api
    };
}

async fn run_telegram() {
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
        start_conversation: start_conversation,
    }) as Box<dyn handlers::BotMessage + Send + Sync>;
    handlers::distributor(tele_msg, processed_text);
}

//---These will be used to generalize telegram messages with other platforms

#[derive(Clone)]
struct TelegramMessage {
    message: TMessage,
    start_conversation: bool,
}

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
    fn send_message(&self, msg: handlers::MsgCount) {
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
#[allow(dead_code)]
async fn download_file(url: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut response = reqwest::get(&url).await?;
    let mut file = tokio::fs::File::open("temp/file.gif").await?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok("temp/file.gif".to_string())
}
