use super::*;
use repositories::covid_repository;
use repositories::covid_repository::Country;

pub async fn get_top_new() -> anyhow::Result<Vec<Country>> {
    Ok(covid_repository::get().await.get_top_new().await?)
}
pub async fn get_top_total() -> anyhow::Result<Vec<Country>> {
    Ok(covid_repository::get().await.get_top_total().await?)
}
pub async fn get_aggreagte() -> (anyhow::Result<i64>, anyhow::Result<i64>) {
    covid_repository::get().await.get_aggregate().await
}
