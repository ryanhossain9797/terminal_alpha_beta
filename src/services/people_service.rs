use super::*;

use repositories::people_repository;
use repositories::people_repository::Person;

///Return's a Some(Person) if name matches, otherwise a None
pub async fn get_person(name: String) -> anyhow::Result<Option<Person>> {
    people_repository::get(name).await
}

///Returns a Some(Vec<Person>) if successful, otherwise a None
pub async fn get_people() -> anyhow::Result<Vec<Person>> {
    people_repository::get_all().await
}
