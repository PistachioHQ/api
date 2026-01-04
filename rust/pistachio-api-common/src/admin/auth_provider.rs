//! Auth provider management types for the Admin API.
//!
//! Auth providers configure authentication methods at project and tenant levels.
//! Supported providers: PDPKA, Anonymous, OAuth (Google, Apple, etc.), and OIDC.

use std::collections::HashMap;

use libgn::project::ProjectId;
use libgn::tenant::TenantId;

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};

// =============================================================================
// Common Types
// =============================================================================

// Re-export ProviderId from libgn for convenience.
pub use libgn::auth_provider::{ProviderId, ProviderIdError};

/// Configuration source for effective auth providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Configuration comes from the project level.
    Project,
    /// Configuration comes from the tenant level (overrides project).
    Tenant,
}

/// Configuration for passphrase-derived public key authentication (PDPKA).
#[derive(Debug, Clone, Default)]
pub struct PdpkaConfig {
    /// Whether new users can sign up using PDPKA.
    pub allow_signup: bool,
}

/// Configuration for anonymous authentication.
#[derive(Debug, Clone, Default)]
pub struct AnonymousConfig {
    /// Duration in seconds for anonymous sessions.
    pub session_duration_seconds: i32,
    /// Whether to automatically upgrade anonymous users to permanent accounts.
    pub auto_upgrade: bool,
}

/// Configuration for OAuth providers (Google, Apple, etc.).
#[derive(Debug, Clone, Default)]
pub struct OAuthConfig {
    /// OAuth client ID from the provider.
    pub client_id: String,
    /// OAuth scopes to request during authentication.
    pub scopes: Vec<String>,
    /// For Google Workspace: restrict sign-in to specific domains.
    pub allowed_hosted_domains: Vec<String>,
}

/// Configuration for generic OpenID Connect providers.
#[derive(Debug, Clone, Default)]
pub struct OidcConfig {
    /// Human-readable name for the provider shown in the sign-in UI.
    pub display_name: String,
    /// The OIDC issuer URL (must support discovery).
    pub issuer_url: String,
    /// OIDC client ID.
    pub client_id: String,
    /// OIDC scopes to request.
    pub scopes: Vec<String>,
    /// Additional parameters to include in authorization requests.
    pub additional_params: HashMap<String, String>,
}

/// Provider-specific configuration.
#[derive(Debug, Clone)]
pub enum AuthProviderConfig {
    /// Passphrase-derived public key authentication.
    Pdpka(PdpkaConfig),
    /// Anonymous authentication.
    Anonymous(AnonymousConfig),
    /// OAuth-based authentication (Google, Apple, etc.).
    OAuth(OAuthConfig),
    /// OpenID Connect authentication.
    Oidc(OidcConfig),
}

