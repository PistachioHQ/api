use libgn::app::App;
use pistachio_api_common::admin::app::{ListAppsError, ListAppsRequest, ListAppsResponse};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{ListAppsError as GenError, list_apps};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::ListApps200Response;
use crate::types::FromJson;

impl From<GenError> for ListAppsError {
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
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    ListAppsError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
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
