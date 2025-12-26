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
use crate::types::FromJson;

impl From<GenError> for ListProjectsError {
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
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    ListProjectsError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
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
