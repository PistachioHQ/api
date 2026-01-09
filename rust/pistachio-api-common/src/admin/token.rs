//! Token management types for the Admin API.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use libgn::pistachio_id::UserId;
use libgn::project::ProjectId;
use libgn::tenant::TenantId;

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};

// =============================================================================
// CreateCustomToken
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateCustomTokenError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("User not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to create a custom token for a user.
///
/// Custom tokens can be used to sign in a user on the client-side with a
/// server-generated token. The token is signed by the service account key and
/// can include custom claims that will be available in the user's ID token.
#[derive(Debug, Clone)]
pub struct CreateCustomTokenRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// Optional tenant ID for multi-tenant scenarios.
    pub tenant_id: Option<TenantId>,
    /// Custom claims to include in the token.
    /// These claims will be available in the user's ID token after sign-in.
    /// Maximum size: 1000 bytes (serialized JSON).
    pub custom_claims: Option<serde_json::Value>,
    /// Token expiration time. If not specified, defaults to 1 hour.
    /// Maximum: 1 hour (3600 seconds).
    pub expires_in_seconds: Option<i32>,
}

impl CreateCustomTokenRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            pistachio_id,
            tenant_id: None,
            custom_claims: None,
            expires_in_seconds: None,
        }
    }

    /// Sets the tenant ID.
    pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Sets the custom claims.
    pub fn with_custom_claims(mut self, claims: serde_json::Value) -> Self {
        self.custom_claims = Some(claims);
        self
    }

    /// Sets the expiration time in seconds.
    pub fn with_expires_in_seconds(mut self, seconds: i32) -> Self {
        self.expires_in_seconds = Some(seconds);
        self
    }
}

/// Response from creating a custom token.
#[derive(Debug, Clone)]
pub struct CreateCustomTokenResponse {
    /// The generated custom token.
    /// This token can be used to sign in on the client-side.
    pub custom_token: String,
    /// When the token expires.
    pub expires_at: Option<DateTime<Utc>>,
}

// =============================================================================
// VerifyIdToken
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum VerifyIdTokenError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    TokenExpired,
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

/// Request to verify an ID token.
///
/// This operation validates the token signature, expiration, and issuer.
/// It returns the decoded token claims if the token is valid.
#[derive(Debug, Clone)]
pub struct VerifyIdTokenRequest {
    /// The project ID to verify the token against.
    pub project_id: ProjectId,
    /// The ID token to verify.
    pub id_token: String,
    /// Whether to check if the user is disabled.
    /// Default: true
    pub check_disabled: bool,
}

impl VerifyIdTokenRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, id_token: impl Into<String>) -> Self {
        Self {
            project_id,
            id_token: id_token.into(),
            check_disabled: true,
        }
    }

    /// Sets whether to check if the user is disabled.
    pub fn with_check_disabled(mut self, check: bool) -> Self {
        self.check_disabled = check;
        self
    }
}

/// Response from verifying an ID token.
#[derive(Debug, Clone)]
pub struct VerifyIdTokenResponse {
    /// Decoded token information.
    pub decoded_token: DecodedIdToken,
}

/// Decoded ID token claims.
#[derive(Debug, Clone)]
pub struct DecodedIdToken {
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// The project ID.
    pub project_id: ProjectId,
    /// Optional tenant ID.
    pub tenant_id: Option<TenantId>,
    /// Token issuer.
    pub issuer: String,
    /// Token subject (same as pistachio_id).
    pub subject: String,
    /// Token audience.
    pub audience: String,
    /// When the token was issued.
    pub issued_at: DateTime<Utc>,
    /// When the token expires.
    pub expires_at: DateTime<Utc>,
    /// Authentication time.
    pub auth_time: DateTime<Utc>,
    /// User's email address, if available.
    pub email: Option<String>,
    /// Whether the email has been verified.
    pub email_verified: Option<bool>,
    /// User's phone number, if available.
    pub phone_number: Option<String>,
    /// User's display name, if available.
    pub name: Option<String>,
    /// User's profile picture URL, if available.
    pub picture: Option<String>,
    /// Sign-in provider (e.g., "password", "google.com").
    pub sign_in_provider: Option<String>,
    /// Custom claims set on the user.
    pub custom_claims: HashMap<String, serde_json::Value>,
}

