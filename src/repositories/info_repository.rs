use super::*;

use mongodb::bson::{doc, Bson};

pub async fn get(title: String, pass: String) -> Option<String> {
    if let Some(db) = database::get_mongo().await {
        let info = db.collection("info");
        let info_result = info
            .find_one(
                doc! {
                    "title": &title,
                    "pass": &pass,
                },
                None,
            )
            .await;
        if let Ok(info) = info_result {
            if let Some(document) = info {
                if let Some(info) = document.get("info").and_then(Bson::as_str) {
                    return Some(info.to_string().replace("\\n", "\n"));
                }
            }
        }
    }
    None
}
