use crate::handlers::root;
use crate::handlers::util;
use std::mem::drop;
use std::time::Instant;
use telegram_bot::*;

extern crate closestmatch;
use closestmatch::*;

//---adds a userstate record with identify state to userstate records map
//---fires wipe history command for identify state
pub async fn start_identify(message: Message) -> root::Msg {
    println!("START_IDENTIFY: identify initiated");

    let mut map = root::RECORDS.lock().await;
    map.entry(message.from.id)
        .or_insert_with(|| root::UserStateRecord {
            username: message.from.first_name.clone(),
            chat: message.chat.id(),
            last: Instant::now(),
            state: root::UserState::Identify,
        });
    drop(map);
    println!("START_IDENTIFY: record added");
    root::wipe_history(message.clone(), root::UserState::Identify);

    root::Msg::Text(format!(
        "Terminal Alpha and Beta:\nGreetings unit {}\
        \nwho do you want to look up?",
        &message.from.first_name
    ))
}

//---finishes identify
//---fires immediate purge history command for identify state
#[allow(unused_variables)]
pub async fn continue_identify(message: Message, processesed_text: String) -> root::Msg {
    root::immediate_purge_history(message.from.clone(), root::UserState::Identify);
    println!("IDENTIFY: beginning identification");
    get_person_go(&processesed_text)
}

fn get_person_go(name: &str) -> root::Msg {
    //---This is a test of the new method to move logic to go

    //---Part one
    if let Some(person) = util::get_person(name.to_string()) {
        return root::Msg::Text(person.description);
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
                            return root::Msg::Text(format!(
                                "We could not find that exact person\
                            \nBut we found {}:\
                            \n{}",
                                person.name, person.description
                            ));
                        }
                    }
                    root::Msg::Text(
                        "We could not find that person, Tagged for future identification"
                            .to_string(),
                    )
                }
                _ => root::Msg::Text(
                    "We could not find that person, Tagged for future identification".to_string(),
                ),
            }
        }
        _ => root::Msg::Text("We could not access the persons database".to_string()),
    }
}
