use pistachio_api_common::admin::app::{
    GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{GetAppConfigError as GenError, get_app_config};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::GetAppConfig200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for GetAppConfigError {
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
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => GetAppConfigError::BadRequest(problem),
                            401 => GetAppConfigError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => GetAppConfigError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => GetAppConfigError::NotFound(problem),
                            500..=599 => GetAppConfigError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => GetAppConfigError::Unknown(format!(
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
                        400 => GetAppConfigError::BadRequest(fallback_error_details(resp.content)),
                        401 => GetAppConfigError::Unauthenticated(resp.content),
                        403 => GetAppConfigError::PermissionDenied(resp.content),
                        404 => GetAppConfigError::NotFound(fallback_error_details(resp.content)),
                        500..=599 => GetAppConfigError::ServiceError(resp.content),
                        _ => {
                            GetAppConfigError::Unknown(format!("HTTP {}: {}", status, resp.content))
                        }
                    }
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
