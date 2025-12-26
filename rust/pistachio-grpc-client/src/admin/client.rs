use pistachio_api_common::admin::client::PistachioAdminClient;
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse, DeleteProjectError,
    DeleteProjectRequest, DeleteProjectResponse, GetAdminSdkConfigError, GetAdminSdkConfigRequest,
    GetAdminSdkConfigResponse, GetProjectError, GetProjectRequest, GetProjectResponse,
    ListProjectsError, ListProjectsRequest, ListProjectsResponse, SearchProjectsError,
    SearchProjectsRequest, SearchProjectsResponse, UndeleteProjectError, UndeleteProjectRequest,
    UndeleteProjectResponse, UpdateProjectError, UpdateProjectRequest, UpdateProjectResponse,
};
use pistachio_api_common::credentials::AdminCredentials;
use pistachio_api_common::error::PistachioApiClientError;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tonic::{Request, Status};
use tracing::{debug, error, info, instrument, warn};

use pistachio_api::pistachio::admin::v1::project_management_client::ProjectManagementClient;

use super::create_project::handle_create_project;
use super::delete_project::handle_delete_project;
use super::get_admin_sdk_config::handle_get_admin_sdk_config;
use super::get_project::handle_get_project;
use super::list_projects::handle_list_projects;
use super::search_projects::handle_search_projects;
use super::undelete_project::handle_undelete_project;
use super::update_project::handle_update_project;

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
    ProjectManagementClient<InterceptedService<Channel, AdminAuthInterceptor>>;

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
        let client = ProjectManagementClient::with_interceptor(channel, interceptor);

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
                            ProjectManagementClient::with_interceptor(channel, interceptor);
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
}
