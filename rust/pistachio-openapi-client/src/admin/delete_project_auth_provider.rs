use pistachio_api_common::admin::auth_provider::{
    DeleteProjectAuthProviderError, DeleteProjectAuthProviderRequest,
    DeleteProjectAuthProviderResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    DeleteProjectAuthProviderError as GenError, delete_project_auth_provider,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::convert_error_details;

impl From<GenError> for DeleteProjectAuthProviderError {
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

pub(crate) async fn handle_delete_project_auth_provider(
    config: &Configuration,
    req: DeleteProjectAuthProviderRequest,
) -> Result<DeleteProjectAuthProviderResponse, DeleteProjectAuthProviderError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let provider_id = req.provider_id.clone();

    debug!(
        ?project_id,
        ?provider_id,
        "Sending delete_project_auth_provider request"
    );

    delete_project_auth_provider(config, &project_id, &provider_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_project_auth_provider response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => DeleteProjectAuthProviderError::BadRequest(problem),
                            401 => DeleteProjectAuthProviderError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => DeleteProjectAuthProviderError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => DeleteProjectAuthProviderError::NotFound(problem),
                            500..=599 => DeleteProjectAuthProviderError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => DeleteProjectAuthProviderError::Unknown(format!(
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
                        400 => DeleteProjectAuthProviderError::BadRequest(fallback_error_details(
                            resp.content,
                        )),
                        401 => DeleteProjectAuthProviderError::Unauthenticated(resp.content),
                        403 => DeleteProjectAuthProviderError::PermissionDenied(resp.content),
                        404 => DeleteProjectAuthProviderError::NotFound(fallback_error_details(
                            resp.content,
                        )),
                        500..=599 => DeleteProjectAuthProviderError::ServiceError(resp.content),
                        _ => DeleteProjectAuthProviderError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    DeleteProjectAuthProviderError::ServiceUnavailable(e.to_string())
                }
                _ => DeleteProjectAuthProviderError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    Ok(DeleteProjectAuthProviderResponse {})
}
