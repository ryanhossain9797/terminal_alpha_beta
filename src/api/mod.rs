use super::*;

///Makes a simple get request to the provided url.  
///Return an Option<serde_json::Value>
pub async fn get_request_json(url: &str) -> Option<serde_json::Value> {
    let source = "GET_JSON";
    let info = util::logger::make_info(source);
    let error = util::logger::make_error(source);
    let req_result = reqwest::get(url).await;
    match req_result {
        //If Request successful
        Ok(result) => match result.text().await {
            //If body text is available
            Ok(body) => {
                info("Fetched json successfully");
                return serde_json::from_str(&body).ok();
            }
            //If request body fails
            Err(err) => {
                error(&format!("{}", err));
            }
        },
        //If request fails
        Err(err) => {
            error(&format!("{}", err));
        }
    }
    None
}
