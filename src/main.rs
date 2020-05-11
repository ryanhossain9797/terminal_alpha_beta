#[macro_use]
extern crate lazy_static;
extern crate snips_nlu_lib;
mod handlers;

extern crate openssl_probe;
use dotenv::dotenv;
use futures::StreamExt;
use handlers::root::handler;
use handlers::root::API;
use regex::Regex;
use std::time::Duration;
use telegram_bot::*;

#[tokio::main]
async fn main() {
    let wait: u64 = 10;
    dotenv().ok();

    println!("Starting up Terminal Alpha Beta");

    let mut stream = API.stream();
    // Fetch new updates via long poll method
    while let Some(update_result) = stream.next().await {
        match update_result {
            // If the received update contains a new message...
            Ok(update) => {
                if let UpdateKind::Message(message) = update.kind {
                    if let MessageKind::Text { ref data, .. } = message.kind {
                        // Print received text message to stdout.
                        println!("<{}>: {}", &message.from.first_name, data);
                        // Spawn a handler for the message.
                        tokio::spawn(async move { filter(&message).await });
                    }
                }
            }
            Err(error) => {
                println!("ALPHA BETA MAIN: Hit problems fetching updates, stopping for {} seconds. error is {}", wait, error);
                tokio::time::delay_for(Duration::from_secs(wait)).await;
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
async fn filter(message: &Message) -> Result<(), Error> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let myname = API.send(GetMe).await?;
        if let Some(name) = myname.username {
            //-----------------------remove self mention from message
            let handle = "@".to_string() + &name;
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
                    handler(&message, msg, true).await?;
                } else {
                    //---false means message won't start a new conversation
                    //---required because ongoing conversation will continue regardless of true or false
                    handler(&message, msg, false).await?;
                }
            } else {
                //---if not in group chat mentions aren't necessary and any message will be replied by the bot
                handler(&message, msg, true).await?;
            }
        }
    }
    Ok(())
}
