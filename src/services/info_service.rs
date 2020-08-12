use super::*;

use repositories::info_repository;

pub async fn get_info(title: String, pass: String) -> Option<String> {
    info_repository::get(title, pass).await
}
