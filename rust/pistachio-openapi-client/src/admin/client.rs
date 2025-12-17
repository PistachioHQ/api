use std::sync::Arc;

use cfg_if::cfg_if;
use pistachio_api_common::admin::client::PistachioAdminClient;
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse,
};
use pistachio_api_common::credentials::AdminCredentials;
use pistachio_api_common::error::PistachioApiClientError;
use tracing::{debug, instrument};

use super::create_project::handle_create_project;

/// OpenAPI/REST client for the Pistachio Admin API.
#[derive(Debug, Clone)]
pub struct AdminClient {
    config: Arc<crate::generated_admin::apis::configuration::Configuration>,
}

cfg_if! {
    if #[cfg(any(feature = "single-threaded", target_arch = "wasm32"))] {
        #[async_trait::async_trait(?Send)]
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
        }
    } else {
        #[async_trait::async_trait]
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
        }
    }
}
