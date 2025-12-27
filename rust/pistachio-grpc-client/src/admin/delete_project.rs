use pistachio_api_common::admin::project::{
    DeleteProjectError, DeleteProjectRequest, DeleteProjectResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::IntoProto;

pub(crate) async fn handle_delete_project<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteProjectRequest,
) -> Result<DeleteProjectResponse, DeleteProjectError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let _response = client
        .delete_project(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_project response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteProjectError::BadRequest(status.message().to_string())
                }
                Code::NotFound => DeleteProjectError::NotFound,
                Code::Unauthenticated => {
                    DeleteProjectError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteProjectError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => DeleteProjectError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    DeleteProjectError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteProjectError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(DeleteProjectResponse {})
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::DeleteProjectRequest> for DeleteProjectRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::DeleteProjectRequest {
        pistachio_api::pistachio::admin::v1::DeleteProjectRequest {
            project_id: self.project_id.to_string(),
        }
    }
}
