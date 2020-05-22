use crate::handlers::responses;
use crate::handlers::root;
use crate::handlers::util;
use std::mem::drop;
use std::time::Instant;

extern crate closestmatch;
use closestmatch::*;

//---adds a userstate record with identify state to userstate records map
//---fires wipe history command for identify state
pub async fn start_identify(m: Box<dyn root::BotMessage + Send + Sync>) {
    println!("START_IDENTIFY: identify initiated");
    let mut map = root::RECORDS.lock().await;
    let id = (*m).get_id();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: (*m).get_name(),
            last: Instant::now(),
            state: root::UserState::Identify,
        });
    drop(map);
    println!("START_IDENTIFY: record added for id {}", id);
    root::wipe_history(m.clone(), root::UserState::Identify);
    (*m).send_msg(root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("identify-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}

//---finishes identify
//---fires immediate purge history command for identify state
pub async fn continue_identify(m: Box<dyn root::BotMessage + Send + Sync>, name: String) {
    root::immediate_purge_history(m.clone(), root::UserState::Identify);
    println!("IDENTIFY: beginning identification");
    match util::get_person(name.to_string()) {
        //---Part one
        Some(person) => {
            (*m).send_msg(root::MsgCount::SingleMsg(root::Msg::Text(
                person.description,
            )));
        }

        //---Part two
        _ => {
            let partial_match = match util::get_people() {
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
                                Some(person) => root::MsgCount::SingleMsg(root::Msg::Text(person)),
                                None => root::MsgCount::SingleMsg(root::Msg::Text(
                                    match responses::load_response("identify-notfound") {
                                        Some(response) => response,
                                        _ => responses::response_unavailable(),
                                    },
                                )),
                            }
                        }
                        _ => root::MsgCount::SingleMsg(root::Msg::Text(
                            match responses::load_response("identify-notfound") {
                                Some(response) => response,
                                _ => responses::response_unavailable(),
                            },
                        )),
                    }
                }
                _ => root::MsgCount::SingleMsg(root::Msg::Text(
                    match responses::load_response("identify-dberror") {
                        Some(response) => response,
                        _ => responses::response_unavailable(),
                    },
                )),
            };
            (*m).send_msg(partial_match);
        }
    }
}
