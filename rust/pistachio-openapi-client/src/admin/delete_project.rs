use libgn::project::Project;
use pistachio_api_common::admin::project::{
    DeleteProjectError, DeleteProjectRequest, DeleteProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{DeleteProjectError as GenError, delete_project};
use crate::generated_admin::models::DeleteProject200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for DeleteProjectError {
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

pub(crate) async fn handle_delete_project(
    config: &Configuration,
    req: DeleteProjectRequest,
) -> Result<DeleteProjectResponse, DeleteProjectError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending delete_project request");

    let response = delete_project(config, &project_id).await.map_err(|e| {
        error!(?e, "Error in delete_project response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();

                // Try to parse RFC 7807 Problem Details from the response content
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => DeleteProjectError::BadRequest(problem),
                        401 => DeleteProjectError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => DeleteProjectError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        404 => DeleteProjectError::NotFound(problem),
                        500..=599 => DeleteProjectError::ServiceError(
                            problem.message.unwrap_or(problem.title),
                        ),
                        _ => DeleteProjectError::Unknown(format!(
                            "HTTP {}: {}",
                            status,
                            problem.message.unwrap_or(problem.title)
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
                    400 => DeleteProjectError::BadRequest(fallback_error_details(resp.content)),
                    401 => DeleteProjectError::Unauthenticated(resp.content),
                    403 => DeleteProjectError::PermissionDenied(resp.content),
                    404 => DeleteProjectError::NotFound(fallback_error_details(resp.content)),
                    500..=599 => DeleteProjectError::ServiceError(resp.content),
                    _ => DeleteProjectError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                DeleteProjectError::ServiceUnavailable(e.to_string())
            }
            _ => DeleteProjectError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    DeleteProjectResponse::from_json(response).map_err(DeleteProjectError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<DeleteProject200Response> for DeleteProjectResponse {
    type Error = ValidationError;

    fn from_json(json: DeleteProject200Response) -> Result<Self, Self::Error> {
        let project = json
            .project
            .map(|p| Project::from_json(*p))
            .transpose()?
            .ok_or(ValidationError::MissingField("project"))?;

        Ok(Self { project })
    }
}
