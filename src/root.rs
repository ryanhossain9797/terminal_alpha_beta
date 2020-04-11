pub mod handlers {
    const LONGWAIT: u64 = 30;
    const SHORTWAIT: u64 = 10;

    const WAITTIME: u64 = LONGWAIT;
    use std::collections::HashMap;
    use std::mem::drop;
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
        println!("processed text is '{}'", processesed_text);
        let map = RECORDS.lock().await;
        let entry_option = map.get(&message.from.id);
        if let Some(record) = entry_option {
            if record.state == "chat".to_string() {
                drop(map);
                println!("continuing chat");
                let handler_assignment =
                    continue_chat(api.clone(), message.clone(), processesed_text.clone()).await;
                match handler_assignment {
                    Err(e) => println!("{:?}", e),
                    _ => (),
                }
            }
        } else {
            drop(map);
            if processesed_text.starts_with("chat") {
                println!("starting chat");
                let start_chat = start_chat(api.clone(), message.clone()).await;
                match start_chat {
                    Err(e) => println!("{:?}", e),
                    _ => (),
                }
            }
        }
        Ok(())
    }

    pub async fn start_chat(api: Api, message: Message) -> Result<(), Error> {
        println!("START_CHAT: chat initiated");

        let mut map = RECORDS.lock().await;
        map.entry(message.from.id)
            .or_insert_with(|| UserStateRecord {
                username: message.from.first_name.clone(),
                chat: message.chat.id(),
                last: Instant::now(),
                state: "chat".to_string(),
                history: Vec::new(),
            });
        drop(map);
        println!("START_CHAT: record added");
        api.send(message.chat.clone().text(format!(
            "Terminal Alpha and Beta:\nGreetings unit {}\
            \nwe will listen to your following queries",
            &message.from.first_name
        )))
        .await?;
        let wipe_launch = wipe_history(message.clone(), api.clone(), "chat".to_string()).await;
        match wipe_launch {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
        Ok(())
    }

    pub async fn continue_chat(
        api: Api,
        message: Message,
        processesed_text: String,
    ) -> Result<(), Error> {
        let mut history = "".to_string();

        let mut map = RECORDS.lock().await;
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
        drop(map);

        api.send(message.chat.clone().text(format!(
            "Terminal Alpha and Beta:\nGreetings unit {}\
            \nyour messages are{}",
            &message.from.first_name, history
        )))
        .await?;
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
                    drop(map);
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
