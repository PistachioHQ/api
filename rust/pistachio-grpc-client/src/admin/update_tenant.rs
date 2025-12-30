use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{
    UpdateTenantError, UpdateTenantRequest, UpdateTenantResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, problem_details_from_status};

pub(crate) async fn handle_update_tenant<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateTenantRequest,
) -> Result<UpdateTenantResponse, UpdateTenantError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_tenant(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_tenant response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateTenantError::BadRequest(problem_details_from_status(&status, 400))
                }
                Code::NotFound => {
                    UpdateTenantError::NotFound(problem_details_from_status(&status, 404))
                }
                Code::Unauthenticated => {
                    UpdateTenantError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateTenantError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UpdateTenantError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UpdateTenantError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateTenantError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateTenantResponse::from_proto(response).map_err(UpdateTenantError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateTenantRequest> for UpdateTenantRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateTenantRequest {
        pistachio_api::pistachio::admin::v1::UpdateTenantRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            display_name: self.display_name.map(|d| d.to_string()),
            allow_pdpka_signup: self.allow_pdpka_signup,
            disable_auth: self.disable_auth,
            mfa_config: self.mfa_config.map(|providers| {
                pistachio_api::pistachio::admin::v1::MfaConfigUpdate { providers }
            }),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateTenantResponse> for UpdateTenantResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateTenantResponse,
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
