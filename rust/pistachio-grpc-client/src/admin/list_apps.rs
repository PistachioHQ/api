use libgn::app::App;
use pistachio_api_common::admin::app::{ListAppsError, ListAppsRequest, ListAppsResponse};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_list_apps<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListAppsRequest,
) -> Result<ListAppsResponse, ListAppsError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_apps(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_apps response");
            match status.code() {
                Code::InvalidArgument => {
                    ListAppsError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => ListAppsError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    ListAppsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListAppsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => ListAppsError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    ListAppsError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListAppsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListAppsResponse::from_proto(response).map_err(ListAppsError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListAppsRequest> for ListAppsRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListAppsRequest {
        pistachio_api::pistachio::admin::v1::ListAppsRequest {
            project_id: self.project_id.to_string(),
            pagination: Some(self.pagination.into_proto()),
            show_deleted: self.show_deleted,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListAppsResponse> for ListAppsResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListAppsResponse,
    ) -> Result<Self, Self::Error> {
        let apps = proto
            .apps
            .into_iter()
            .map(App::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = proto
            .pagination
            .map(PaginationMeta::from_proto)
            .transpose()?
            .unwrap_or_default();

        Ok(Self { apps, pagination })
    }
}
