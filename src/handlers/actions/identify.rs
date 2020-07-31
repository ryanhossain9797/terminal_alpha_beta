use super::*;
use closestmatch::*;

///Adds a userstate record with identify state to userstate records map.  
///Fires wipe history command for identify state.
pub async fn start_identify(bot_message: impl BotMessage + 'static) {
    println!("START_IDENTIFY: identify initiated");
    let id = bot_message.get_id();
    set_state(id.clone(), UserState::Identify).await;
    println!("START_IDENTIFY: record added for id {}", id);
    let arc_message = Arc::new(bot_message);
    wipe_history(Arc::clone(&arc_message), UserState::Identify);
    arc_message
        .send_message(responses::load("identify-start").into())
        .await;
}

///Finishes identify.  
///Fires immediate purge history command for identify state.
pub async fn continue_identify(bot_message: impl BotMessage + 'static, name: String) {
    let arc_message = Arc::new(bot_message);
    immediate_purge_history(Arc::clone(&arc_message), UserState::Identify);
    println!("IDENTIFY: beginning identification");
    match people_service::get_person(name.to_string()).await {
        //---If exact match on name
        Some(person) => {
            arc_message.send_message(person.description.into()).await;
        }

        //---Else, try to get closes match
        _ => {
            let partial_match = match people_service::get_people().await {
                Some(people) => {
                    let mut names: Vec<String> = vec![];
                    people
                        .iter()
                        .for_each(|person| names.push(person.name.clone()));
                    let cm = ClosestMatch::new(names, [name.len() / 2, name.len()].to_vec());
                    let closest_name = cm.get_closest(name.to_string());
                    match closest_name {
                        Some(name) => {
                            println!("closest name is {}", name);
                            let mut matched_option: Option<String> = None;
                            people.iter().for_each(|person| {
                                if person.name == name {
                                    matched_option = match responses::load("identify-partialmatch")
                                    {
                                        Some(response) => Some(
                                            response
                                                .replace("{name}", &person.name)
                                                .replace("{description}", &person.description),
                                        ),
                                        _ => None,
                                    }
                                }
                            });
                            match matched_option {
                                Some(person) => Some(person),
                                None => responses::load("identify-notfound"),
                            }
                        }
                        _ => responses::load("identify-notfound"),
                    }
                }
                _ => responses::load("identify-dberror"),
            };
            arc_message.send_message(partial_match.into()).await;
        }
    }
}
