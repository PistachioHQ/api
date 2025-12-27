use pistachio_api_common::admin::app::{
    GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{GetAppConfigError as GenError, get_app_config};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::GetAppConfig200Response;
use crate::types::FromJson;

impl From<GenError> for GetAppConfigError {
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

pub(crate) async fn handle_get_app_config(
    config: &Configuration,
    req: GetAppConfigRequest,
) -> Result<GetAppConfigResponse, GetAppConfigError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    debug!(?project_id, ?app_id, "Sending get_app_config request");

    let response = get_app_config(config, &project_id, &app_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_app_config response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        GetAppConfigError::Unknown(format!(
                            "HTTP {}: {}",
                            resp.status, resp.content
                        ))
                    })
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    GetAppConfigError::ServiceUnavailable(e.to_string())
                }
                _ => GetAppConfigError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    GetAppConfigResponse::from_json(response).map_err(GetAppConfigError::ResponseValidationError)
}

impl FromJson<GetAppConfig200Response> for GetAppConfigResponse {
    type Error = ValidationError;

    fn from_json(json: GetAppConfig200Response) -> Result<Self, Self::Error> {
        Ok(Self {
            config_file_contents: json
                .config_file_contents
                .ok_or(ValidationError::MissingField("config_file_contents"))?,
            config_filename: json
                .config_filename
                .ok_or(ValidationError::MissingField("config_filename"))?,
        })
    }
}
