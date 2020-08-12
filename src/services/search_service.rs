use super::*;
use repositories::search_repository;
use repositories::search_repository::Result;

pub async fn get_search_results_by_query(query: &str) -> Option<Vec<Result>> {
    search_repository::get_by_keyword_and_limit(query, None).await
}
