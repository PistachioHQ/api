use pistachio_api_common::admin::app::{
    CreateAppError, CreateAppRequest, CreateAppResponse, DeleteAppError, DeleteAppRequest,
    DeleteAppResponse, GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse, GetAppError,
    GetAppRequest, GetAppResponse, ListAppsError, ListAppsRequest, ListAppsResponse,
    SearchAppsError, SearchAppsRequest, SearchAppsResponse, UndeleteAppError, UndeleteAppRequest,
    UndeleteAppResponse, UpdateAppError, UpdateAppRequest, UpdateAppResponse,
};
use pistachio_api_common::admin::auth_provider::{
    DeleteProjectAuthProviderError, DeleteProjectAuthProviderRequest,
    DeleteProjectAuthProviderResponse, DeleteTenantAuthProviderError,
    DeleteTenantAuthProviderRequest, DeleteTenantAuthProviderResponse,
    GetEffectiveTenantAuthProvidersError, GetEffectiveTenantAuthProvidersRequest,
    GetEffectiveTenantAuthProvidersResponse, GetProjectAuthProviderError,
    GetProjectAuthProviderRequest, GetProjectAuthProviderResponse, ListProjectAuthProvidersError,
    ListProjectAuthProvidersRequest, ListProjectAuthProvidersResponse,
    ListTenantAuthProvidersError, ListTenantAuthProvidersRequest, ListTenantAuthProvidersResponse,
    UpdateProjectAuthProviderError, UpdateProjectAuthProviderRequest,
    UpdateProjectAuthProviderResponse, UpdateTenantAuthProviderError,
    UpdateTenantAuthProviderRequest, UpdateTenantAuthProviderResponse,
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
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tonic::{Request, Status};
use tracing::{debug, error, info, instrument, warn};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient as GrpcPistachioAdminClient;

use super::create_app::handle_create_app;
use super::create_project::handle_create_project;
use super::create_tenant::handle_create_tenant;
use super::delete_app::handle_delete_app;
use super::delete_project::handle_delete_project;
use super::delete_project_auth_provider::handle_delete_project_auth_provider;
use super::delete_tenant::handle_delete_tenant;
use super::delete_tenant_auth_provider::handle_delete_tenant_auth_provider;
use super::get_admin_sdk_config::handle_get_admin_sdk_config;
use super::get_app::handle_get_app;
use super::get_app_config::handle_get_app_config;
use super::get_effective_tenant_auth_providers::handle_get_effective_tenant_auth_providers;
use super::get_project::handle_get_project;
use super::get_project_auth_provider::handle_get_project_auth_provider;
use super::get_tenant::handle_get_tenant;
use super::list_apps::handle_list_apps;
use super::list_project_auth_providers::handle_list_project_auth_providers;
use super::list_projects::handle_list_projects;
use super::list_tenant_auth_providers::handle_list_tenant_auth_providers;
use super::list_tenants::handle_list_tenants;
use super::search_apps::handle_search_apps;
use super::search_projects::handle_search_projects;
use super::search_tenants::handle_search_tenants;
use super::undelete_app::handle_undelete_app;
use super::undelete_project::handle_undelete_project;
use super::update_app::handle_update_app;
use super::update_project::handle_update_project;
use super::update_project_auth_provider::handle_update_project_auth_provider;
use super::update_tenant::handle_update_tenant;
use super::update_tenant_auth_provider::handle_update_tenant_auth_provider;

/// Interceptor that adds admin credentials to requests.
#[derive(Debug, Clone)]
struct AdminAuthInterceptor {
    api_key: String,
    service_account_token: String,
}

impl Interceptor for AdminAuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.metadata_mut().insert(
            "x-api-key",
            self.api_key
                .parse()
                .map_err(|_| Status::internal("Invalid API key format"))?,
        );
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.service_account_token)
                .parse()
                .map_err(|_| Status::internal("Invalid service account token format"))?,
        );
        Ok(request)
    }
}

type AuthenticatedClient =
    GrpcPistachioAdminClient<InterceptedService<Channel, AdminAuthInterceptor>>;

/// gRPC client for the Pistachio Admin API.
#[derive(Debug, Clone)]
pub struct AdminClient {
    endpoint: Option<tonic::transport::Endpoint>,
    credentials: AdminCredentials,
    inner: Option<AuthenticatedClient>,
}

