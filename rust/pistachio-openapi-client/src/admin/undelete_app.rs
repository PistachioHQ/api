use libgn::app::App;
use pistachio_api_common::admin::app::{UndeleteAppError, UndeleteAppRequest, UndeleteAppResponse};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{UndeleteAppError as GenError, undelete_app};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::UndeleteApp200Response;
use crate::types::FromJson;

impl From<GenError> for UndeleteAppError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status404(_) => Self::NotFound,
            GenError::Status409(_) => Self::FailedPrecondition,
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
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        UndeleteAppError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                    })
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
