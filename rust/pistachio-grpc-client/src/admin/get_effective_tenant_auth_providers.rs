use pistachio_api_common::admin::auth_provider::{
    AuthProvider, ConfigSource, EffectiveAuthProvider, GetEffectiveTenantAuthProvidersError,
    GetEffectiveTenantAuthProvidersRequest, GetEffectiveTenantAuthProvidersResponse,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_get_effective_tenant_auth_providers<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetEffectiveTenantAuthProvidersRequest,
) -> Result<GetEffectiveTenantAuthProvidersResponse, GetEffectiveTenantAuthProvidersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .get_effective_tenant_auth_providers(request)
        .await
        .map_err(|status| {
            error!(
                ?status,
                "Error in get_effective_tenant_auth_providers response"
            );
            match status.code() {
                Code::InvalidArgument => GetEffectiveTenantAuthProvidersError::BadRequest(
                    error_details_from_status(&status),
                ),
                Code::NotFound => GetEffectiveTenantAuthProvidersError::NotFound(
                    error_details_from_status(&status),
                ),
                Code::Unauthenticated => GetEffectiveTenantAuthProvidersError::Unauthenticated(
                    status.message().to_string(),
                ),
                Code::PermissionDenied => GetEffectiveTenantAuthProvidersError::PermissionDenied(
                    status.message().to_string(),
                ),
                Code::Internal => {
                    GetEffectiveTenantAuthProvidersError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => GetEffectiveTenantAuthProvidersError::ServiceUnavailable(
                    status.message().to_string(),
                ),
                _ => GetEffectiveTenantAuthProvidersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    GetEffectiveTenantAuthProvidersResponse::from_proto(response)
        .map_err(GetEffectiveTenantAuthProvidersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::GetEffectiveTenantAuthProvidersRequest>
    for GetEffectiveTenantAuthProvidersRequest
{
    fn into_proto(
        self,
    ) -> pistachio_api::pistachio::admin::v1::GetEffectiveTenantAuthProvidersRequest {
        pistachio_api::pistachio::admin::v1::GetEffectiveTenantAuthProvidersRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            enabled_only: self.enabled_only,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::GetEffectiveTenantAuthProvidersResponse>
    for GetEffectiveTenantAuthProvidersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::GetEffectiveTenantAuthProvidersResponse,
    ) -> Result<Self, Self::Error> {
        let providers = proto
            .providers
            .into_iter()
            .map(EffectiveAuthProvider::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { providers })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::EffectiveAuthProvider>
    for EffectiveAuthProvider
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::EffectiveAuthProvider,
    ) -> Result<Self, Self::Error> {
        use pistachio_api::pistachio::types::v1::ConfigSource as ProtoConfigSource;

        let provider_proto =
            proto
                .provider
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "provider",
                ))?;

        let provider = AuthProvider::from_proto(provider_proto)?;

        let source = match ProtoConfigSource::try_from(proto.source) {
            Ok(ProtoConfigSource::Project) => ConfigSource::Project,
            Ok(ProtoConfigSource::Tenant) => ConfigSource::Tenant,
            _ => ConfigSource::Project, // Default to project if unspecified
        };

        Ok(Self { provider, source })
    }
}
