use crate::error::{PistachioApiClientError, ValidationError};

/// Project is the top-level container for apps, users, and resources.
#[derive(Debug, Clone)]
pub struct Project {
    /// Unique project identifier.
    pub project_id: String,
    /// Resource name in the format "projects/{project_id}".
    pub name: String,
    /// Internal project identifier (pistachio_id).
    pub pistachio_id: String,
    /// Human-readable display name for the project.
    pub display_name: String,
    /// Current state of the project.
    pub state: ProjectState,
    /// Default resources provisioned for this project.
    pub resources: Option<ProjectResources>,
}

/// ProjectState indicates the lifecycle state of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectState {
    /// Unspecified state.
    Unspecified,
    /// The project is active and operational.
    Active,
    /// The project has been soft-deleted.
    Deleted,
}

/// ProjectResources contains default resources provisioned for a project.
#[derive(Debug, Clone)]
pub struct ProjectResources {
    /// Default hosting site name for the project.
    pub hosting_site: String,
    /// Default realtime database instance name.
    pub realtime_database_instance: String,
    /// Default storage bucket for the project.
    pub storage_bucket: String,
}

// =============================================================================
// CreateProject
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectError {
    #[error("Bad request: {0}")]
    BadRequest(String),
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
    pub project_id: Option<String>,
    /// Human-readable display name for the project.
    /// If not provided, project_id will be used as the display name.
    pub display_name: Option<String>,
}

impl CreateProjectRequest {
    /// Creates a new request with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the project ID.
    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}

/// Response from creating a project.
#[derive(Debug, Clone)]
pub struct CreateProjectResponse {
    /// The created project.
    pub project: Project,
}
