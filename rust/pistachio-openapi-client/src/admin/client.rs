use std::sync::Arc;

use cfg_if::cfg_if;
use pistachio_api_common::admin::client::PistachioAdminClient;
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse, DeleteProjectError,
    DeleteProjectRequest, DeleteProjectResponse, GetProjectError, GetProjectRequest,
    GetProjectResponse, ListProjectsError, ListProjectsRequest, ListProjectsResponse,
    SearchProjectsError, SearchProjectsRequest, SearchProjectsResponse, UndeleteProjectError,
    UndeleteProjectRequest, UndeleteProjectResponse, UpdateProjectError, UpdateProjectRequest,
    UpdateProjectResponse,
};
use pistachio_api_common::credentials::AdminCredentials;
use pistachio_api_common::error::PistachioApiClientError;
use tracing::{debug, instrument};

use super::create_project::handle_create_project;
use super::delete_project::handle_delete_project;
use super::get_project::handle_get_project;
use super::list_projects::handle_list_projects;
use super::search_projects::handle_search_projects;
use super::undelete_project::handle_undelete_project;
use super::update_project::handle_update_project;

/// OpenAPI/REST client for the Pistachio Admin API.
#[derive(Debug, Clone)]
pub struct AdminClient {
    config: Arc<crate::generated_admin::apis::configuration::Configuration>,
}

/// Macro to implement PistachioAdminClient trait with the appropriate async_trait attribute.
/// This avoids duplicating the entire implementation for single-threaded vs multi-threaded.
macro_rules! impl_admin_client {
    ($($attr:tt)*) => {
        $($attr)*
        impl PistachioAdminClient for AdminClient {
            #[instrument(skip(endpoint, credentials), level = "debug")]
            fn new(
                endpoint: impl AsRef<str>,
                credentials: AdminCredentials,
            ) -> Result<Self, PistachioApiClientError> {
                debug!(
                    "Creating new Pistachio Admin API OpenAPI client with endpoint: {}",
                    endpoint.as_ref()
                );

                let mut config = crate::generated_admin::apis::configuration::Configuration::new();
                config.base_path = endpoint.as_ref().to_string();

                // Set up API key authentication
                config.api_key = Some(crate::generated_admin::apis::configuration::ApiKey {
                    prefix: None,
                    key: credentials.api_key().to_string(),
                });

                // Set up bearer token authentication for service account
                config.bearer_access_token = Some(credentials.service_account_token().to_string());

                Ok(Self {
                    config: Arc::new(config),
                })
            }

            #[instrument(skip(self), level = "debug")]
            async fn connect(self) -> Result<Self, PistachioApiClientError> {
                // HTTP client doesn't need explicit connection
                debug!("OpenAPI client ready (no connection needed for HTTP)");
                Ok(self)
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_project(
                &mut self,
                req: CreateProjectRequest,
            ) -> Result<CreateProjectResponse, CreateProjectError> {
                debug!("Attempting create_project via OpenAPI");
                handle_create_project(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_project(
                &mut self,
                req: GetProjectRequest,
            ) -> Result<GetProjectResponse, GetProjectError> {
                debug!("Attempting get_project via OpenAPI");
                handle_get_project(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_project(
                &mut self,
                req: UpdateProjectRequest,
            ) -> Result<UpdateProjectResponse, UpdateProjectError> {
                debug!("Attempting update_project via OpenAPI");
                handle_update_project(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_project(
                &mut self,
                req: DeleteProjectRequest,
            ) -> Result<DeleteProjectResponse, DeleteProjectError> {
                debug!("Attempting delete_project via OpenAPI");
                handle_delete_project(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn undelete_project(
                &mut self,
                req: UndeleteProjectRequest,
            ) -> Result<UndeleteProjectResponse, UndeleteProjectError> {
                debug!("Attempting undelete_project via OpenAPI");
                handle_undelete_project(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_projects(
                &mut self,
                req: ListProjectsRequest,
            ) -> Result<ListProjectsResponse, ListProjectsError> {
                debug!("Attempting list_projects via OpenAPI");
                handle_list_projects(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn search_projects(
                &mut self,
                req: SearchProjectsRequest,
            ) -> Result<SearchProjectsResponse, SearchProjectsError> {
                debug!("Attempting search_projects via OpenAPI");
                handle_search_projects(&self.config, req).await
            }
        }
    };
}

cfg_if! {
    if #[cfg(any(feature = "single-threaded", target_arch = "wasm32"))] {
        impl_admin_client!(#[async_trait::async_trait(?Send)]);
    } else {
        impl_admin_client!(#[async_trait::async_trait]);
    }
}
