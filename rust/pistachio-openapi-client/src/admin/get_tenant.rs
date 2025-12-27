use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{GetTenantError, GetTenantRequest, GetTenantResponse};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{GetTenantError as GenError, get_tenant};
use crate::generated_admin::models::GetTenant200Response;
use crate::types::FromJson;

impl From<GenError> for GetTenantError {
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

pub(crate) async fn handle_get_tenant(
    config: &Configuration,
    req: GetTenantRequest,
) -> Result<GetTenantResponse, GetTenantError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();

    debug!(?project_id, ?tenant_id, "Sending get_tenant request");

    let response = get_tenant(config, &project_id, &tenant_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_tenant response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        GetTenantError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                    })
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    GetTenantError::ServiceUnavailable(e.to_string())
                }
                _ => GetTenantError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    GetTenantResponse::from_json(response).map_err(GetTenantError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<GetTenant200Response> for GetTenantResponse {
    type Error = ValidationError;

    fn from_json(json: GetTenant200Response) -> Result<Self, Self::Error> {
        let tenant = json
            .tenant
            .map(|t| Tenant::from_json(*t))
            .transpose()?
            .ok_or(ValidationError::MissingField("tenant"))?;

        Ok(Self { tenant })
    }
}
