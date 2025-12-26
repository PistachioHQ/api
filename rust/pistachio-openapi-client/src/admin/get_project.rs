use libgn::project::Project;
use pistachio_api_common::admin::project::{
    GetProjectError, GetProjectRequest, GetProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{GetProjectError as GenError, get_project};
use crate::generated_admin::models::GetProject200Response;
use crate::types::FromJson;

impl From<GenError> for GetProjectError {
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
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    GetProjectError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
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
