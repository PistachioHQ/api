use pistachio_api_common::admin::user::{
    SearchProjectUsersError, SearchProjectUsersRequest, SearchProjectUsersResponse, User,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_search_project_users<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: SearchProjectUsersRequest,
) -> Result<SearchProjectUsersResponse, SearchProjectUsersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .search_project_users(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in search_project_users response");
            match status.code() {
                Code::InvalidArgument => {
                    SearchProjectUsersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    SearchProjectUsersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    SearchProjectUsersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    SearchProjectUsersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    SearchProjectUsersError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    SearchProjectUsersError::ServiceUnavailable(status.message().to_string())
                }
                _ => SearchProjectUsersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    SearchProjectUsersResponse::from_proto(response)
        .map_err(SearchProjectUsersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::SearchProjectUsersRequest>
    for SearchProjectUsersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::SearchProjectUsersRequest {
        pistachio_api::pistachio::admin::v1::SearchProjectUsersRequest {
            project_id: self.project_id.to_string(),
            params: Some(self.params.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::SearchProjectUsersResponse>
    for SearchProjectUsersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::SearchProjectUsersResponse,
    ) -> Result<Self, Self::Error> {
        let users = proto
            .users
            .into_iter()
            .map(User::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = proto
            .pagination
            .map(PaginationMeta::from_proto)
            .transpose()?
            .unwrap_or_default();

        Ok(Self { users, pagination })
    }
}
