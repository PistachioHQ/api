use libgn::project::ProjectId;
use pistachio_api_common::admin::project::{
    GetAdminSdkConfigError, GetAdminSdkConfigRequest, GetAdminSdkConfigResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{
    GetAdminSdkConfigError as GenError, get_admin_sdk_config,
};
use crate::generated_admin::models::GetAdminSdkConfig200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for GetAdminSdkConfigError {
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

pub(crate) async fn handle_get_admin_sdk_config(
    config: &Configuration,
    req: GetAdminSdkConfigRequest,
) -> Result<GetAdminSdkConfigResponse, GetAdminSdkConfigError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending get_admin_sdk_config request");

    let response = get_admin_sdk_config(config, &project_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_admin_sdk_config response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => GetAdminSdkConfigError::BadRequest(problem),
                            401 => GetAdminSdkConfigError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => GetAdminSdkConfigError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => GetAdminSdkConfigError::NotFound(problem),
                            500..=599 => GetAdminSdkConfigError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => GetAdminSdkConfigError::Unknown(format!(
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
                        400 => {
                            GetAdminSdkConfigError::BadRequest(fallback_error_details(resp.content))
                        }
                        401 => GetAdminSdkConfigError::Unauthenticated(resp.content),
                        403 => GetAdminSdkConfigError::PermissionDenied(resp.content),
                        404 => {
                            GetAdminSdkConfigError::NotFound(fallback_error_details(resp.content))
                        }
                        500..=599 => GetAdminSdkConfigError::ServiceError(resp.content),
                        _ => GetAdminSdkConfigError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    GetAdminSdkConfigError::ServiceUnavailable(e.to_string())
                }
                _ => GetAdminSdkConfigError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    GetAdminSdkConfigResponse::from_json(response)
        .map_err(GetAdminSdkConfigError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<GetAdminSdkConfig200Response> for GetAdminSdkConfigResponse {
    type Error = ValidationError;

    fn from_json(json: GetAdminSdkConfig200Response) -> Result<Self, Self::Error> {
        let project_id_str = json
            .project_id
            .ok_or(ValidationError::MissingField("project_id"))?;

        let project_id = ProjectId::parse(&project_id_str)
            .map_err(|_| ValidationError::InvalidValue("project_id"))?;

        let storage_bucket = json
            .storage_bucket
            .ok_or(ValidationError::MissingField("storage_bucket"))?;

        let location_id = json
            .location_id
            .ok_or(ValidationError::MissingField("location_id"))?;

        Ok(Self {
            project_id,
            storage_bucket,
            location_id,
        })
    }
}
