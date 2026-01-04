use pistachio_api_common::admin::auth_provider::{
    ListTenantAuthProvidersError, ListTenantAuthProvidersRequest, ListTenantAuthProvidersResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    ListTenantAuthProvidersError as GenError, list_tenant_auth_providers,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{convert_error_details, override_from_json};

impl From<GenError> for ListTenantAuthProvidersError {
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

pub(crate) async fn handle_list_tenant_auth_providers(
    config: &Configuration,
    req: ListTenantAuthProvidersRequest,
) -> Result<ListTenantAuthProvidersResponse, ListTenantAuthProvidersError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();

    debug!(
        ?project_id,
        ?tenant_id,
        "Sending list_tenant_auth_providers request"
    );

    let response = list_tenant_auth_providers(config, &project_id, &tenant_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in list_tenant_auth_providers response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => ListTenantAuthProvidersError::BadRequest(problem),
                            401 => ListTenantAuthProvidersError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => ListTenantAuthProvidersError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => ListTenantAuthProvidersError::NotFound(problem),
                            500..=599 => ListTenantAuthProvidersError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => ListTenantAuthProvidersError::Unknown(format!(
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
                        400 => ListTenantAuthProvidersError::BadRequest(fallback_error_details(
                            resp.content,
                        )),
                        401 => ListTenantAuthProvidersError::Unauthenticated(resp.content),
                        403 => ListTenantAuthProvidersError::PermissionDenied(resp.content),
                        404 => ListTenantAuthProvidersError::NotFound(fallback_error_details(
                            resp.content,
                        )),
                        500..=599 => ListTenantAuthProvidersError::ServiceError(resp.content),
                        _ => ListTenantAuthProvidersError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    ListTenantAuthProvidersError::ServiceUnavailable(e.to_string())
                }
                _ => ListTenantAuthProvidersError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let overrides = response
        .overrides
        .unwrap_or_default()
        .into_iter()
        .map(override_from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ListTenantAuthProvidersError::ResponseValidationError)?;

    // TODO: Extract pagination from response once OpenAPI spec is regenerated
    Ok(ListTenantAuthProvidersResponse {
        overrides,
        pagination: None,
    })
}
