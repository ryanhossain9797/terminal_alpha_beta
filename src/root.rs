pub mod handlers {
    const LONGWAIT: u64 = 30;
    const SHORTWAIT: u64 = 10;

    const WAITTIME: u64 = LONGWAIT;
    use std::collections::HashMap;
    // use std::sync::Mutex;
    use std::time::{Duration, Instant};
    use telegram_bot::*;
    lazy_static! {
        static ref RECORDS: tokio::sync::Mutex<HashMap<UserId, UserStateRecord>> =
            { tokio::sync::Mutex::new(HashMap::new()) };
    }

    struct UserStateRecord {
        username: String,
        state: String,
        last: Instant,
        chat: ChatId,
        history: Vec<String>,
    }
    pub async fn handler(
        api: &Api,
        message: &Message,
        processesed_text: String,
    ) -> Result<(), Error> {
        if processesed_text.starts_with("chat ") {
            let msg = processesed_text.trim_start_matches("chat ").to_string();
            let handler_assignment = chat(api.clone(), message.clone(), msg).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
        Ok(())
    }

    pub async fn chat(api: Api, message: Message, processesed_text: String) -> Result<(), Error> {
        let mut map = RECORDS.lock().await;
        let mut history = "".to_string();
        let entry = map
            .entry(message.from.id)
            .or_insert_with(|| UserStateRecord {
                username: message.from.first_name.clone(),
                chat: message.chat.id(),
                last: Instant::now(),
                state: "chat".to_string(),
                history: Vec::new(),
            });
        entry.history.push(processesed_text.clone());
        entry.last = Instant::now();

        if let Some(record) = map.get(&message.from.id) {
            for val in &record.history {
                history += &("\n'".to_string() + &val + &"'".to_string());
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
        let wipe_launch = wipe_history(message.clone(), api.clone(), "chat".to_string()).await;
        match wipe_launch {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
        Ok(())
    }

    pub async fn wipe_history(message: Message, api: Api, state: String) -> Result<(), Error> {
        tokio::spawn(async move {
            tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
            let mut map = RECORDS.lock().await;
            if let Some(r) = map.get(&message.from.id) {
                if r.last.elapsed() > Duration::from_secs(WAITTIME) && r.state == state {
                    map.remove(&message.from.id);
                    println!("deleted chat record for {}", state);
                    let notice_result = api
                        .send(message.chat.text(format!(
                            "you have been silent for too long\nwe cannot wait for you any longer"
                        )))
                        .await;
                    match notice_result {
                        Err(e) => println!("{:?}", e),
                        _ => (),
                    }
                }
            }
        });
        Ok(())
    }
}
