//! MFA (Multi-Factor Authentication) management types for the Admin API.

use chrono::{DateTime, Utc};
use libgn::pistachio_id::UserId;
use libgn::project::ProjectId;
use libgn::tenant::TenantId;

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};

// =============================================================================
// MFA Domain Types
// =============================================================================

/// A multi-factor authentication factor enrolled by a user.
#[derive(Debug, Clone)]
pub struct MfaFactor {
    /// Unique factor identifier.
    pub factor_id: String,
    /// The type of MFA factor.
    pub factor_type: MfaFactorType,
    /// User-provided name for the factor.
    pub display_name: Option<String>,
    /// Phone number for SMS factor (E.164 format).
    pub phone_number: Option<String>,
    /// Email address for email factor.
    pub email: Option<String>,
    /// Whether the factor has been verified.
    pub verified: bool,
    /// Timestamp when the factor was enrolled.
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp when the factor was last used for authentication.
    pub last_used_at: Option<DateTime<Utc>>,
}

/// The type of multi-factor authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MfaFactorType {
    /// Unspecified factor type.
    #[default]
    Unspecified,
    /// Time-based One-Time Password using authenticator apps.
    Totp,
    /// SMS-based verification code.
    Sms,
    /// Email-based verification code.
    Email,
}

// =============================================================================
// ListProjectUserMfaFactors
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListProjectUserMfaFactorsError {
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

/// Request to list MFA factors for a project-level user.
#[derive(Debug, Clone)]
pub struct ListProjectUserMfaFactorsRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
}

impl ListProjectUserMfaFactorsRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            pistachio_id,
        }
    }
}

/// Response from listing MFA factors for a project user.
#[derive(Debug, Clone)]
pub struct ListProjectUserMfaFactorsResponse {
    /// The list of enrolled MFA factors.
    pub factors: Vec<MfaFactor>,
}

// =============================================================================
// DeleteProjectUserMfaFactor
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteProjectUserMfaFactorError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Factor not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to delete an MFA factor from a project-level user.
#[derive(Debug, Clone)]
pub struct DeleteProjectUserMfaFactorRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// The factor ID to delete.
    pub factor_id: String,
}

impl DeleteProjectUserMfaFactorRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, pistachio_id: UserId, factor_id: impl Into<String>) -> Self {
        Self {
            project_id,
            pistachio_id,
            factor_id: factor_id.into(),
        }
    }
}

/// Response from deleting an MFA factor.
#[derive(Debug, Clone)]
pub struct DeleteProjectUserMfaFactorResponse {
    // Empty response - the MFA factor has been removed.
}

// =============================================================================
// ListTenantUserMfaFactors
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListTenantUserMfaFactorsError {
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

/// Request to list MFA factors for a tenant-level user.
#[derive(Debug, Clone)]
pub struct ListTenantUserMfaFactorsRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID that owns the user.
    pub tenant_id: TenantId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
}

impl ListTenantUserMfaFactorsRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            tenant_id,
            pistachio_id,
        }
    }
}

/// Response from listing MFA factors for a tenant user.
#[derive(Debug, Clone)]
pub struct ListTenantUserMfaFactorsResponse {
    /// The list of enrolled MFA factors.
    pub factors: Vec<MfaFactor>,
}

// =============================================================================
// DeleteTenantUserMfaFactor
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteTenantUserMfaFactorError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Factor not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to delete an MFA factor from a tenant-level user.
#[derive(Debug, Clone)]
pub struct DeleteTenantUserMfaFactorRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID that owns the user.
    pub tenant_id: TenantId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// The factor ID to delete.
    pub factor_id: String,
}

impl DeleteTenantUserMfaFactorRequest {
    /// Creates a new request.
    pub fn new(
        project_id: ProjectId,
        tenant_id: TenantId,
        pistachio_id: UserId,
        factor_id: impl Into<String>,
    ) -> Self {
        Self {
            project_id,
            tenant_id,
            pistachio_id,
            factor_id: factor_id.into(),
        }
    }
}

/// Response from deleting an MFA factor.
#[derive(Debug, Clone)]
pub struct DeleteTenantUserMfaFactorResponse {
    // Empty response - the MFA factor has been removed.
}

// =============================================================================
// SetProjectUserCustomClaims
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SetProjectUserCustomClaimsError {
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

/// Request to set custom claims for a project-level user.
#[derive(Debug, Clone)]
pub struct SetProjectUserCustomClaimsRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// Custom claims to set on the user's ID token.
    pub custom_claims: serde_json::Value,
}

impl SetProjectUserCustomClaimsRequest {
    /// Creates a new request.
    pub fn new(
        project_id: ProjectId,
        pistachio_id: UserId,
        custom_claims: serde_json::Value,
    ) -> Self {
        Self {
            project_id,
            pistachio_id,
            custom_claims,
        }
    }
}

/// Response from setting custom claims.
#[derive(Debug, Clone)]
pub struct SetProjectUserCustomClaimsResponse {
    // Empty response - custom claims have been set.
}
