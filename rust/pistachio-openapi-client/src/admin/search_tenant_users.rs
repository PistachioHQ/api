use pistachio_api_common::admin::user::{
    SearchTenantUsersError, SearchTenantUsersRequest, SearchTenantUsersResponse, User,
};
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    SearchTenantUsersError as GenError, search_tenant_users,
};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::FromJson;

impl From<GenError> for SearchTenantUsersError {
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

pub(crate) async fn handle_search_tenant_users(
    config: &Configuration,
    req: SearchTenantUsersRequest,
) -> Result<SearchTenantUsersResponse, SearchTenantUsersError> {
    debug!("Creating OpenAPI request for search_tenant_users");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let query = req.params.query;
    let page_size = req.params.pagination.page_size;
    let cursor = req.params.pagination.cursor;
    let sort_string = format_sort_fields(&req.params.pagination.sort);

    debug!(
        ?project_id,
        ?tenant_id,
        ?query,
        ?page_size,
        ?cursor,
        ?sort_string,
        "Sending search_tenant_users request"
    );

    let response = search_tenant_users(
        config,
        &project_id,
        &tenant_id,
        Some(&query),
        page_size,
        cursor.as_deref(),
        sort_string.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in search_tenant_users response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => SearchTenantUsersError::BadRequest(problem),
                        401 => SearchTenantUsersError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => SearchTenantUsersError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        404 => SearchTenantUsersError::NotFound(problem),
                        500..=599 => SearchTenantUsersError::ServiceError(
                            problem.message.unwrap_or(problem.title),
                        ),
                        _ => SearchTenantUsersError::Unknown(format!(
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
                    400 => SearchTenantUsersError::BadRequest(fallback_error_details(resp.content)),
                    401 => SearchTenantUsersError::Unauthenticated(resp.content),
                    403 => SearchTenantUsersError::PermissionDenied(resp.content),
                    404 => SearchTenantUsersError::NotFound(fallback_error_details(resp.content)),
                    500..=599 => SearchTenantUsersError::ServiceError(resp.content),
                    _ => SearchTenantUsersError::Unknown(format!(
                        "HTTP {}: {}",
                        status, resp.content
                    )),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                SearchTenantUsersError::ServiceUnavailable(e.to_string())
            }
            _ => SearchTenantUsersError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    let users: Vec<User> = response
        .users
        .unwrap_or_default()
        .into_iter()
        .map(User::from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(SearchTenantUsersError::ResponseValidationError)?;

    let next_cursor = response
        .pagination
        .as_ref()
        .and_then(|p| p.next_cursor.as_ref().filter(|s| !s.is_empty()).cloned());
    let total_count = response.pagination.as_ref().and_then(|p| p.total_count);

    Ok(SearchTenantUsersResponse {
        users,
        pagination: PaginationMeta {
            next_cursor,
            total_count,
        },
    })
}
