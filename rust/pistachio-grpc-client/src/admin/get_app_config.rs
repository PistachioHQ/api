use pistachio_api_common::admin::app::{
    GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_app_config<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetAppConfigRequest,
) -> Result<GetAppConfigResponse, GetAppConfigError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_app_config(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_app_config response");
            match status.code() {
                Code::InvalidArgument => {
                    GetAppConfigError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => GetAppConfigError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    GetAppConfigError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetAppConfigError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetAppConfigError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetAppConfigError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetAppConfigError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetAppConfigResponse::from_proto(response).map_err(GetAppConfigError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetAppConfigRequest> for GetAppConfigRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetAppConfigRequest {
        pistachio_api::pistachio::admin::v1::GetAppConfigRequest {
            project_id: self.project_id.to_string(),
            app_id: self.app_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetAppConfigResponse> for GetAppConfigResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetAppConfigResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            config_file_contents: proto.config_file_contents,
            config_filename: proto.config_filename,
        })
    }
}
