use pistachio_api_common::admin::tenant::{
    DeleteTenantError, DeleteTenantRequest, DeleteTenantResponse,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{DeleteTenantError as GenError, delete_tenant};

impl From<GenError> for DeleteTenantError {
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

pub(crate) async fn handle_delete_tenant(
    config: &Configuration,
    req: DeleteTenantRequest,
) -> Result<DeleteTenantResponse, DeleteTenantError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();

    debug!(?project_id, ?tenant_id, "Sending delete_tenant request");

    let _response = delete_tenant(config, &project_id, &tenant_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_tenant response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        DeleteTenantError::Unknown(format!(
                            "HTTP {}: {}",
                            resp.status, resp.content
                        ))
                    })
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    DeleteTenantError::ServiceUnavailable(e.to_string())
                }
                _ => DeleteTenantError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    Ok(DeleteTenantResponse {})
}
