use pistachio_api_common::admin::app::{DeleteAppError, DeleteAppRequest, DeleteAppResponse};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::IntoProto;

pub(crate) async fn handle_delete_app<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteAppRequest,
) -> Result<DeleteAppResponse, DeleteAppError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let _response = client
        .delete_app(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_app response");
            match status.code() {
                Code::InvalidArgument => DeleteAppError::BadRequest(status.message().to_string()),
                Code::NotFound => DeleteAppError::NotFound,
                Code::Unauthenticated => {
                    DeleteAppError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteAppError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => DeleteAppError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    DeleteAppError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteAppError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(DeleteAppResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteAppRequest> for DeleteAppRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteAppRequest {
        pistachio_api::pistachio::admin::v1::DeleteAppRequest {
            project_id: self.project_id.to_string(),
            app_id: self.app_id.to_string(),
        }
    }
}
