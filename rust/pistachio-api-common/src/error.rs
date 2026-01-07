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
    pub const ALREADY_USED: &str = "already_used";

    // State errors
    pub const NOT_DELETED: &str = "not_deleted";

    // Infrastructure errors
    pub const SERVICE_UNAVAILABLE: &str = "service_unavailable";
    pub const INTERNAL_ERROR: &str = "internal_error";
    pub const NOT_IMPLEMENTED: &str = "not_implemented";

    // Auth errors
    pub const UNAUTHENTICATED: &str = "unauthenticated";
    pub const PERMISSION_DENIED: &str = "permission_denied";
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

/// Protocol-agnostic error details.
///
/// Contains semantic information about an error without exposing
/// transport-specific details (HTTP status codes, gRPC codes, etc.).
///
/// SDK users care about:
/// - What kind of error occurred (via enum variant: `NotFound`, `FailedPrecondition`, etc.)
/// - Semantic details to understand/fix the problem
///
/// SDK users should NOT see:
/// - HTTP status codes
/// - gRPC status codes (tonic::Code)
/// - Any hint of which transport is being used
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDetails {
    /// Error type slug for programmatic handling.
    ///
    /// Example: "not_found", "already_used", "invalid_request"
    pub error_type: String,

    /// Human-readable error title.
    ///
    /// Example: "Project not found"
    pub title: String,

    /// Detailed error message specific to this occurrence.
    ///
    /// Example: "No project exists with ID proj_123abc"
    pub message: Option<String>,

    /// Invalid parameters for validation errors.
    pub invalid_params: Vec<InvalidParam>,
}

impl ErrorDetails {
    /// Create error details from a message string.
    ///
    /// Used as a fallback when structured error details are not available.
    pub fn from_message(message: impl Into<String>) -> Self {
        let msg = message.into();
        Self {
            error_type: error_type::INTERNAL_ERROR.to_string(),
            title: "Error".to_string(),
            message: Some(msg),
            invalid_params: Vec::new(),
        }
    }

    /// Create error details with a specific type and title.
    pub fn new(error_type: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            title: title.into(),
            message: None,
            invalid_params: Vec::new(),
        }
    }

    /// Set the detail message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Add an invalid parameter.
    #[must_use]
    pub fn with_invalid_param(mut self, param: InvalidParam) -> Self {
        self.invalid_params.push(param);
        self
    }
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
    #[error("Invalid provider ID: {0}")]
    InvalidProviderId(#[from] libgn::auth_provider::ProviderIdError),
}
