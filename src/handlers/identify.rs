use super::*;
use std::mem::drop;
use std::time::Instant;

extern crate closestmatch;
use closestmatch::*;

//---adds a userstate record with identify state to userstate records map
//---fires wipe history command for identify state
pub async fn start_identify(m: Box<dyn BotMessage + Send + Sync>) {
    println!("START_IDENTIFY: identify initiated");
    let mut map = RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| UserStateRecord {
            username: (*m).get_name(),
            last: Instant::now(),
            state: UserState::Identify,
        });
    drop(map);
    println!("START_IDENTIFY: record added for id {}", id);
    wipe_history(m.clone(), UserState::Identify);
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("identify-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}

//---finishes identify
//---fires immediate purge history command for identify state
pub async fn continue_identify(m: Box<dyn BotMessage + Send + Sync>, name: String) {
    immediate_purge_history(m.clone(), UserState::Identify);
    println!("IDENTIFY: beginning identification");
    match general::get_person(name.to_string()) {
        //---Part one
        Some(person) => {
            (*m).send_message(MsgCount::SingleMsg(Msg::Text(person.description)));
        }

        //---Part two
        _ => {
            let partial_match = match general::get_people() {
                Some(people) => {
                    let mut names: Vec<String> = vec![];
                    people
                        .iter()
                        .for_each(|person| names.push(person.name.clone()));
                    let cm = ClosestMatch::new(names, [4, 5, 6].to_vec());
                    let closest_name = cm.get_closest(name.to_string());
                    match closest_name {
                        Some(name) => {
                            println!("closest name is {}", name);
                            let mut matched_option: Option<String> = None;
                            for person in people {
                                if person.name == name {
                                    matched_option = Some(
                                        match responses::load_response("identify-partialmatch") {
                                            Some(response) => response
                                                .replace("{name}", &person.name)
                                                .replace("{description}", &person.description),
                                            _ => responses::response_unavailable(),
                                        },
                                    )
                                }
                            }
                            match matched_option {
                                Some(person) => MsgCount::SingleMsg(Msg::Text(person)),
                                None => MsgCount::SingleMsg(Msg::Text(
                                    match responses::load_response("identify-notfound") {
                                        Some(response) => response,
                                        _ => responses::response_unavailable(),
                                    },
                                )),
                            }
                        }
                        _ => MsgCount::SingleMsg(Msg::Text(
                            match responses::load_response("identify-notfound") {
                                Some(response) => response,
                                _ => responses::response_unavailable(),
                            },
                        )),
                    }
                }
                _ => MsgCount::SingleMsg(Msg::Text(
                    match responses::load_response("identify-dberror") {
                        Some(response) => response,
                        _ => responses::response_unavailable(),
                    },
                )),
            };
            (*m).send_message(partial_match);
        }
    }
}
