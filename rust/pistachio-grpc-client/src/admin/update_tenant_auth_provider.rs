use pistachio_api_common::admin::auth_provider::{
    TenantAuthProviderOverride, UpdateTenantAuthProviderError, UpdateTenantAuthProviderRequest,
    UpdateTenantAuthProviderResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use super::list_project_auth_providers::auth_provider_config_to_proto;
use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_update_tenant_auth_provider<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateTenantAuthProviderRequest,
) -> Result<UpdateTenantAuthProviderResponse, UpdateTenantAuthProviderError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .update_tenant_auth_provider(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_tenant_auth_provider response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateTenantAuthProviderError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    UpdateTenantAuthProviderError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    UpdateTenantAuthProviderError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateTenantAuthProviderError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    UpdateTenantAuthProviderError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    UpdateTenantAuthProviderError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateTenantAuthProviderError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    UpdateTenantAuthProviderResponse::from_proto(response)
        .map_err(UpdateTenantAuthProviderError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::UpdateTenantAuthProviderRequest>
    for UpdateTenantAuthProviderRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::UpdateTenantAuthProviderRequest {
        pistachio_api::pistachio::admin::v1::UpdateTenantAuthProviderRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            provider_id: self.provider_id,
            enabled: self.enabled,
            display_order: self.display_order,
            config: self.config.as_ref().map(auth_provider_config_to_proto),
            client_secret: self.client_secret,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::UpdateTenantAuthProviderResponse>
    for UpdateTenantAuthProviderResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::UpdateTenantAuthProviderResponse,
    ) -> Result<Self, Self::Error> {
        let override_proto =
            proto
                .r#override
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "override",
                ))?;

        let override_ = TenantAuthProviderOverride::from_proto(override_proto)?;

        Ok(Self { override_ })
    }
}
