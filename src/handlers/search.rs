use crate::handlers::*;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;

use select::document::Document;
use select::predicate::*;

use std::str;

//---adds a userstate record with search state to userstate records map
//---fires wipe history command for search state
pub async fn start_search(message: Message) -> root::Msg {
    println!("START_SEARCH: search initiated");

    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: root::UserState::Search,
        });
    drop(map);
    println!("START_SEARCH: record added");
    root::wipe_history(message.clone(), root::UserState::Search);

    root::Msg::Text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
        \nwhat do you want to search for?",
        &message.from.first_name
    ))
}

//---finishes search
//---fires immediate purge history command for search state
pub async fn continue_search(message: Message, processesed_text: String) -> root::Msg {
    let mut search_results = "".to_string();
    if let Ok(results) = search_google(&processesed_text, 5).await {
        println!("{}", results.iter().len());
        for result in results {
            search_results += &format!("{}\n", result);
        }
    } else {
        search_results = "search failed".to_string();
    }

    root::immediate_purge_history(message.from.clone(), root::UserState::Search);

    root::Msg::Text(format!(
        "Terminal Alpha and Beta:\
        \nhere's your search results \n{}",
        search_results
    ))
}

//--------------WEB scraper to search through google
pub async fn search_google(query: &str, limit: u32) -> Result<Vec<String>, Error> {
    let request_string = format!(
        "https://www.google.com/search?q={}&gws_rd=ssl&num={}&hl=en",
        query, limit
    );

    let body = Client::new()
        .get(&request_string)
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:34.0) Gecko/20100101 Firefox/34.0",
        )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let document = Document::from(body.as_str());

    let mut sections: Vec<String> = Vec::new();

    for node in document.find(
        Attr("id", "rso")
            .descendant(Class("g"))
            .descendant(Class("rc"))
            .descendant(Class("r"))
            .descendant(Name("a")),
    ) {
        if let Some(link) = node.attr("href") {
            sections.push(link.to_string());
        }
    }
    for section in &sections {
        println!("{}", section);
    }
    Ok(sections)
}
