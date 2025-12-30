use libgn::project::Project;
use pistachio_api_common::admin::project::{
    UndeleteProjectError, UndeleteProjectRequest, UndeleteProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{
    UndeleteProjectError as GenError, undelete_project,
};
use crate::generated_admin::models::UndeleteProject200Response;
use crate::problem_details::{fallback_problem_details, parse_problem_details};
use crate::types::{FromJson, convert_problem_details};

impl From<GenError> for UndeleteProjectError {
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
            GenError::Status409(e) => {
                Self::FailedPrecondition(e.detail.unwrap_or_else(|| e.title.clone()))
            }
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

pub(crate) async fn handle_undelete_project(
    config: &Configuration,
    req: UndeleteProjectRequest,
) -> Result<UndeleteProjectResponse, UndeleteProjectError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending undelete_project request");

    let response = undelete_project(config, &project_id).await.map_err(|e| {
        error!(?e, "Error in undelete_project response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();

                if let Some(problem) = parse_problem_details(&resp.content, status) {
                    return match status {
                        400 => UndeleteProjectError::BadRequest(problem),
                        401 => UndeleteProjectError::Unauthenticated(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        403 => UndeleteProjectError::PermissionDenied(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        404 => UndeleteProjectError::NotFound(problem),
                        409 => UndeleteProjectError::FailedPrecondition(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        500..=599 => UndeleteProjectError::ServiceError(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        _ => UndeleteProjectError::Unknown(format!(
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
                    400 => UndeleteProjectError::BadRequest(fallback_problem_details(
                        400,
                        resp.content,
                    )),
                    401 => UndeleteProjectError::Unauthenticated(resp.content),
                    403 => UndeleteProjectError::PermissionDenied(resp.content),
                    404 => {
                        UndeleteProjectError::NotFound(fallback_problem_details(404, resp.content))
                    }
                    409 => UndeleteProjectError::FailedPrecondition(resp.content),
                    500..=599 => UndeleteProjectError::ServiceError(resp.content),
                    _ => {
                        UndeleteProjectError::Unknown(format!("HTTP {}: {}", status, resp.content))
                    }
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                UndeleteProjectError::ServiceUnavailable(e.to_string())
            }
            _ => UndeleteProjectError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    UndeleteProjectResponse::from_json(response)
        .map_err(UndeleteProjectError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<UndeleteProject200Response> for UndeleteProjectResponse {
    type Error = ValidationError;

    fn from_json(json: UndeleteProject200Response) -> Result<Self, Self::Error> {
        let project = json
            .project
            .map(|p| Project::from_json(*p))
            .transpose()?
            .ok_or(ValidationError::MissingField("project"))?;

        Ok(Self { project })
    }
}
