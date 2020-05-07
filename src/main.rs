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
use telegram_bot::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    openssl_probe::init_ssl_cert_env_vars();
    dotenv().ok();

    println!("Starting up Terminal Alpha Beta");
    // Fetch new updates via long poll method
    let mut stream = API.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);
                // Spawn a handler for the message.
                tokio::spawn(async move { filter(&message).await });
            }
        }
    }
    Ok(())
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
