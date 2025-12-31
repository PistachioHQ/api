use pistachio_api_common::admin::tenant::{
    DeleteTenantError, DeleteTenantRequest, DeleteTenantResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{IntoProto, error_details_from_status};

pub(crate) async fn handle_delete_tenant<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteTenantRequest,
) -> Result<DeleteTenantResponse, DeleteTenantError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let _response = client
        .delete_tenant(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_tenant response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteTenantError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => DeleteTenantError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    DeleteTenantError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteTenantError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => DeleteTenantError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    DeleteTenantError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteTenantError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(DeleteTenantResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteTenantRequest> for DeleteTenantRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteTenantRequest {
        pistachio_api::pistachio::admin::v1::DeleteTenantRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
        }
    }
}
