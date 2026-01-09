//! API key management types for the Admin API.

use chrono::{DateTime, Utc};
use libgn::app::AppId;
use libgn::project::ProjectId;

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};
use crate::pagination::{PaginationMeta, PaginationParams};

// =============================================================================
// API Key Domain Types
// =============================================================================

/// An API key with the full key string (only returned on creation/rotation).
#[derive(Debug, Clone)]
pub struct ApiKey {
    /// Resource name in the format:
    /// "projects/{project_id}/apps/{app_id}/apiKeys/{key_id}"
    pub name: String,
    /// Unique key identifier.
    pub key_id: String,
    /// The full API key string to use for authentication.
    /// IMPORTANT: Only returned once at creation/rotation time.
    pub key_string: String,
    /// Human-readable display name for the key.
    pub display_name: Option<String>,
    /// Restrictions applied to this API key.
    pub restrictions: Option<ApiKeyRestrictions>,
    /// Timestamp when the key was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp when the key was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

/// API key metadata with masked key value (for list/get operations).
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// Resource name in the format:
    /// "projects/{project_id}/apps/{app_id}/apiKeys/{key_id}"
    pub name: String,
    /// Unique key identifier.
    pub key_id: String,
    /// Masked prefix of the API key for identification purposes.
    pub key_prefix: String,
    /// Human-readable display name for the key.
    pub display_name: Option<String>,
    /// Restrictions applied to this API key.
    pub restrictions: Option<ApiKeyRestrictions>,
    /// Timestamp when the key was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp when the key was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

/// Restrictions that specify how an API key can be used.
#[derive(Debug, Clone)]
pub struct ApiKeyRestrictions {
    /// Platform-specific restrictions.
    pub platform_restrictions: Option<PlatformRestrictions>,
}

/// Platform-specific API key restrictions.
#[derive(Debug, Clone)]
pub enum PlatformRestrictions {
    /// Restrictions for browser/web API keys.
    Browser(BrowserKeyRestrictions),
    /// Restrictions for server API keys.
    Server(ServerKeyRestrictions),
    /// Restrictions for Android API keys.
    Android(AndroidKeyRestrictions),
    /// Restrictions for iOS API keys.
    Ios(IosKeyRestrictions),
}

/// Restrictions for browser/web API keys.
#[derive(Debug, Clone)]
pub struct BrowserKeyRestrictions {
    /// List of allowed HTTP referrer patterns.
    pub allowed_referrers: Vec<String>,
}

/// Restrictions for server API keys.
#[derive(Debug, Clone)]
pub struct ServerKeyRestrictions {
    /// List of allowed IP addresses or CIDR ranges.
    pub allowed_ips: Vec<String>,
}

/// Restrictions for Android API keys.
#[derive(Debug, Clone)]
pub struct AndroidKeyRestrictions {
    /// List of allowed Android applications.
    pub allowed_applications: Vec<AndroidApplication>,
}

/// Android application identification.
#[derive(Debug, Clone)]
pub struct AndroidApplication {
    /// The Android package name.
    pub package_name: String,
    /// SHA-256 fingerprint of the signing certificate.
    pub sha256_cert_fingerprint: String,
}

/// Restrictions for iOS API keys.
#[derive(Debug, Clone)]
pub struct IosKeyRestrictions {
    /// List of allowed iOS bundle identifiers.
    pub allowed_bundle_ids: Vec<String>,
}

// =============================================================================
// CreateApiKey
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateApiKeyError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("App not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to create a new API key for an app.
#[derive(Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID that will own this API key.
    pub app_id: AppId,
    /// Human-readable display name for the key.
    pub display_name: Option<String>,
    /// Restrictions to apply to this API key.
    pub restrictions: Option<ApiKeyRestrictions>,
}

impl CreateApiKeyRequest {
    /// Creates a new request with required fields only.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self {
            project_id,
            app_id,
            display_name: None,
            restrictions: None,
        }
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the restrictions.
    pub fn with_restrictions(mut self, restrictions: ApiKeyRestrictions) -> Self {
        self.restrictions = Some(restrictions);
        self
    }
}

/// Response from creating an API key.
///
/// IMPORTANT: The key_string is only returned in this response.
#[derive(Debug, Clone)]
pub struct CreateApiKeyResponse {
    /// The created API key with full key_string.
    pub api_key: ApiKey,
}

