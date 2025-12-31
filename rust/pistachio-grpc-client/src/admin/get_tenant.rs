use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{GetTenantError, GetTenantRequest, GetTenantResponse};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_tenant<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetTenantRequest,
) -> Result<GetTenantResponse, GetTenantError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_tenant(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_tenant response");
            match status.code() {
                Code::InvalidArgument => {
                    GetTenantError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => GetTenantError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    GetTenantError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetTenantError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetTenantError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetTenantError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetTenantError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetTenantResponse::from_proto(response).map_err(GetTenantError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetTenantRequest> for GetTenantRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::GetTenantRequest {
        pistachio_api::pistachio::admin::v1::GetTenantRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetTenantResponse> for GetTenantResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetTenantResponse,
    ) -> Result<Self, Self::Error> {
        let tenant_proto =
            proto
                .tenant
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "tenant",
                ))?;

        let tenant = Tenant::from_proto(tenant_proto)?;

        Ok(Self { tenant })
    }
}
