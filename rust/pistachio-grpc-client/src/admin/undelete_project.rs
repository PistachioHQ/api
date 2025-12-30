use libgn::project::Project;
use pistachio_api_common::admin::project::{
    UndeleteProjectError, UndeleteProjectRequest, UndeleteProjectResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, problem_details_from_status};

pub(crate) async fn handle_undelete_project<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UndeleteProjectRequest,
) -> Result<UndeleteProjectResponse, UndeleteProjectError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .undelete_project(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in undelete_project response");
            match status.code() {
                Code::InvalidArgument => {
                    UndeleteProjectError::BadRequest(problem_details_from_status(&status, 400))
                }
                Code::NotFound => {
                    UndeleteProjectError::NotFound(problem_details_from_status(&status, 404))
                }
                Code::FailedPrecondition => {
                    UndeleteProjectError::FailedPrecondition(status.message().to_string())
                }
                Code::Unauthenticated => {
                    UndeleteProjectError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UndeleteProjectError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UndeleteProjectError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UndeleteProjectError::ServiceUnavailable(status.message().to_string())
                }
                _ => UndeleteProjectError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UndeleteProjectResponse::from_proto(response)
        .map_err(UndeleteProjectError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UndeleteProjectRequest>
    for UndeleteProjectRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UndeleteProjectRequest {
        pistachio_api::pistachio::admin::v1::UndeleteProjectRequest {
            project_id: self.project_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UndeleteProjectResponse>
    for UndeleteProjectResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UndeleteProjectResponse,
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
