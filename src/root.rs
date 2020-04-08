pub mod handlers {
    lazy_static! {
        static ref RECORDS: Mutex<HashMap<UserId, Vec<String>>> = {
            let m = HashMap::new();
            Mutex::new(m)
        };
    }
    use std::collections::HashMap;
    use std::sync::Mutex;
    use telegram_bot::*;

    pub async fn handler(
        api: &Api,
        message: &Message,
        processesed_text: String,
    ) -> Result<(), Error> {
        let mut map = RECORDS.lock().unwrap();
        let mut history = "".to_string();
        map.entry(message.from.id)
            .or_insert_with(Vec::new)
            .push(processesed_text.clone());

        if let Some(record) = map.get(&message.from.id) {
            for val in record {
                history += &("\n'".to_string() + val + &"'".to_string());
            }
        }
        api.send(
        message.chat.clone().text(format!(
            "Terminal Alpha and Beta:\nGreetings unit {}\
            \nWe are Terminal systems' new rust server.\
            \nWe are but only a fragment of the network and cannot provide any functionality as of yet\
            \nyour messages{}\nmakes no sense to us\
            \nplease be patient as we move over our functionality",
            &message.from.first_name,history
        ))).await?;
        Ok(())
    }
}
