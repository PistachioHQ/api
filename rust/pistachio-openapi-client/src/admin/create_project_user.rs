use pistachio_api_common::admin::user::{
    CreateProjectUserError, CreateProjectUserRequest, CreateProjectUserResponse, User,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    CreateProjectUserError as GenError, create_project_user,
};
use crate::generated_admin::models::CreateProjectUserRequest as GenRequest;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::FromJson;

impl From<GenError> for CreateProjectUserError {
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

pub(crate) async fn handle_create_project_user(
    config: &Configuration,
    req: CreateProjectUserRequest,
) -> Result<CreateProjectUserResponse, CreateProjectUserError> {
    debug!("Creating OpenAPI request for create_project_user");

    let project_id = req.project_id.to_string();

    let gen_request = GenRequest {
        email: req.email.map(|e| Some(e.to_string())),
        email_verified: Some(req.email_verified),
        phone_number: req.phone_number.map(|p| Some(p.to_string())),
        display_name: req.display_name.map(|d| Some(d.to_string())),
        photo_url: req.photo_url.map(|p| Some(p.to_string())),
        disabled: Some(req.disabled),
        custom_claims: req
            .custom_claims
            .map(|c| c.into_iter().map(|(k, v)| (k, v.to_json())).collect()),
    };

    debug!(?project_id, "Sending create_project_user request");

    let response = create_project_user(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in create_project_user response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => CreateProjectUserError::BadRequest(problem),
                            401 => CreateProjectUserError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => CreateProjectUserError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => CreateProjectUserError::NotFound(problem),
                            409 => CreateProjectUserError::AlreadyExists,
                            500..=599 => CreateProjectUserError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => CreateProjectUserError::Unknown(format!(
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
                            CreateProjectUserError::BadRequest(fallback_error_details(resp.content))
                        }
                        401 => CreateProjectUserError::Unauthenticated(resp.content),
                        403 => CreateProjectUserError::PermissionDenied(resp.content),
                        404 => {
                            CreateProjectUserError::NotFound(fallback_error_details(resp.content))
                        }
                        409 => CreateProjectUserError::AlreadyExists,
                        500..=599 => CreateProjectUserError::ServiceError(resp.content),
                        _ => CreateProjectUserError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    CreateProjectUserError::ServiceUnavailable(e.to_string())
                }
                _ => CreateProjectUserError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let user = response
        .user
        .map(|u| User::from_json(*u))
        .transpose()
        .map_err(CreateProjectUserError::ResponseValidationError)?
        .ok_or(CreateProjectUserError::ResponseValidationError(
            pistachio_api_common::error::ValidationError::MissingField("user"),
        ))?;

    Ok(CreateProjectUserResponse { user })
}
