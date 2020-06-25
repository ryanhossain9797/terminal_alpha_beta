use super::*;

///Adds a userstate record with search state to userstate records map.  
///Fires wipe history command for search state.
pub async fn start_search(bot_message: impl BotMessage + 'static) {
    println!("START_SEARCH: search initiated");

    let id = bot_message.get_id();
    //---Insert Search intent
    set_state(id.clone(), UserState::Search).await;

    println!("START_SEARCH: record added for id {}", id);
    //---Make a cloneable ARC version of the Message
    let arc_message = Arc::new(bot_message);
    //---Fire off wipe
    wipe_history(Arc::clone(&arc_message), UserState::Search);

    arc_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            responses::load_named("search-start").unwrap_or_else(responses::unavailable),
        )))
        .await;
}

///Finishes search
///Fires immediate purge history command for search state
pub async fn continue_search(bot_message: impl BotMessage + 'static, processesed_text: String) {
    let arc_message = Arc::new(bot_message);
    //---Delete the UserState Record
    immediate_purge_history(Arc::clone(&arc_message), UserState::Search);
    let search_option = golib::google_search(processesed_text);

    let response = match search_option {
        Some(results) => {
            let mut msgs: Vec<Msg> = vec![Msg::Text(
                responses::load_named("search-success").unwrap_or_else(responses::unavailable),
            )];
            let search_template =
                responses::load_named("search-content").unwrap_or_else(responses::unavailable);
            for result in results {
                msgs.push(Msg::Text(
                    search_template
                        .replace("{description}", &result.description)
                        .replace("{url}", &result.link),
                ));
            }
            MsgCount::MultiMsg(msgs)
        }
        _ => MsgCount::SingleMsg(Msg::Text(
            responses::load_named("search-fail").unwrap_or_else(responses::unavailable),
        )),
    };
    arc_message.send_message(response).await;
}
