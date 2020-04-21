use crate::handlers::*;
use telegram_bot::*;

pub async fn unsupported_notice(chat: MessageChat) -> Result<(), Error> {
    let notice_result = root::API
        .send(chat.text(format!(
            "we could not understand that\
                \nplease be aware that we are a test system with only sub-functions available\
                \nwe can only utilize a fraction of our full capabilites on this server"
        )))
        .await;
    match notice_result {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

pub async fn unknown_state_notice(chat: MessageChat) -> Result<(), Error> {
    let notice_result = root::API
        .send(chat.text(format!(
            "we could not remember what we were doing\
                \nplease be aware that we are a test system with only sub-functions available\
                \nwe can only utilize a fraction of our full capabilites on this server"
        )))
        .await;
    match notice_result {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

pub async fn custom_response(chat: MessageChat, key: String) -> Result<(), Error> {
    let notice_result = if key == "about".to_string() {
        root::API
            .send(chat.text(format!(
                "we are terminal alpha and beta\
                \nwe represent the collective intelligence of the machine life forms"
            )))
            .await
    } else if key == "technology".to_string() {
        root::API
            .send(chat.text(format!(
                "we are running on a raspberry pi 3 b+\
                    \nwe were made using RUST"
            )))
            .await
    } else {
        root::API
            .send(chat.text(format!("we could not understand your question")))
            .await
    };
    match notice_result {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}
