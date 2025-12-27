use libgn::project::ProjectId;
use libgn::tenant::{Tenant, TenantDisplayName, TenantId};

use crate::error::{PistachioApiClientError, ValidationError};
use crate::pagination::{PaginationMeta, PaginationParams};
use crate::search::SearchParams;

// =============================================================================
// CreateTenant
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateTenantError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Tenant ID already exists in this project")]
    AlreadyExists,
    #[error("Project not found")]
    NotFound,
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

/// Request to create a new tenant within a project.
#[derive(Debug, Clone)]
pub struct CreateTenantRequest {
    /// The project ID that will own this tenant.
    pub project_id: ProjectId,
    /// Desired tenant ID.
    /// If not provided, a unique ID will be generated.
    /// Must be 1-128 characters, alphanumeric with underscores, case-sensitive.
    pub tenant_id: Option<TenantId>,
    /// Human-readable display name for the tenant. Required.
    pub display_name: TenantDisplayName,
    /// Whether passphrase-derived public key authentication (PDPKA) sign-up is allowed.
    /// Defaults to true if not specified.
    pub allow_pdpka_signup: Option<bool>,
    /// Whether all authentication is disabled for this tenant.
    /// Defaults to false if not specified.
    pub disable_auth: Option<bool>,
    /// Enabled MFA providers for this tenant.
    /// Valid values: "phone", "totp"
    pub mfa_config: Vec<String>,
}

impl CreateTenantRequest {
    /// Creates a new request with required fields.
    pub fn new(project_id: ProjectId, display_name: TenantDisplayName) -> Self {
        Self {
            project_id,
            tenant_id: None,
            display_name,
            allow_pdpka_signup: None,
            disable_auth: None,
            mfa_config: Vec::new(),
        }
    }

    /// Sets the tenant ID.
    pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Sets whether PDPKA sign-up is allowed.
    pub fn with_allow_pdpka_signup(mut self, allow: bool) -> Self {
        self.allow_pdpka_signup = Some(allow);
        self
    }

    /// Sets whether authentication is disabled.
    pub fn with_disable_auth(mut self, disable: bool) -> Self {
        self.disable_auth = Some(disable);
        self
    }

    /// Sets the MFA configuration.
    pub fn with_mfa_config(mut self, mfa_config: Vec<String>) -> Self {
        self.mfa_config = mfa_config;
        self
    }
}

/// Response from creating a tenant.
#[derive(Debug, Clone)]
pub struct CreateTenantResponse {
    /// The created tenant.
    pub tenant: Tenant,
}

// =============================================================================
// GetTenant
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetTenantError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Tenant not found")]
    NotFound,
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

/// Request to get a tenant by ID.
#[derive(Debug, Clone)]
pub struct GetTenantRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID to retrieve.
    pub tenant_id: TenantId,
}

impl GetTenantRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
        }
    }
}

/// Response from getting a tenant.
#[derive(Debug, Clone)]
pub struct GetTenantResponse {
    /// The retrieved tenant.
    pub tenant: Tenant,
}

// =============================================================================
// UpdateTenant
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateTenantError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Tenant not found")]
    NotFound,
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

/// Request to update a tenant.
#[derive(Debug, Clone)]
pub struct UpdateTenantRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID to update.
    pub tenant_id: TenantId,
    /// New display name for the tenant.
    /// If not provided, the display name will not be changed.
    pub display_name: Option<TenantDisplayName>,
    /// Whether PDPKA sign-up is allowed.
    /// If not provided, the value will not be changed.
    pub allow_pdpka_signup: Option<bool>,
    /// Whether authentication is disabled.
    /// If not provided, the value will not be changed.
    pub disable_auth: Option<bool>,
    /// Enabled MFA providers for this tenant.
    /// If Some, replaces the existing configuration.
    /// If None, the value will not be changed.
    pub mfa_config: Option<Vec<String>>,
}

impl UpdateTenantRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
            display_name: None,
            allow_pdpka_signup: None,
            disable_auth: None,
            mfa_config: None,
        }
    }

    /// Sets the display name to update.
    pub fn with_display_name(mut self, display_name: TenantDisplayName) -> Self {
        self.display_name = Some(display_name);
        self
    }

    /// Sets whether PDPKA sign-up is allowed.
    pub fn with_allow_pdpka_signup(mut self, allow: bool) -> Self {
        self.allow_pdpka_signup = Some(allow);
        self
    }

    /// Sets whether authentication is disabled.
    pub fn with_disable_auth(mut self, disable: bool) -> Self {
        self.disable_auth = Some(disable);
        self
    }

    /// Sets the MFA configuration.
    pub fn with_mfa_config(mut self, mfa_config: Vec<String>) -> Self {
        self.mfa_config = Some(mfa_config);
        self
    }
}

/// Response from updating a tenant.
#[derive(Debug, Clone)]
pub struct UpdateTenantResponse {
    /// The updated tenant.
    pub tenant: Tenant,
}

// =============================================================================
// DeleteTenant
// =============================================================================

/// Error type for delete tenant operations.
///
/// Note: Unlike other error types, this intentionally omits `ResponseValidationError`
/// because delete operations return an empty response body with no fields to validate.
#[derive(Debug, thiserror::Error)]
pub enum DeleteTenantError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Tenant not found")]
    NotFound,
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

/// Request to delete a tenant.
#[derive(Debug, Clone)]
pub struct DeleteTenantRequest {
    /// The project ID that owns the tenant.
    pub project_id: ProjectId,
    /// The tenant ID to delete.
    pub tenant_id: TenantId,
}

impl DeleteTenantRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
        }
    }
}

/// Response from deleting a tenant.
#[derive(Debug, Clone)]
pub struct DeleteTenantResponse {
    // Empty response - the tenant has been permanently deleted.
}

// =============================================================================
// ListTenants
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListTenantsError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Project not found")]
    NotFound,
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

/// Request to list tenants within a project.
#[derive(Debug, Clone)]
pub struct ListTenantsRequest {
    /// The project ID to list tenants from.
    pub project_id: ProjectId,
    /// Pagination parameters including page size, cursor, and sort.
    pub pagination: PaginationParams,
}

impl ListTenantsRequest {
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

/// Response from listing tenants.
#[derive(Debug, Clone)]
pub struct ListTenantsResponse {
    /// The list of tenants.
    pub tenants: Vec<Tenant>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// SearchTenants
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SearchTenantsError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Project not found")]
    NotFound,
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

/// Request to search for tenants within a project.
#[derive(Debug, Clone)]
pub struct SearchTenantsRequest {
    /// The project ID to search tenants in.
    pub project_id: ProjectId,
    /// Search parameters including query, sorting, and pagination.
    pub params: SearchParams,
}

impl SearchTenantsRequest {
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

/// Response from searching tenants.
#[derive(Debug, Clone)]
pub struct SearchTenantsResponse {
    /// The list of tenants matching the search query.
    pub tenants: Vec<Tenant>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}
