use libgn::project::Project;
use pistachio_api_common::admin::project::{
    GetProjectError, GetProjectRequest, GetProjectResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_project<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetProjectRequest,
) -> Result<GetProjectResponse, GetProjectError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_project(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_project response");
            match status.code() {
                Code::InvalidArgument => {
                    GetProjectError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => GetProjectError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    GetProjectError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetProjectError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetProjectError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetProjectError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetProjectError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetProjectResponse::from_proto(response).map_err(GetProjectError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetProjectRequest> for GetProjectRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetProjectRequest {
        pistachio_api::pistachio::admin::v1::GetProjectRequest {
            project_id: self.project_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetProjectResponse> for GetProjectResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetProjectResponse,
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
