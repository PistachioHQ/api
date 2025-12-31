use libgn::project::{Project, ProjectDisplayName, ProjectId, ProjectInvitationCode};

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};
use crate::pagination::{PaginationMeta, PaginationParams};
use crate::search::SearchParams;

// =============================================================================
// CreateProject
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project ID already exists")]
    AlreadyExists,
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

/// Request to create a new project.
#[derive(Debug, Clone, Default)]
pub struct CreateProjectRequest {
    /// Desired project ID.
    /// If not provided, a unique ID will be generated.
    /// Must be 6-30 characters, lowercase alphanumeric with hyphens.
    pub project_id: Option<ProjectId>,
    /// Human-readable display name for the project.
    /// If not provided, project_id will be used as the display name.
    pub display_name: Option<ProjectDisplayName>,
    /// Invitation code for project creation.
    /// Must be a valid 16 hex character code.
    pub invitation_code: Option<ProjectInvitationCode>,
}

impl CreateProjectRequest {
    /// Creates a new request with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the project ID.
    pub fn with_project_id(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, display_name: ProjectDisplayName) -> Self {
        self.display_name = Some(display_name);
        self
    }

    /// Sets the invitation code.
    pub fn with_invitation_code(mut self, invitation_code: ProjectInvitationCode) -> Self {
        self.invitation_code = Some(invitation_code);
        self
    }
}

/// Response from creating a project.
#[derive(Debug, Clone)]
pub struct CreateProjectResponse {
    /// The created project.
    pub project: Project,
}

// =============================================================================
// GetProject
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetProjectError {
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

/// Request to get a project by ID.
#[derive(Debug, Clone)]
pub struct GetProjectRequest {
    /// The project ID to retrieve.
    pub project_id: ProjectId,
}

impl GetProjectRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

/// Response from getting a project.
#[derive(Debug, Clone)]
pub struct GetProjectResponse {
    /// The retrieved project.
    pub project: Project,
}

// =============================================================================
// UpdateProject
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateProjectError {
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

/// Request to update a project.
#[derive(Debug, Clone)]
pub struct UpdateProjectRequest {
    /// The project ID to update.
    pub project_id: ProjectId,
    /// New display name for the project.
    /// If not provided, the display name will not be changed.
    pub display_name: Option<ProjectDisplayName>,
}

impl UpdateProjectRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            display_name: None,
        }
    }

    /// Sets the display name to update.
    pub fn with_display_name(mut self, display_name: ProjectDisplayName) -> Self {
        self.display_name = Some(display_name);
        self
    }
}

/// Response from updating a project.
#[derive(Debug, Clone)]
pub struct UpdateProjectResponse {
    /// The updated project.
    pub project: Project,
}

// =============================================================================
// DeleteProject
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeleteProjectError {
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

/// Request to delete a project.
#[derive(Debug, Clone)]
pub struct DeleteProjectRequest {
    /// The project ID to delete.
    pub project_id: ProjectId,
}

impl DeleteProjectRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

/// Response from deleting a project.
#[derive(Debug, Clone)]
pub struct DeleteProjectResponse {
    /// The soft-deleted project with updated state.
    pub project: Project,
}

// =============================================================================
// UndeleteProject
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UndeleteProjectError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Project is not in DELETED state")]
    FailedPrecondition(String),
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

/// Request to restore a soft-deleted project.
#[derive(Debug, Clone)]
pub struct UndeleteProjectRequest {
    /// The project ID to restore.
    pub project_id: ProjectId,
}

impl UndeleteProjectRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

/// Response from restoring a project.
#[derive(Debug, Clone)]
pub struct UndeleteProjectResponse {
    /// The restored project.
    pub project: Project,
}

// =============================================================================
// ListProjects
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ListProjectsError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
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

/// Request to list projects.
#[derive(Debug, Clone, Default)]
pub struct ListProjectsRequest {
    /// Pagination parameters including page size, cursor, and sort.
    pub pagination: PaginationParams,
    /// If true, include deleted projects in the results.
    /// Defaults to false.
    pub show_deleted: bool,
}

impl ListProjectsRequest {
    /// Creates a new request with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pagination parameters.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    /// Sets whether to include deleted projects.
    pub fn with_show_deleted(mut self, show_deleted: bool) -> Self {
        self.show_deleted = show_deleted;
        self
    }
}

/// Response from listing projects.
#[derive(Debug, Clone)]
pub struct ListProjectsResponse {
    /// The list of projects.
    pub projects: Vec<Project>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// SearchProjects
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SearchProjectsError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
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

/// Request to search for projects.
#[derive(Debug, Clone, Default)]
pub struct SearchProjectsRequest {
    /// Search parameters including query, sorting, and pagination.
    pub params: SearchParams,
}

impl SearchProjectsRequest {
    /// Creates a new request with default values.
    pub fn new() -> Self {
        Self::default()
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

/// Response from searching projects.
#[derive(Debug, Clone)]
pub struct SearchProjectsResponse {
    /// The list of projects matching the search query.
    pub projects: Vec<Project>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

// =============================================================================
// GetAdminSdkConfig
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetAdminSdkConfigError {
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

/// Request to get Admin SDK configuration for a project.
#[derive(Debug, Clone)]
pub struct GetAdminSdkConfigRequest {
    /// The project ID.
    pub project_id: ProjectId,
}

impl GetAdminSdkConfigRequest {
    /// Creates a new request for the given project ID.
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

/// Response containing the Admin SDK configuration.
#[derive(Debug, Clone)]
pub struct GetAdminSdkConfigResponse {
    /// Project ID.
    pub project_id: ProjectId,
    /// Storage bucket name.
    pub storage_bucket: String,
    /// Location/region for default resources.
    pub location_id: String,
}
