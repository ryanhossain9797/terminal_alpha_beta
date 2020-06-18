use super::*;

extern crate closestmatch;
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
        .send_message(MsgCount::SingleMsg(Msg::Text(
            match responses::load("identify-start") {
                Some(response) => response,
                _ => responses::unavailable(),
            },
        )))
        .await;
}

///Finishes identify.  
///Fires immediate purge history command for identify state.
pub async fn continue_identify(bot_message: impl BotMessage + 'static, name: String) {
    let arc_message = Arc::new(bot_message);
    immediate_purge_history(Arc::clone(&arc_message), UserState::Identify);
    println!("IDENTIFY: beginning identification");
    match general::get_person(name.to_string()).await {
        //---Part one
        Some(person) => {
            arc_message
                .send_message(MsgCount::SingleMsg(Msg::Text(person.description)))
                .await;
        }

        //---Part two
        _ => {
            let partial_match = match general::get_people().await {
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
                            for person in people {
                                if person.name == name {
                                    matched_option =
                                        Some(match responses::load("identify-partialmatch") {
                                            Some(response) => response
                                                .replace("{name}", &person.name)
                                                .replace("{description}", &person.description),
                                            _ => responses::unavailable(),
                                        })
                                }
                            }
                            match matched_option {
                                Some(person) => MsgCount::SingleMsg(Msg::Text(person)),
                                None => MsgCount::SingleMsg(Msg::Text(
                                    match responses::load("identify-notfound") {
                                        Some(response) => response,
                                        _ => responses::unavailable(),
                                    },
                                )),
                            }
                        }
                        _ => MsgCount::SingleMsg(Msg::Text(
                            match responses::load("identify-notfound") {
                                Some(response) => response,
                                _ => responses::unavailable(),
                            },
                        )),
                    }
                }
                _ => MsgCount::SingleMsg(Msg::Text(match responses::load("identify-dberror") {
                    Some(response) => response,
                    _ => responses::unavailable(),
                })),
            };
            arc_message.send_message(partial_match).await;
        }
    }
}
