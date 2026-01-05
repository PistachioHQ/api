use pistachio_api_common::admin::user::{
    UpdateProjectUserError, UpdateProjectUserRequest, UpdateProjectUserResponse, User,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_update_project_user<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateProjectUserRequest,
) -> Result<UpdateProjectUserResponse, UpdateProjectUserError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_project_user(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_project_user response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateProjectUserError::BadRequest(error_details_from_status(&status))
                }
                Code::AlreadyExists => UpdateProjectUserError::AlreadyExists,
                Code::NotFound => {
                    UpdateProjectUserError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    UpdateProjectUserError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateProjectUserError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    UpdateProjectUserError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    UpdateProjectUserError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateProjectUserError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateProjectUserResponse::from_proto(response)
        .map_err(UpdateProjectUserError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateProjectUserRequest>
    for UpdateProjectUserRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateProjectUserRequest {
        pistachio_api::pistachio::admin::v1::UpdateProjectUserRequest {
            project_id: self.project_id.to_string(),
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

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateProjectUserResponse>
    for UpdateProjectUserResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateProjectUserResponse,
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
