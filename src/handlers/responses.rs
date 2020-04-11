use telegram_bot::*;

pub async fn unsupported_notice(api: Api, message: Message) -> Result<(), Error> {
    let notice_result = api
        .send(message.chat.text(format!(
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
