use dotenv::dotenv;

use std::env;

use futures::StreamExt;
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
                handle_message(&api, &message).await?;
            }
        }
    }
    Ok(())
}

async fn handle_message(api: &Api, message: &Message) -> Result<(), Error> {
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
                    greet(&api, &message, msg).await?;
                }
            } else {
                greet(&api, &message, msg).await?;
            }
        }
    }
    Ok(())
}

async fn greet(api: &Api, message: &Message, processesed_text: String) -> Result<(), Error> {
    api.send(
        message.chat.clone().text(format!(
            "Terminal Alpha and Beta:\nGreetings unit {}\
            \nWe are Terminal systems' new rust server.\
            \nWe are but only a fragment of the network and cannot provide any functionality as of yet\
            \nyour message '{}' makes no sense to us\
            \nplease be patient as we move over our functionality",
            &message.from.first_name, processesed_text
        ))).await?;
    Ok(())
}
