//! User management types for the Admin API.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use libgn::email::Email;
use libgn::pistachio_id::UserId;
use libgn::project::ProjectId;
use libgn::tenant::TenantId;
use libgn::user::{DisplayName, PhoneNumber, PhotoUrl, UserResourceName};

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};
use crate::pagination::{PaginationMeta, PaginationParams};
use crate::search::SearchParams;

/// Represents custom claims for a user's ID token.
///
/// Custom claims are stored as key-value pairs where values are JSON-encoded strings.
/// This type is transport-agnostic; transport layers (gRPC, OpenAPI) handle
/// conversion to/from their native representations.
pub type CustomClaims = HashMap<String, String>;

// =============================================================================
// User Domain Type
// =============================================================================

/// A user account within a project or tenant.
#[derive(Debug, Clone)]
pub struct User {
    /// Resource name in the format:
    /// - Project users: "projects/{project_id}/users/{pistachio_id}"
    /// - Tenant users: "projects/{project_id}/tenants/{tenant_id}/users/{pistachio_id}"
    pub name: UserResourceName,
    /// System-generated user identifier (32 hex chars ending in '00').
    pub pistachio_id: UserId,
    /// Tenant ID this user belongs to. None for project-level users.
    pub tenant_id: Option<TenantId>,
    /// User's primary email address.
    pub email: Option<Email>,
    /// Whether the user's email address has been verified.
    pub email_verified: bool,
    /// User's phone number in E.164 format.
    pub phone_number: Option<PhoneNumber>,
    /// Human-readable display name for the user.
    pub display_name: Option<DisplayName>,
    /// URL to the user's profile photo.
    pub photo_url: Option<PhotoUrl>,
    /// Whether the user account is disabled.
    pub disabled: bool,
    /// Custom claims to include in the user's ID token.
    pub custom_claims: Option<CustomClaims>,
    /// Timestamp when the user account was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp when the user last signed in.
    pub last_sign_in_at: Option<DateTime<Utc>>,
    /// Timestamp when the user's tokens were last refreshed.
    pub last_refresh_at: Option<DateTime<Utc>>,
    /// Timestamp when the user account was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

/// A user record for batch import.
#[derive(Debug, Clone, Default)]
pub struct ImportUserRecord {
    /// User's primary email address.
    pub email: Option<String>,
    /// Whether the email has been verified.
    pub email_verified: bool,
    /// User's phone number in E.164 format.
    pub phone_number: Option<String>,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// URL to the user's profile photo.
    pub photo_url: Option<String>,
    /// Whether the account is disabled.
    pub disabled: bool,
    /// Custom claims for the ID token.
    pub custom_claims: Option<CustomClaims>,
    /// Base64-encoded password hash for PDPKA import.
    pub password_hash: Option<String>,
    /// Base64-encoded salt used with the password hash.
    pub password_salt: Option<String>,
}

/// Hash algorithm for password hashes during import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Scrypt,
    Bcrypt,
    Argon2,
    Pbkdf2Sha256,
}

/// Algorithm-specific configuration for password hashing.
#[derive(Debug, Clone, Default)]
pub struct HashConfig {
    /// Number of rounds for BCRYPT/SCRYPT.
    pub rounds: Option<i32>,
    /// Memory cost parameter for SCRYPT/ARGON2.
    pub memory_cost: Option<i32>,
    /// Parallelization parameter for SCRYPT/ARGON2.
    pub parallelization: Option<i32>,
    /// Base64-encoded salt separator for SCRYPT.
    pub salt_separator: Option<String>,
    /// Base64-encoded signer key for SCRYPT.
    pub signer_key: Option<String>,
}

/// Details about a failed user import.
#[derive(Debug, Clone)]
pub struct ImportUserError {
    /// Zero-based index of the user in the request array.
    pub index: i32,
    /// Error message describing the failure.
    pub message: String,
    /// Field that caused the error, if applicable.
    pub field: Option<String>,
}

// =============================================================================
// CreateProjectUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectUserError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Email or phone number already exists")]
    AlreadyExists,
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to create a new user within a project.
#[derive(Debug, Clone)]
pub struct CreateProjectUserRequest {
    /// The project ID that will own this user.
    pub project_id: ProjectId,
    /// User's email address.
    pub email: Option<String>,
    /// Whether the email has been verified.
    pub email_verified: bool,
    /// User's phone number in E.164 format.
    pub phone_number: Option<String>,
    /// Human-readable display name for the user.
    pub display_name: Option<String>,
    /// URL to the user's profile photo.
    pub photo_url: Option<String>,
    /// Whether to create the user in disabled state.
    pub disabled: bool,
    /// Custom claims for the ID token.
    pub custom_claims: Option<CustomClaims>,
}

