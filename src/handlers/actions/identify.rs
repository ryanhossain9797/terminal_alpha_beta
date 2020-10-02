use super::*;
use closestmatch::*;

///Adds a userstate record with identify state to userstate records map.  
///Fires wipe history command for identify state.
pub async fn start(bot_message: Box<dyn BotMessage>) {
    let source = "START_IDENTIFY";
    let info = util::logger::info(source);
    info("identify initiated");
    let arc_message = Arc::new(bot_message);
    set_timed_state(Arc::clone(&arc_message), UserState::Identify).await;
    arc_message
        .send_message(responses::load("identify-start").into())
        .await;
}

///Finishes identify.  
///Fires immediate purge history command for identify state.
pub async fn resume(bot_message: Box<dyn BotMessage>, name: String) {
    let source = "CONTINUE_IDENTIFY";
    let info = util::logger::info(source);
    let arc_message = Arc::new(bot_message);
    cancel_matching_state(Arc::clone(&arc_message), UserState::Identify).await;
    info("beginning identification");

    //---If exact match on name
    if let Ok(Some(person)) = people_service::get_person(name.to_string()).await {
        info("Found direct match");
        arc_message.send_message(person.description.into()).await;
    }
    //---Else, try to get closest match
    else {
        info("No direct match, trying closest match");
        arc_message
            .send_message(responses::load("identify-nodirect").into())
            .await;
        task::sleep(Duration::from_secs(2)).await;
        let partial_match = match people_service::get_people().await {
            Ok(people) => {
                let mut names: Vec<String> = vec![];
                people
                    .iter()
                    .for_each(|person| names.push(person.name.clone()));
                let cm = ClosestMatch::new(names, [name.len() / 2, name.len()].to_vec());
                let closest_name = cm.get_closest(name.to_string());
                match closest_name {
                    Some(name) => {
                        info(format!("closest name is {}", name).as_str());
                        let mut matched_option: Option<String> = None;
                        people.iter().for_each(|person| {
                            if person.name == name {
                                matched_option =
                                    responses::load("identify-partialmatch").map(|response| {
                                        response
                                            .replace("{name}", person.name.as_str())
                                            .replace("{description}", person.description.as_str())
                                    });
                            }
                        });
                        match matched_option {
                            Some(person) => Some(person),
                            None => responses::load("identify-notfound"),
                        }
                    }
                    None => responses::load("identify-notfound"),
                }
            }
            Err(_) => responses::load("identify-dberror"),
        };
        arc_message.send_message(partial_match.into()).await;
    }
    info("Identification complete");
}
