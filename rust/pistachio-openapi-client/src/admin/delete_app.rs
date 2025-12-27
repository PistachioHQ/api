use pistachio_api_common::admin::app::{DeleteAppError, DeleteAppRequest, DeleteAppResponse};
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{DeleteAppError as GenError, delete_app};
use crate::generated_admin::apis::configuration::Configuration;

impl From<GenError> for DeleteAppError {
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

pub(crate) async fn handle_delete_app(
    config: &Configuration,
    req: DeleteAppRequest,
) -> Result<DeleteAppResponse, DeleteAppError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    debug!(?project_id, ?app_id, "Sending delete_app request");

    let _response = delete_app(config, &project_id, &app_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_app response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        DeleteAppError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                    })
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    DeleteAppError::ServiceUnavailable(e.to_string())
                }
                _ => DeleteAppError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    Ok(DeleteAppResponse {})
}
