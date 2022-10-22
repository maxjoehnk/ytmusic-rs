use thiserror::Error;

#[derive(Debug, Error)]
pub enum YoutubeMusicError {
    #[error("HTTP Error {0}")]
    HttpError(surf::Error),
    #[error("Deserialization error {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Unexpected api response")]
    ApiError,
    #[error("Missing SAPISID cookie")]
    MissingSapisidCookieError,
    #[error("Invalid cookie: {0}")]
    InvalidCookieError(String),
    #[error("Unable to fetch visitor id")]
    MissingVisitorId,
}

impl From<surf::Error> for YoutubeMusicError {
    fn from(error: surf::Error) -> Self {
        Self::HttpError(error)
    }
}
