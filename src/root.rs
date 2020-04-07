pub mod handlers {
    use telegram_bot::*;
    pub async fn handler(
        api: &Api,
        message: &Message,
        processesed_text: String,
    ) -> Result<(), Error> {
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
}