impl AdminClient {
    /// Creates a new client from an existing connected channel.
    ///
    /// This allows sharing a single HTTP/2 connection across multiple clients.
    /// The channel should already be connected.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tonic::transport::Channel;
    ///
    /// let channel = Channel::from_static("https://admin.pistachiohq.com")
    ///     .connect()
    ///     .await?;
    ///
    /// let client = AdminClient::from_channel(channel, credentials);
    /// ```
    pub fn from_channel(channel: Channel, credentials: AdminCredentials) -> Self {
        let interceptor = AdminAuthInterceptor {
            api_key: credentials.api_key().to_string(),
            service_account_token: credentials.service_account_token().to_string(),
        };
        let client = GrpcPistachioAdminClient::with_interceptor(channel, interceptor);

        Self {
            endpoint: None,
            credentials,
            inner: Some(client),
        }
    }
}

#[cfg_attr(
    any(feature = "single-threaded", target_arch = "wasm32"),
    async_trait::async_trait(?Send)
)]
#[cfg_attr(
    not(any(feature = "single-threaded", target_arch = "wasm32")),
    async_trait::async_trait
)]
impl PistachioAdminClient for AdminClient {
    #[instrument(skip(endpoint, credentials), level = "debug")]
    fn new(
        endpoint: impl AsRef<str>,
        credentials: AdminCredentials,
    ) -> Result<Self, PistachioApiClientError> {
        debug!(
            "Creating new Pistachio Admin API client with endpoint: {}",
            endpoint.as_ref()
        );
        let endpoint = Channel::from_shared(endpoint.as_ref().to_string())
            .map_err(|e| PistachioApiClientError::InvalidUri(e.to_string()))?;

        Ok(Self {
            endpoint: Some(endpoint),
            credentials,
            inner: None,
        })
    }

