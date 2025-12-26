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
use crate::types::FromJson;

impl From<GenError> for UndeleteProjectError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status404(_) => Self::NotFound,
            GenError::Status409(e) => {
                Self::FailedPrecondition(format!("{}: {}", e.code, e.message))
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
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    UndeleteProjectError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
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
