use pistachio_api_common::admin::user::{
    SearchTenantUsersError, SearchTenantUsersRequest, SearchTenantUsersResponse, User,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_search_tenant_users<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: SearchTenantUsersRequest,
) -> Result<SearchTenantUsersResponse, SearchTenantUsersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .search_tenant_users(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in search_tenant_users response");
            match status.code() {
                Code::InvalidArgument => {
                    SearchTenantUsersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    SearchTenantUsersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    SearchTenantUsersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    SearchTenantUsersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    SearchTenantUsersError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    SearchTenantUsersError::ServiceUnavailable(status.message().to_string())
                }
                _ => SearchTenantUsersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    SearchTenantUsersResponse::from_proto(response)
        .map_err(SearchTenantUsersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::SearchTenantUsersRequest>
    for SearchTenantUsersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::SearchTenantUsersRequest {
        pistachio_api::pistachio::admin::v1::SearchTenantUsersRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            params: Some(self.params.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::SearchTenantUsersResponse>
    for SearchTenantUsersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::SearchTenantUsersResponse,
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
