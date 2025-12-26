use libgn::project::ProjectId;
use pistachio_api_common::admin::project::{
    GetAdminSdkConfigError, GetAdminSdkConfigRequest, GetAdminSdkConfigResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{
    GetAdminSdkConfigError as GenError, get_admin_sdk_config,
};
use crate::generated_admin::models::GetAdminSdkConfig200Response;
use crate::types::FromJson;

impl From<GenError> for GetAdminSdkConfigError {
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

pub(crate) async fn handle_get_admin_sdk_config(
    config: &Configuration,
    req: GetAdminSdkConfigRequest,
) -> Result<GetAdminSdkConfigResponse, GetAdminSdkConfigError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();

    debug!(?project_id, "Sending get_admin_sdk_config request");

    let response = get_admin_sdk_config(config, &project_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_admin_sdk_config response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        GetAdminSdkConfigError::Unknown(format!(
                            "HTTP {}: {}",
                            resp.status, resp.content
                        ))
                    })
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    GetAdminSdkConfigError::ServiceUnavailable(e.to_string())
                }
                _ => GetAdminSdkConfigError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    GetAdminSdkConfigResponse::from_json(response)
        .map_err(GetAdminSdkConfigError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<GetAdminSdkConfig200Response> for GetAdminSdkConfigResponse {
    type Error = ValidationError;

    fn from_json(json: GetAdminSdkConfig200Response) -> Result<Self, Self::Error> {
        let project_id_str = json
            .project_id
            .ok_or(ValidationError::MissingField("project_id"))?;

        let project_id = ProjectId::parse(&project_id_str)
            .map_err(|_| ValidationError::InvalidValue("project_id"))?;

        let storage_bucket = json
            .storage_bucket
            .ok_or(ValidationError::MissingField("storage_bucket"))?;

        let location_id = json
            .location_id
            .ok_or(ValidationError::MissingField("location_id"))?;

        Ok(Self {
            project_id,
            storage_bucket,
            location_id,
        })
    }
}
