use libgn::app::{App, PlatformConfig};
use pistachio_api_common::admin::app::{UpdateAppError, UpdateAppRequest, UpdateAppResponse};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto};

pub(crate) async fn handle_update_app<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateAppRequest,
) -> Result<UpdateAppResponse, UpdateAppError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_app(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_app response");
            match status.code() {
                Code::InvalidArgument => UpdateAppError::BadRequest(status.message().to_string()),
                Code::NotFound => UpdateAppError::NotFound,
                Code::Unauthenticated => {
                    UpdateAppError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateAppError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UpdateAppError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UpdateAppError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateAppError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateAppResponse::from_proto(response).map_err(UpdateAppError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateAppRequest> for UpdateAppRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateAppRequest {
        use pistachio_api::pistachio::admin::v1::update_app_request::PlatformConfig as ProtoPlatformConfig;

        let platform_config = self.platform_config.map(|config| match config {
            PlatformConfig::Ios(c) => ProtoPlatformConfig::Ios(c.into_proto()),
            PlatformConfig::Android(c) => ProtoPlatformConfig::Android(c.into_proto()),
            PlatformConfig::Macos(c) => ProtoPlatformConfig::Macos(c.into_proto()),
            PlatformConfig::Windows(c) => ProtoPlatformConfig::Windows(c.into_proto()),
            PlatformConfig::Linux(c) => ProtoPlatformConfig::Linux(c.into_proto()),
            PlatformConfig::Web(c) => ProtoPlatformConfig::Web(c.into_proto()),
        });

        pistachio_api::pistachio::admin::v1::UpdateAppRequest {
            project_id: self.project_id.to_string(),
            app_id: self.app_id.to_string(),
            display_name: self.display_name.map(|d| d.to_string()),
            platform_config,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateAppResponse> for UpdateAppResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateAppResponse,
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
