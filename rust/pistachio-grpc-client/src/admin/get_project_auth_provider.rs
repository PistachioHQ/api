use pistachio_api_common::admin::auth_provider::{
    AuthProvider, GetProjectAuthProviderError, GetProjectAuthProviderRequest,
    GetProjectAuthProviderResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_project_auth_provider<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetProjectAuthProviderRequest,
) -> Result<GetProjectAuthProviderResponse, GetProjectAuthProviderError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_project_auth_provider(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_project_auth_provider response");
            match status.code() {
                Code::InvalidArgument => {
                    GetProjectAuthProviderError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    GetProjectAuthProviderError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    GetProjectAuthProviderError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetProjectAuthProviderError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    GetProjectAuthProviderError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    GetProjectAuthProviderError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetProjectAuthProviderError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetProjectAuthProviderResponse::from_proto(response)
        .map_err(GetProjectAuthProviderError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetProjectAuthProviderRequest>
    for GetProjectAuthProviderRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetProjectAuthProviderRequest {
        pistachio_api::pistachio::admin::v1::GetProjectAuthProviderRequest {
            project_id: self.project_id.to_string(),
            provider_id: self.provider_id,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetProjectAuthProviderResponse>
    for GetProjectAuthProviderResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetProjectAuthProviderResponse,
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
