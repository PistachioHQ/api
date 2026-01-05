use pistachio_api_common::admin::user::{
    GetTenantUserError, GetTenantUserRequest, GetTenantUserResponse, User,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_tenant_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetTenantUserRequest,
) -> Result<GetTenantUserResponse, GetTenantUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_tenant_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_tenant_user response");
            match status.code() {
                Code::InvalidArgument => {
                    GetTenantUserError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => GetTenantUserError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    GetTenantUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetTenantUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetTenantUserError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetTenantUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetTenantUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetTenantUserResponse::from_proto(response).map_err(GetTenantUserError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetTenantUserRequest> for GetTenantUserRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetTenantUserRequest {
        pistachio_api::pistachio::admin::v1::GetTenantUserRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            pistachio_id: self.pistachio_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetTenantUserResponse>
    for GetTenantUserResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetTenantUserResponse,
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
