use super::*;
use std::mem::drop;
use std::time::Instant;

//---adds a userstate record with search state to userstate records map
//---fires wipe history command for search state
pub async fn start_search(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_SEARCH: search initiated");

    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| UserStateRecord {
            username: (*m).get_name(),
            last: Instant::now(),
            state: UserState::Search,
        });
    drop(map);
    println!("START_SEARCH: record added for id {}", id);
    wipe_history(m.clone(), UserState::Search);
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("search-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}

//---finishes search
//---fires immediate purge history command for search state
pub async fn continue_search(m: Box<dyn BotMessage + Send + Sync>, processesed_text: String) {
    immediate_purge_history(m.clone(), UserState::Search);
    let search_option = general::google_search(processesed_text);

    let response = match search_option {
        Some(results) => {
            let mut msgs: Vec<Msg> = vec![Msg::Text(
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
                msgs.push(Msg::Text(
                    search_template
                        .replace("{description}", &result.description)
                        .replace("{url}", &result.link),
                ));
            }
            MsgCount::MultiMsg(msgs)
        }
        _ => MsgCount::SingleMsg(Msg::Text(match responses::load_response("search-fail") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        })),
    };
    (*m).send_message(response);
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
