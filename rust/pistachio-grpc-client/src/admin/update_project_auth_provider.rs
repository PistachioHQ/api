use pistachio_api_common::admin::auth_provider::{
    AuthProvider, UpdateProjectAuthProviderError, UpdateProjectAuthProviderRequest,
    UpdateProjectAuthProviderResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use super::list_project_auth_providers::auth_provider_config_to_proto;
use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_update_project_auth_provider<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateProjectAuthProviderRequest,
) -> Result<UpdateProjectAuthProviderResponse, UpdateProjectAuthProviderError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_project_auth_provider(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_project_auth_provider response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateProjectAuthProviderError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    UpdateProjectAuthProviderError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    UpdateProjectAuthProviderError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateProjectAuthProviderError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    UpdateProjectAuthProviderError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    UpdateProjectAuthProviderError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateProjectAuthProviderError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateProjectAuthProviderResponse::from_proto(response)
        .map_err(UpdateProjectAuthProviderError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateProjectAuthProviderRequest>
    for UpdateProjectAuthProviderRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateProjectAuthProviderRequest {
        pistachio_api::pistachio::admin::v1::UpdateProjectAuthProviderRequest {
            project_id: self.project_id.to_string(),
            provider_id: self.provider_id,
            enabled: self.enabled,
            display_order: self.display_order,
            config: self.config.as_ref().map(auth_provider_config_to_proto),
            client_secret: self.client_secret,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateProjectAuthProviderResponse>
    for UpdateProjectAuthProviderResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateProjectAuthProviderResponse,
    ) -> Result<Self, Self::Error> {
        let provider_proto =
            proto
                .provider
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "provider",
                ))?;

        let provider = AuthProvider::from_proto(provider_proto)?;

        Ok(Self { provider })
    }
}