/// An authentication provider configured at the project level.
#[derive(Debug, Clone)]
pub struct AuthProvider {
    /// Provider identifier (e.g., "pdpka", "google", "oidc:okta").
    pub provider_id: ProviderId,
    /// Whether this provider is enabled for authentication.
    pub enabled: bool,
    /// Display order in sign-in UI. Lower values appear first.
    pub display_order: i32,
    /// Provider-specific configuration.
    pub config: Option<AuthProviderConfig>,
    /// Timestamp when the provider was configured.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp when the provider was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Tenant-level override for an auth provider configuration.
#[derive(Debug, Clone)]
pub struct TenantAuthProviderOverride {
    /// Provider identifier being overridden.
    pub provider_id: ProviderId,
    /// Override for enabled state. None = inherit from project.
    pub enabled: Option<bool>,
    /// Override for display order. None = inherit from project.
    pub display_order: Option<i32>,
    /// Provider-specific configuration override.
    pub config: Option<AuthProviderConfig>,
    /// Timestamp when the override was created.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp when the override was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// The effective auth provider configuration after merging project and tenant.
#[derive(Debug, Clone)]
pub struct EffectiveAuthProvider {
    /// The merged provider configuration.
    pub provider: AuthProvider,
    /// Source of the configuration (PROJECT or TENANT).
    pub source: ConfigSource,
}

// =============================================================================
// ListProjectAuthProviders
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListProjectAuthProvidersError {
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

/// Request to list all auth providers for a project.
#[derive(Debug, Clone)]
pub struct ListProjectAuthProvidersRequest {
    /// The project ID to list auth providers for.
    pub project_id: ProjectId,
}

impl ListProjectAuthProvidersRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

/// Response containing the list of project auth providers.
#[derive(Debug, Clone)]
pub struct ListProjectAuthProvidersResponse {
    /// List of configured auth providers, ordered by display_order.
    pub providers: Vec<AuthProvider>,
}

// =============================================================================
// GetProjectAuthProvider
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetProjectAuthProviderError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Auth provider not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to get a specific auth provider for a project.
#[derive(Debug, Clone)]
pub struct GetProjectAuthProviderRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The provider ID to retrieve.
    pub provider_id: ProviderId,
}

impl GetProjectAuthProviderRequest {
    /// Creates a new request for the given project and provider IDs.
    pub fn new(project_id: ProjectId, provider_id: ProviderId) -> Self {
        Self {
            project_id,
            provider_id,
        }
    }
}

/// Response containing a project auth provider.
#[derive(Debug, Clone)]
pub struct GetProjectAuthProviderResponse {
    /// The auth provider configuration.
    pub provider: AuthProvider,
}

// =============================================================================
// UpdateProjectAuthProvider
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateProjectAuthProviderError {
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

/// Request to create or update an auth provider for a project.
#[derive(Debug, Clone)]
pub struct UpdateProjectAuthProviderRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The provider ID to create or update.
    pub provider_id: ProviderId,
    /// Whether this provider is enabled.
    pub enabled: Option<bool>,
    /// Display order in sign-in UI.
    pub display_order: Option<i32>,
    /// Provider-specific configuration.
    pub config: Option<AuthProviderConfig>,
    /// OAuth/OIDC client secret (write-only).
    pub client_secret: Option<String>,
}

impl UpdateProjectAuthProviderRequest {
    /// Creates a new request for the given project and provider IDs.
    pub fn new(project_id: ProjectId, provider_id: ProviderId) -> Self {
        Self {
            project_id,
            provider_id,
            enabled: None,
            display_order: None,
            config: None,
            client_secret: None,
        }
    }

    /// Sets whether the provider is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the display order.
    pub fn with_display_order(mut self, display_order: i32) -> Self {
        self.display_order = Some(display_order);
        self
    }

    /// Sets the provider configuration.
    pub fn with_config(mut self, config: AuthProviderConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the client secret (for OAuth/OIDC providers).
    pub fn with_client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }
}

/// Response containing the created or updated auth provider.
#[derive(Debug, Clone)]
pub struct UpdateProjectAuthProviderResponse {
    /// The auth provider configuration.
    pub provider: AuthProvider,
}

// =============================================================================
// DeleteProjectAuthProvider
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteProjectAuthProviderError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Auth provider not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to delete an auth provider from a project.
#[derive(Debug, Clone)]
pub struct DeleteProjectAuthProviderRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The provider ID to delete.
    pub provider_id: ProviderId,
}

impl DeleteProjectAuthProviderRequest {
    /// Creates a new request for the given project and provider IDs.
    pub fn new(project_id: ProjectId, provider_id: ProviderId) -> Self {
        Self {
            project_id,
            provider_id,
        }
    }
}

/// Response from deleting an auth provider.
#[derive(Debug, Clone)]
pub struct DeleteProjectAuthProviderResponse {
    // Empty response - the provider has been permanently deleted.
}

// =============================================================================
// ListTenantAuthProviders
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListTenantAuthProvidersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to list tenant auth provider overrides.
#[derive(Debug, Clone)]
pub struct ListTenantAuthProvidersRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The tenant ID.
    pub tenant_id: TenantId,
}

impl ListTenantAuthProvidersRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
        }
    }
}

/// Response containing tenant auth provider overrides.
#[derive(Debug, Clone)]
pub struct ListTenantAuthProvidersResponse {
    /// List of tenant-level overrides.
    pub overrides: Vec<TenantAuthProviderOverride>,
}

// =============================================================================
// UpdateTenantAuthProvider
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateTenantAuthProviderError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to create or update a tenant auth provider override.
#[derive(Debug, Clone)]
pub struct UpdateTenantAuthProviderRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The tenant ID.
    pub tenant_id: TenantId,
    /// The provider ID to override.
    pub provider_id: ProviderId,
    /// Override for enabled state. None = inherit from project.
    pub enabled: Option<bool>,
    /// Override for display order. None = inherit from project.
    pub display_order: Option<i32>,
    /// Provider-specific configuration override.
    pub config: Option<AuthProviderConfig>,
    /// OAuth/OIDC client secret override (write-only).
    pub client_secret: Option<String>,
}

impl UpdateTenantAuthProviderRequest {
    /// Creates a new request for the given project, tenant, and provider IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, provider_id: ProviderId) -> Self {
        Self {
            project_id,
            tenant_id,
            provider_id,
            enabled: None,
            display_order: None,
            config: None,
            client_secret: None,
        }
    }

    /// Sets the enabled override.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the display order override.
    pub fn with_display_order(mut self, display_order: i32) -> Self {
        self.display_order = Some(display_order);
        self
    }

    /// Sets the config override.
    pub fn with_config(mut self, config: AuthProviderConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the client secret override.
    pub fn with_client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }
}

/// Response containing the created or updated tenant override.
#[derive(Debug, Clone)]
pub struct UpdateTenantAuthProviderResponse {
    /// The tenant auth provider override.
    pub override_: TenantAuthProviderOverride,
}

// =============================================================================
// DeleteTenantAuthProvider
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteTenantAuthProviderError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Tenant auth provider not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to delete a tenant auth provider override.
#[derive(Debug, Clone)]
pub struct DeleteTenantAuthProviderRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The tenant ID.
    pub tenant_id: TenantId,
    /// The provider ID override to delete.
    pub provider_id: ProviderId,
}

impl DeleteTenantAuthProviderRequest {
    /// Creates a new request for the given project, tenant, and provider IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId, provider_id: ProviderId) -> Self {
        Self {
            project_id,
            tenant_id,
            provider_id,
        }
    }
}

/// Response from deleting a tenant auth provider override.
#[derive(Debug, Clone)]
pub struct DeleteTenantAuthProviderResponse {
    // Empty response - the override has been deleted.
}

// =============================================================================
// GetEffectiveTenantAuthProviders
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetEffectiveTenantAuthProvidersError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Tenant not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
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

/// Request to get effective tenant auth providers.
#[derive(Debug, Clone)]
pub struct GetEffectiveTenantAuthProvidersRequest {
    /// The project ID.
    pub project_id: ProjectId,
    /// The tenant ID.
    pub tenant_id: TenantId,
    /// If true, only return enabled providers.
    pub enabled_only: bool,
}

impl GetEffectiveTenantAuthProvidersRequest {
    /// Creates a new request for the given project and tenant IDs.
    pub fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
            enabled_only: false,
        }
    }

    /// Sets whether to only return enabled providers.
    pub fn with_enabled_only(mut self, enabled_only: bool) -> Self {
        self.enabled_only = enabled_only;
        self
    }
}

/// Response containing effective auth provider configuration.
#[derive(Debug, Clone)]
pub struct GetEffectiveTenantAuthProvidersResponse {
    /// List of effective auth providers, ordered by display_order.
    pub providers: Vec<EffectiveAuthProvider>,
}
