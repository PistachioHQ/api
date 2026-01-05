use pistachio_api_common::admin::user::{
    DeleteTenantUserError, DeleteTenantUserRequest, DeleteTenantUserResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{IntoProto, error_details_from_status};

pub(crate) async fn handle_delete_tenant_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteTenantUserRequest,
) -> Result<DeleteTenantUserResponse, DeleteTenantUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let _response = client
        .delete_tenant_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_tenant_user response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteTenantUserError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteTenantUserError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteTenantUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteTenantUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => DeleteTenantUserError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    DeleteTenantUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteTenantUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(DeleteTenantUserResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteTenantUserRequest>
    for DeleteTenantUserRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteTenantUserRequest {
        pistachio_api::pistachio::admin::v1::DeleteTenantUserRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            pistachio_id: self.pistachio_id.to_string(),
        }
    }
}
