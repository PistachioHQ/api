use pistachio_api_common::admin::user::{
    CreateTenantUserError, CreateTenantUserRequest, CreateTenantUserResponse, User,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_create_tenant_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateTenantUserRequest,
) -> Result<CreateTenantUserResponse, CreateTenantUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .create_tenant_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_tenant_user response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateTenantUserError::BadRequest(error_details_from_status(&status))
                }
                Code::AlreadyExists => CreateTenantUserError::AlreadyExists,
                Code::NotFound => {
                    CreateTenantUserError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    CreateTenantUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateTenantUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => CreateTenantUserError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    CreateTenantUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateTenantUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    CreateTenantUserResponse::from_proto(response)
        .map_err(CreateTenantUserError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::CreateTenantUserRequest>
    for CreateTenantUserRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::CreateTenantUserRequest {
        pistachio_api::pistachio::admin::v1::CreateTenantUserRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            email: self.email.unwrap_or_default(),
            email_verified: self.email_verified,
            phone_number: self.phone_number.unwrap_or_default(),
            display_name: self.display_name.unwrap_or_default(),
            photo_url: self.photo_url.unwrap_or_default(),
            disabled: self.disabled,
            custom_claims: self.custom_claims.map(|claims| prost_types::Struct {
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
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::CreateTenantUserResponse>
    for CreateTenantUserResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::CreateTenantUserResponse,
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
