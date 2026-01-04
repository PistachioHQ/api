use pistachio_api_common::admin::auth_provider::{
    ListProjectAuthProvidersError, ListProjectAuthProvidersRequest,
    ListProjectAuthProvidersResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    ListProjectAuthProvidersError as GenError, list_project_auth_providers,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{convert_error_details, provider_from_json};

impl From<GenError> for ListProjectAuthProvidersError {
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

pub(crate) async fn handle_list_project_auth_providers(
    config: &Configuration,
    req: ListProjectAuthProvidersRequest,
) -> Result<ListProjectAuthProvidersResponse, ListProjectAuthProvidersError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending list_project_auth_providers request");

    let response = list_project_auth_providers(config, &project_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in list_project_auth_providers response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();

                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => ListProjectAuthProvidersError::BadRequest(problem),
                            401 => ListProjectAuthProvidersError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => ListProjectAuthProvidersError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => ListProjectAuthProvidersError::NotFound(problem),
                            500..=599 => ListProjectAuthProvidersError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => ListProjectAuthProvidersError::Unknown(format!(
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
                        400 => ListProjectAuthProvidersError::BadRequest(fallback_error_details(
                            resp.content,
                        )),
                        401 => ListProjectAuthProvidersError::Unauthenticated(resp.content),
                        403 => ListProjectAuthProvidersError::PermissionDenied(resp.content),
                        404 => ListProjectAuthProvidersError::NotFound(fallback_error_details(
                            resp.content,
                        )),
                        500..=599 => ListProjectAuthProvidersError::ServiceError(resp.content),
                        _ => ListProjectAuthProvidersError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    ListProjectAuthProvidersError::ServiceUnavailable(e.to_string())
                }
                _ => ListProjectAuthProvidersError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let providers = response
        .providers
        .unwrap_or_default()
        .into_iter()
        .map(provider_from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ListProjectAuthProvidersError::ResponseValidationError)?;

    // TODO: Extract pagination from response once OpenAPI spec is regenerated
    Ok(ListProjectAuthProvidersResponse {
        providers,
        pagination: None,
    })
}
