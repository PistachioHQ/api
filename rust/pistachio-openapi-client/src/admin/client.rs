use std::sync::Arc;

use cfg_if::cfg_if;
use pistachio_api_common::admin::api_key::{
    CreateApiKeyError, CreateApiKeyRequest, CreateApiKeyResponse, DeleteApiKeyError,
    DeleteApiKeyRequest, DeleteApiKeyResponse, GetApiKeyError, GetApiKeyRequest, GetApiKeyResponse,
    ListApiKeysError, ListApiKeysRequest, ListApiKeysResponse, RotateApiKeyError,
    RotateApiKeyRequest, RotateApiKeyResponse, UpdateApiKeyError, UpdateApiKeyRequest,
    UpdateApiKeyResponse,
};
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
use pistachio_api_common::admin::mfa::{
    DeleteProjectUserMfaFactorError, DeleteProjectUserMfaFactorRequest,
    DeleteProjectUserMfaFactorResponse, DeleteTenantUserMfaFactorError,
    DeleteTenantUserMfaFactorRequest, DeleteTenantUserMfaFactorResponse,
    ListProjectUserMfaFactorsError, ListProjectUserMfaFactorsRequest,
    ListProjectUserMfaFactorsResponse, ListTenantUserMfaFactorsError,
    ListTenantUserMfaFactorsRequest, ListTenantUserMfaFactorsResponse,
};
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse, DeleteProjectError,
    DeleteProjectRequest, DeleteProjectResponse, GetAdminSdkConfigError, GetAdminSdkConfigRequest,
    GetAdminSdkConfigResponse, GetProjectError, GetProjectRequest, GetProjectResponse,
    ListProjectsError, ListProjectsRequest, ListProjectsResponse, SearchProjectsError,
    SearchProjectsRequest, SearchProjectsResponse, UndeleteProjectError, UndeleteProjectRequest,
    UndeleteProjectResponse, UpdateProjectError, UpdateProjectRequest, UpdateProjectResponse,
};
use pistachio_api_common::admin::service_account::{
    CreateServiceAccountError, CreateServiceAccountRequest, CreateServiceAccountResponse,
    DeleteServiceAccountError, DeleteServiceAccountKeyError, DeleteServiceAccountKeyRequest,
    DeleteServiceAccountKeyResponse, DeleteServiceAccountRequest, DeleteServiceAccountResponse,
    DisableServiceAccountKeyError, DisableServiceAccountKeyRequest,
    DisableServiceAccountKeyResponse, EnableServiceAccountKeyError, EnableServiceAccountKeyRequest,
    EnableServiceAccountKeyResponse, GenerateServiceAccountKeyError,
    GenerateServiceAccountKeyRequest, GenerateServiceAccountKeyResponse, GetServiceAccountError,
    GetServiceAccountKeyError, GetServiceAccountKeyRequest, GetServiceAccountKeyResponse,
    GetServiceAccountRequest, GetServiceAccountResponse, ListServiceAccountKeysError,
    ListServiceAccountKeysRequest, ListServiceAccountKeysResponse, ListServiceAccountsError,
    ListServiceAccountsRequest, ListServiceAccountsResponse, SearchServiceAccountsError,
    SearchServiceAccountsRequest, SearchServiceAccountsResponse, UpdateServiceAccountError,
    UpdateServiceAccountRequest, UpdateServiceAccountResponse,
};
use pistachio_api_common::admin::tenant::{
    CreateTenantError, CreateTenantRequest, CreateTenantResponse, DeleteTenantError,
    DeleteTenantRequest, DeleteTenantResponse, GetTenantError, GetTenantRequest, GetTenantResponse,
    ListTenantsError, ListTenantsRequest, ListTenantsResponse, SearchTenantsError,
    SearchTenantsRequest, SearchTenantsResponse, UpdateTenantError, UpdateTenantRequest,
    UpdateTenantResponse,
};
use pistachio_api_common::admin::token::{
    CreateCustomTokenError, CreateCustomTokenRequest, CreateCustomTokenResponse,
    CreateSessionCookieError, CreateSessionCookieRequest, CreateSessionCookieResponse,
    RevokeRefreshTokensError, RevokeRefreshTokensRequest, RevokeRefreshTokensResponse,
    VerifyIdTokenError, VerifyIdTokenRequest, VerifyIdTokenResponse, VerifySessionCookieError,
    VerifySessionCookieRequest, VerifySessionCookieResponse,
};
use pistachio_api_common::admin::usage::{
    GetProjectQuotasError, GetProjectQuotasRequest, GetProjectQuotasResponse, GetProjectUsageError,
    GetProjectUsageRequest, GetProjectUsageResponse, UpdateProjectQuotasError,
    UpdateProjectQuotasRequest, UpdateProjectQuotasResponse,
};
use pistachio_api_common::admin::user::{
    CreateProjectUserError, CreateProjectUserRequest, CreateProjectUserResponse,
    CreateTenantUserError, CreateTenantUserRequest, CreateTenantUserResponse,
    DeleteProjectUserError, DeleteProjectUserRequest, DeleteProjectUserResponse,
    DeleteTenantUserError, DeleteTenantUserRequest, DeleteTenantUserResponse, GetProjectUserError,
    GetProjectUserRequest, GetProjectUserResponse, GetTenantUserError, GetTenantUserRequest,
    GetTenantUserResponse, ImportProjectUsersError, ImportProjectUsersRequest,
    ImportProjectUsersResponse, ImportTenantUsersError, ImportTenantUsersRequest,
    ImportTenantUsersResponse, ListProjectUsersError, ListProjectUsersRequest,
    ListProjectUsersResponse, ListTenantUsersError, ListTenantUsersRequest,
    ListTenantUsersResponse, SearchProjectUsersError, SearchProjectUsersRequest,
    SearchProjectUsersResponse, SearchTenantUsersError, SearchTenantUsersRequest,
    SearchTenantUsersResponse, UpdateProjectUserError, UpdateProjectUserRequest,
    UpdateProjectUserResponse, UpdateTenantUserError, UpdateTenantUserRequest,
    UpdateTenantUserResponse,
};
use pistachio_api_common::credentials::AdminCredentials;
use pistachio_api_common::error::PistachioApiClientError;
use tracing::{debug, info, instrument, warn};

