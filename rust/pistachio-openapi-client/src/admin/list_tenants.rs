use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{
    ListTenantsError, ListTenantsRequest, ListTenantsResponse,
};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{ListTenantsError as GenError, list_tenants};
use crate::generated_admin::models::ListTenants200Response;
use crate::problem_details::{fallback_problem_details, parse_problem_details};
use crate::types::{FromJson, convert_problem_details};

impl From<GenError> for ListTenantsError {
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

pub(crate) async fn handle_list_tenants(
    config: &Configuration,
    req: ListTenantsRequest,
) -> Result<ListTenantsResponse, ListTenantsError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let page_size = req.pagination.page_size;
    let cursor = req.pagination.cursor;
    let sort_string = format_sort_fields(&req.pagination.sort);

    debug!(
        ?project_id,
        ?page_size,
        ?cursor,
        ?sort_string,
        "Sending list_tenants request"
    );

    let response = list_tenants(
        config,
        &project_id,
        page_size,
        cursor.as_deref(),
        sort_string.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in list_tenants response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Some(problem) = parse_problem_details(&resp.content, status) {
                    return match status {
                        400 => ListTenantsError::BadRequest(problem),
                        401 => ListTenantsError::Unauthenticated(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        403 => ListTenantsError::PermissionDenied(
                            problem.detail.unwrap_or(problem.title),
                        ),
                        404 => ListTenantsError::NotFound(problem),
                        500..=599 => {
                            ListTenantsError::ServiceError(problem.detail.unwrap_or(problem.title))
                        }
                        _ => ListTenantsError::Unknown(format!(
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
                        ListTenantsError::BadRequest(fallback_problem_details(400, resp.content))
                    }
                    401 => ListTenantsError::Unauthenticated(resp.content),
                    403 => ListTenantsError::PermissionDenied(resp.content),
                    404 => ListTenantsError::NotFound(fallback_problem_details(404, resp.content)),
                    500..=599 => ListTenantsError::ServiceError(resp.content),
                    _ => ListTenantsError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                ListTenantsError::ServiceUnavailable(e.to_string())
            }
            _ => ListTenantsError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    ListTenantsResponse::from_json(response).map_err(ListTenantsError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<ListTenants200Response> for ListTenantsResponse {
    type Error = ValidationError;

    fn from_json(json: ListTenants200Response) -> Result<Self, Self::Error> {
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
