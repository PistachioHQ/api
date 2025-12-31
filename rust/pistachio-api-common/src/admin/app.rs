use libgn::app::{App, AppDisplayName, AppId, Platform, PlatformConfig};
use libgn::project::ProjectId;

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};
use crate::pagination::{PaginationMeta, PaginationParams};
use crate::search::SearchParams;

// =============================================================================
// CreateApp
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateAppError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("App with same identifier already exists in this project")]
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

/// Request to create a new app within a project.
#[derive(Debug, Clone)]
pub struct CreateAppRequest {
    /// The project ID that will own this app.
    pub project_id: ProjectId,
    /// Human-readable display name for the app. Required.
    pub display_name: AppDisplayName,
    /// Target platform for the app. Required.
    pub platform: Platform,
    /// Platform-specific configuration. Required.
    pub platform_config: PlatformConfig,
}

impl CreateAppRequest {
    /// Creates a new request with required fields.
    pub fn new(
        project_id: ProjectId,
        display_name: AppDisplayName,
        platform_config: PlatformConfig,
    ) -> Self {
        Self {
            project_id,
            display_name,
            platform: platform_config.platform(),
            platform_config,
        }
    }
}

/// Response from creating an app.
#[derive(Debug, Clone)]
pub struct CreateAppResponse {
    /// The created app (includes API key on creation).
    pub app: App,
}

// =============================================================================
// GetApp
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetAppError {
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

/// Request to get an app by ID.
#[derive(Debug, Clone)]
pub struct GetAppRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID to retrieve.
    pub app_id: AppId,
}

impl GetAppRequest {
    /// Creates a new request for the given project and app IDs.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self { project_id, app_id }
    }
}

/// Response from getting an app.
#[derive(Debug, Clone)]
pub struct GetAppResponse {
    /// The retrieved app.
    pub app: App,
}

// =============================================================================
// UpdateApp
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateAppError {
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

/// Request to update an app.
#[derive(Debug, Clone)]
pub struct UpdateAppRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID to update.
    pub app_id: AppId,
    /// New display name for the app.
    /// If not provided, the display name will not be changed.
    pub display_name: Option<AppDisplayName>,
    /// Updated platform-specific configuration.
    /// If not provided, the configuration will not be changed.
    pub platform_config: Option<PlatformConfig>,
}

impl UpdateAppRequest {
    /// Creates a new request for the given project and app IDs.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self {
            project_id,
            app_id,
            display_name: None,
            platform_config: None,
        }
    }

    /// Sets the display name to update.
    pub fn with_display_name(mut self, display_name: AppDisplayName) -> Self {
        self.display_name = Some(display_name);
        self
    }

    /// Sets the platform configuration to update.
    pub fn with_platform_config(mut self, platform_config: PlatformConfig) -> Self {
        self.platform_config = Some(platform_config);
        self
    }
}

/// Response from updating an app.
#[derive(Debug, Clone)]
pub struct UpdateAppResponse {
    /// The updated app.
    pub app: App,
}

// =============================================================================
// DeleteApp
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteAppError {
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

/// Request to delete an app (soft delete).
#[derive(Debug, Clone)]
pub struct DeleteAppRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID to delete.
    pub app_id: AppId,
}

impl DeleteAppRequest {
    /// Creates a new request for the given project and app IDs.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self { project_id, app_id }
    }
}

/// Response from deleting an app.
#[derive(Debug, Clone)]
pub struct DeleteAppResponse {
    /// The soft-deleted app with updated state.
    pub app: App,
}

// =============================================================================
// UndeleteApp
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UndeleteAppError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("App not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("App is not in DELETED state")]
    FailedPrecondition,
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

/// Request to restore a soft-deleted app.
#[derive(Debug, Clone)]
pub struct UndeleteAppRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID to restore.
    pub app_id: AppId,
}

impl UndeleteAppRequest {
    /// Creates a new request for the given project and app IDs.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self { project_id, app_id }
    }
}

/// Response from restoring an app.
#[derive(Debug, Clone)]
pub struct UndeleteAppResponse {
    /// The restored app.
    pub app: App,
}

// =============================================================================
// ListApps
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListAppsError {
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

/// Request to list apps within a project.
#[derive(Debug, Clone)]
pub struct ListAppsRequest {
    /// The project ID to list apps from.
    pub project_id: ProjectId,
    /// Whether to include deleted apps in the results.
    pub show_deleted: bool,
    /// Pagination parameters including page size, cursor, and sort.
    pub pagination: PaginationParams,
}

impl ListAppsRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            show_deleted: false,
            pagination: PaginationParams::default(),
        }
    }

    /// Sets whether to include deleted apps.
    pub fn with_show_deleted(mut self, show_deleted: bool) -> Self {
        self.show_deleted = show_deleted;
        self
    }

    /// Sets the pagination parameters.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Response from listing apps.
#[derive(Debug, Clone)]
pub struct ListAppsResponse {
    /// The list of apps.
    pub apps: Vec<App>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// SearchApps
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SearchAppsError {
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

/// Request to search for apps within a project.
#[derive(Debug, Clone)]
pub struct SearchAppsRequest {
    /// The project ID to search apps in.
    pub project_id: ProjectId,
    /// Search parameters including query, sorting, and pagination.
    pub params: SearchParams,
}

impl SearchAppsRequest {
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

/// Response from searching apps.
#[derive(Debug, Clone)]
pub struct SearchAppsResponse {
    /// The list of apps matching the search query.
    pub apps: Vec<App>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// GetAppConfig
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetAppConfigError {
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

/// Request to get SDK configuration for an app.
#[derive(Debug, Clone)]
pub struct GetAppConfigRequest {
    /// The project ID that owns the app.
    pub project_id: ProjectId,
    /// The app ID to get config for.
    pub app_id: AppId,
}

impl GetAppConfigRequest {
    /// Creates a new request for the given project and app IDs.
    pub fn new(project_id: ProjectId, app_id: AppId) -> Self {
        Self { project_id, app_id }
    }
}

/// Response from getting app SDK configuration.
#[derive(Debug, Clone)]
pub struct GetAppConfigResponse {
    /// Contents of the platform-specific config file.
    pub config_file_contents: String,
    /// Recommended filename for the config file.
    pub config_filename: String,
}
