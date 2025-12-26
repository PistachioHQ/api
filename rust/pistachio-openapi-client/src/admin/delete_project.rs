use pistachio_api_common::admin::project::{
    DeleteProjectError, DeleteProjectRequest, DeleteProjectResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{DeleteProjectError as GenError, delete_project};

impl From<GenError> for DeleteProjectError {
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

pub(crate) async fn handle_delete_project(
    config: &Configuration,
    req: DeleteProjectRequest,
) -> Result<DeleteProjectResponse, DeleteProjectError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending delete_project request");

    let _response = delete_project(config, &project_id).await.map_err(|e| {
        error!(?e, "Error in delete_project response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    DeleteProjectError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                DeleteProjectError::ServiceUnavailable(e.to_string())
            }
            _ => DeleteProjectError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    Ok(DeleteProjectResponse {})
}
