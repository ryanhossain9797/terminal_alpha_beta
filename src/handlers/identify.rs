use crate::handlers::responses;
use crate::handlers::root;
use crate::handlers::util;
use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;

extern crate closestmatch;
use closestmatch::*;

//---adds a userstate record with identify state to userstate records map
//---fires wipe history command for identify state
pub async fn start_identify(message: Message) -> root::MsgCount {
    println!("START_IDENTIFY: identify initiated");

    let mut map = root::RECORDS.lock().await;
    let id: i64 = message.from.id.into();
    map.entry(format!("{}", id))
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: root::UserState::Identify,
        });
    drop(map);
    println!("START_IDENTIFY: record added for id {}", id);
    root::wipe_history(message.clone(), root::UserState::Identify);

    root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("identify-start") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    ))
}

//---finishes identify
//---fires immediate purge history command for identify state
#[allow(unused_variables)]
pub async fn continue_identify(message: Message, processesed_text: String) -> root::MsgCount {
    root::immediate_purge_history(message.from.clone(), root::UserState::Identify);
    println!("IDENTIFY: beginning identification");
    get_person_go(&processesed_text)
}

fn get_person_go(name: &str) -> root::MsgCount {
    //---This is a test of the new method to move logic to go

    //---Part one
    if let Some(person) = util::get_person(name.to_string()) {
        return root::MsgCount::SingleMsg(root::Msg::Text(person.description));
    }

    //---Part two
    match util::get_people() {
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
                    for person in people {
                        if person.name == name {
                            return root::MsgCount::SingleMsg(root::Msg::Text(
                                match responses::load_response("identify-partialmatch") {
                                    Some(response) => response
                                        .replace("{name}", &person.name)
                                        .replace("{description}", &person.description),
                                    _ => responses::response_unavailable(),
                                },
                            ));
                        }
                    }
                    root::MsgCount::SingleMsg(root::Msg::Text(
                        match responses::load_response("identify-notfound") {
                            Some(response) => response,
                            _ => responses::response_unavailable(),
                        },
                    ))
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
    }
}
