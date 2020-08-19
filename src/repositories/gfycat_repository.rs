use super::*;
use serde_json::Value;

pub async fn get_gfycat_by_keyword(keyword: &str) -> anyhow::Result<String> {
    let url = format!(
        "https://api.gfycat.com/v1/gfycats/search?search_text={}&count=1",
        keyword
    );
    // Get json value from request
    if let Value::Object(map) = api::get_request_json(&url).await? {
        // Get desired stuff from json
        if let Value::Array(gfycats) = map
            .get("gfycats")
            .ok_or_else(|| anyhow::anyhow!("Can't find value for key 'gfycats'"))?
        {
            if let Value::String(url) = gfycats
                .first()
                .ok_or_else(|| anyhow::anyhow!("'gfycats' has no result"))?
                .get("max2mbGif")
                .ok_or_else(|| anyhow::anyhow!("Can't find value for key 'max2mbGif'"))?
            {
                return Ok(url.to_string());
            } else {
                return Err(anyhow::anyhow!("'max2mbGif' isn't a url"));
            }
        }
    } else {
        return Err(anyhow::anyhow!("{}", "'gfycats' isn't a valid json"));
    }
    Err(anyhow::anyhow!("Something went wrong"))
}
