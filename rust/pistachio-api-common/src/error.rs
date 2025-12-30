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
    pub const INVALID_SORT_FIELD: &str = "invalid_sort_field";

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

/// RFC 7807 Problem Details response.
///
/// This structure represents a standardized error response format as defined
/// in RFC 7807 (Problem Details for HTTP APIs). It provides a consistent way
/// to communicate error information to API consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemDetails {
    /// A URI reference that identifies the problem type.
    /// Example: `https://docs.pistachiohq.com/errors/not_found`
    pub problem_type: String,

    /// A short, human-readable summary of the problem type.
    /// Example: "Project not found"
    pub title: String,

    /// The HTTP status code for this occurrence of the problem.
    pub status: u16,

    /// A human-readable explanation specific to this occurrence.
    /// Example: "Project 'my-project' not found"
    pub detail: Option<String>,

    /// A URI reference that identifies the specific occurrence of the problem.
    /// Example: "/admin/v1/projects/my-project"
    pub instance: Option<String>,

    /// Extension member for validation errors containing details about
    /// which parameters failed validation and why.
    pub invalid_params: Vec<InvalidParam>,
}

/// Details about an invalid parameter in a validation error.
///
/// This is an RFC 7807 extension member used to provide structured
/// information about validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidParam {
    /// The name/path of the invalid parameter.
    /// Example: "sort"
    pub name: String,

    /// A human-readable explanation of why the parameter is invalid.
    /// Example: "Invalid value"
    pub reason: String,

    /// The invalid value that was provided (if applicable).
    /// Example: "foo"
    pub value: Option<String>,

    /// The list of valid/expected values for this parameter (if applicable).
    /// Example: ["project_id", "created_at", "display_name"]
    pub expected_values: Vec<String>,
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
