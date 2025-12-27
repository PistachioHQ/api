use std::sync::Arc;

use cfg_if::cfg_if;
use pistachio_api_common::admin::app::{
    CreateAppError, CreateAppRequest, CreateAppResponse, DeleteAppError, DeleteAppRequest,
    DeleteAppResponse, GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse, GetAppError,
    GetAppRequest, GetAppResponse, ListAppsError, ListAppsRequest, ListAppsResponse,
    SearchAppsError, SearchAppsRequest, SearchAppsResponse, UndeleteAppError, UndeleteAppRequest,
    UndeleteAppResponse, UpdateAppError, UpdateAppRequest, UpdateAppResponse,
};
use pistachio_api_common::admin::client::PistachioAdminClient;
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse, DeleteProjectError,
    DeleteProjectRequest, DeleteProjectResponse, GetAdminSdkConfigError, GetAdminSdkConfigRequest,
    GetAdminSdkConfigResponse, GetProjectError, GetProjectRequest, GetProjectResponse,
    ListProjectsError, ListProjectsRequest, ListProjectsResponse, SearchProjectsError,
    SearchProjectsRequest, SearchProjectsResponse, UndeleteProjectError, UndeleteProjectRequest,
    UndeleteProjectResponse, UpdateProjectError, UpdateProjectRequest, UpdateProjectResponse,
};
use pistachio_api_common::admin::tenant::{
    CreateTenantError, CreateTenantRequest, CreateTenantResponse, DeleteTenantError,
    DeleteTenantRequest, DeleteTenantResponse, GetTenantError, GetTenantRequest, GetTenantResponse,
    ListTenantsError, ListTenantsRequest, ListTenantsResponse, SearchTenantsError,
    SearchTenantsRequest, SearchTenantsResponse, UpdateTenantError, UpdateTenantRequest,
    UpdateTenantResponse,
};
use pistachio_api_common::credentials::AdminCredentials;
use pistachio_api_common::error::PistachioApiClientError;
use tracing::{debug, instrument};

use super::create_app::handle_create_app;
use super::create_project::handle_create_project;
use super::create_tenant::handle_create_tenant;
use super::delete_app::handle_delete_app;
use super::delete_project::handle_delete_project;
use super::delete_tenant::handle_delete_tenant;
use super::get_admin_sdk_config::handle_get_admin_sdk_config;
use super::get_app::handle_get_app;
use super::get_app_config::handle_get_app_config;
use super::get_project::handle_get_project;
use super::get_tenant::handle_get_tenant;
use super::list_apps::handle_list_apps;
use super::list_projects::handle_list_projects;
use super::list_tenants::handle_list_tenants;
use super::search_apps::handle_search_apps;
use super::search_projects::handle_search_projects;
use super::search_tenants::handle_search_tenants;
use super::undelete_app::handle_undelete_app;
use super::undelete_project::handle_undelete_project;
use super::update_app::handle_update_app;
use super::update_project::handle_update_project;
use super::update_tenant::handle_update_tenant;

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

            #[instrument(skip(self, req), level = "debug")]
            async fn get_admin_sdk_config(
                &mut self,
                req: GetAdminSdkConfigRequest,
            ) -> Result<GetAdminSdkConfigResponse, GetAdminSdkConfigError> {
                debug!("Attempting get_admin_sdk_config via OpenAPI");
                handle_get_admin_sdk_config(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_tenant(
                &mut self,
                req: CreateTenantRequest,
            ) -> Result<CreateTenantResponse, CreateTenantError> {
                debug!("Attempting create_tenant via OpenAPI");
                handle_create_tenant(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_tenant(
                &mut self,
                req: GetTenantRequest,
            ) -> Result<GetTenantResponse, GetTenantError> {
                debug!("Attempting get_tenant via OpenAPI");
                handle_get_tenant(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_tenant(
                &mut self,
                req: UpdateTenantRequest,
            ) -> Result<UpdateTenantResponse, UpdateTenantError> {
                debug!("Attempting update_tenant via OpenAPI");
                handle_update_tenant(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_tenant(
                &mut self,
                req: DeleteTenantRequest,
            ) -> Result<DeleteTenantResponse, DeleteTenantError> {
                debug!("Attempting delete_tenant via OpenAPI");
                handle_delete_tenant(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_tenants(
                &mut self,
                req: ListTenantsRequest,
            ) -> Result<ListTenantsResponse, ListTenantsError> {
                debug!("Attempting list_tenants via OpenAPI");
                handle_list_tenants(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn search_tenants(
                &mut self,
                req: SearchTenantsRequest,
            ) -> Result<SearchTenantsResponse, SearchTenantsError> {
                debug!("Attempting search_tenants via OpenAPI");
                handle_search_tenants(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_app(
                &mut self,
                req: CreateAppRequest,
            ) -> Result<CreateAppResponse, CreateAppError> {
                debug!("Attempting create_app via OpenAPI");
                handle_create_app(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_app(
                &mut self,
                req: GetAppRequest,
            ) -> Result<GetAppResponse, GetAppError> {
                debug!("Attempting get_app via OpenAPI");
                handle_get_app(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_app(
                &mut self,
                req: UpdateAppRequest,
            ) -> Result<UpdateAppResponse, UpdateAppError> {
                debug!("Attempting update_app via OpenAPI");
                handle_update_app(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_app(
                &mut self,
                req: DeleteAppRequest,
            ) -> Result<DeleteAppResponse, DeleteAppError> {
                debug!("Attempting delete_app via OpenAPI");
                handle_delete_app(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn undelete_app(
                &mut self,
                req: UndeleteAppRequest,
            ) -> Result<UndeleteAppResponse, UndeleteAppError> {
                debug!("Attempting undelete_app via OpenAPI");
                handle_undelete_app(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_apps(
                &mut self,
                req: ListAppsRequest,
            ) -> Result<ListAppsResponse, ListAppsError> {
                debug!("Attempting list_apps via OpenAPI");
                handle_list_apps(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn search_apps(
                &mut self,
                req: SearchAppsRequest,
            ) -> Result<SearchAppsResponse, SearchAppsError> {
                debug!("Attempting search_apps via OpenAPI");
                handle_search_apps(&self.config, req).await
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_app_config(
                &mut self,
                req: GetAppConfigRequest,
            ) -> Result<GetAppConfigResponse, GetAppConfigError> {
                debug!("Attempting get_app_config via OpenAPI");
                handle_get_app_config(&self.config, req).await
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
