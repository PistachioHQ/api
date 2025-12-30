use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{
    SearchTenantsError, SearchTenantsRequest, SearchTenantsResponse,
};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{SearchTenantsError as GenError, search_tenants};
use crate::generated_admin::models::SearchTenants200Response;
use crate::problem_details::{fallback_problem_details, parse_problem_details};
use crate::types::{FromJson, convert_problem_details};

impl From<GenError> for SearchTenantsError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(convert_problem_details(e)),
            GenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status404(e) => Self::NotFound(convert_problem_details(e)),
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

pub(crate) async fn handle_search_tenants(
    config: &Configuration,
    req: SearchTenantsRequest,
) -> Result<SearchTenantsResponse, SearchTenantsError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let query = if req.params.query.is_empty() {
        None
    } else {
        Some(req.params.query.as_str())
    };
    let page_size = req.params.pagination.page_size;
    let cursor = req.params.pagination.cursor.as_deref();
    let sort_string = format_sort_fields(&req.params.pagination.sort);

    debug!(
        ?project_id,
        ?query,
        ?page_size,
        ?cursor,
        ?sort_string,
        "Sending search_tenants request"
    );

    let response = search_tenants(
        config,
        &project_id,
        query,
        page_size,
        cursor,
        sort_string.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in search_tenants response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Some(problem) = parse_problem_details(&resp.content, status) {
                    return match status {
                        400 => SearchTenantsError::BadRequest(problem),
                        401 => SearchTenantsError::Unauthenticated(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        403 => SearchTenantsError::PermissionDenied(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        404 => SearchTenantsError::NotFound(problem),
                        500..=599 => SearchTenantsError::ServiceError(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        _ => SearchTenantsError::Unknown(format!(
                            "HTTP {}: {}",
                            status,
                            problem.detail.unwrap_or(problem.title)
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
                        SearchTenantsError::BadRequest(fallback_problem_details(400, resp.content))
                    }
                    401 => SearchTenantsError::Unauthenticated(resp.content),
                    403 => SearchTenantsError::PermissionDenied(resp.content),
                    404 => {
                        SearchTenantsError::NotFound(fallback_problem_details(404, resp.content))
                    }
                    500..=599 => SearchTenantsError::ServiceError(resp.content),
                    _ => SearchTenantsError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                SearchTenantsError::ServiceUnavailable(e.to_string())
            }
            _ => SearchTenantsError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    SearchTenantsResponse::from_json(response).map_err(SearchTenantsError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<SearchTenants200Response> for SearchTenantsResponse {
    type Error = ValidationError;

    fn from_json(json: SearchTenants200Response) -> Result<Self, Self::Error> {
        let tenants = json
            .tenants
            .unwrap_or_default()
            .into_iter()
            .map(Tenant::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let next_cursor = json
            .pagination
            .as_ref()
            .and_then(|p| p.next_cursor.as_ref().filter(|s| !s.is_empty()).cloned());
        let total_count = json.pagination.as_ref().and_then(|p| p.total_count);

        Ok(Self {
            tenants,
            pagination: PaginationMeta {
                next_cursor,
                total_count,
            },
        })
    }
}
