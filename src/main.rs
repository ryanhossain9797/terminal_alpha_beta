#[macro_use]
extern crate lazy_static;
mod root;
use dotenv::dotenv;

use futures::StreamExt;
use root::handlers::handler;
use std::env;
use telegram_bot::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Answer message with "Hi".
                filter(&api, &message).await?;
            }
        }
    }
    Ok(())
}

async fn filter(api: &Api, message: &Message) -> Result<(), Error> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let myname = api.send(GetMe).await?;
        if let Some(name) = myname.username {
            //-----------------------remove self mention from message
            let handle = "@".to_string() + &name;
            let mut msg = data.replace(&handle, "");
            msg = msg.trim().to_string();
            //-----------------------check if message is from a group chat.......
            if let MessageChat::Group(group) = &message.chat {
                println!("{:?}", group);
                //-----------------------......and check if handle is present if message IS from group chat
                if data.contains(&handle) {
                    handler(&api, &message, msg).await?;
                }
            } else {
                handler(&api, &message, msg).await?;
            }
        }
    }
    Ok(())
}
