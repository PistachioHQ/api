use libgn::pistachio_id::TenantId as PistachioTenantId;
use libgn::tenant::{Tenant, TenantDisplayName, TenantId, TenantName};
use pistachio_api_common::admin::tenant::{
    CreateTenantError, CreateTenantRequest, CreateTenantResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tenants_api::{CreateTenantError as GenError, create_tenant};
use crate::generated_admin::models::{
    CreateTenant201Response, CreateTenantRequest as GenRequest, ListTenants200ResponseTenantsInner,
};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details, parse_timestamp};

impl From<GenError> for CreateTenantError {
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
            GenError::Status409(_) => Self::AlreadyExists,
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

pub(crate) async fn handle_create_tenant(
    config: &Configuration,
    req: CreateTenantRequest,
) -> Result<CreateTenantResponse, CreateTenantError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    // Convert string MFA config to enum values
    let mfa_config = if req.mfa_config.is_empty() {
        None
    } else {
        Some(
            req.mfa_config
                .into_iter()
                .filter_map(|s| match s.as_str() {
                    "phone" => Some(
                        crate::generated_admin::models::create_tenant_request::MfaConfig::Phone,
                    ),
                    "totp" => {
                        Some(crate::generated_admin::models::create_tenant_request::MfaConfig::Totp)
                    }
                    _ => None,
                })
                .collect(),
        )
    };

    // Convert Option<TenantDisplayName> to Option<Option<String>>
    // None -> None (field omitted, server generates)
    // Some(name) -> Some(Some(name.to_string())) (explicit value)
    let display_name = req.display_name.map(|name| Some(name.to_string()));

    let gen_request = GenRequest {
        tenant_id: req.tenant_id.map(|id| id.to_string()),
        display_name,
        allow_pdpka_signup: req.allow_pdpka_signup,
        disable_auth: req.disable_auth,
        mfa_config,
    };

    debug!(?project_id, "Sending create_tenant request");

    let response = create_tenant(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in create_tenant response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => CreateTenantError::BadRequest(problem),
                            401 => CreateTenantError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => CreateTenantError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => CreateTenantError::NotFound(problem),
                            409 => CreateTenantError::AlreadyExists,
                            500..=599 => CreateTenantError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => CreateTenantError::Unknown(format!(
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
                        400 => CreateTenantError::BadRequest(fallback_error_details(resp.content)),
                        401 => CreateTenantError::Unauthenticated(resp.content),
                        403 => CreateTenantError::PermissionDenied(resp.content),
                        404 => CreateTenantError::NotFound(fallback_error_details(resp.content)),
                        409 => CreateTenantError::AlreadyExists,
                        500..=599 => CreateTenantError::ServiceError(resp.content),
                        _ => {
                            CreateTenantError::Unknown(format!("HTTP {}: {}", status, resp.content))
                        }
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    CreateTenantError::ServiceUnavailable(e.to_string())
                }
                _ => CreateTenantError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    CreateTenantResponse::from_json(response).map_err(CreateTenantError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<CreateTenant201Response> for CreateTenantResponse {
    type Error = ValidationError;

    fn from_json(json: CreateTenant201Response) -> Result<Self, Self::Error> {
        let tenant = json
            .tenant
            .map(|t| Tenant::from_json(*t))
            .transpose()?
            .ok_or(ValidationError::MissingField("tenant"))?;

        Ok(Self { tenant })
    }
}

impl FromJson<ListTenants200ResponseTenantsInner> for Tenant {
    type Error = ValidationError;

    fn from_json(json: ListTenants200ResponseTenantsInner) -> Result<Self, Self::Error> {
        let name_str = json.name.ok_or(ValidationError::MissingField("name"))?;
        let name =
            TenantName::parse(&name_str).map_err(|_| ValidationError::InvalidValue("name"))?;
        let project_id = name.project_id();

        let tenant_id_str = json
            .tenant_id
            .ok_or(ValidationError::MissingField("tenant_id"))?;
        let tenant_id = TenantId::parse(&tenant_id_str)
            .map_err(|_| ValidationError::InvalidValue("tenant_id"))?;

        let pistachio_id_str = json
            .pistachio_id
            .ok_or(ValidationError::MissingField("pistachio_id"))?;
        let pistachio_id = PistachioTenantId::parse(&pistachio_id_str)?;

        let display_name_str = json
            .display_name
            .ok_or(ValidationError::MissingField("display_name"))?;
        let display_name = TenantDisplayName::parse(&display_name_str)
            .map_err(|_| ValidationError::InvalidValue("display_name"))?;

        let created_at = parse_timestamp(json.created_at)?;
        let updated_at = parse_timestamp(json.updated_at)?;

        // Convert MfaConfig enum to strings
        let mfa_config = json
            .mfa_config
            .unwrap_or_default()
            .into_iter()
            .map(|mfa| match mfa {
                crate::generated_admin::models::list_tenants_200_response_tenants_inner::MfaConfig::Phone => {
                    "phone".to_string()
                }
                crate::generated_admin::models::list_tenants_200_response_tenants_inner::MfaConfig::Totp => {
                    "totp".to_string()
                }
            })
            .collect();

        // Security-critical fields: require explicit values from server
        let allow_pdpka_signup = json
            .allow_pdpka_signup
            .ok_or(ValidationError::MissingField("allow_pdpka_signup"))?;
        let disable_auth = json
            .disable_auth
            .ok_or(ValidationError::MissingField("disable_auth"))?;

        Ok(Self {
            project_id,
            tenant_id,
            name,
            pistachio_id,
            display_name,
            allow_pdpka_signup,
            disable_auth,
            mfa_config,
            created_at,
            updated_at,
        })
    }
}
