use pistachio_api_common::admin::user::{
    SearchProjectUsersError, SearchProjectUsersRequest, SearchProjectUsersResponse, User,
};
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    SearchProjectUsersError as GenError, search_project_users,
};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::FromJson;

impl From<GenError> for SearchProjectUsersError {
    fn from(error: GenError) -> Self {
        use crate::types::convert_error_details;
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

pub(crate) async fn handle_search_project_users(
    config: &Configuration,
    req: SearchProjectUsersRequest,
) -> Result<SearchProjectUsersResponse, SearchProjectUsersError> {
    debug!("Creating OpenAPI request for search_project_users");

    let project_id = req.project_id.to_string();
    let query = req.params.query;
    let page_size = req.params.pagination.page_size;
    let cursor = req.params.pagination.cursor;
    let sort_string = format_sort_fields(&req.params.pagination.sort);

    debug!(
        ?project_id,
        ?query,
        ?page_size,
        ?cursor,
        ?sort_string,
        "Sending search_project_users request"
    );

    let response = search_project_users(
        config,
        &project_id,
        Some(&query),
        page_size,
        cursor.as_deref(),
        sort_string.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in search_project_users response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => SearchProjectUsersError::BadRequest(problem),
                        401 => SearchProjectUsersError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => SearchProjectUsersError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        404 => SearchProjectUsersError::NotFound(problem),
                        500..=599 => SearchProjectUsersError::ServiceError(
                            problem.message.unwrap_or(problem.title),
                        ),
                        _ => SearchProjectUsersError::Unknown(format!(
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
                        SearchProjectUsersError::BadRequest(fallback_error_details(resp.content))
                    }
                    401 => SearchProjectUsersError::Unauthenticated(resp.content),
                    403 => SearchProjectUsersError::PermissionDenied(resp.content),
                    404 => SearchProjectUsersError::NotFound(fallback_error_details(resp.content)),
                    500..=599 => SearchProjectUsersError::ServiceError(resp.content),
                    _ => SearchProjectUsersError::Unknown(format!(
                        "HTTP {}: {}",
                        status, resp.content
                    )),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                SearchProjectUsersError::ServiceUnavailable(e.to_string())
            }
            _ => SearchProjectUsersError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    let users: Vec<User> = response
        .users
        .unwrap_or_default()
        .into_iter()
        .map(User::from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(SearchProjectUsersError::ResponseValidationError)?;

    let next_cursor = response
        .pagination
        .as_ref()
        .and_then(|p| p.next_cursor.as_ref().filter(|s| !s.is_empty()).cloned());
    let total_count = response.pagination.as_ref().and_then(|p| p.total_count);

    Ok(SearchProjectUsersResponse {
        users,
        pagination: PaginationMeta {
            next_cursor,
            total_count,
        },
    })
}
