use pistachio_api_common::admin::user::{
    UpdateTenantUserError, UpdateTenantUserRequest, UpdateTenantUserResponse, User,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    UpdateTenantUserError as GenError, update_tenant_user,
};
use crate::generated_admin::models::UpdateProjectUserRequest as GenRequest;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::FromJson;

impl From<GenError> for UpdateTenantUserError {
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
            GenError::Status409(_) => Self::AlreadyExists,
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

pub(crate) async fn handle_update_tenant_user(
    config: &Configuration,
    req: UpdateTenantUserRequest,
) -> Result<UpdateTenantUserResponse, UpdateTenantUserError> {
    debug!("Creating OpenAPI request for update_tenant_user");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    let gen_request = GenRequest {
        email: req.email.map(|e| Some(e.to_string())),
        email_verified: req.email_verified,
        phone_number: req.phone_number.map(|p| Some(p.to_string())),
        display_name: req.display_name.map(|d| Some(d.to_string())),
        photo_url: req.photo_url.map(|p| Some(p.to_string())),
        disabled: req.disabled,
        custom_claims: req.custom_claims.map(|c| {
            c.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect()
        }),
    };

    debug!(
        ?project_id,
        ?tenant_id,
        ?pistachio_id,
        "Sending update_tenant_user request"
    );

    let response = update_tenant_user(config, &project_id, &tenant_id, &pistachio_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_tenant_user response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => UpdateTenantUserError::BadRequest(problem),
                            401 => UpdateTenantUserError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => UpdateTenantUserError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => UpdateTenantUserError::NotFound(problem),
                            409 => UpdateTenantUserError::AlreadyExists,
                            500..=599 => UpdateTenantUserError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => UpdateTenantUserError::Unknown(format!(
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
                            UpdateTenantUserError::BadRequest(fallback_error_details(resp.content))
                        }
                        401 => UpdateTenantUserError::Unauthenticated(resp.content),
                        403 => UpdateTenantUserError::PermissionDenied(resp.content),
                        404 => {
                            UpdateTenantUserError::NotFound(fallback_error_details(resp.content))
                        }
                        409 => UpdateTenantUserError::AlreadyExists,
                        500..=599 => UpdateTenantUserError::ServiceError(resp.content),
                        _ => UpdateTenantUserError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    UpdateTenantUserError::ServiceUnavailable(e.to_string())
                }
                _ => UpdateTenantUserError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let user = response
        .user
        .map(|u| User::from_json(*u))
        .transpose()
        .map_err(UpdateTenantUserError::ResponseValidationError)?
        .ok_or(UpdateTenantUserError::ResponseValidationError(
            pistachio_api_common::error::ValidationError::MissingField("user"),
        ))?;

    Ok(UpdateTenantUserResponse { user })
}
