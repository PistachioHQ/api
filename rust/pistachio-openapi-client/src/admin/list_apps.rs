use libgn::app::App;
use pistachio_api_common::admin::app::{ListAppsError, ListAppsRequest, ListAppsResponse};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{ListAppsError as GenError, list_apps};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::ListApps200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for ListAppsError {
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

pub(crate) async fn handle_list_apps(
    config: &Configuration,
    req: ListAppsRequest,
) -> Result<ListAppsResponse, ListAppsError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let page_size = req.pagination.page_size;
    let cursor = req.pagination.cursor;
    let sort_string = format_sort_fields(&req.pagination.sort);
    let show_deleted = Some(req.show_deleted);

    debug!(
        ?project_id,
        ?page_size,
        ?cursor,
        ?sort_string,
        "Sending list_apps request"
    );

    let response = list_apps(
        config,
        &project_id,
        page_size,
        cursor.as_deref(),
        sort_string.as_deref(),
        show_deleted,
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in list_apps response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => ListAppsError::BadRequest(problem),
                        401 => {
                            ListAppsError::Unauthenticated(problem.message.unwrap_or(problem.title))
                        }
                        403 => ListAppsError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        404 => ListAppsError::NotFound(problem),
                        500..=599 => {
                            ListAppsError::ServiceError(problem.message.unwrap_or(problem.title))
                        }
                        _ => ListAppsError::Unknown(format!(
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
                    400 => ListAppsError::BadRequest(fallback_error_details(resp.content)),
                    401 => ListAppsError::Unauthenticated(resp.content),
                    403 => ListAppsError::PermissionDenied(resp.content),
                    404 => ListAppsError::NotFound(fallback_error_details(resp.content)),
                    500..=599 => ListAppsError::ServiceError(resp.content),
                    _ => ListAppsError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                ListAppsError::ServiceUnavailable(e.to_string())
            }
            _ => ListAppsError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    ListAppsResponse::from_json(response).map_err(ListAppsError::ResponseValidationError)
}

impl FromJson<ListApps200Response> for ListAppsResponse {
    type Error = ValidationError;

    fn from_json(json: ListApps200Response) -> Result<Self, Self::Error> {
        let apps = json
            .apps
            .unwrap_or_default()
            .into_iter()
            .map(App::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let next_cursor = json
            .pagination
            .as_ref()
            .and_then(|p| p.next_cursor.as_ref().filter(|s| !s.is_empty()).cloned());
        let total_count = json.pagination.as_ref().and_then(|p| p.total_count);

        Ok(Self {
            apps,
            pagination: PaginationMeta {
                next_cursor,
                total_count,
            },
        })
    }
}
