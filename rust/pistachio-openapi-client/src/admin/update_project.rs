use libgn::project::Project;
use pistachio_api_common::admin::project::{
    UpdateProjectError, UpdateProjectRequest, UpdateProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{UpdateProjectError as GenError, update_project};
use crate::generated_admin::models::{
    UpdateProject200Response, UpdateProjectRequest as GenUpdateProjectRequest,
};
use crate::problem_details::{fallback_problem_details, parse_problem_details};
use crate::types::{FromJson, convert_problem_details};

impl From<GenError> for UpdateProjectError {
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

pub(crate) async fn handle_update_project(
    config: &Configuration,
    req: UpdateProjectRequest,
) -> Result<UpdateProjectResponse, UpdateProjectError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let request = GenUpdateProjectRequest {
        display_name: req.display_name.map(|name| name.to_string()),
    };

    debug!(?project_id, ?request, "Sending update_project request");

    let response = update_project(config, &project_id, request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_project response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    // Try to parse RFC 7807 Problem Details from the response content
                    if let Some(problem) = parse_problem_details(&resp.content, status) {
                        return match status {
                            400 => UpdateProjectError::BadRequest(problem),
                            401 => UpdateProjectError::Unauthenticated(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            403 => UpdateProjectError::PermissionDenied(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            404 => UpdateProjectError::NotFound(problem),
                            500..=599 => UpdateProjectError::ServiceError(
                                problem.detail.unwrap_or(problem.title),
                            ),
                            _ => UpdateProjectError::Unknown(format!(
                                "HTTP {}: {}",
                                status,
                                problem.detail.unwrap_or(problem.title)
                            )),
                        };
                    }

                    // Fall back to entity parsing if RFC 7807 parsing failed
                    if let Some(entity) = resp.entity
                        && !matches!(entity, GenError::UnknownValue(_))
                    {
                        return entity.into();
                    }

                    // Last resort: status code mapping with raw content
                    match status {
                        400 => UpdateProjectError::BadRequest(fallback_problem_details(
                            400,
                            resp.content,
                        )),
                        401 => UpdateProjectError::Unauthenticated(resp.content),
                        403 => UpdateProjectError::PermissionDenied(resp.content),
                        404 => UpdateProjectError::NotFound(fallback_problem_details(
                            404,
                            resp.content,
                        )),
                        500..=599 => UpdateProjectError::ServiceError(resp.content),
                        _ => UpdateProjectError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    UpdateProjectError::ServiceUnavailable(e.to_string())
                }
                _ => UpdateProjectError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    UpdateProjectResponse::from_json(response).map_err(UpdateProjectError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<UpdateProject200Response> for UpdateProjectResponse {
    type Error = ValidationError;

    fn from_json(json: UpdateProject200Response) -> Result<Self, Self::Error> {
        let project = json
            .project
            .map(|p| Project::from_json(*p))
            .transpose()?
            .ok_or(ValidationError::MissingField("project"))?;

        Ok(Self { project })
    }
}