// =============================================================================
// CreateSessionCookie
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateSessionCookieError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Invalid ID token: {0}")]
    InvalidIdToken(String),
    #[error("Token expired")]
    TokenExpired,
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

/// Request to create a session cookie from an ID token.
///
/// Session cookies are used for server-side session management and are more
/// secure than client-side tokens because they can be invalidated on sign-out.
#[derive(Debug, Clone)]
pub struct CreateSessionCookieRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The ID token to exchange for a session cookie.
    pub id_token: String,
    /// Session cookie duration in seconds.
    /// Minimum: 300 (5 minutes), Maximum: 1209600 (14 days).
    /// Default: 1209600 (14 days).
    pub expires_in_seconds: Option<i64>,
}

impl CreateSessionCookieRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, id_token: impl Into<String>) -> Self {
        Self {
            project_id,
            id_token: id_token.into(),
            expires_in_seconds: None,
        }
    }

    /// Sets the expiration time in seconds.
    pub fn with_expires_in_seconds(mut self, seconds: i64) -> Self {
        self.expires_in_seconds = Some(seconds);
        self
    }
}

/// Response from creating a session cookie.
#[derive(Debug, Clone)]
pub struct CreateSessionCookieResponse {
    /// The session cookie value.
    /// This should be set as an HTTP-only secure cookie.
    pub session_cookie: String,
    /// When the session cookie expires.
    pub expires_at: Option<DateTime<Utc>>,
}

// =============================================================================
// VerifySessionCookie
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum VerifySessionCookieError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Invalid session cookie: {0}")]
    InvalidSessionCookie(String),
    #[error("Session expired")]
    SessionExpired,
    #[error("Session revoked")]
    SessionRevoked,
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

/// Request to verify a session cookie.
#[derive(Debug, Clone)]
pub struct VerifySessionCookieRequest {
    /// The project ID to verify the session cookie against.
    pub project_id: ProjectId,
    /// The session cookie to verify.
    pub session_cookie: String,
    /// Whether to check if the user's refresh tokens have been revoked.
    /// This adds latency but ensures the session hasn't been invalidated.
    /// Default: false
    pub check_revoked: bool,
}

impl VerifySessionCookieRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, session_cookie: impl Into<String>) -> Self {
        Self {
            project_id,
            session_cookie: session_cookie.into(),
            check_revoked: false,
        }
    }

    /// Sets whether to check if the session has been revoked.
    pub fn with_check_revoked(mut self, check: bool) -> Self {
        self.check_revoked = check;
        self
    }
}

/// Response from verifying a session cookie.
#[derive(Debug, Clone)]
pub struct VerifySessionCookieResponse {
    /// Decoded token information.
    pub decoded_token: DecodedIdToken,
}

// =============================================================================
// RevokeRefreshTokens
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RevokeRefreshTokensError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("User not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to revoke all refresh tokens for a user.
///
/// This effectively signs out the user from all devices and sessions.
/// After revocation, the user will need to sign in again on all devices.
#[derive(Debug, Clone)]
pub struct RevokeRefreshTokensRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// Optional tenant ID for multi-tenant scenarios.
    pub tenant_id: Option<TenantId>,
}

impl RevokeRefreshTokensRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            pistachio_id,
            tenant_id: None,
        }
    }

    /// Sets the tenant ID.
    pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}

/// Response from revoking refresh tokens.
#[derive(Debug, Clone)]
pub struct RevokeRefreshTokensResponse {
    // Empty response - all refresh tokens have been revoked.
}