impl CreateProjectUserRequest {
    /// Creates a new request with required fields only.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            email: None,
            email_verified: false,
            phone_number: None,
            display_name: None,
            photo_url: None,
            disabled: false,
            custom_claims: None,
        }
    }

    /// Sets the email address.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the email verified status.
    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = verified;
        self
    }

    /// Sets the phone number.
    pub fn with_phone_number(mut self, phone: impl Into<String>) -> Self {
        self.phone_number = Some(phone.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the photo URL.
    pub fn with_photo_url(mut self, url: impl Into<String>) -> Self {
        self.photo_url = Some(url.into());
        self
    }

    /// Sets the disabled status.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the custom claims.
    pub fn with_custom_claims(mut self, claims: CustomClaims) -> Self {
        self.custom_claims = Some(claims);
        self
    }
}

/// Response from creating a project user.
#[derive(Debug, Clone)]
pub struct CreateProjectUserResponse {
    /// The created user.
    pub user: User,
}

// =============================================================================
// GetProjectUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetProjectUserError {
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

/// Request to get a project user by pistachio_id.
#[derive(Debug, Clone)]
pub struct GetProjectUserRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
}

impl GetProjectUserRequest {
    /// Creates a new request for the given project and user IDs.
    pub fn new(project_id: ProjectId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            pistachio_id,
        }
    }
}

/// Response from getting a project user.
#[derive(Debug, Clone)]
pub struct GetProjectUserResponse {
    /// The retrieved user.
    pub user: User,
}

// =============================================================================
// UpdateProjectUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateProjectUserError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Email or phone number already exists")]
    AlreadyExists,
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

/// Request to update a project user.
#[derive(Debug, Clone)]
pub struct UpdateProjectUserRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// New email address.
    pub email: Option<String>,
    /// New email verification status.
    pub email_verified: Option<bool>,
    /// New phone number in E.164 format.
    pub phone_number: Option<String>,
    /// New display name.
    pub display_name: Option<String>,
    /// New photo URL.
    pub photo_url: Option<String>,
    /// New disabled status.
    pub disabled: Option<bool>,
    /// New custom claims. Replaces existing claims if provided.
    pub custom_claims: Option<CustomClaims>,
}

impl UpdateProjectUserRequest {
    /// Creates a new request for the given project and user IDs.
    pub fn new(project_id: ProjectId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            pistachio_id,
            email: None,
            email_verified: None,
            phone_number: None,
            display_name: None,
            photo_url: None,
            disabled: None,
            custom_claims: None,
        }
    }

    /// Sets the email address to update.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the email verified status.
    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = Some(verified);
        self
    }

    /// Sets the phone number.
    pub fn with_phone_number(mut self, phone: impl Into<String>) -> Self {
        self.phone_number = Some(phone.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the photo URL.
    pub fn with_photo_url(mut self, url: impl Into<String>) -> Self {
        self.photo_url = Some(url.into());
        self
    }

    /// Sets the disabled status.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Sets the custom claims.
    pub fn with_custom_claims(mut self, claims: CustomClaims) -> Self {
        self.custom_claims = Some(claims);
        self
    }
}

/// Response from updating a project user.
#[derive(Debug, Clone)]
pub struct UpdateProjectUserResponse {
    /// The updated user.
    pub user: User,
}

// =============================================================================
// DeleteProjectUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteProjectUserError {
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

/// Request to delete a project user.
#[derive(Debug, Clone)]
pub struct DeleteProjectUserRequest {
    /// The project ID that owns the user.
    pub project_id: ProjectId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
}

impl DeleteProjectUserRequest {
    /// Creates a new request for the given project and user IDs.
    pub fn new(project_id: ProjectId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            pistachio_id,
        }
    }
}

/// Response from deleting a project user.
#[derive(Debug, Clone)]
pub struct DeleteProjectUserResponse {
    // Empty response - the user has been permanently deleted.
}

// =============================================================================
// ListProjectUsers
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListProjectUsersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to list users within a project.
#[derive(Debug, Clone)]
pub struct ListProjectUsersRequest {
    /// The project ID to list users from.
    pub project_id: ProjectId,
    /// Pagination parameters.
    pub pagination: PaginationParams,
}

impl ListProjectUsersRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            pagination: PaginationParams::default(),
        }
    }

    /// Sets the pagination parameters.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Response from listing project users.
