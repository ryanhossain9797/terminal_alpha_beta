use super::*;
use serde_json::Value;

pub async fn get_gfycat_by_keyword(keyword: &str) -> Option<String> {
    let url = format!(
        "https://api.gfycat.com/v1/gfycats/search?search_text={}&count=1",
        keyword
    );
    // Get json value from request
    if let Ok(Value::Object(map)) = api::get_request_json(&url).await {
        // Get desired stuff from json
        if let Some(Value::Array(gfycats)) = map.get("gfycats") {
            for gif in gfycats {
                if let Some(Value::String(url)) = gif.get("max2mbGif") {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}
