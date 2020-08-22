use super::*;

///Adds a userstate record with search state to userstate records map.  
///Fires wipe history command for search state.
pub async fn start_search(bot_message: impl BotMessage + 'static) {
    let source = "START_SEARCH";

    let info = util::logger::make_info(source);
    info("search initiated");

    //---Make a cloneable ARC version of the Message
    let arc_message = Arc::new(bot_message);
    //---Fire off wipe
    set_timed_state(Arc::clone(&arc_message), UserState::Search).await;

    arc_message
        .send_message(responses::load("search-start").into())
        .await;
}

///Finishes search
///Fires immediate purge history command for search state
pub async fn continue_search(bot_message: impl BotMessage + 'static, processed_text: String) {
    let source = "CONTINUE_SEARCH";
    let info = util::logger::make_info(source);
    let arc_message = Arc::new(bot_message);
    //---Delete the UserState Record
    cancel_matching_state(Arc::clone(&arc_message), UserState::Search).await;

    let search_result =
        services::search_service::get_search_results_by_query(&processed_text).await;

    match search_result {
        Ok(results) => {
            arc_message
                .send_message(responses::load("search-success").into())
                .await;
            task::sleep(Duration::from_secs(1)).await;
            //Load template for search results
            let search_template = responses::load_text("search-content")
                .unwrap_or_else(|| "{description}\nURL: {url}".to_string());
            info("Sending search results");
            arc_message
                .send_message(
                    results
                        .into_iter()
                        .map(|result| {
                            search_template
                                .replace("{description}", &result.description)
                                .replace("{url}", &result.link)
                                .into()
                        })
                        .collect::<Vec<Msg>>()
                        .into(),
                )
                .await;
        }
        _ => {
            arc_message
                .send_message(responses::load("search-fail").into())
                .await
        }
    };
}
