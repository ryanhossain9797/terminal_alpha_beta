use super::*;
use repositories::gfycat_repository;

pub async fn get_gfycat_by_keyword(keyword: &str) -> Option<String> {
    let source = "GFYCAT_SERVICE";
    let info = util::logger::make_info(source);
    let error = util::logger::make_error(source);

    match gfycat_repository::get_gfycat_by_keyword(keyword).await {
        Some(url) => {
            info(&format!("The url is: {}", url));
            Some(url)
        }
        None => {
            error("Couldn't get gif from gfycat");
            None
        }
    }
}
