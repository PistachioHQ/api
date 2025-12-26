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
use crate::types::FromJson;

impl From<GenError> for UpdateProjectError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status404(_) => Self::NotFound,
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
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        UpdateProjectError::Unknown(format!(
                            "HTTP {}: {}",
                            resp.status, resp.content
                        ))
                    })
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
