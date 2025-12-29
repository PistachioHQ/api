/// External error type slugs.
///
/// These slugs are exposed to API consumers and become part of the error type URL:
/// `https://docs.pistachiohq.com/errors/{slug}`
///
/// Each slug represents a documented error condition that clients may need
/// to handle programmatically.
pub mod error_type {
    // Resource not found errors
    pub const NOT_FOUND: &str = "not_found";

    // Validation errors
    pub const INVALID_PARAMETER: &str = "invalid_parameter";
    pub const INVALID_REQUEST: &str = "invalid_request";

    // Conflict errors
    pub const ALREADY_EXISTS: &str = "already_exists";

    // State errors
    pub const NOT_DELETED: &str = "not_deleted";

    // Infrastructure errors
    pub const SERVICE_UNAVAILABLE: &str = "service_unavailable";
    pub const INTERNAL_ERROR: &str = "internal_error";

    // Auth errors
    pub const UNAUTHENTICATED: &str = "unauthenticated";
    pub const PERMISSION_DENIED: &str = "permission_denied";
}

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
    #[error("Invalid project ID: {0}")]
    InvalidProjectId(#[from] libgn::pistachio_id::ProjectIdError),
    #[error("Invalid tenant ID: {0}")]
    InvalidTenantId(#[from] libgn::pistachio_id::TenantIdError),
    #[error("Invalid app ID: {0}")]
    InvalidAppId(#[from] libgn::pistachio_id::AppIdError),
}
