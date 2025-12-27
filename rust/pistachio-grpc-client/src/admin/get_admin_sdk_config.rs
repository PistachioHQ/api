use libgn::project::ProjectId;
use pistachio_api_common::admin::project::{
    GetAdminSdkConfigError, GetAdminSdkConfigRequest, GetAdminSdkConfigResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto};

pub(crate) async fn handle_get_admin_sdk_config<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetAdminSdkConfigRequest,
) -> Result<GetAdminSdkConfigResponse, GetAdminSdkConfigError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_admin_sdk_config(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_admin_sdk_config response");
            match status.code() {
                Code::InvalidArgument => {
                    GetAdminSdkConfigError::BadRequest(status.message().to_string())
                }
                Code::NotFound => GetAdminSdkConfigError::NotFound,
                Code::Unauthenticated => {
                    GetAdminSdkConfigError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetAdminSdkConfigError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    GetAdminSdkConfigError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    GetAdminSdkConfigError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetAdminSdkConfigError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetAdminSdkConfigResponse::from_proto(response)
        .map_err(GetAdminSdkConfigError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetAdminSdkConfigRequest>
    for GetAdminSdkConfigRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetAdminSdkConfigRequest {
        pistachio_api::pistachio::admin::v1::GetAdminSdkConfigRequest {
            project_id: self.project_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetAdminSdkConfigResponse>
    for GetAdminSdkConfigResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetAdminSdkConfigResponse,
    ) -> Result<Self, Self::Error> {
        let project_id = ProjectId::parse(&proto.project_id).map_err(|_| {
            pistachio_api_common::error::ValidationError::InvalidValue("project_id")
        })?;

        Ok(Self {
            project_id,
            storage_bucket: proto.storage_bucket,
            location_id: proto.location_id,
        })
    }
}
