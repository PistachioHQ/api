#[derive(Debug, thiserror::Error)]
pub enum PistachioApiClientError {
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Not connected")]
    NotConnected,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Invalid field value: {0}")]
    InvalidValue(&'static str),
}
