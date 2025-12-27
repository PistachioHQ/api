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
    CreateTenant200Response, CreateTenantRequest as GenRequest, ListTenants200ResponseTenantsInner,
};
use crate::types::{FromJson, parse_timestamp};

impl From<GenError> for CreateTenantError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status404(_) => Self::NotFound,
            GenError::Status409(_) => Self::AlreadyExists,
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

    let gen_request = GenRequest {
        tenant_id: req.tenant_id.map(|id| id.to_string()),
        display_name: req.display_name.to_string(),
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
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        CreateTenantError::Unknown(format!(
                            "HTTP {}: {}",
                            resp.status, resp.content
                        ))
                    })
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

impl FromJson<CreateTenant200Response> for CreateTenantResponse {
    type Error = ValidationError;

    fn from_json(json: CreateTenant200Response) -> Result<Self, Self::Error> {
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

        Ok(Self {
            project_id,
            tenant_id,
            name,
            pistachio_id,
            display_name,
            allow_pdpka_signup: json.allow_pdpka_signup.unwrap_or(true),
            disable_auth: json.disable_auth.unwrap_or(false),
            mfa_config,
            created_at,
            updated_at,
        })
    }
}