use super::api_key::{
    handle_create_api_key, handle_delete_api_key, handle_get_api_key, handle_list_api_keys,
    handle_rotate_api_key, handle_update_api_key,
};
use super::create_app::handle_create_app;
use super::create_project::handle_create_project;
use super::create_project_user::handle_create_project_user;
use super::create_tenant::handle_create_tenant;
use super::create_tenant_user::handle_create_tenant_user;
use super::delete_app::handle_delete_app;
use super::delete_project::handle_delete_project;
use super::delete_project_auth_provider::handle_delete_project_auth_provider;
use super::delete_project_user::handle_delete_project_user;
use super::delete_tenant::handle_delete_tenant;
use super::delete_tenant_auth_provider::handle_delete_tenant_auth_provider;
use super::delete_tenant_user::handle_delete_tenant_user;
use super::get_admin_sdk_config::handle_get_admin_sdk_config;
use super::get_app::handle_get_app;
use super::get_app_config::handle_get_app_config;
use super::get_effective_tenant_auth_providers::handle_get_effective_tenant_auth_providers;
use super::get_project::handle_get_project;
use super::get_project_auth_provider::handle_get_project_auth_provider;
use super::get_project_user::handle_get_project_user;
use super::get_tenant::handle_get_tenant;
use super::get_tenant_user::handle_get_tenant_user;
use super::import_project_users::handle_import_project_users;
use super::import_tenant_users::handle_import_tenant_users;
use super::list_apps::handle_list_apps;
use super::list_project_auth_providers::handle_list_project_auth_providers;
use super::list_project_users::handle_list_project_users;
use super::list_projects::handle_list_projects;
use super::list_tenant_auth_providers::handle_list_tenant_auth_providers;
use super::list_tenant_users::handle_list_tenant_users;
use super::list_tenants::handle_list_tenants;
use super::mfa::{
    handle_delete_project_user_mfa_factor, handle_delete_tenant_user_mfa_factor,
    handle_list_project_user_mfa_factors, handle_list_tenant_user_mfa_factors,
};
use super::search_apps::handle_search_apps;
use super::search_project_users::handle_search_project_users;
use super::search_projects::handle_search_projects;
use super::search_tenant_users::handle_search_tenant_users;
use super::search_tenants::handle_search_tenants;
use super::service_account::{
    handle_create_service_account, handle_delete_service_account,
    handle_delete_service_account_key, handle_disable_service_account_key,
    handle_enable_service_account_key, handle_generate_service_account_key,
    handle_get_service_account, handle_get_service_account_key, handle_list_service_account_keys,
    handle_list_service_accounts, handle_search_service_accounts, handle_update_service_account,
};
use super::token::{
    handle_create_custom_token, handle_create_session_cookie, handle_revoke_refresh_tokens,
    handle_verify_id_token, handle_verify_session_cookie,
};
use super::undelete_app::handle_undelete_app;
use super::undelete_project::handle_undelete_project;
use super::update_app::handle_update_app;
use super::update_project::handle_update_project;
use super::update_project_auth_provider::handle_update_project_auth_provider;
use super::update_project_user::handle_update_project_user;
use super::update_tenant::handle_update_tenant;
use super::update_tenant_auth_provider::handle_update_tenant_auth_provider;
use super::update_tenant_user::handle_update_tenant_user;
use super::usage::{
    handle_get_project_quotas, handle_get_project_usage, handle_update_project_quotas,
};

