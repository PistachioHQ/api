use pistachio_api_common::admin::auth_provider::{
    DeleteTenantAuthProviderError, DeleteTenantAuthProviderRequest,
    DeleteTenantAuthProviderResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{IntoProto, error_details_from_status};

pub(crate) async fn handle_delete_tenant_auth_provider<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteTenantAuthProviderRequest,
) -> Result<DeleteTenantAuthProviderResponse, DeleteTenantAuthProviderError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    client
        .delete_tenant_auth_provider(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_tenant_auth_provider response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteTenantAuthProviderError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteTenantAuthProviderError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteTenantAuthProviderError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteTenantAuthProviderError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteTenantAuthProviderError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DeleteTenantAuthProviderError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteTenantAuthProviderError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(DeleteTenantAuthProviderResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteTenantAuthProviderRequest>
    for DeleteTenantAuthProviderRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteTenantAuthProviderRequest {
        pistachio_api::pistachio::admin::v1::DeleteTenantAuthProviderRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            provider_id: self.provider_id,
        }
    }
}
