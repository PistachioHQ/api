use libgn::app::App;
use pistachio_api_common::admin::app::{SearchAppsError, SearchAppsRequest, SearchAppsResponse};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{SearchAppsError as GenError, search_apps};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::SearchApps200Response;
use crate::types::FromJson;

impl From<GenError> for SearchAppsError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status404(_) => Self::NotFound,
            GenError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

pub(crate) async fn handle_search_apps(
    config: &Configuration,
    req: SearchAppsRequest,
) -> Result<SearchAppsResponse, SearchAppsError> {
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
        "Sending search_apps request"
    );

    let response = search_apps(
        config,
        &project_id,
        query,
        page_size,
        cursor,
        sort_string.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in search_apps response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    SearchAppsError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                SearchAppsError::ServiceUnavailable(e.to_string())
            }
            _ => SearchAppsError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    SearchAppsResponse::from_json(response).map_err(SearchAppsError::ResponseValidationError)
}

impl FromJson<SearchApps200Response> for SearchAppsResponse {
    type Error = ValidationError;

    fn from_json(json: SearchApps200Response) -> Result<Self, Self::Error> {
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
