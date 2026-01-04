use pistachio_api_common::admin::auth_provider::{
    DeleteTenantAuthProviderError, DeleteTenantAuthProviderRequest,
    DeleteTenantAuthProviderResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    DeleteTenantAuthProviderError as GenError, delete_tenant_auth_provider,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::convert_error_details;

impl From<GenError> for DeleteTenantAuthProviderError {
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

pub(crate) async fn handle_delete_tenant_auth_provider(
    config: &Configuration,
    req: DeleteTenantAuthProviderRequest,
) -> Result<DeleteTenantAuthProviderResponse, DeleteTenantAuthProviderError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let provider_id = req.provider_id.to_string();

    debug!(
        ?project_id,
        ?tenant_id,
        ?provider_id,
        "Sending delete_tenant_auth_provider request"
    );

    delete_tenant_auth_provider(config, &project_id, &tenant_id, &provider_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_tenant_auth_provider response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => DeleteTenantAuthProviderError::BadRequest(problem),
                            401 => DeleteTenantAuthProviderError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => DeleteTenantAuthProviderError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => DeleteTenantAuthProviderError::NotFound(problem),
                            500..=599 => DeleteTenantAuthProviderError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => DeleteTenantAuthProviderError::Unknown(format!(
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
                        400 => DeleteTenantAuthProviderError::BadRequest(fallback_error_details(
                            resp.content,
                        )),
                        401 => DeleteTenantAuthProviderError::Unauthenticated(resp.content),
                        403 => DeleteTenantAuthProviderError::PermissionDenied(resp.content),
                        404 => DeleteTenantAuthProviderError::NotFound(fallback_error_details(
                            resp.content,
                        )),
                        500..=599 => DeleteTenantAuthProviderError::ServiceError(resp.content),
                        _ => DeleteTenantAuthProviderError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    DeleteTenantAuthProviderError::ServiceUnavailable(e.to_string())
                }
                _ => DeleteTenantAuthProviderError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    Ok(DeleteTenantAuthProviderResponse {})
}
