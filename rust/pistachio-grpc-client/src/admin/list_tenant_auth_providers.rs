use pistachio_api_common::admin::auth_provider::{
    AuthProviderConfig, ListTenantAuthProvidersError, ListTenantAuthProvidersRequest,
    ListTenantAuthProvidersResponse, ProviderId, TenantAuthProviderOverride,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_list_tenant_auth_providers<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListTenantAuthProvidersRequest,
) -> Result<ListTenantAuthProvidersResponse, ListTenantAuthProvidersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_tenant_auth_providers(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_tenant_auth_providers response");
            match status.code() {
                Code::InvalidArgument => {
                    ListTenantAuthProvidersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListTenantAuthProvidersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListTenantAuthProvidersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListTenantAuthProvidersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ListTenantAuthProvidersError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ListTenantAuthProvidersError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListTenantAuthProvidersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListTenantAuthProvidersResponse::from_proto(response)
        .map_err(ListTenantAuthProvidersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListTenantAuthProvidersRequest>
    for ListTenantAuthProvidersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListTenantAuthProvidersRequest {
        pistachio_api::pistachio::admin::v1::ListTenantAuthProvidersRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            pagination: self
                .pagination
                .map(crate::types::pagination_params_to_proto),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListTenantAuthProvidersResponse>
    for ListTenantAuthProvidersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListTenantAuthProvidersResponse,
    ) -> Result<Self, Self::Error> {
        let overrides = proto
            .overrides
            .into_iter()
            .map(TenantAuthProviderOverride::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = proto
            .pagination
            .map(crate::types::pagination_meta_from_proto);

        Ok(Self {
            overrides,
            pagination,
        })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::TenantAuthProviderOverride>
    for TenantAuthProviderOverride
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::TenantAuthProviderOverride,
    ) -> Result<Self, Self::Error> {
        let config = proto
            .config
            .map(AuthProviderConfig::from_proto)
            .transpose()?;

        let created_at = proto
            .created_at
            .map(|ts| crate::types::timestamp_to_datetime(Some(ts)))
            .transpose()?;
        let updated_at = proto
            .updated_at
            .map(|ts| crate::types::timestamp_to_datetime(Some(ts)))
            .transpose()?;

        let provider_id = ProviderId::parse(&proto.provider_id)?;

        Ok(Self {
            provider_id,
            enabled: proto.enabled,
            display_order: proto.display_order,
            config,
            created_at,
            updated_at,
        })
    }
}