/// OpenAPI/REST client for the Pistachio Admin API.
///
/// Like the gRPC client, this client requires calling `connect()` after `new()`
/// before making API calls. This ensures a consistent initialization pattern
/// across all transport types.
#[derive(Debug, Clone)]
pub struct AdminClient {
    endpoint: String,
    credentials: AdminCredentials,
    config: Option<Arc<crate::generated_admin::apis::configuration::Configuration>>,
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

                Ok(Self {
                    endpoint: endpoint.as_ref().to_string(),
                    credentials,
                    config: None,
                })
            }

            #[instrument(skip(self), level = "debug")]
            async fn connect(self) -> Result<Self, PistachioApiClientError> {
                match self.config {
                    Some(_) => {
                        debug!("Client already connected");
                        Ok(self)
                    }
                    None => {
                        debug!("Initializing OpenAPI client configuration");

                        let mut config = crate::generated_admin::apis::configuration::Configuration::new();
                        config.base_path = self.endpoint.clone();

                        // Set up API key authentication
                        config.api_key = Some(crate::generated_admin::apis::configuration::ApiKey {
                            prefix: None,
                            key: self.credentials.api_key().to_string(),
                        });

                        // Set up bearer token authentication for service account
                        config.bearer_access_token = Some(self.credentials.service_account_token().to_string());

                        info!("OpenAPI client connected to {}", self.endpoint);

                        Ok(Self {
                            endpoint: self.endpoint,
                            credentials: self.credentials,
                            config: Some(Arc::new(config)),
                        })
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_project(
                &mut self,
                req: CreateProjectRequest,
            ) -> Result<CreateProjectResponse, CreateProjectError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_project via OpenAPI");
                        handle_create_project(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_project via OpenAPI");
                        handle_get_project(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_project via OpenAPI");
                        handle_update_project(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_project via OpenAPI");
                        handle_delete_project(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting undelete_project via OpenAPI");
                        handle_undelete_project(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_projects via OpenAPI");
                        handle_list_projects(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting search_projects via OpenAPI");
                        handle_search_projects(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_admin_sdk_config via OpenAPI");
                        handle_get_admin_sdk_config(config, req).await
                    }
                    None => {
                        warn!("Attempted get_admin_sdk_config with unconnected client");
                        Err(GetAdminSdkConfigError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_tenant(
                &mut self,
                req: CreateTenantRequest,
            ) -> Result<CreateTenantResponse, CreateTenantError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_tenant via OpenAPI");
                        handle_create_tenant(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_tenant via OpenAPI");
                        handle_get_tenant(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_tenant via OpenAPI");
                        handle_update_tenant(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_tenant via OpenAPI");
                        handle_delete_tenant(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_tenants via OpenAPI");
                        handle_list_tenants(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting search_tenants via OpenAPI");
                        handle_search_tenants(config, req).await
                    }
                    None => {
                        warn!("Attempted search_tenants with unconnected client");
                        Err(SearchTenantsError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_app(
                &mut self,
                req: CreateAppRequest,
            ) -> Result<CreateAppResponse, CreateAppError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_app via OpenAPI");
                        handle_create_app(config, req).await
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
            async fn get_app(
                &mut self,
                req: GetAppRequest,
            ) -> Result<GetAppResponse, GetAppError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_app via OpenAPI");
                        handle_get_app(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_app via OpenAPI");
                        handle_update_app(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_app via OpenAPI");
                        handle_delete_app(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting undelete_app via OpenAPI");
                        handle_undelete_app(config, req).await
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
            async fn list_apps(
                &mut self,
                req: ListAppsRequest,
            ) -> Result<ListAppsResponse, ListAppsError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_apps via OpenAPI");
                        handle_list_apps(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting search_apps via OpenAPI");
                        handle_search_apps(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_app_config via OpenAPI");
                        handle_get_app_config(config, req).await
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
            // Auth Provider methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn list_project_auth_providers(
                &mut self,
                req: ListProjectAuthProvidersRequest,
            ) -> Result<ListProjectAuthProvidersResponse, ListProjectAuthProvidersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_project_auth_providers via OpenAPI");
                        handle_list_project_auth_providers(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_project_auth_provider via OpenAPI");
                        handle_get_project_auth_provider(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_project_auth_provider via OpenAPI");
                        handle_update_project_auth_provider(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_project_auth_provider via OpenAPI");
                        handle_delete_project_auth_provider(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_tenant_auth_providers via OpenAPI");
                        handle_list_tenant_auth_providers(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_tenant_auth_provider via OpenAPI");
                        handle_update_tenant_auth_provider(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_tenant_auth_provider via OpenAPI");
                        handle_delete_tenant_auth_provider(config, req).await
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
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_effective_tenant_auth_providers via OpenAPI");
                        handle_get_effective_tenant_auth_providers(config, req).await
                    }
                    None => {
                        warn!("Attempted get_effective_tenant_auth_providers with unconnected client");
                        Err(GetEffectiveTenantAuthProvidersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // Project User methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn create_project_user(
                &mut self,
                req: CreateProjectUserRequest,
            ) -> Result<CreateProjectUserResponse, CreateProjectUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_project_user via OpenAPI");
                        handle_create_project_user(config, req).await
                    }
                    None => {
                        warn!("Attempted create_project_user with unconnected client");
                        Err(CreateProjectUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_project_user(
                &mut self,
                req: GetProjectUserRequest,
            ) -> Result<GetProjectUserResponse, GetProjectUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_project_user via OpenAPI");
                        handle_get_project_user(config, req).await
                    }
                    None => {
                        warn!("Attempted get_project_user with unconnected client");
                        Err(GetProjectUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_project_user(
                &mut self,
                req: UpdateProjectUserRequest,
            ) -> Result<UpdateProjectUserResponse, UpdateProjectUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_project_user via OpenAPI");
                        handle_update_project_user(config, req).await
                    }
                    None => {
                        warn!("Attempted update_project_user with unconnected client");
                        Err(UpdateProjectUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_project_user(
                &mut self,
                req: DeleteProjectUserRequest,
            ) -> Result<DeleteProjectUserResponse, DeleteProjectUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_project_user via OpenAPI");
                        handle_delete_project_user(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_project_user with unconnected client");
                        Err(DeleteProjectUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_project_users(
                &mut self,
                req: ListProjectUsersRequest,
            ) -> Result<ListProjectUsersResponse, ListProjectUsersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_project_users via OpenAPI");
                        handle_list_project_users(config, req).await
                    }
                    None => {
                        warn!("Attempted list_project_users with unconnected client");
                        Err(ListProjectUsersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn import_project_users(
                &mut self,
                req: ImportProjectUsersRequest,
            ) -> Result<ImportProjectUsersResponse, ImportProjectUsersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting import_project_users via OpenAPI");
                        handle_import_project_users(config, req).await
                    }
                    None => {
                        warn!("Attempted import_project_users with unconnected client");
                        Err(ImportProjectUsersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn search_project_users(
                &mut self,
                req: SearchProjectUsersRequest,
            ) -> Result<SearchProjectUsersResponse, SearchProjectUsersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting search_project_users via OpenAPI");
                        handle_search_project_users(config, req).await
                    }
                    None => {
                        warn!("Attempted search_project_users with unconnected client");
                        Err(SearchProjectUsersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // Tenant User methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn create_tenant_user(
                &mut self,
                req: CreateTenantUserRequest,
            ) -> Result<CreateTenantUserResponse, CreateTenantUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_tenant_user via OpenAPI");
                        handle_create_tenant_user(config, req).await
                    }
                    None => {
                        warn!("Attempted create_tenant_user with unconnected client");
                        Err(CreateTenantUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_tenant_user(
                &mut self,
                req: GetTenantUserRequest,
            ) -> Result<GetTenantUserResponse, GetTenantUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_tenant_user via OpenAPI");
                        handle_get_tenant_user(config, req).await
                    }
                    None => {
                        warn!("Attempted get_tenant_user with unconnected client");
                        Err(GetTenantUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_tenant_user(
                &mut self,
                req: UpdateTenantUserRequest,
            ) -> Result<UpdateTenantUserResponse, UpdateTenantUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_tenant_user via OpenAPI");
                        handle_update_tenant_user(config, req).await
                    }
                    None => {
                        warn!("Attempted update_tenant_user with unconnected client");
                        Err(UpdateTenantUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_tenant_user(
                &mut self,
                req: DeleteTenantUserRequest,
            ) -> Result<DeleteTenantUserResponse, DeleteTenantUserError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_tenant_user via OpenAPI");
                        handle_delete_tenant_user(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_tenant_user with unconnected client");
                        Err(DeleteTenantUserError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_tenant_users(
                &mut self,
                req: ListTenantUsersRequest,
            ) -> Result<ListTenantUsersResponse, ListTenantUsersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_tenant_users via OpenAPI");
                        handle_list_tenant_users(config, req).await
                    }
                    None => {
                        warn!("Attempted list_tenant_users with unconnected client");
                        Err(ListTenantUsersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn import_tenant_users(
                &mut self,
                req: ImportTenantUsersRequest,
            ) -> Result<ImportTenantUsersResponse, ImportTenantUsersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting import_tenant_users via OpenAPI");
                        handle_import_tenant_users(config, req).await
                    }
                    None => {
                        warn!("Attempted import_tenant_users with unconnected client");
                        Err(ImportTenantUsersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn search_tenant_users(
                &mut self,
                req: SearchTenantUsersRequest,
            ) -> Result<SearchTenantUsersResponse, SearchTenantUsersError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting search_tenant_users via OpenAPI");
                        handle_search_tenant_users(config, req).await
                    }
                    None => {
                        warn!("Attempted search_tenant_users with unconnected client");
                        Err(SearchTenantUsersError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // Service Account methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn create_service_account(
                &mut self,
                req: CreateServiceAccountRequest,
            ) -> Result<CreateServiceAccountResponse, CreateServiceAccountError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_service_account via OpenAPI");
                        handle_create_service_account(config, req).await
                    }
                    None => {
                        warn!("Attempted create_service_account with unconnected client");
                        Err(CreateServiceAccountError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_service_account(
                &mut self,
                req: GetServiceAccountRequest,
            ) -> Result<GetServiceAccountResponse, GetServiceAccountError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_service_account via OpenAPI");
                        handle_get_service_account(config, req).await
                    }
                    None => {
                        warn!("Attempted get_service_account with unconnected client");
                        Err(GetServiceAccountError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_service_account(
                &mut self,
                req: UpdateServiceAccountRequest,
            ) -> Result<UpdateServiceAccountResponse, UpdateServiceAccountError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_service_account via OpenAPI");
                        handle_update_service_account(config, req).await
                    }
                    None => {
                        warn!("Attempted update_service_account with unconnected client");
                        Err(UpdateServiceAccountError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_service_account(
                &mut self,
                req: DeleteServiceAccountRequest,
            ) -> Result<DeleteServiceAccountResponse, DeleteServiceAccountError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_service_account via OpenAPI");
                        handle_delete_service_account(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_service_account with unconnected client");
                        Err(DeleteServiceAccountError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_service_accounts(
                &mut self,
                req: ListServiceAccountsRequest,
            ) -> Result<ListServiceAccountsResponse, ListServiceAccountsError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_service_accounts via OpenAPI");
                        handle_list_service_accounts(config, req).await
                    }
                    None => {
                        warn!("Attempted list_service_accounts with unconnected client");
                        Err(ListServiceAccountsError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn search_service_accounts(
                &mut self,
                req: SearchServiceAccountsRequest,
            ) -> Result<SearchServiceAccountsResponse, SearchServiceAccountsError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting search_service_accounts via OpenAPI");
                        handle_search_service_accounts(config, req).await
                    }
                    None => {
                        warn!("Attempted search_service_accounts with unconnected client");
                        Err(SearchServiceAccountsError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn generate_service_account_key(
                &mut self,
                req: GenerateServiceAccountKeyRequest,
            ) -> Result<GenerateServiceAccountKeyResponse, GenerateServiceAccountKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting generate_service_account_key via OpenAPI");
                        handle_generate_service_account_key(config, req).await
                    }
                    None => {
                        warn!("Attempted generate_service_account_key with unconnected client");
                        Err(GenerateServiceAccountKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_service_account_keys(
                &mut self,
                req: ListServiceAccountKeysRequest,
            ) -> Result<ListServiceAccountKeysResponse, ListServiceAccountKeysError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_service_account_keys via OpenAPI");
                        handle_list_service_account_keys(config, req).await
                    }
                    None => {
                        warn!("Attempted list_service_account_keys with unconnected client");
                        Err(ListServiceAccountKeysError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_service_account_key(
                &mut self,
                req: GetServiceAccountKeyRequest,
            ) -> Result<GetServiceAccountKeyResponse, GetServiceAccountKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_service_account_key via OpenAPI");
                        handle_get_service_account_key(config, req).await
                    }
                    None => {
                        warn!("Attempted get_service_account_key with unconnected client");
                        Err(GetServiceAccountKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_service_account_key(
                &mut self,
                req: DeleteServiceAccountKeyRequest,
            ) -> Result<DeleteServiceAccountKeyResponse, DeleteServiceAccountKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_service_account_key via OpenAPI");
                        handle_delete_service_account_key(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_service_account_key with unconnected client");
                        Err(DeleteServiceAccountKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn disable_service_account_key(
                &mut self,
                req: DisableServiceAccountKeyRequest,
            ) -> Result<DisableServiceAccountKeyResponse, DisableServiceAccountKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting disable_service_account_key via OpenAPI");
                        handle_disable_service_account_key(config, req).await
                    }
                    None => {
                        warn!("Attempted disable_service_account_key with unconnected client");
                        Err(DisableServiceAccountKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn enable_service_account_key(
                &mut self,
                req: EnableServiceAccountKeyRequest,
            ) -> Result<EnableServiceAccountKeyResponse, EnableServiceAccountKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting enable_service_account_key via OpenAPI");
                        handle_enable_service_account_key(config, req).await
                    }
                    None => {
                        warn!("Attempted enable_service_account_key with unconnected client");
                        Err(EnableServiceAccountKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // API Key methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn create_api_key(
                &mut self,
                req: CreateApiKeyRequest,
            ) -> Result<CreateApiKeyResponse, CreateApiKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_api_key via OpenAPI");
                        handle_create_api_key(config, req).await
                    }
                    None => {
                        warn!("Attempted create_api_key with unconnected client");
                        Err(CreateApiKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_api_key(
                &mut self,
                req: GetApiKeyRequest,
            ) -> Result<GetApiKeyResponse, GetApiKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_api_key via OpenAPI");
                        handle_get_api_key(config, req).await
                    }
                    None => {
                        warn!("Attempted get_api_key with unconnected client");
                        Err(GetApiKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_api_key(
                &mut self,
                req: UpdateApiKeyRequest,
            ) -> Result<UpdateApiKeyResponse, UpdateApiKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_api_key via OpenAPI");
                        handle_update_api_key(config, req).await
                    }
                    None => {
                        warn!("Attempted update_api_key with unconnected client");
                        Err(UpdateApiKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_api_key(
                &mut self,
                req: DeleteApiKeyRequest,
            ) -> Result<DeleteApiKeyResponse, DeleteApiKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_api_key via OpenAPI");
                        handle_delete_api_key(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_api_key with unconnected client");
                        Err(DeleteApiKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_api_keys(
                &mut self,
                req: ListApiKeysRequest,
            ) -> Result<ListApiKeysResponse, ListApiKeysError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_api_keys via OpenAPI");
                        handle_list_api_keys(config, req).await
                    }
                    None => {
                        warn!("Attempted list_api_keys with unconnected client");
                        Err(ListApiKeysError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn rotate_api_key(
                &mut self,
                req: RotateApiKeyRequest,
            ) -> Result<RotateApiKeyResponse, RotateApiKeyError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting rotate_api_key via OpenAPI");
                        handle_rotate_api_key(config, req).await
                    }
                    None => {
                        warn!("Attempted rotate_api_key with unconnected client");
                        Err(RotateApiKeyError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // MFA methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn list_project_user_mfa_factors(
                &mut self,
                req: ListProjectUserMfaFactorsRequest,
            ) -> Result<ListProjectUserMfaFactorsResponse, ListProjectUserMfaFactorsError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_project_user_mfa_factors via OpenAPI");
                        handle_list_project_user_mfa_factors(config, req).await
                    }
                    None => {
                        warn!("Attempted list_project_user_mfa_factors with unconnected client");
                        Err(ListProjectUserMfaFactorsError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_project_user_mfa_factor(
                &mut self,
                req: DeleteProjectUserMfaFactorRequest,
            ) -> Result<DeleteProjectUserMfaFactorResponse, DeleteProjectUserMfaFactorError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_project_user_mfa_factor via OpenAPI");
                        handle_delete_project_user_mfa_factor(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_project_user_mfa_factor with unconnected client");
                        Err(DeleteProjectUserMfaFactorError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn list_tenant_user_mfa_factors(
                &mut self,
                req: ListTenantUserMfaFactorsRequest,
            ) -> Result<ListTenantUserMfaFactorsResponse, ListTenantUserMfaFactorsError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting list_tenant_user_mfa_factors via OpenAPI");
                        handle_list_tenant_user_mfa_factors(config, req).await
                    }
                    None => {
                        warn!("Attempted list_tenant_user_mfa_factors with unconnected client");
                        Err(ListTenantUserMfaFactorsError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn delete_tenant_user_mfa_factor(
                &mut self,
                req: DeleteTenantUserMfaFactorRequest,
            ) -> Result<DeleteTenantUserMfaFactorResponse, DeleteTenantUserMfaFactorError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting delete_tenant_user_mfa_factor via OpenAPI");
                        handle_delete_tenant_user_mfa_factor(config, req).await
                    }
                    None => {
                        warn!("Attempted delete_tenant_user_mfa_factor with unconnected client");
                        Err(DeleteTenantUserMfaFactorError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // Token methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn create_custom_token(
                &mut self,
                req: CreateCustomTokenRequest,
            ) -> Result<CreateCustomTokenResponse, CreateCustomTokenError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_custom_token via OpenAPI");
                        handle_create_custom_token(config, req).await
                    }
                    None => {
                        warn!("Attempted create_custom_token with unconnected client");
                        Err(CreateCustomTokenError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn verify_id_token(
                &mut self,
                req: VerifyIdTokenRequest,
            ) -> Result<VerifyIdTokenResponse, VerifyIdTokenError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting verify_id_token via OpenAPI");
                        handle_verify_id_token(config, req).await
                    }
                    None => {
                        warn!("Attempted verify_id_token with unconnected client");
                        Err(VerifyIdTokenError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn create_session_cookie(
                &mut self,
                req: CreateSessionCookieRequest,
            ) -> Result<CreateSessionCookieResponse, CreateSessionCookieError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting create_session_cookie via OpenAPI");
                        handle_create_session_cookie(config, req).await
                    }
                    None => {
                        warn!("Attempted create_session_cookie with unconnected client");
                        Err(CreateSessionCookieError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn verify_session_cookie(
                &mut self,
                req: VerifySessionCookieRequest,
            ) -> Result<VerifySessionCookieResponse, VerifySessionCookieError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting verify_session_cookie via OpenAPI");
                        handle_verify_session_cookie(config, req).await
                    }
                    None => {
                        warn!("Attempted verify_session_cookie with unconnected client");
                        Err(VerifySessionCookieError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn revoke_refresh_tokens(
                &mut self,
                req: RevokeRefreshTokensRequest,
            ) -> Result<RevokeRefreshTokensResponse, RevokeRefreshTokensError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting revoke_refresh_tokens via OpenAPI");
                        handle_revoke_refresh_tokens(config, req).await
                    }
                    None => {
                        warn!("Attempted revoke_refresh_tokens with unconnected client");
                        Err(RevokeRefreshTokensError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            // =========================================================================
            // Usage & Quota methods
            // =========================================================================

            #[instrument(skip(self, req), level = "debug")]
            async fn get_project_usage(
                &mut self,
                req: GetProjectUsageRequest,
            ) -> Result<GetProjectUsageResponse, GetProjectUsageError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_project_usage via OpenAPI");
                        handle_get_project_usage(config, req).await
                    }
                    None => {
                        warn!("Attempted get_project_usage with unconnected client");
                        Err(GetProjectUsageError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn get_project_quotas(
                &mut self,
                req: GetProjectQuotasRequest,
            ) -> Result<GetProjectQuotasResponse, GetProjectQuotasError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting get_project_quotas via OpenAPI");
                        handle_get_project_quotas(config, req).await
                    }
                    None => {
                        warn!("Attempted get_project_quotas with unconnected client");
                        Err(GetProjectQuotasError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
            }

            #[instrument(skip(self, req), level = "debug")]
            async fn update_project_quotas(
                &mut self,
                req: UpdateProjectQuotasRequest,
            ) -> Result<UpdateProjectQuotasResponse, UpdateProjectQuotasError> {
                match &self.config {
                    Some(config) => {
                        debug!("Attempting update_project_quotas via OpenAPI");
                        handle_update_project_quotas(config, req).await
                    }
                    None => {
                        warn!("Attempted update_project_quotas with unconnected client");
                        Err(UpdateProjectQuotasError::PistachioApiClientError(
                            PistachioApiClientError::NotConnected,
                        ))
                    }
                }
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
