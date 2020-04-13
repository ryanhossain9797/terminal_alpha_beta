use crate::handlers::*;
use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;

//---adds a userstate record with search state to userstate records map
//---fires wipe history command for search state
pub async fn start_search(api: Api, message: Message) -> Result<(), Error> {
    println!("START_SEARCH: search initiated");

    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: "search".to_string(),
            history: Vec::new(),
        });
    drop(map);
    println!("START_SEARCH: record added");
    api.send(message.chat.clone().text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
            \nwhat do you want to search for?",
        &message.from.first_name
    )))
    .await?;
    let wipe_launch = root::wipe_history(message.clone(), api.clone(), "search".to_string()).await;
    match wipe_launch {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

//---finishes search
//---fires immediate purge history command for search state
pub async fn continue_search(
    api: Api,
    message: Message,
    processesed_text: String,
) -> Result<(), Error> {
    let mut search_results = "sorry\nour search functionality is still offline";
    // let results = google("roblox", None);
    // for result in results {
    //     search_results += format!("\n\n{}\n{}\n\n", result.title, result.link);
    // }
    api.send(message.chat.clone().text(format!(
        "Terminal Alpha and Beta:\
            \nhere's your search results \n{}",
        search_results
    )))
    .await?;
    let purge_launch = root::imeediate_purge_history(message.clone(), "search".to_string()).await;
    match purge_launch {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}
