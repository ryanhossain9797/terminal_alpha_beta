pub mod handlers {
    use futures::executor::block_on;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::thread;
    use std::time::{Duration, Instant};
    use telegram_bot::*;
    lazy_static! {
        static ref RECORDS: Mutex<HashMap<UserId, UserStateRecord>> = {
            let m = HashMap::new();
            Mutex::new(m)
        };
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
        let mut map = RECORDS.lock().unwrap();
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
        // .history
        // .push(processesed_text.clone());
        // .last = Instant::now();

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
        let msg = message.clone();
        let copy_api = api.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            let mut map = RECORDS.lock().unwrap();
            if let Some(r) = map.get(&msg.from.id) {
                if r.last.elapsed() > Duration::from_secs(10) {
                    map.remove(&msg.from.id);
                    println!("deleted chat record");
                    //let res = block_on(copy_api.send(msg.chat.text(format!("bye"))));
                }
            }
        });
        Ok(())
    }
}