    #[instrument(skip(self), level = "debug")]
    async fn connect(self) -> Result<Self, PistachioApiClientError> {
        match self.inner {
            Some(_) => {
                debug!("Client already connected");
                Ok(self)
            }
            None => {
                let endpoint = self
                    .endpoint
                    .as_ref()
                    .ok_or(PistachioApiClientError::NotConnected)?;

                debug!("Attempting to connect to Pistachio Admin API");
                match endpoint.connect().await {
                    Ok(channel) => {
                        info!("Successfully connected to Pistachio Admin API");
                        let interceptor = AdminAuthInterceptor {
                            api_key: self.credentials.api_key().to_string(),
                            service_account_token: self
                                .credentials
                                .service_account_token()
                                .to_string(),
                        };
                        let client =
                            GrpcPistachioAdminClient::with_interceptor(channel, interceptor);
                        Ok(Self {
                            endpoint: self.endpoint,
                            credentials: self.credentials,
                            inner: Some(client),
                        })
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            error_debug = ?e,
                            "Failed to connect to Pistachio Admin API"
                        );
                        Err(PistachioApiClientError::ConnectionError(e.to_string()))
                    }
                }
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn create_project(
        &mut self,
        req: CreateProjectRequest,
    ) -> Result<CreateProjectResponse, CreateProjectError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting create_project");
                handle_create_project(client, req).await
            }
            None => {
                warn!("Attempted create_project with unconnected client");
                Err(CreateProjectError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_project(
        &mut self,
        req: GetProjectRequest,
    ) -> Result<GetProjectResponse, GetProjectError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_project");
                handle_get_project(client, req).await
            }
            None => {
                warn!("Attempted get_project with unconnected client");
                Err(GetProjectError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn update_project(
        &mut self,
        req: UpdateProjectRequest,
    ) -> Result<UpdateProjectResponse, UpdateProjectError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting update_project");
                handle_update_project(client, req).await
            }
            None => {
                warn!("Attempted update_project with unconnected client");
                Err(UpdateProjectError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn delete_project(
        &mut self,
        req: DeleteProjectRequest,
    ) -> Result<DeleteProjectResponse, DeleteProjectError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting delete_project");
                handle_delete_project(client, req).await
            }
            None => {
                warn!("Attempted delete_project with unconnected client");
                Err(DeleteProjectError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn undelete_project(
        &mut self,
        req: UndeleteProjectRequest,
    ) -> Result<UndeleteProjectResponse, UndeleteProjectError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting undelete_project");
                handle_undelete_project(client, req).await
            }
            None => {
                warn!("Attempted undelete_project with unconnected client");
                Err(UndeleteProjectError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn list_projects(
        &mut self,
        req: ListProjectsRequest,
    ) -> Result<ListProjectsResponse, ListProjectsError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting list_projects");
                handle_list_projects(client, req).await
            }
            None => {
                warn!("Attempted list_projects with unconnected client");
                Err(ListProjectsError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn search_projects(
        &mut self,
        req: SearchProjectsRequest,
    ) -> Result<SearchProjectsResponse, SearchProjectsError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting search_projects");
                handle_search_projects(client, req).await
            }
            None => {
                warn!("Attempted search_projects with unconnected client");
                Err(SearchProjectsError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_admin_sdk_config(
        &mut self,
        req: GetAdminSdkConfigRequest,
    ) -> Result<GetAdminSdkConfigResponse, GetAdminSdkConfigError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_admin_sdk_config");
                handle_get_admin_sdk_config(client, req).await
            }
            None => {
                warn!("Attempted get_admin_sdk_config with unconnected client");
                Err(GetAdminSdkConfigError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    // =========================================================================
    // Tenant Operations
    // =========================================================================

    #[instrument(skip(self, req), level = "debug")]
    async fn create_tenant(
        &mut self,
        req: CreateTenantRequest,
    ) -> Result<CreateTenantResponse, CreateTenantError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting create_tenant");
                handle_create_tenant(client, req).await
            }
            None => {
                warn!("Attempted create_tenant with unconnected client");
                Err(CreateTenantError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_tenant(
        &mut self,
        req: GetTenantRequest,
    ) -> Result<GetTenantResponse, GetTenantError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_tenant");
                handle_get_tenant(client, req).await
            }
            None => {
                warn!("Attempted get_tenant with unconnected client");
                Err(GetTenantError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn update_tenant(
        &mut self,
        req: UpdateTenantRequest,
    ) -> Result<UpdateTenantResponse, UpdateTenantError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting update_tenant");
                handle_update_tenant(client, req).await
            }
            None => {
                warn!("Attempted update_tenant with unconnected client");
                Err(UpdateTenantError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn delete_tenant(
        &mut self,
        req: DeleteTenantRequest,
    ) -> Result<DeleteTenantResponse, DeleteTenantError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting delete_tenant");
                handle_delete_tenant(client, req).await
            }
            None => {
                warn!("Attempted delete_tenant with unconnected client");
                Err(DeleteTenantError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn list_tenants(
        &mut self,
        req: ListTenantsRequest,
    ) -> Result<ListTenantsResponse, ListTenantsError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting list_tenants");
                handle_list_tenants(client, req).await
            }
            None => {
                warn!("Attempted list_tenants with unconnected client");
                Err(ListTenantsError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn search_tenants(
        &mut self,
        req: SearchTenantsRequest,
    ) -> Result<SearchTenantsResponse, SearchTenantsError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting search_tenants");
                handle_search_tenants(client, req).await
            }
            None => {
                warn!("Attempted search_tenants with unconnected client");
                Err(SearchTenantsError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    // =========================================================================
    // App Operations
    // =========================================================================

    #[instrument(skip(self, req), level = "debug")]
    async fn create_app(
        &mut self,
        req: CreateAppRequest,
    ) -> Result<CreateAppResponse, CreateAppError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting create_app");
                handle_create_app(client, req).await
            }
            None => {
                warn!("Attempted create_app with unconnected client");
                Err(CreateAppError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_app(&mut self, req: GetAppRequest) -> Result<GetAppResponse, GetAppError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_app");
                handle_get_app(client, req).await
            }
            None => {
                warn!("Attempted get_app with unconnected client");
                Err(GetAppError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn update_app(
        &mut self,
        req: UpdateAppRequest,
    ) -> Result<UpdateAppResponse, UpdateAppError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting update_app");
                handle_update_app(client, req).await
            }
            None => {
                warn!("Attempted update_app with unconnected client");
                Err(UpdateAppError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn delete_app(
        &mut self,
        req: DeleteAppRequest,
    ) -> Result<DeleteAppResponse, DeleteAppError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting delete_app");
                handle_delete_app(client, req).await
            }
            None => {
                warn!("Attempted delete_app with unconnected client");
                Err(DeleteAppError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn undelete_app(
        &mut self,
        req: UndeleteAppRequest,
    ) -> Result<UndeleteAppResponse, UndeleteAppError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting undelete_app");
                handle_undelete_app(client, req).await
            }
            None => {
                warn!("Attempted undelete_app with unconnected client");
                Err(UndeleteAppError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn list_apps(&mut self, req: ListAppsRequest) -> Result<ListAppsResponse, ListAppsError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting list_apps");
                handle_list_apps(client, req).await
            }
            None => {
                warn!("Attempted list_apps with unconnected client");
                Err(ListAppsError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn search_apps(
        &mut self,
        req: SearchAppsRequest,
    ) -> Result<SearchAppsResponse, SearchAppsError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting search_apps");
                handle_search_apps(client, req).await
            }
            None => {
                warn!("Attempted search_apps with unconnected client");
                Err(SearchAppsError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_app_config(
        &mut self,
        req: GetAppConfigRequest,
    ) -> Result<GetAppConfigResponse, GetAppConfigError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_app_config");
                handle_get_app_config(client, req).await
            }
            None => {
                warn!("Attempted get_app_config with unconnected client");
                Err(GetAppConfigError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    // =========================================================================
    // Auth Provider Operations
    // =========================================================================

    #[instrument(skip(self, req), level = "debug")]
    async fn list_project_auth_providers(
        &mut self,
        req: ListProjectAuthProvidersRequest,
    ) -> Result<ListProjectAuthProvidersResponse, ListProjectAuthProvidersError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting list_project_auth_providers");
                handle_list_project_auth_providers(client, req).await
            }
            None => {
                warn!("Attempted list_project_auth_providers with unconnected client");
                Err(ListProjectAuthProvidersError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_project_auth_provider(
        &mut self,
        req: GetProjectAuthProviderRequest,
    ) -> Result<GetProjectAuthProviderResponse, GetProjectAuthProviderError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_project_auth_provider");
                handle_get_project_auth_provider(client, req).await
            }
            None => {
                warn!("Attempted get_project_auth_provider with unconnected client");
                Err(GetProjectAuthProviderError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn update_project_auth_provider(
        &mut self,
        req: UpdateProjectAuthProviderRequest,
    ) -> Result<UpdateProjectAuthProviderResponse, UpdateProjectAuthProviderError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting update_project_auth_provider");
                handle_update_project_auth_provider(client, req).await
            }
            None => {
                warn!("Attempted update_project_auth_provider with unconnected client");
                Err(UpdateProjectAuthProviderError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn delete_project_auth_provider(
        &mut self,
        req: DeleteProjectAuthProviderRequest,
    ) -> Result<DeleteProjectAuthProviderResponse, DeleteProjectAuthProviderError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting delete_project_auth_provider");
                handle_delete_project_auth_provider(client, req).await
            }
            None => {
                warn!("Attempted delete_project_auth_provider with unconnected client");
                Err(DeleteProjectAuthProviderError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn list_tenant_auth_providers(
        &mut self,
        req: ListTenantAuthProvidersRequest,
    ) -> Result<ListTenantAuthProvidersResponse, ListTenantAuthProvidersError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting list_tenant_auth_providers");
                handle_list_tenant_auth_providers(client, req).await
            }
            None => {
                warn!("Attempted list_tenant_auth_providers with unconnected client");
                Err(ListTenantAuthProvidersError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn update_tenant_auth_provider(
        &mut self,
        req: UpdateTenantAuthProviderRequest,
    ) -> Result<UpdateTenantAuthProviderResponse, UpdateTenantAuthProviderError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting update_tenant_auth_provider");
                handle_update_tenant_auth_provider(client, req).await
            }
            None => {
                warn!("Attempted update_tenant_auth_provider with unconnected client");
                Err(UpdateTenantAuthProviderError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn delete_tenant_auth_provider(
        &mut self,
        req: DeleteTenantAuthProviderRequest,
    ) -> Result<DeleteTenantAuthProviderResponse, DeleteTenantAuthProviderError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting delete_tenant_auth_provider");
                handle_delete_tenant_auth_provider(client, req).await
            }
            None => {
                warn!("Attempted delete_tenant_auth_provider with unconnected client");
                Err(DeleteTenantAuthProviderError::PistachioApiClientError(
                    PistachioApiClientError::NotConnected,
                ))
            }
        }
    }

    #[instrument(skip(self, req), level = "debug")]
    async fn get_effective_tenant_auth_providers(
        &mut self,
        req: GetEffectiveTenantAuthProvidersRequest,
    ) -> Result<GetEffectiveTenantAuthProvidersResponse, GetEffectiveTenantAuthProvidersError> {
        match &mut self.inner {
            Some(client) => {
                debug!("Attempting get_effective_tenant_auth_providers");
                handle_get_effective_tenant_auth_providers(client, req).await
            }
            None => {
                warn!("Attempted get_effective_tenant_auth_providers with unconnected client");
                Err(
                    GetEffectiveTenantAuthProvidersError::PistachioApiClientError(
                        PistachioApiClientError::NotConnected,
                    ),
                )
            }
        }
    }
}
