use pistachio_api_common::admin::user::{
    UpdateTenantUserError, UpdateTenantUserRequest, UpdateTenantUserResponse, User,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_update_tenant_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateTenantUserRequest,
) -> Result<UpdateTenantUserResponse, UpdateTenantUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_tenant_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_tenant_user response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateTenantUserError::BadRequest(error_details_from_status(&status))
                }
                Code::AlreadyExists => UpdateTenantUserError::AlreadyExists,
                Code::NotFound => {
                    UpdateTenantUserError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    UpdateTenantUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateTenantUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UpdateTenantUserError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UpdateTenantUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateTenantUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateTenantUserResponse::from_proto(response)
        .map_err(UpdateTenantUserError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateTenantUserRequest>
    for UpdateTenantUserRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateTenantUserRequest {
        pistachio_api::pistachio::admin::v1::UpdateTenantUserRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            pistachio_id: self.pistachio_id.to_string(),
            email: self.email,
            email_verified: self.email_verified,
            phone_number: self.phone_number,
            display_name: self.display_name,
            photo_url: self.photo_url,
            disabled: self.disabled,
            custom_claims: self.custom_claims.map(|claims| {
                pistachio_api::pistachio::admin::v1::CustomClaimsUpdate {
                    claims: Some(prost_types::Struct {
                        fields: claims
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    k,
                                    prost_types::Value {
                                        kind: Some(prost_types::value::Kind::StringValue(v)),
                                    },
                                )
                            })
                            .collect(),
                    }),
                }
            }),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateTenantUserResponse>
    for UpdateTenantUserResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateTenantUserResponse,
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
