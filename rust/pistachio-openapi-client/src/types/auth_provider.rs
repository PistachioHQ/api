//! Type conversions for auth provider API responses.

use pistachio_api_common::admin::auth_provider::{
    AnonymousConfig, AuthProvider, AuthProviderConfig, ConfigSource, EffectiveAuthProvider,
    OAuthConfig, OidcConfig, PdpkaConfig, ProviderId, TenantAuthProviderOverride,
};
use pistachio_api_common::error::ValidationError;

use crate::generated_admin::models::{
    GetEffectiveTenantAuthProviders200ResponseProvidersInner,
    ListProjectAuthProviders200ResponseProvidersInner,
    ListProjectAuthProviders200ResponseProvidersInnerAnonymous,
    ListProjectAuthProviders200ResponseProvidersInnerOauth,
    ListProjectAuthProviders200ResponseProvidersInnerOidc,
    ListProjectAuthProviders200ResponseProvidersInnerPdpka,
    ListTenantAuthProviders200ResponseOverridesInner, UpdateProjectAuthProviderRequest,
    UpdateTenantAuthProviderRequest,
};

use super::parse_timestamp;

/// Convert a generated provider inner to domain AuthProvider.
pub(crate) fn provider_from_json(
    json: ListProjectAuthProviders200ResponseProvidersInner,
) -> Result<AuthProvider, ValidationError> {
    // Parse config before moving any fields
    let config = parse_provider_config(&json);

    let provider_id_str = json
        .provider_id
        .ok_or(ValidationError::MissingField("providerId"))?;

    let provider_id = ProviderId::parse(&provider_id_str)?;

    Ok(AuthProvider {
        provider_id,
        enabled: json.enabled.unwrap_or(false),
        display_order: json.display_order.unwrap_or(0),
        config,
        created_at: json
            .created_at
            .map(|s| parse_timestamp(Some(s)))
            .transpose()?,
        updated_at: json
            .updated_at
            .map(|s| parse_timestamp(Some(s)))
            .transpose()?,
    })
}

/// Parse provider-specific config from the JSON response.
fn parse_provider_config(
    json: &ListProjectAuthProviders200ResponseProvidersInner,
) -> Option<AuthProviderConfig> {
    if let Some(pdpka) = &json.pdpka {
        return Some(AuthProviderConfig::Pdpka(PdpkaConfig {
            allow_signup: pdpka.allow_signup.unwrap_or(false),
        }));
    }

    if let Some(anonymous) = &json.anonymous {
        return Some(AuthProviderConfig::Anonymous(AnonymousConfig {
            session_duration_seconds: anonymous.session_duration_seconds.unwrap_or(3600),
            auto_upgrade: anonymous.auto_upgrade.unwrap_or(false),
        }));
    }

    if let Some(oauth) = &json.oauth {
        return Some(AuthProviderConfig::OAuth(OAuthConfig {
            client_id: oauth.client_id.clone().unwrap_or_default(),
            scopes: oauth.scopes.clone().unwrap_or_default(),
            allowed_hosted_domains: oauth.allowed_hosted_domains.clone().unwrap_or_default(),
        }));
    }

    if let Some(oidc) = &json.oidc {
        return Some(AuthProviderConfig::Oidc(OidcConfig {
            display_name: oidc.display_name.clone().unwrap_or_default(),
            issuer_url: oidc.issuer_url.clone().unwrap_or_default(),
            client_id: oidc.client_id.clone().unwrap_or_default(),
            scopes: oidc.scopes.clone().unwrap_or_default(),
            additional_params: oidc.additional_params.clone().unwrap_or_default(),
        }));
    }

    None
}

/// Convert a generated tenant override to domain TenantAuthProviderOverride.
pub(crate) fn override_from_json(
    json: ListTenantAuthProviders200ResponseOverridesInner,
) -> Result<TenantAuthProviderOverride, ValidationError> {
    // Parse config before moving any fields
    let config = parse_override_config(&json);

    let provider_id_str = json
        .provider_id
        .ok_or(ValidationError::MissingField("providerId"))?;

    let provider_id = ProviderId::parse(&provider_id_str)?;

    Ok(TenantAuthProviderOverride {
        provider_id,
        enabled: json.enabled.flatten(),
        display_order: json.display_order.flatten(),
        config,
        created_at: json
            .created_at
            .map(|s| parse_timestamp(Some(s)))
            .transpose()?,
        updated_at: json
            .updated_at
            .map(|s| parse_timestamp(Some(s)))
            .transpose()?,
    })
}

