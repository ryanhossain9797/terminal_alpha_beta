use once_cell::sync::Lazy;
use search_with_google::Client;

static SEARCH_CLIENT: Lazy<Client> = Lazy::new(Default::default);

pub struct SearchResult {
    pub description: String,
    pub link: String,
}

pub async fn get_by_keyword_and_limit(
    query: &str,
    limit: impl Into<Option<u32>>,
) -> anyhow::Result<Vec<SearchResult>> {
    Ok(SEARCH_CLIENT
        .search(
            query,
            limit.into(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0".to_string(),
        )
        .await?
        .into_iter()
        .map(|result| SearchResult {
            link: result.link,
            description: result.description,
        })
        .collect())
}
