use super::*;
use repositories::gfycat_repository;

pub async fn get_by_keyword(keyword: &str) -> anyhow::Result<String> {
    Ok(gfycat_repository::get_gfycat_by_keyword(keyword).await?)
}
