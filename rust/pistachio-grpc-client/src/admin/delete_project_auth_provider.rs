use pistachio_api_common::admin::auth_provider::{
    DeleteProjectAuthProviderError, DeleteProjectAuthProviderRequest,
    DeleteProjectAuthProviderResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{IntoProto, error_details_from_status};

pub(crate) async fn handle_delete_project_auth_provider<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteProjectAuthProviderRequest,
) -> Result<DeleteProjectAuthProviderResponse, DeleteProjectAuthProviderError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    client
        .delete_project_auth_provider(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_project_auth_provider response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteProjectAuthProviderError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteProjectAuthProviderError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteProjectAuthProviderError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteProjectAuthProviderError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteProjectAuthProviderError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DeleteProjectAuthProviderError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteProjectAuthProviderError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(DeleteProjectAuthProviderResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteProjectAuthProviderRequest>
    for DeleteProjectAuthProviderRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteProjectAuthProviderRequest {
        pistachio_api::pistachio::admin::v1::DeleteProjectAuthProviderRequest {
            project_id: self.project_id.to_string(),
            provider_id: self.provider_id.to_string(),
        }
    }
}
