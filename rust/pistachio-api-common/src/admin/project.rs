use libgn::project::{Project, ProjectDisplayName, ProjectId, ProjectInvitationCode};

use crate::error::{PistachioApiClientError, ValidationError};

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
