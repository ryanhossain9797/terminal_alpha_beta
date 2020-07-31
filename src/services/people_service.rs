use super::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, Bson};

pub struct Person {
    pub name: String,
    pub description: String,
}
impl Person {
    fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Person {
            name: name.into(),
            description: description.into(),
        }
    }
}

pub async fn get_person(name: String) -> Option<Person> {
    println!("GETTING PERSON: {}", name);
    if let Some(db) = database::get_mongo().await {
        if let Ok(Some(document)) = db
            .collection("people")
            .find_one(doc! {"name": &name}, None)
            .await
        {
            if let Some(description) = document.get("description").and_then(Bson::as_str) {
                return Some(Person::new(name, description));
            }
        }
    }
    None
}

pub async fn get_people() -> Option<Vec<Person>> {
    if let Some(db) = database::get_mongo().await {
        if let Ok(people) = db.collection("people").find(None, None).await {
            return Some(
                people
                    .fold(vec![], |mut people_list, person| async {
                        if let Ok(document) = person {
                            if let (Some(name), Some(description)) = (
                                document.get("name").and_then(Bson::as_str),
                                document.get("description").and_then(Bson::as_str),
                            ) {
                                people_list.push(Person::new(name, description));
                            }
                        }
                        people_list
                    })
                    .await,
            );
        }
    }
    None
}
