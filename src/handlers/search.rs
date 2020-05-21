use crate::handlers::util::*;
use crate::handlers::*;
use std::mem::drop;
use std::time::Instant;

//---adds a userstate record with search state to userstate records map
//---fires wipe history command for search state
pub async fn start_search(m: Box<dyn root::BotMessage + Send + Sync>) -> root::MsgCount {
    println!("START_SEARCH: search initiated");

    let mut map = root::RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: (*m).get_name(),
            last: Instant::now(),
            state: root::UserState::Search,
        });
    drop(map);
    println!("START_SEARCH: record added for id {}", id);
    root::wipe_history(m.clone(), root::UserState::Search);
    (*m).send_msg(root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("search-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )))
    .await;
    root::MsgCount::NoMsg
}

//---finishes search
//---fires immediate purge history command for search state
pub async fn continue_search(
    m: Box<dyn root::BotMessage + Send + Sync>,
    processesed_text: String,
) -> root::MsgCount {
    root::immediate_purge_history(m.clone(), root::UserState::Search);
    let search_option = google_search(processesed_text);

    match search_option {
        Some(results) => {
            let mut msgs: Vec<root::Msg> = vec![root::Msg::Text(
                match responses::load_response("search-success") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                },
            )];
            let search_template = match responses::load_response("search-content") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            };
            for result in results {
                msgs.push(root::Msg::Text(
                    search_template
                        .replace("{description}", &result.description)
                        .replace("{url}", &result.link),
                ));
            }
            root::MsgCount::MultiMsg(msgs)
        }
        _ => root::MsgCount::SingleMsg(root::Msg::Text(
            match responses::load_response("search-fail") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            },
        )),
    }
}

//--------------WEB scraper to search through google
// pub async fn search_google(query: &str, limit: u32) -> Result<Vec<String>, Error> {
//     let request_string = format!(
//         "https://www.google.com/search?q={}&gws_rd=ssl&num={}&hl=en",
//         query, limit
//     );

//     let body = Client::new()
//         .get(&request_string)
//         .header(
//             USER_AGENT,
//             "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:34.0) Gecko/20100101 Firefox/34.0",
//         )
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     let document = Document::from(body.as_str());

//     let mut sections: Vec<String> = Vec::new();

//     for node in document.find(
//         Attr("id", "rso")
//             .descendant(Class("g"))
//             .descendant(Class("rc"))
//             .descendant(Class("r"))
//             .descendant(Name("a")),
//     ) {
//         if let Some(link) = node.attr("href") {
//             sections.push(link.to_string());
//         }
//     }
//     for section in &sections {
//         println!("{}", section);
//     }
//     Ok(sections)
// }
