use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AlphaBetaApiError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
}

impl Display for AlphaBetaApiError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AlphaBetaApiError::ReqwestError(err) => write!(f, "{}", err),
            AlphaBetaApiError::SerdeJsonError(err) => write!(f, "{}", err),
        }
    }
}

impl From<reqwest::Error> for AlphaBetaApiError {
    fn from(err: reqwest::Error) -> AlphaBetaApiError {
        AlphaBetaApiError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for AlphaBetaApiError {
    fn from(err: serde_json::Error) -> AlphaBetaApiError {
        AlphaBetaApiError::SerdeJsonError(err)
    }
}

impl std::error::Error for AlphaBetaApiError {}

///Makes a simple get request to the provided url.  
///Return an Option<serde_json::Value>
pub async fn get_request_json(url: &str) -> Result<serde_json::Value, AlphaBetaApiError> {
    Ok(serde_json::from_str(
        &(reqwest::get(url).await?.text().await?),
    )?)
}
