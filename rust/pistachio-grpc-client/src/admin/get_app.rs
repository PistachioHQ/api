use libgn::app::App;
use pistachio_api_common::admin::app::{GetAppError, GetAppRequest, GetAppResponse};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto};

pub(crate) async fn handle_get_app<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetAppRequest,
) -> Result<GetAppResponse, GetAppError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_app(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_app response");
            match status.code() {
                Code::InvalidArgument => GetAppError::BadRequest(status.message().to_string()),
                Code::NotFound => GetAppError::NotFound,
                Code::Unauthenticated => GetAppError::Unauthenticated(status.message().to_string()),
                Code::PermissionDenied => {
                    GetAppError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetAppError::ServiceError(status.message().to_string()),
                Code::Unavailable => GetAppError::ServiceUnavailable(status.message().to_string()),
                _ => GetAppError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetAppResponse::from_proto(response).map_err(GetAppError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetAppRequest> for GetAppRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetAppRequest {
        pistachio_api::pistachio::admin::v1::GetAppRequest {
            project_id: self.project_id.to_string(),
            app_id: self.app_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetAppResponse> for GetAppResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetAppResponse,
    ) -> Result<Self, Self::Error> {
        let app_proto =
            proto
                .app
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "app",
                ))?;

        let app = App::from_proto(app_proto)?;

        Ok(Self { app })
    }
}
