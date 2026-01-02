use libgn::tenant::Tenant;
use pistachio_api_common::admin::tenant::{
    UpdateTenantError, UpdateTenantRequest, UpdateTenantResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{UpdateTenantError as GenError, update_tenant};
use crate::generated_admin::models::{UpdateTenant200Response, UpdateTenantRequest as GenRequest};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for UpdateTenantError {
    fn from(error: GenError) -> Self {
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

pub(crate) async fn handle_update_tenant(
    config: &Configuration,
    req: UpdateTenantRequest,
) -> Result<UpdateTenantResponse, UpdateTenantError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    // Convert string MFA config to enum values
    // The generated model uses Option<Option<Vec<MfaConfig>>> to represent nullable arrays:
    // - None (outer) = field omitted, don't change
    // - Some(Some([])) = explicitly set to empty, clear all MFA
    // - Some(Some([Phone])) = set MFA providers
    let mfa_config = req.mfa_config.map(|configs| {
        Some(
            configs
                .into_iter()
                .filter_map(|s| match s.as_str() {
                    "phone" => Some(
                        crate::generated_admin::models::update_tenant_request::MfaConfig::Phone,
                    ),
                    "totp" => {
                        Some(crate::generated_admin::models::update_tenant_request::MfaConfig::Totp)
                    }
                    _ => None,
                })
                .collect(),
        )
    });

    let gen_request = GenRequest {
        display_name: req.display_name.map(|d| d.to_string()),
        allow_pdpka_signup: req.allow_pdpka_signup,
        disable_auth: req.disable_auth,
        mfa_config,
    };

    debug!(?project_id, ?tenant_id, "Sending update_tenant request");

    let response = update_tenant(config, &project_id, &tenant_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_tenant response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => UpdateTenantError::BadRequest(problem),
                            401 => UpdateTenantError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => UpdateTenantError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => UpdateTenantError::NotFound(problem),
                            500..=599 => UpdateTenantError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => UpdateTenantError::Unknown(format!(
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
                        400 => UpdateTenantError::BadRequest(fallback_error_details(resp.content)),
                        401 => UpdateTenantError::Unauthenticated(resp.content),
                        403 => UpdateTenantError::PermissionDenied(resp.content),
                        404 => UpdateTenantError::NotFound(fallback_error_details(resp.content)),
                        500..=599 => UpdateTenantError::ServiceError(resp.content),
                        _ => {
                            UpdateTenantError::Unknown(format!("HTTP {}: {}", status, resp.content))
                        }
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    UpdateTenantError::ServiceUnavailable(e.to_string())
                }
                _ => UpdateTenantError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    UpdateTenantResponse::from_json(response).map_err(UpdateTenantError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<UpdateTenant200Response> for UpdateTenantResponse {
    type Error = ValidationError;

    fn from_json(json: UpdateTenant200Response) -> Result<Self, Self::Error> {
        let tenant = json
            .tenant
            .map(|t| Tenant::from_json(*t))
            .transpose()?
            .ok_or(ValidationError::MissingField("tenant"))?;

        Ok(Self { tenant })
    }
}
