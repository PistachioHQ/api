use libgn::project::Project;
use pistachio_api_common::admin::project::{
    SearchProjectsError, SearchProjectsRequest, SearchProjectsResponse,
};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{
    SearchProjectsError as GenError, search_projects,
};
use crate::generated_admin::models::SearchProjects200Response;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for SearchProjectsError {
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

pub(crate) async fn handle_search_projects(
    config: &Configuration,
    req: SearchProjectsRequest,
) -> Result<SearchProjectsResponse, SearchProjectsError> {
    debug!("Creating OpenAPI request");

    let query = req.params.query;
    let page_size = req.params.pagination.page_size;
    let cursor = req.params.pagination.cursor;
    let sort_string = format_sort_fields(&req.params.pagination.sort);

    debug!(
        ?query,
        ?page_size,
        ?cursor,
        ?sort_string,
        "Sending search_projects request"
    );

    let response = search_projects(
        config,
        Some(&query),
        page_size,
        cursor.as_deref(),
        sort_string.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in search_projects response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();

                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => SearchProjectsError::BadRequest(problem),
                        401 => SearchProjectsError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => SearchProjectsError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        500..=599 => SearchProjectsError::ServiceError(
                            problem.message.unwrap_or(problem.title),
                        ),
                        _ => SearchProjectsError::Unknown(format!(
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
                    400 => SearchProjectsError::BadRequest(fallback_error_details(resp.content)),
                    401 => SearchProjectsError::Unauthenticated(resp.content),
                    403 => SearchProjectsError::PermissionDenied(resp.content),
                    500..=599 => SearchProjectsError::ServiceError(resp.content),
                    _ => SearchProjectsError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                SearchProjectsError::ServiceUnavailable(e.to_string())
            }
            _ => SearchProjectsError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    SearchProjectsResponse::from_json(response)
        .map_err(SearchProjectsError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<SearchProjects200Response> for SearchProjectsResponse {
    type Error = ValidationError;

    fn from_json(json: SearchProjects200Response) -> Result<Self, Self::Error> {
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
