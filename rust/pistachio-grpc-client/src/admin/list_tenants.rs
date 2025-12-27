use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{
    ListTenantsError, ListTenantsRequest, ListTenantsResponse,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto};

pub(crate) async fn handle_list_tenants<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListTenantsRequest,
) -> Result<ListTenantsResponse, ListTenantsError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_tenants(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_tenants response");
            match status.code() {
                Code::InvalidArgument => ListTenantsError::BadRequest(status.message().to_string()),
                Code::NotFound => ListTenantsError::NotFound,
                Code::Unauthenticated => {
                    ListTenantsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListTenantsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => ListTenantsError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    ListTenantsError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListTenantsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListTenantsResponse::from_proto(response).map_err(ListTenantsError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListTenantsRequest> for ListTenantsRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListTenantsRequest {
        pistachio_api::pistachio::admin::v1::ListTenantsRequest {
            project_id: self.project_id.to_string(),
            pagination: Some(self.pagination.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListTenantsResponse> for ListTenantsResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListTenantsResponse,
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