#[derive(Debug, Clone)]
pub struct ListProjectUsersResponse {
    /// The list of users.
    pub users: Vec<User>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// ImportProjectUsers
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ImportProjectUsersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to import users into a project.
#[derive(Debug, Clone)]
pub struct ImportProjectUsersRequest {
    /// The project ID to import users into.
    pub project_id: ProjectId,
    /// Array of user records to import (max 1000).
    pub users: Vec<ImportUserRecord>,
    /// Hash algorithm for password hashes.
    pub hash_algorithm: Option<HashAlgorithm>,
    /// Configuration for the hash algorithm.
    pub hash_config: Option<HashConfig>,
}

impl ImportProjectUsersRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId, users: Vec<ImportUserRecord>) -> Self {
        Self {
            project_id,
            users,
            hash_algorithm: None,
            hash_config: None,
        }
    }

    /// Sets the hash algorithm.
    pub fn with_hash_algorithm(mut self, algorithm: HashAlgorithm) -> Self {
        self.hash_algorithm = Some(algorithm);
        self
    }

    /// Sets the hash configuration.
    pub fn with_hash_config(mut self, config: HashConfig) -> Self {
        self.hash_config = Some(config);
        self
    }
}

/// Response from importing project users.
#[derive(Debug, Clone)]
pub struct ImportProjectUsersResponse {
    /// Number of users successfully imported.
    pub success_count: i32,
    /// Number of users that failed to import.
    pub failure_count: i32,
    /// Details of import failures.
    pub errors: Vec<ImportUserError>,
}

// =============================================================================
// SearchProjectUsers
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SearchProjectUsersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to search for users within a project.
#[derive(Debug, Clone)]
pub struct SearchProjectUsersRequest {
    /// The project ID to search users in.
    pub project_id: ProjectId,
    /// Search parameters including query, sorting, and pagination.
    pub params: SearchParams,
}

impl SearchProjectUsersRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            params: SearchParams::default(),
        }
    }

    /// Sets the search parameters.
    pub fn with_params(mut self, params: SearchParams) -> Self {
        self.params = params;
        self
    }

    /// Sets the search query.
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.params.query = query.into();
        self
    }
}

/// Response from searching project users.
#[derive(Debug, Clone)]
pub struct SearchProjectUsersResponse {
    /// The list of users matching the search query.
    pub users: Vec<User>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// CreateTenantUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateTenantUserError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Email or phone number already exists")]
    AlreadyExists,
    #[error("Project or tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to create a new user within a tenant.
#[derive(Debug, Clone)]
pub struct CreateTenantUserRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID that will own this user.
    pub tenant_id: TenantId,
    /// User's email address.
    pub email: Option<String>,
    /// Whether the email has been verified.
    pub email_verified: bool,
    /// User's phone number in E.164 format.
    pub phone_number: Option<String>,
    /// Human-readable display name for the user.
    pub display_name: Option<String>,
    /// URL to the user's profile photo.
    pub photo_url: Option<String>,
    /// Whether to create the user in disabled state.
    pub disabled: bool,
    /// Custom claims for the ID token.
    pub custom_claims: Option<CustomClaims>,
}

impl CreateTenantUserRequest {
    /// Creates a new request with required fields only.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
            email: None,
            email_verified: false,
            phone_number: None,
            display_name: None,
            photo_url: None,
            disabled: false,
            custom_claims: None,
        }
    }

    /// Sets the email address.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the email verified status.
    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = verified;
        self
    }

    /// Sets the phone number.
    pub fn with_phone_number(mut self, phone: impl Into<String>) -> Self {
        self.phone_number = Some(phone.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the photo URL.
    pub fn with_photo_url(mut self, url: impl Into<String>) -> Self {
        self.photo_url = Some(url.into());
        self
    }

    /// Sets the disabled status.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the custom claims.
    pub fn with_custom_claims(mut self, claims: CustomClaims) -> Self {
        self.custom_claims = Some(claims);
        self
    }
}

/// Response from creating a tenant user.
#[derive(Debug, Clone)]
pub struct CreateTenantUserResponse {
    /// The created user.
    pub user: User,
}

// =============================================================================
// GetTenantUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetTenantUserError {
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

/// Request to get a tenant user by pistachio_id.
#[derive(Debug, Clone)]
pub struct GetTenantUserRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID that owns the user.
    pub tenant_id: TenantId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
}

impl GetTenantUserRequest {
    /// Creates a new request for the given project, tenant, and user IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            tenant_id,
            pistachio_id,
        }
    }
}

/// Response from getting a tenant user.
#[derive(Debug, Clone)]
pub struct GetTenantUserResponse {
    /// The retrieved user.
    pub user: User,
}

// =============================================================================
// UpdateTenantUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateTenantUserError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Email or phone number already exists")]
    AlreadyExists,
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

/// Request to update a tenant user.
#[derive(Debug, Clone)]
pub struct UpdateTenantUserRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID that owns the user.
    pub tenant_id: TenantId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
    /// New email address.
    pub email: Option<String>,
    /// New email verification status.
    pub email_verified: Option<bool>,
    /// New phone number in E.164 format.
    pub phone_number: Option<String>,
    /// New display name.
    pub display_name: Option<String>,
    /// New photo URL.
    pub photo_url: Option<String>,
    /// New disabled status.
    pub disabled: Option<bool>,
    /// New custom claims. Replaces existing claims if provided.
    pub custom_claims: Option<CustomClaims>,
}

