use super::*;
use repositories::covid_repository;
use repositories::covid_repository::Country;

pub async fn get_top_new() -> Option<Vec<Country>> {
    covid_repository::get().await.get_top_new().await
}
pub async fn get_top_total() -> Option<Vec<Country>> {
    covid_repository::get().await.get_top_total().await
}
pub async fn get_aggreagte() -> (Option<i64>, Option<i64>) {
    covid_repository::get().await.get_aggregate().await
}
