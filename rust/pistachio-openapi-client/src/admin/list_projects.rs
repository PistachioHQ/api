use libgn::project::Project;
use pistachio_api_common::admin::project::{
    ListProjectsError, ListProjectsRequest, ListProjectsResponse,
};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{ListProjectsError as GenError, list_projects};
use crate::generated_admin::models::ListProjects200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for ListProjectsError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
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

pub(crate) async fn handle_list_projects(
    config: &Configuration,
    req: ListProjectsRequest,
) -> Result<ListProjectsResponse, ListProjectsError> {
    debug!("Creating OpenAPI request");

    let page_size = req.pagination.page_size;
    let cursor = req.pagination.cursor;
    let show_deleted = Some(req.show_deleted);
    let sort_string = format_sort_fields(&req.pagination.sort);

    debug!(
        ?page_size,
        ?cursor,
        ?sort_string,
        ?show_deleted,
        "Sending list_projects request"
    );

    let response = list_projects(
        config,
        page_size,
        cursor.as_deref(),
        sort_string.as_deref(),
        show_deleted,
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in list_projects response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();

                // Try to parse RFC 7807 Problem Details from the response content
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => ListProjectsError::BadRequest(problem),
                        401 => ListProjectsError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => ListProjectsError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        500..=599 => ListProjectsError::ServiceError(
                            problem.message.unwrap_or(problem.title),
                        ),
                        _ => ListProjectsError::Unknown(format!(
                            "HTTP {}: {}",
                            status,
                            problem.message.unwrap_or(problem.title)
                        )),
                    };
                }

                // Fall back to entity parsing if RFC 7807 parsing failed
                if let Some(entity) = resp.entity
                    && !matches!(entity, GenError::UnknownValue(_))
                {
                    return entity.into();
                }

                // Last resort: status code mapping with raw content
                match status {
                    400 => ListProjectsError::BadRequest(fallback_error_details(resp.content)),
                    401 => ListProjectsError::Unauthenticated(resp.content),
                    403 => ListProjectsError::PermissionDenied(resp.content),
                    500..=599 => ListProjectsError::ServiceError(resp.content),
                    _ => ListProjectsError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                ListProjectsError::ServiceUnavailable(e.to_string())
            }
            _ => ListProjectsError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    ListProjectsResponse::from_json(response).map_err(ListProjectsError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<ListProjects200Response> for ListProjectsResponse {
    type Error = ValidationError;

    fn from_json(json: ListProjects200Response) -> Result<Self, Self::Error> {
        let projects = json
            .projects
            .unwrap_or_default()
            .into_iter()
            .map(Project::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let next_cursor = json
            .pagination
            .as_ref()
            .and_then(|p| p.next_cursor.as_ref().filter(|s| !s.is_empty()).cloned());
        let total_count = json.pagination.as_ref().and_then(|p| p.total_count);

        Ok(Self {
            projects,
            pagination: PaginationMeta {
                next_cursor,
                total_count,
            },
        })
    }
}
