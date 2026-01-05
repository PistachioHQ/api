use pistachio_api_common::admin::user::{
    GetProjectUserError, GetProjectUserRequest, GetProjectUserResponse, User,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_project_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetProjectUserRequest,
) -> Result<GetProjectUserResponse, GetProjectUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_project_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_project_user response");
            match status.code() {
                Code::InvalidArgument => {
                    GetProjectUserError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => GetProjectUserError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    GetProjectUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetProjectUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetProjectUserError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetProjectUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetProjectUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetProjectUserResponse::from_proto(response)
        .map_err(GetProjectUserError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetProjectUserRequest>
    for GetProjectUserRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetProjectUserRequest {
        pistachio_api::pistachio::admin::v1::GetProjectUserRequest {
            project_id: self.project_id.to_string(),
            pistachio_id: self.pistachio_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetProjectUserResponse>
    for GetProjectUserResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetProjectUserResponse,
    ) -> Result<Self, Self::Error> {
        let user_proto =
            proto
                .user
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "user",
                ))?;

        let user = User::from_proto(user_proto)?;

        Ok(Self { user })
    }
}
