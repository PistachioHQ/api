use pistachio_api_common::admin::auth_provider::{
    AuthProvider, AuthProviderConfig, ListProjectAuthProvidersError,
    ListProjectAuthProvidersRequest, ListProjectAuthProvidersResponse, ProviderId,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_list_project_auth_providers<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListProjectAuthProvidersRequest,
) -> Result<ListProjectAuthProvidersResponse, ListProjectAuthProvidersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_project_auth_providers(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_project_auth_providers response");
            match status.code() {
                Code::InvalidArgument => {
                    ListProjectAuthProvidersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListProjectAuthProvidersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListProjectAuthProvidersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListProjectAuthProvidersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ListProjectAuthProvidersError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ListProjectAuthProvidersError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListProjectAuthProvidersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListProjectAuthProvidersResponse::from_proto(response)
        .map_err(ListProjectAuthProvidersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListProjectAuthProvidersRequest>
    for ListProjectAuthProvidersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListProjectAuthProvidersRequest {
        pistachio_api::pistachio::admin::v1::ListProjectAuthProvidersRequest {
            project_id: self.project_id.to_string(),
            pagination: self
                .pagination
                .map(crate::types::pagination_params_to_proto),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListProjectAuthProvidersResponse>
    for ListProjectAuthProvidersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListProjectAuthProvidersResponse,
    ) -> Result<Self, Self::Error> {
        let providers = proto
            .providers
            .into_iter()
            .map(AuthProvider::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = proto
            .pagination
            .map(crate::types::pagination_meta_from_proto);

        Ok(Self {
            providers,
            pagination,
        })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::AuthProvider> for AuthProvider {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::AuthProvider,
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

impl FromProto<pistachio_api::pistachio::types::v1::AuthProviderConfig> for AuthProviderConfig {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::AuthProviderConfig,
    ) -> Result<Self, Self::Error> {
        use pistachio_api::pistachio::types::v1::auth_provider_config::Config;
        use pistachio_api_common::admin::auth_provider::{
            AnonymousConfig, OAuthConfig, OidcConfig, PdpkaConfig,
        };

        match proto.config {
            Some(Config::Pdpka(p)) => Ok(AuthProviderConfig::Pdpka(PdpkaConfig {
                allow_signup: p.allow_signup,
            })),
            Some(Config::Anonymous(a)) => Ok(AuthProviderConfig::Anonymous(AnonymousConfig {
                session_duration_seconds: a.session_duration_seconds,
                auto_upgrade: a.auto_upgrade,
            })),
            Some(Config::Oauth(o)) => Ok(AuthProviderConfig::OAuth(OAuthConfig {
                client_id: o.client_id,
                scopes: o.scopes,
                allowed_hosted_domains: o.allowed_hosted_domains,
            })),
            Some(Config::Oidc(o)) => Ok(AuthProviderConfig::Oidc(OidcConfig {
                display_name: o.display_name,
                issuer_url: o.issuer_url,
                client_id: o.client_id,
                scopes: o.scopes,
                additional_params: o.additional_params,
            })),
            None => Err(pistachio_api_common::error::ValidationError::MissingField(
                "config",
            )),
        }
    }
}

pub(crate) fn auth_provider_config_to_proto(
    config: &pistachio_api_common::admin::auth_provider::AuthProviderConfig,
) -> pistachio_api::pistachio::types::v1::AuthProviderConfig {
    use pistachio_api::pistachio::types::v1::auth_provider_config::Config;
    use pistachio_api_common::admin::auth_provider::AuthProviderConfig;

    let config = match config {
        AuthProviderConfig::Pdpka(p) => {
            Config::Pdpka(pistachio_api::pistachio::types::v1::PdpkaConfig {
                allow_signup: p.allow_signup,
            })
        }
        AuthProviderConfig::Anonymous(a) => {
            Config::Anonymous(pistachio_api::pistachio::types::v1::AnonymousConfig {
                session_duration_seconds: a.session_duration_seconds,
                auto_upgrade: a.auto_upgrade,
            })
        }
        AuthProviderConfig::OAuth(o) => {
            Config::Oauth(pistachio_api::pistachio::types::v1::OAuthConfig {
                client_id: o.client_id.clone(),
                scopes: o.scopes.clone(),
                allowed_hosted_domains: o.allowed_hosted_domains.clone(),
            })
        }
        AuthProviderConfig::Oidc(o) => {
            Config::Oidc(pistachio_api::pistachio::types::v1::OidcConfig {
                display_name: o.display_name.clone(),
                issuer_url: o.issuer_url.clone(),
                client_id: o.client_id.clone(),
                scopes: o.scopes.clone(),
                additional_params: o.additional_params.clone(),
            })
        }
    };

    pistachio_api::pistachio::types::v1::AuthProviderConfig {
        config: Some(config),
    }
}
