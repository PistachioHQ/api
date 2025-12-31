use libgn::project::Project;
use pistachio_api_common::admin::project::{
    GetProjectError, GetProjectRequest, GetProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{GetProjectError as GenError, get_project};
use crate::generated_admin::models::GetProject200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for GetProjectError {
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

pub(crate) async fn handle_get_project(
    config: &Configuration,
    req: GetProjectRequest,
) -> Result<GetProjectResponse, GetProjectError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending get_project request");

    let response = get_project(config, &project_id).await.map_err(|e| {
        error!(?e, "Error in get_project response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => GetProjectError::BadRequest(problem),
                        401 => GetProjectError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => GetProjectError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        404 => GetProjectError::NotFound(problem),
                        500..=599 => {
                            GetProjectError::ServiceError(problem.message.unwrap_or(problem.title))
                        }
                        _ => GetProjectError::Unknown(format!(
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
                    400 => GetProjectError::BadRequest(fallback_error_details(resp.content)),
                    401 => GetProjectError::Unauthenticated(resp.content),
                    403 => GetProjectError::PermissionDenied(resp.content),
                    404 => GetProjectError::NotFound(fallback_error_details(resp.content)),
                    500..=599 => GetProjectError::ServiceError(resp.content),
                    _ => GetProjectError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                GetProjectError::ServiceUnavailable(e.to_string())
            }
            _ => GetProjectError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    GetProjectResponse::from_json(response).map_err(GetProjectError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<GetProject200Response> for GetProjectResponse {
    type Error = ValidationError;

    fn from_json(json: GetProject200Response) -> Result<Self, Self::Error> {
        let project = json
            .project
            .map(|p| Project::from_json(*p))
            .transpose()?
            .ok_or(ValidationError::MissingField("project"))?;

        Ok(Self { project })
    }
}
