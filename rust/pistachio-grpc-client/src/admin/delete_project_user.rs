use pistachio_api_common::admin::user::{
    DeleteProjectUserError, DeleteProjectUserRequest, DeleteProjectUserResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{IntoProto, error_details_from_status};

pub(crate) async fn handle_delete_project_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteProjectUserRequest,
) -> Result<DeleteProjectUserResponse, DeleteProjectUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let _response = client
        .delete_project_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_project_user response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteProjectUserError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteProjectUserError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteProjectUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteProjectUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteProjectUserError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DeleteProjectUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteProjectUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(DeleteProjectUserResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteProjectUserRequest>
    for DeleteProjectUserRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteProjectUserRequest {
        pistachio_api::pistachio::admin::v1::DeleteProjectUserRequest {
            project_id: self.project_id.to_string(),
            pistachio_id: self.pistachio_id.to_string(),
        }
    }
}
