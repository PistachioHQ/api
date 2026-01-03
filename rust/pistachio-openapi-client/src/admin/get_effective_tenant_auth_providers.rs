use pistachio_api_common::admin::auth_provider::{
    GetEffectiveTenantAuthProvidersError, GetEffectiveTenantAuthProvidersRequest,
    GetEffectiveTenantAuthProvidersResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    GetEffectiveTenantAuthProvidersError as GenError, get_effective_tenant_auth_providers,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{convert_error_details, effective_provider_from_json};

impl From<GenError> for GetEffectiveTenantAuthProvidersError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

pub(crate) async fn handle_get_effective_tenant_auth_providers(
    config: &Configuration,
    req: GetEffectiveTenantAuthProvidersRequest,
) -> Result<GetEffectiveTenantAuthProvidersResponse, GetEffectiveTenantAuthProvidersError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let enabled_only = if req.enabled_only { Some(true) } else { None };

    debug!(
        ?project_id,
        ?tenant_id,
        ?enabled_only,
        "Sending get_effective_tenant_auth_providers request"
    );

    let response =
        get_effective_tenant_auth_providers(config, &project_id, &tenant_id, enabled_only)
            .await
            .map_err(|e| {
                error!(?e, "Error in get_effective_tenant_auth_providers response");
                match e {
                    crate::generated_admin::apis::Error::ResponseError(resp) => {
                        let status = resp.status.as_u16();

                        if let Some(problem) = parse_error_details(&resp.content) {
                            return match status {
                                400 => GetEffectiveTenantAuthProvidersError::BadRequest(problem),
                                401 => GetEffectiveTenantAuthProvidersError::Unauthenticated(
                                    problem.message.unwrap_or(problem.title),
                                ),
                                403 => GetEffectiveTenantAuthProvidersError::PermissionDenied(
                                    problem.message.unwrap_or(problem.title),
                                ),
                                404 => GetEffectiveTenantAuthProvidersError::NotFound(problem),
                                500..=599 => GetEffectiveTenantAuthProvidersError::ServiceError(
                                    problem.message.unwrap_or(problem.title),
                                ),
                                _ => GetEffectiveTenantAuthProvidersError::Unknown(format!(
                                    "HTTP {}: {}",
                                    status,
                                    problem.message.unwrap_or(problem.title)
                                )),
                            };
                        }

                        if let Some(entity) = resp.entity
                            && !matches!(entity, GenError::UnknownValue(_))
                        {
                            return entity.into();
                        }

                        match status {
                            400 => GetEffectiveTenantAuthProvidersError::BadRequest(
                                fallback_error_details(resp.content),
                            ),
                            401 => {
                                GetEffectiveTenantAuthProvidersError::Unauthenticated(resp.content)
                            }
                            403 => {
                                GetEffectiveTenantAuthProvidersError::PermissionDenied(resp.content)
                            }
                            404 => GetEffectiveTenantAuthProvidersError::NotFound(
                                fallback_error_details(resp.content),
                            ),
                            500..=599 => {
                                GetEffectiveTenantAuthProvidersError::ServiceError(resp.content)
                            }
                            _ => GetEffectiveTenantAuthProvidersError::Unknown(format!(
                                "HTTP {}: {}",
                                status, resp.content
                            )),
                        }
                    }
                    crate::generated_admin::apis::Error::Reqwest(e) => {
                        GetEffectiveTenantAuthProvidersError::ServiceUnavailable(e.to_string())
                    }
                    _ => GetEffectiveTenantAuthProvidersError::ServiceError(
                        "Unknown error occurred".into(),
                    ),
                }
            })?;

    let providers = response
        .providers
        .unwrap_or_default()
        .into_iter()
        .map(effective_provider_from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(GetEffectiveTenantAuthProvidersError::ResponseValidationError)?;

    Ok(GetEffectiveTenantAuthProvidersResponse { providers })
}
