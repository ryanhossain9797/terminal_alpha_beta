use super::*;

use futures::stream::StreamExt;
use mongodb::bson::{doc, Bson};

///A single note
/// - `name` person's name
/// - `description` the person's description
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

///Return's a Some(Person) if name matches, otherwise a None
pub async fn get(name: String) -> anyhow::Result<Option<Person>> {
    if let Some(document) = database::mongo::get_mongo()
        .await
        .ok_or_else(|| anyhow::anyhow!("Couldn't fetch db connection"))?
        .collection("people")
        .find_one(doc! {"name": &name}, None)
        .await?
    {
        let description = document
            .get("description")
            .and_then(Bson::as_str)
            .ok_or_else(|| anyhow::anyhow!("No field named description"))?;
        return Ok(Some(Person::new(name, description)));
    }
    Ok(None)
}

///Returns a Some(Vec<Person>) if successful, otherwise a None
pub async fn get_all() -> anyhow::Result<Vec<Person>> {
    Ok(database::mongo::get_mongo()
        .await
        .ok_or_else(|| anyhow::anyhow!("Couldn't fetch db connection"))?
        .collection("people")
        .find(None, None)
        .await?
        .filter_map(async move |person_result| {
            if let Ok(document) = person_result {
                if let (Some(name), Some(description)) = (
                    document.get("name").and_then(Bson::as_str),
                    document.get("description").and_then(Bson::as_str),
                ) {
                    return Some(Person::new(name, description));
                }
            }
            None
        })
        .collect()
        .await)
}
