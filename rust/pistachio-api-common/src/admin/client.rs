use crate::credentials::AdminCredentials;
use crate::error::PistachioApiClientError;

use super::project::{CreateProjectError, CreateProjectRequest, CreateProjectResponse};

/// Trait for Pistachio Admin API clients.
///
/// This trait defines the operations available for administrative tasks
/// such as project management. All operations require service account
/// authentication.
#[cfg_attr(
    any(feature = "single-threaded", target_arch = "wasm32"),
    async_trait::async_trait(?Send)
)]
#[cfg_attr(
    not(any(feature = "single-threaded", target_arch = "wasm32")),
    async_trait::async_trait
)]
pub trait PistachioAdminClient: Sized {
    /// Creates a new client with the given endpoint and credentials.
    fn new(
        endpoint: impl AsRef<str>,
        credentials: AdminCredentials,
    ) -> Result<Self, PistachioApiClientError>;

    /// Connects to the API server.
    async fn connect(self) -> Result<Self, PistachioApiClientError>;

    /// Creates a new project.
    ///
    /// Projects are the top-level container for apps, users, and resources.
    /// Each project can have multiple apps across different platforms
    /// that share the same user pool.
    async fn create_project(
        &mut self,
        req: CreateProjectRequest,
    ) -> Result<CreateProjectResponse, CreateProjectError>;
}
