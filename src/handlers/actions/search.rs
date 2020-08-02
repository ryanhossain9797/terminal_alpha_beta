use super::*;
use once_cell::sync::Lazy;
use search_with_google::Client;

static SEARCH_CLIENT: Lazy<Client> = Lazy::new(Default::default);

///Adds a userstate record with search state to userstate records map.  
///Fires wipe history command for search state.
pub async fn start_search(bot_message: impl BotMessage + 'static) {
    let source = "START_SEARCH";

    let info =util::logger::make_info(source);
    info("search initiated");

    info(&format!("record added for if: {}", bot_message.get_id()));

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
    let arc_message = Arc::new(bot_message);
    //---Delete the UserState Record
    cancel_matching_state(Arc::clone(&arc_message), UserState::Search).await;

    let search_result = SEARCH_CLIENT
        .search(
            &processed_text,
            None,
            "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0".to_string(),
        )
        .await;

    let response = match search_result {
        Ok(results) => {
            let mut msgs: Vec<Msg> = vec![responses::load("search-success").into()];
            //Load template for search results
            let search_template = responses::load_text("search-content")
                .unwrap_or_else(|| "{description}\nURL: {url}".to_string());
            for result in results {
                msgs.push(
                    search_template
                        .replace("{description}", &result.description)
                        .replace("{url}", &result.link)
                        .into(),
                );
            }
            MsgCount::MultiMsg(msgs)
        }
        Err(error) => {
            println!("{:?}", error);
            responses::load("search-fail").into()
        }
    };
    arc_message.send_message(response).await;
}
