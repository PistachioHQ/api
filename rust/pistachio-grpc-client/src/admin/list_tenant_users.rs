use pistachio_api_common::admin::user::{
    ListTenantUsersError, ListTenantUsersRequest, ListTenantUsersResponse, User,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_list_tenant_users<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListTenantUsersRequest,
) -> Result<ListTenantUsersResponse, ListTenantUsersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_tenant_users(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_tenant_users response");
            match status.code() {
                Code::InvalidArgument => {
                    ListTenantUsersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListTenantUsersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListTenantUsersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListTenantUsersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => ListTenantUsersError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    ListTenantUsersError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListTenantUsersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListTenantUsersResponse::from_proto(response)
        .map_err(ListTenantUsersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListTenantUsersRequest>
    for ListTenantUsersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListTenantUsersRequest {
        pistachio_api::pistachio::admin::v1::ListTenantUsersRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            pagination: Some(self.pagination.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListTenantUsersResponse>
    for ListTenantUsersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListTenantUsersResponse,
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