// =============================================================================
// GetApiKey
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetApiKeyError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("API key not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to get an API key by ID.
#[derive(Debug, Clone)]
pub struct GetApiKeyRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID that owns the API key.
    pub app_id: AppId,
    /// The API key ID.
    pub key_id: String,
}

impl GetApiKeyRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, app_id: AppId, key_id: impl Into<String>) -> Self {
        Self {
            project_id,
            app_id,
            key_id: key_id.into(),
        }
    }
}

/// Response from getting an API key.
#[derive(Debug, Clone)]
pub struct GetApiKeyResponse {
    /// The API key with masked key_prefix.
    pub api_key: ApiKeyInfo,
}

// =============================================================================
// UpdateApiKey
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateApiKeyError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("API key not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to update an existing API key.
#[derive(Debug, Clone)]
pub struct UpdateApiKeyRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID that owns the API key.
    pub app_id: AppId,
    /// The API key ID.
    pub key_id: String,
    /// New display name.
    pub display_name: Option<String>,
    /// New restrictions. If provided, replaces existing restrictions.
    pub restrictions: Option<ApiKeyRestrictions>,
}

impl UpdateApiKeyRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, app_id: AppId, key_id: impl Into<String>) -> Self {
        Self {
            project_id,
            app_id,
            key_id: key_id.into(),
            display_name: None,
            restrictions: None,
        }
    }

    /// Sets the display name to update.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the restrictions.
    pub fn with_restrictions(mut self, restrictions: ApiKeyRestrictions) -> Self {
        self.restrictions = Some(restrictions);
        self
    }
}

/// Response from updating an API key.
#[derive(Debug, Clone)]
pub struct UpdateApiKeyResponse {
    /// The updated API key with masked key_prefix.
    pub api_key: ApiKeyInfo,
}

// =============================================================================
// DeleteApiKey
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteApiKeyError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("API key not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to delete an API key.
#[derive(Debug, Clone)]
pub struct DeleteApiKeyRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID that owns the API key.
    pub app_id: AppId,
    /// The API key ID.
    pub key_id: String,
}

impl DeleteApiKeyRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, app_id: AppId, key_id: impl Into<String>) -> Self {
        Self {
            project_id,
            app_id,
            key_id: key_id.into(),
        }
    }
}

/// Response from deleting an API key.
#[derive(Debug, Clone)]
pub struct DeleteApiKeyResponse {
    // Empty response - the API key has been permanently deleted.
}

// =============================================================================
// ListApiKeys
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListApiKeysError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("App not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to list API keys for an app.
#[derive(Debug, Clone)]
pub struct ListApiKeysRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID to list API keys from.
    pub app_id: AppId,
    /// Pagination parameters.
    pub pagination: PaginationParams,
}

impl ListApiKeysRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self {
            project_id,
            app_id,
            pagination: PaginationParams::default(),
        }
    }

    /// Sets the pagination parameters.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Response from listing API keys.
#[derive(Debug, Clone)]
pub struct ListApiKeysResponse {
    /// The list of API keys with masked key values.
    pub api_keys: Vec<ApiKeyInfo>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// RotateApiKey
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RotateApiKeyError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("API key not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to rotate an API key.
#[derive(Debug, Clone)]
pub struct RotateApiKeyRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID that owns the API key.
    pub app_id: AppId,
    /// The API key ID.
    pub key_id: String,
    /// Grace period in seconds during which the previous key remains valid.
    /// Default: 86400 (24 hours). Maximum: 604800 (7 days).
    pub grace_period_seconds: Option<i64>,
}

impl RotateApiKeyRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, app_id: AppId, key_id: impl Into<String>) -> Self {
        Self {
            project_id,
            app_id,
            key_id: key_id.into(),
            grace_period_seconds: None,
        }
    }

    /// Sets the grace period in seconds.
    pub fn with_grace_period_seconds(mut self, seconds: i64) -> Self {
        self.grace_period_seconds = Some(seconds);
        self
    }
}

/// Response from rotating an API key.
///
/// IMPORTANT: The new key_string is only returned once.
#[derive(Debug, Clone)]
pub struct RotateApiKeyResponse {
    /// The API key with new key_string.
    pub api_key: ApiKey,
    /// The previous key string (remains valid during grace period).
    pub previous_key_string: String,
    /// When the grace period expires.
    pub grace_period_expires_at: Option<DateTime<Utc>>,
}
