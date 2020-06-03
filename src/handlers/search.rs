use super::*;
use std::mem::drop;
use std::time::Instant;

//---adds a userstate record with search state to userstate records map
//---fires wipe history command for search state
pub async fn start_search(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_SEARCH: search initiated");

    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    map.insert(
        format!("{}", id),
        UserStateRecord {
            last: Instant::now(),
            state: UserState::Search,
        },
    );
    drop(map);
    println!("START_SEARCH: record added for id {}", id);
    wipe_history(m.clone(), UserState::Search);
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("search-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )))
    .await;
}

//---finishes search
//---fires immediate purge history command for search state
pub async fn continue_search(m: Box<dyn BotMessage + Send + Sync>, processesed_text: String) {
    immediate_purge_history(m.clone(), UserState::Search);
    let search_option = golib::google_search(processesed_text);

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
    (*m).send_message(response).await;
}
