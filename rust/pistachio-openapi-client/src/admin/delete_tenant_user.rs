use pistachio_api_common::admin::user::{
    DeleteTenantUserError, DeleteTenantUserRequest, DeleteTenantUserResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    DeleteTenantUserError as GenError, delete_tenant_user,
};
use crate::problem_details::{fallback_error_details, parse_error_details};

impl From<GenError> for DeleteTenantUserError {
    fn from(error: GenError) -> Self {
        use crate::types::convert_error_details;
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

pub(crate) async fn handle_delete_tenant_user(
    config: &Configuration,
    req: DeleteTenantUserRequest,
) -> Result<DeleteTenantUserResponse, DeleteTenantUserError> {
    debug!("Creating OpenAPI request for delete_tenant_user");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    debug!(
        ?project_id,
        ?tenant_id,
        ?pistachio_id,
        "Sending delete_tenant_user request"
    );

    delete_tenant_user(config, &project_id, &tenant_id, &pistachio_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_tenant_user response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => DeleteTenantUserError::BadRequest(problem),
                            401 => DeleteTenantUserError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => DeleteTenantUserError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => DeleteTenantUserError::NotFound(problem),
                            500..=599 => DeleteTenantUserError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => DeleteTenantUserError::Unknown(format!(
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
                        400 => {
                            DeleteTenantUserError::BadRequest(fallback_error_details(resp.content))
                        }
                        401 => DeleteTenantUserError::Unauthenticated(resp.content),
                        403 => DeleteTenantUserError::PermissionDenied(resp.content),
                        404 => {
                            DeleteTenantUserError::NotFound(fallback_error_details(resp.content))
                        }
                        500..=599 => DeleteTenantUserError::ServiceError(resp.content),
                        _ => DeleteTenantUserError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    DeleteTenantUserError::ServiceUnavailable(e.to_string())
                }
                _ => DeleteTenantUserError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    Ok(DeleteTenantUserResponse {})
}
