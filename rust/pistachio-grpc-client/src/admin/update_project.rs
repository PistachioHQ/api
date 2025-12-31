use libgn::project::Project;
use pistachio_api_common::admin::project::{
    UpdateProjectError, UpdateProjectRequest, UpdateProjectResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_update_project<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateProjectRequest,
) -> Result<UpdateProjectResponse, UpdateProjectError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_project(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_project response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateProjectError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => UpdateProjectError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    UpdateProjectError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateProjectError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UpdateProjectError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UpdateProjectError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateProjectError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateProjectResponse::from_proto(response).map_err(UpdateProjectError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateProjectRequest> for UpdateProjectRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateProjectRequest {
        pistachio_api::pistachio::admin::v1::UpdateProjectRequest {
            project_id: self.project_id.to_string(),
            // With `optional` in proto3, None means "don't change" and Some("") would be invalid
            display_name: self.display_name.map(|name| name.to_string()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateProjectResponse>
    for UpdateProjectResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateProjectResponse,
    ) -> Result<Self, Self::Error> {
        let project_proto =
            proto
                .project
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "project",
                ))?;

        let project = Project::from_proto(project_proto)?;

        Ok(Self { project })
    }
}