impl UpdateTenantUserRequest {
    /// Creates a new request for the given project, tenant, and user IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            tenant_id,
            pistachio_id,
            email: None,
            email_verified: None,
            phone_number: None,
            display_name: None,
            photo_url: None,
            disabled: None,
            custom_claims: None,
        }
    }

    /// Sets the email address to update.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the email verified status.
    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = Some(verified);
        self
    }

    /// Sets the phone number.
    pub fn with_phone_number(mut self, phone: impl Into<String>) -> Self {
        self.phone_number = Some(phone.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the photo URL.
    pub fn with_photo_url(mut self, url: impl Into<String>) -> Self {
        self.photo_url = Some(url.into());
        self
    }

    /// Sets the disabled status.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Sets the custom claims.
    pub fn with_custom_claims(mut self, claims: CustomClaims) -> Self {
        self.custom_claims = Some(claims);
        self
    }
}

/// Response from updating a tenant user.
#[derive(Debug, Clone)]
pub struct UpdateTenantUserResponse {
    /// The updated user.
    pub user: User,
}

// =============================================================================
// DeleteTenantUser
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteTenantUserError {
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

/// Request to delete a tenant user.
#[derive(Debug, Clone)]
pub struct DeleteTenantUserRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID that owns the user.
    pub tenant_id: TenantId,
    /// The user's pistachio_id.
    pub pistachio_id: UserId,
}

impl DeleteTenantUserRequest {
    /// Creates a new request for the given project, tenant, and user IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, pistachio_id: UserId) -> Self {
        Self {
            project_id,
            tenant_id,
            pistachio_id,
        }
    }
}

/// Response from deleting a tenant user.
#[derive(Debug, Clone)]
pub struct DeleteTenantUserResponse {
    // Empty response - the user has been permanently deleted.
}

// =============================================================================
// ListTenantUsers
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListTenantUsersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project or tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to list users within a tenant.
#[derive(Debug, Clone)]
pub struct ListTenantUsersRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID to list users from.
    pub tenant_id: TenantId,
    /// Pagination parameters.
    pub pagination: PaginationParams,
}

impl ListTenantUsersRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
            pagination: PaginationParams::default(),
        }
    }

    /// Sets the pagination parameters.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Response from listing tenant users.
#[derive(Debug, Clone)]
pub struct ListTenantUsersResponse {
    /// The list of users.
    pub users: Vec<User>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// ImportTenantUsers
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ImportTenantUsersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project or tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to import users into a tenant.
#[derive(Debug, Clone)]
pub struct ImportTenantUsersRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID to import users into.
    pub tenant_id: TenantId,
    /// Array of user records to import (max 1000).
    pub users: Vec<ImportUserRecord>,
    /// Hash algorithm for password hashes.
    pub hash_algorithm: Option<HashAlgorithm>,
    /// Configuration for the hash algorithm.
    pub hash_config: Option<HashConfig>,
}

impl ImportTenantUsersRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, users: Vec<ImportUserRecord>) -> Self {
        Self {
            project_id,
            tenant_id,
            users,
            hash_algorithm: None,
            hash_config: None,
        }
    }

    /// Sets the hash algorithm.
    pub fn with_hash_algorithm(mut self, algorithm: HashAlgorithm) -> Self {
        self.hash_algorithm = Some(algorithm);
        self
    }

    /// Sets the hash configuration.
    pub fn with_hash_config(mut self, config: HashConfig) -> Self {
        self.hash_config = Some(config);
        self
    }
}

/// Response from importing tenant users.
#[derive(Debug, Clone)]
pub struct ImportTenantUsersResponse {
    /// Number of users successfully imported.
    pub success_count: i32,
    /// Number of users that failed to import.
    pub failure_count: i32,
    /// Details of import failures.
    pub errors: Vec<ImportUserError>,
}

// =============================================================================
// SearchTenantUsers
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SearchTenantUsersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project or tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to search for users within a tenant.
#[derive(Debug, Clone)]
pub struct SearchTenantUsersRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID to search users in.
    pub tenant_id: TenantId,
    /// Search parameters including query, sorting, and pagination.
    pub params: SearchParams,
}

impl SearchTenantUsersRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
            params: SearchParams::default(),
        }
    }

    /// Sets the search parameters.
    pub fn with_params(mut self, params: SearchParams) -> Self {
        self.params = params;
        self
    }

    /// Sets the search query.
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.params.query = query.into();
        self
    }
}

/// Response from searching tenant users.
#[derive(Debug, Clone)]
pub struct SearchTenantUsersResponse {
    /// The list of users matching the search query.
    pub users: Vec<User>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}
