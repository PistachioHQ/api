use pistachio_api_common::admin::user::{
    GetProjectUserError, GetProjectUserRequest, GetProjectUserResponse, User,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{GetProjectUserError as GenError, get_project_user};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::FromJson;

impl From<GenError> for GetProjectUserError {
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

pub(crate) async fn handle_get_project_user(
    config: &Configuration,
    req: GetProjectUserRequest,
) -> Result<GetProjectUserResponse, GetProjectUserError> {
    debug!("Creating OpenAPI request for get_project_user");

    let project_id = req.project_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    debug!(
        ?project_id,
        ?pistachio_id,
        "Sending get_project_user request"
    );

    let response = get_project_user(config, &project_id, &pistachio_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_project_user response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => GetProjectUserError::BadRequest(problem),
                            401 => GetProjectUserError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => GetProjectUserError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => GetProjectUserError::NotFound(problem),
                            500..=599 => GetProjectUserError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => GetProjectUserError::Unknown(format!(
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
                            GetProjectUserError::BadRequest(fallback_error_details(resp.content))
                        }
                        401 => GetProjectUserError::Unauthenticated(resp.content),
                        403 => GetProjectUserError::PermissionDenied(resp.content),
                        404 => GetProjectUserError::NotFound(fallback_error_details(resp.content)),
                        500..=599 => GetProjectUserError::ServiceError(resp.content),
                        _ => GetProjectUserError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    GetProjectUserError::ServiceUnavailable(e.to_string())
                }
                _ => GetProjectUserError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let user = response
        .user
        .map(|u| User::from_json(*u))
        .transpose()
        .map_err(GetProjectUserError::ResponseValidationError)?
        .ok_or(GetProjectUserError::ResponseValidationError(
            pistachio_api_common::error::ValidationError::MissingField("user"),
        ))?;

    Ok(GetProjectUserResponse { user })
}
