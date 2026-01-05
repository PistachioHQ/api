use pistachio_api_common::admin::user::{
    GetTenantUserError, GetTenantUserRequest, GetTenantUserResponse, User,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{GetTenantUserError as GenError, get_tenant_user};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::FromJson;

impl From<GenError> for GetTenantUserError {
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

pub(crate) async fn handle_get_tenant_user(
    config: &Configuration,
    req: GetTenantUserRequest,
) -> Result<GetTenantUserResponse, GetTenantUserError> {
    debug!("Creating OpenAPI request for get_tenant_user");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    debug!(
        ?project_id,
        ?tenant_id,
        ?pistachio_id,
        "Sending get_tenant_user request"
    );

    let response = get_tenant_user(config, &project_id, &tenant_id, &pistachio_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_tenant_user response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => GetTenantUserError::BadRequest(problem),
                            401 => GetTenantUserError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => GetTenantUserError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => GetTenantUserError::NotFound(problem),
                            500..=599 => GetTenantUserError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => GetTenantUserError::Unknown(format!(
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
                        400 => GetTenantUserError::BadRequest(fallback_error_details(resp.content)),
                        401 => GetTenantUserError::Unauthenticated(resp.content),
                        403 => GetTenantUserError::PermissionDenied(resp.content),
                        404 => GetTenantUserError::NotFound(fallback_error_details(resp.content)),
                        500..=599 => GetTenantUserError::ServiceError(resp.content),
                        _ => GetTenantUserError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    GetTenantUserError::ServiceUnavailable(e.to_string())
                }
                _ => GetTenantUserError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let user = response
        .user
        .map(|u| User::from_json(*u))
        .transpose()
        .map_err(GetTenantUserError::ResponseValidationError)?
        .ok_or(GetTenantUserError::ResponseValidationError(
            pistachio_api_common::error::ValidationError::MissingField("user"),
        ))?;

    Ok(GetTenantUserResponse { user })
}