/// Parse provider-specific config from override JSON.
fn parse_override_config(
    json: &ListTenantAuthProviders200ResponseOverridesInner,
) -> Option<AuthProviderConfig> {
    if let Some(pdpka) = &json.pdpka {
        return Some(AuthProviderConfig::Pdpka(PdpkaConfig {
            allow_signup: pdpka.allow_signup.unwrap_or(false),
        }));
    }

    if let Some(anonymous) = &json.anonymous {
        return Some(AuthProviderConfig::Anonymous(AnonymousConfig {
            session_duration_seconds: anonymous.session_duration_seconds.unwrap_or(3600),
            auto_upgrade: anonymous.auto_upgrade.unwrap_or(false),
        }));
    }

    if let Some(oauth) = &json.oauth {
        return Some(AuthProviderConfig::OAuth(OAuthConfig {
            client_id: oauth.client_id.clone().unwrap_or_default(),
            scopes: oauth.scopes.clone().unwrap_or_default(),
            allowed_hosted_domains: oauth.allowed_hosted_domains.clone().unwrap_or_default(),
        }));
    }

    if let Some(oidc) = &json.oidc {
        return Some(AuthProviderConfig::Oidc(OidcConfig {
            display_name: oidc.display_name.clone().unwrap_or_default(),
            issuer_url: oidc.issuer_url.clone().unwrap_or_default(),
            client_id: oidc.client_id.clone().unwrap_or_default(),
            scopes: oidc.scopes.clone().unwrap_or_default(),
            additional_params: oidc.additional_params.clone().unwrap_or_default(),
        }));
    }

    None
}

/// Convert a generated effective provider to domain EffectiveAuthProvider.
pub(crate) fn effective_provider_from_json(
    json: GetEffectiveTenantAuthProviders200ResponseProvidersInner,
) -> Result<EffectiveAuthProvider, ValidationError> {
    use crate::generated_admin::models::get_effective_tenant_auth_providers_200_response_providers_inner::Source;

    let provider_json = json
        .provider
        .ok_or(ValidationError::MissingField("provider"))?;

    let provider = provider_from_json(*provider_json)?;

    let source = match json.source {
        Some(Source::Project) => ConfigSource::Project,
        Some(Source::Tenant) => ConfigSource::Tenant,
        None => ConfigSource::Project,
    };

    Ok(EffectiveAuthProvider { provider, source })
}

/// Type alias for the config fields tuple to reduce complexity.
type ConfigFields = (
    Option<Box<ListProjectAuthProviders200ResponseProvidersInnerPdpka>>,
    Option<Box<ListProjectAuthProviders200ResponseProvidersInnerAnonymous>>,
    Option<Box<ListProjectAuthProviders200ResponseProvidersInnerOauth>>,
    Option<Box<ListProjectAuthProviders200ResponseProvidersInnerOidc>>,
);

/// Convert domain AuthProviderConfig to request config fields.
pub(crate) fn config_to_request(config: &AuthProviderConfig) -> ConfigFields {
    match config {
        AuthProviderConfig::Pdpka(pdpka) => (
            Some(Box::new(
                ListProjectAuthProviders200ResponseProvidersInnerPdpka {
                    allow_signup: Some(pdpka.allow_signup),
                },
            )),
            None,
            None,
            None,
        ),
        AuthProviderConfig::Anonymous(anonymous) => (
            None,
            Some(Box::new(
                ListProjectAuthProviders200ResponseProvidersInnerAnonymous {
                    session_duration_seconds: Some(anonymous.session_duration_seconds),
                    auto_upgrade: Some(anonymous.auto_upgrade),
                },
            )),
            None,
            None,
        ),
        AuthProviderConfig::OAuth(oauth) => (
            None,
            None,
            Some(Box::new(
                ListProjectAuthProviders200ResponseProvidersInnerOauth {
                    client_id: Some(oauth.client_id.clone()),
                    scopes: Some(oauth.scopes.clone()),
                    allowed_hosted_domains: Some(oauth.allowed_hosted_domains.clone()),
                },
            )),
            None,
        ),
        AuthProviderConfig::Oidc(oidc) => (
            None,
            None,
            None,
            Some(Box::new(
                ListProjectAuthProviders200ResponseProvidersInnerOidc {
                    display_name: Some(oidc.display_name.clone()),
                    issuer_url: Some(oidc.issuer_url.clone()),
                    client_id: Some(oidc.client_id.clone()),
                    scopes: Some(oidc.scopes.clone()),
                    additional_params: Some(oidc.additional_params.clone()),
                },
            )),
        ),
    }
}

/// Build an UpdateProjectAuthProviderRequest from domain types.
pub(crate) fn build_update_project_request(
    enabled: Option<bool>,
    display_order: Option<i32>,
    config: Option<&AuthProviderConfig>,
    client_secret: Option<String>,
) -> UpdateProjectAuthProviderRequest {
    let (pdpka, anonymous, oauth, oidc) = config
        .map(config_to_request)
        .unwrap_or((None, None, None, None));

    UpdateProjectAuthProviderRequest {
        enabled,
        display_order,
        pdpka,
        anonymous,
        oauth,
        oidc,
        client_secret,
    }
}

/// Build an UpdateTenantAuthProviderRequest from domain types.
pub(crate) fn build_update_tenant_request(
    enabled: Option<bool>,
    display_order: Option<i32>,
    config: Option<&AuthProviderConfig>,
    client_secret: Option<String>,
) -> UpdateTenantAuthProviderRequest {
    let (pdpka, anonymous, oauth, oidc) = config
        .map(config_to_request)
        .unwrap_or((None, None, None, None));

    UpdateTenantAuthProviderRequest {
        enabled: enabled.map(Some),
        display_order: display_order.map(Some),
        pdpka,
        anonymous,
        oauth,
        oidc,
        client_secret,
    }
}
