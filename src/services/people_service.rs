use super::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, Bson};

pub struct Person {
    pub name: String,
    pub description: String,
}

pub async fn get_person(name: String) -> Option<Person> {
    println!("GETTING PERSON: {}", name);
    if let Some(db) = database::get_mongo().await {
        let notes = db.collection("people");
        let person_result = notes
            .find_one(
                doc! {
                    "name": &name
                },
                None,
            )
            .await;
        if let Ok(person) = person_result {
            if let Some(document) = person {
                if let Some(description) = document.get("description").and_then(Bson::as_str) {
                    return Some(Person {
                        name,
                        description: description.to_string(),
                    });
                }
            }
        }
    }

    None
}

pub async fn get_people() -> Option<Vec<Person>> {
    if let Some(db) = database::get_mongo().await {
        let people = db.collection("people");
        let people_result = people.find(None, None).await;
        if let Ok(mut my_notes) = people_result {
            let mut people_list: Vec<Person> = vec![];

            while let Some(result) = my_notes.next().await {
                if let Ok(document) = result {
                    if let (Some(name), Some(description)) = (
                        document.get("name").and_then(Bson::as_str),
                        document.get("description").and_then(Bson::as_str),
                    ) {
                        people_list.push(Person {
                            name: name.to_string(),
                            description: description.to_string(),
                        });
                    }
                }
            }
            return Some(people_list);
        }
    }

    None
}
