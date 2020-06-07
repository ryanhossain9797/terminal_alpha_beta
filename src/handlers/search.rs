use super::*;
use std::mem::drop;
use std::time::Instant;

//---adds a userstate record with search state to userstate records map
//---fires wipe history command for search state
pub async fn start_search(bot_message: impl BotMessage + 'static) {
    println!("START_SEARCH: search initiated");

    let mut map = RECORDS.lock().await;
    let id = bot_message.get_id();
    map.insert(
        format!("{}", id),
        UserStateRecord {
            last: Instant::now(),
            state: UserState::Search,
        },
    );
    drop(map);
    println!("START_SEARCH: record added for id {}", id);
    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Search);
    arc_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match responses::load_response("search-start") {
                Some(response) => response,
                _ => responses::response_unavailable(),
            },
        )))
        .await;
}

//---finishes search
//---fires immediate purge history command for search state
pub async fn continue_search(bot_message: impl BotMessage + 'static, processesed_text: String) {
    let arc_message = Arc::new(bot_message);

    immediate_purge_history(Arc::clone(&arc_message), UserState::Search);
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
    arc_message.send_message(response).await;
}
