use super::*;

use repositories::info_repository;

pub async fn get_info(title: String, pass: String) -> anyhow::Result<Option<String>> {
    info_repository::get(title, pass).await
}
