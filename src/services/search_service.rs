use super::*;
use repositories::search_repository;
use repositories::search_repository::SearchResult;

pub async fn get_search_results_by_query(query: &str) -> anyhow::Result<Vec<SearchResult>> {
    search_repository::get_by_keyword_and_limit(query, None).await
}
