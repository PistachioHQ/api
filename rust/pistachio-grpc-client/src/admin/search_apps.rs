use libgn::app::App;
use pistachio_api_common::admin::app::{SearchAppsError, SearchAppsRequest, SearchAppsResponse};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_search_apps<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: SearchAppsRequest,
) -> Result<SearchAppsResponse, SearchAppsError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .search_apps(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in search_apps response");
            match status.code() {
                Code::InvalidArgument => {
                    SearchAppsError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => SearchAppsError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    SearchAppsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    SearchAppsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => SearchAppsError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    SearchAppsError::ServiceUnavailable(status.message().to_string())
                }
                _ => SearchAppsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    SearchAppsResponse::from_proto(response).map_err(SearchAppsError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::SearchAppsRequest> for SearchAppsRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::SearchAppsRequest {
        pistachio_api::pistachio::admin::v1::SearchAppsRequest {
            project_id: self.project_id.to_string(),
            params: Some(self.params.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::SearchAppsResponse> for SearchAppsResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::SearchAppsResponse,
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
