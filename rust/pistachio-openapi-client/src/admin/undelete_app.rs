use libgn::app::App;
use pistachio_api_common::admin::app::{UndeleteAppError, UndeleteAppRequest, UndeleteAppResponse};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{UndeleteAppError as GenError, undelete_app};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::UndeleteApp200Response;
use crate::problem_details::{fallback_problem_details, parse_problem_details};
use crate::types::{FromJson, convert_problem_details};

impl From<GenError> for UndeleteAppError {
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
            GenError::Status409(_) => Self::FailedPrecondition,
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

pub(crate) async fn handle_undelete_app(
    config: &Configuration,
    req: UndeleteAppRequest,
) -> Result<UndeleteAppResponse, UndeleteAppError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    debug!(?project_id, ?app_id, "Sending undelete_app request");

    let response = undelete_app(config, &project_id, &app_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in undelete_app response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_problem_details(&resp.content, status) {
                        return match status {
                            400 => UndeleteAppError::BadRequest(problem),
                            401 => UndeleteAppError::Unauthenticated(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            403 => UndeleteAppError::PermissionDenied(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            404 => UndeleteAppError::NotFound(problem),
                            409 => UndeleteAppError::FailedPrecondition,
                            500..=599 => UndeleteAppError::ServiceError(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            _ => UndeleteAppError::Unknown(format!(
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
                        400 => UndeleteAppError::BadRequest(fallback_problem_details(
                            400,
                            resp.content,
                        )),
                        401 => UndeleteAppError::Unauthenticated(resp.content),
                        403 => UndeleteAppError::PermissionDenied(resp.content),
                        404 => {
                            UndeleteAppError::NotFound(fallback_problem_details(404, resp.content))
                        }
                        409 => UndeleteAppError::FailedPrecondition,
                        500..=599 => UndeleteAppError::ServiceError(resp.content),
                        _ => {
                            UndeleteAppError::Unknown(format!("HTTP {}: {}", status, resp.content))
                        }
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    UndeleteAppError::ServiceUnavailable(e.to_string())
                }
                _ => UndeleteAppError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    UndeleteAppResponse::from_json(response).map_err(UndeleteAppError::ResponseValidationError)
}

impl FromJson<UndeleteApp200Response> for UndeleteAppResponse {
    type Error = ValidationError;

    fn from_json(json: UndeleteApp200Response) -> Result<Self, Self::Error> {
        let app = json
            .app
            .map(|a| App::from_json(*a))
            .transpose()?
            .ok_or(ValidationError::MissingField("app"))?;

        Ok(Self { app })
    }
}
