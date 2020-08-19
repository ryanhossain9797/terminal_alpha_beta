///Makes a simple get request to the provided url.  
///Return an Option<serde_json::Value>
pub async fn get_request_json(url: &str) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::from_str(
        &(reqwest::get(url).await?.text().await?),
    )?)
}
