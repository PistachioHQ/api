use pistachio_api_common::admin::tenant::{
    DeleteTenantError, DeleteTenantRequest, DeleteTenantResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{DeleteTenantError as GenError, delete_tenant};
use crate::problem_details::{fallback_problem_details, parse_problem_details};
use crate::types::convert_problem_details;

impl From<GenError> for DeleteTenantError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(convert_problem_details(e)),
            GenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status404(e) => Self::NotFound(convert_problem_details(e)),
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

pub(crate) async fn handle_delete_tenant(
    config: &Configuration,
    req: DeleteTenantRequest,
) -> Result<DeleteTenantResponse, DeleteTenantError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();

    debug!(?project_id, ?tenant_id, "Sending delete_tenant request");

    let _response = delete_tenant(config, &project_id, &tenant_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_tenant response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_problem_details(&resp.content, status) {
                        return match status {
                            400 => DeleteTenantError::BadRequest(problem),
                            401 => DeleteTenantError::Unauthenticated(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            403 => DeleteTenantError::PermissionDenied(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            404 => DeleteTenantError::NotFound(problem),
                            500..=599 => DeleteTenantError::ServiceError(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            _ => DeleteTenantError::Unknown(format!(
                                "HTTP {}: {}",
                                status,
                                problem.detail.unwrap_or(problem.title)
                            )),
                        };
                    }
                    if let Some(entity) = resp.entity
                        && !matches!(entity, GenError::UnknownValue(_))
                    {
                        return entity.into();
                    }
                    match status {
                        400 => DeleteTenantError::BadRequest(fallback_problem_details(
                            400,
                            resp.content,
                        )),
                        401 => DeleteTenantError::Unauthenticated(resp.content),
                        403 => DeleteTenantError::PermissionDenied(resp.content),
                        404 => {
                            DeleteTenantError::NotFound(fallback_problem_details(404, resp.content))
                        }
                        500..=599 => DeleteTenantError::ServiceError(resp.content),
                        _ => {
                            DeleteTenantError::Unknown(format!("HTTP {}: {}", status, resp.content))
                        }
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    DeleteTenantError::ServiceUnavailable(e.to_string())
                }
                _ => DeleteTenantError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    Ok(DeleteTenantResponse {})
}
