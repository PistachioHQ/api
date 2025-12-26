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
use crate::types::FromJson;

impl From<GenError> for SearchProjectsError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
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
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    SearchProjectsError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
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
