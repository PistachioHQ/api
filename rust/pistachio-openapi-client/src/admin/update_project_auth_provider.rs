use pistachio_api_common::admin::auth_provider::{
    UpdateProjectAuthProviderError, UpdateProjectAuthProviderRequest,
    UpdateProjectAuthProviderResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    UpdateProjectAuthProviderError as GenError, update_project_auth_provider,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{build_update_project_request, convert_error_details, provider_from_json};

impl From<GenError> for UpdateProjectAuthProviderError {
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

pub(crate) async fn handle_update_project_auth_provider(
    config: &Configuration,
    req: UpdateProjectAuthProviderRequest,
) -> Result<UpdateProjectAuthProviderResponse, UpdateProjectAuthProviderError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let provider_id = req.provider_id.to_string();

    let request_body = build_update_project_request(
        req.enabled,
        req.display_order,
        req.config.as_ref(),
        req.client_secret,
    );

    debug!(
        ?project_id,
        ?provider_id,
        "Sending update_project_auth_provider request"
    );

    let response = update_project_auth_provider(config, &project_id, &provider_id, request_body)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_project_auth_provider response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => UpdateProjectAuthProviderError::BadRequest(problem),
                            401 => UpdateProjectAuthProviderError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => UpdateProjectAuthProviderError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => UpdateProjectAuthProviderError::NotFound(problem),
                            500..=599 => UpdateProjectAuthProviderError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => UpdateProjectAuthProviderError::Unknown(format!(
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
                        400 => UpdateProjectAuthProviderError::BadRequest(fallback_error_details(
                            resp.content,
                        )),
                        401 => UpdateProjectAuthProviderError::Unauthenticated(resp.content),
                        403 => UpdateProjectAuthProviderError::PermissionDenied(resp.content),
                        404 => UpdateProjectAuthProviderError::NotFound(fallback_error_details(
                            resp.content,
                        )),
                        500..=599 => UpdateProjectAuthProviderError::ServiceError(resp.content),
                        _ => UpdateProjectAuthProviderError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    UpdateProjectAuthProviderError::ServiceUnavailable(e.to_string())
                }
                _ => UpdateProjectAuthProviderError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let provider = response
        .provider
        .map(|p| provider_from_json(*p))
        .transpose()
        .map_err(UpdateProjectAuthProviderError::ResponseValidationError)?
        .ok_or(UpdateProjectAuthProviderError::ResponseValidationError(
            ValidationError::MissingField("provider"),
        ))?;

    Ok(UpdateProjectAuthProviderResponse { provider })
}
