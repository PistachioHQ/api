use pistachio_api_common::admin::user::{
    ListProjectUsersError, ListProjectUsersRequest, ListProjectUsersResponse, User,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_list_project_users<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListProjectUsersRequest,
) -> Result<ListProjectUsersResponse, ListProjectUsersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_project_users(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_project_users response");
            match status.code() {
                Code::InvalidArgument => {
                    ListProjectUsersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListProjectUsersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListProjectUsersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListProjectUsersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => ListProjectUsersError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    ListProjectUsersError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListProjectUsersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListProjectUsersResponse::from_proto(response)
        .map_err(ListProjectUsersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListProjectUsersRequest>
    for ListProjectUsersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListProjectUsersRequest {
        pistachio_api::pistachio::admin::v1::ListProjectUsersRequest {
            project_id: self.project_id.to_string(),
            pagination: Some(self.pagination.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListProjectUsersResponse>
    for ListProjectUsersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListProjectUsersResponse,
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
