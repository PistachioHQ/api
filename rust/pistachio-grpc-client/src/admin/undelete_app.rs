use libgn::app::App;
use pistachio_api_common::admin::app::{UndeleteAppError, UndeleteAppRequest, UndeleteAppResponse};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, problem_details_from_status};

pub(crate) async fn handle_undelete_app<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UndeleteAppRequest,
) -> Result<UndeleteAppResponse, UndeleteAppError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .undelete_app(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in undelete_app response");
            match status.code() {
                Code::InvalidArgument => {
                    UndeleteAppError::BadRequest(problem_details_from_status(&status, 400))
                }
                Code::NotFound => {
                    UndeleteAppError::NotFound(problem_details_from_status(&status, 404))
                }
                Code::FailedPrecondition => UndeleteAppError::FailedPrecondition,
                Code::Unauthenticated => {
                    UndeleteAppError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UndeleteAppError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UndeleteAppError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UndeleteAppError::ServiceUnavailable(status.message().to_string())
                }
                _ => UndeleteAppError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UndeleteAppResponse::from_proto(response).map_err(UndeleteAppError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UndeleteAppRequest> for UndeleteAppRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UndeleteAppRequest {
        pistachio_api::pistachio::admin::v1::UndeleteAppRequest {
            project_id: self.project_id.to_string(),
            app_id: self.app_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UndeleteAppResponse> for UndeleteAppResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UndeleteAppResponse,
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
