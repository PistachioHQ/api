use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{
    SearchTenantsError, SearchTenantsRequest, SearchTenantsResponse,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, problem_details_from_status};

pub(crate) async fn handle_search_tenants<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: SearchTenantsRequest,
) -> Result<SearchTenantsResponse, SearchTenantsError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .search_tenants(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in search_tenants response");
            match status.code() {
                Code::InvalidArgument => {
                    SearchTenantsError::BadRequest(problem_details_from_status(&status, 400))
                }
                Code::NotFound => {
                    SearchTenantsError::NotFound(problem_details_from_status(&status, 404))
                }
                Code::Unauthenticated => {
                    SearchTenantsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    SearchTenantsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => SearchTenantsError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    SearchTenantsError::ServiceUnavailable(status.message().to_string())
                }
                _ => SearchTenantsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    SearchTenantsResponse::from_proto(response).map_err(SearchTenantsError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::SearchTenantsRequest> for SearchTenantsRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::SearchTenantsRequest {
        pistachio_api::pistachio::admin::v1::SearchTenantsRequest {
            project_id: self.project_id.to_string(),
            params: Some(self.params.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::SearchTenantsResponse>
    for SearchTenantsResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::SearchTenantsResponse,
    ) -> Result<Self, Self::Error> {
        let tenants = proto
            .tenants
            .into_iter()
            .map(Tenant::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = proto
            .pagination
            .map(PaginationMeta::from_proto)
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            tenants,
            pagination,
        })
    }
}
