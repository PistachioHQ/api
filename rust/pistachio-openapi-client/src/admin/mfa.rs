//! MFA operation handlers for the OpenAPI client.

use pistachio_api_common::admin::mfa::{
    DeleteProjectUserMfaFactorError, DeleteProjectUserMfaFactorRequest,
    DeleteProjectUserMfaFactorResponse, DeleteTenantUserMfaFactorError,
    DeleteTenantUserMfaFactorRequest, DeleteTenantUserMfaFactorResponse,
    ListProjectUserMfaFactorsError, ListProjectUserMfaFactorsRequest,
    ListProjectUserMfaFactorsResponse, ListTenantUserMfaFactorsError,
    ListTenantUserMfaFactorsRequest, ListTenantUserMfaFactorsResponse, MfaFactor, MfaFactorType,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::mfa_api::{
    DeleteProjectUserMfaFactorError as GenDeleteProjectError,
    DeleteTenantUserMfaFactorError as GenDeleteTenantError,
    ListProjectUserMfaFactorsError as GenListProjectError,
    ListTenantUserMfaFactorsError as GenListTenantError, delete_project_user_mfa_factor,
    delete_tenant_user_mfa_factor, list_project_user_mfa_factors, list_tenant_user_mfa_factors,
};
use crate::generated_admin::models::{
    ListProjectUserMfaFactors200Response, ListProjectUserMfaFactors200ResponseFactorsInner,
};
use crate::problem_details::fallback_error_details;
use crate::types::{FromJson, convert_error_details, parse_timestamp};

// =============================================================================
// Type Conversions
// =============================================================================

impl FromJson<ListProjectUserMfaFactors200ResponseFactorsInner> for MfaFactor {
    type Error = ValidationError;

    fn from_json(
        json: ListProjectUserMfaFactors200ResponseFactorsInner,
    ) -> Result<Self, Self::Error> {
        use crate::generated_admin::models::list_project_user_mfa_factors_200_response_factors_inner::FactorType;

        let factor_type = match json.factor_type {
            FactorType::Totp => MfaFactorType::Totp,
            FactorType::Sms => MfaFactorType::Sms,
            FactorType::Email => MfaFactorType::Email,
        };

        Ok(Self {
            factor_id: json.factor_id,
            factor_type,
            display_name: json.display_name.flatten(),
            phone_number: json.phone_number.flatten(),
            email: json.email.flatten(),
            verified: json.verified.unwrap_or(false),
            created_at: parse_timestamp(Some(json.created_at)).ok(),
            last_used_at: json
                .last_used_at
                .flatten()
                .and_then(|s| parse_timestamp(Some(s)).ok()),
        })
    }
}

impl FromJson<ListProjectUserMfaFactors200Response> for ListProjectUserMfaFactorsResponse {
    type Error = ValidationError;

    fn from_json(json: ListProjectUserMfaFactors200Response) -> Result<Self, Self::Error> {
        let factors = json
            .factors
            .unwrap_or_default()
            .into_iter()
            .map(MfaFactor::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { factors })
    }
}

impl FromJson<ListProjectUserMfaFactors200Response> for ListTenantUserMfaFactorsResponse {
    type Error = ValidationError;

    fn from_json(json: ListProjectUserMfaFactors200Response) -> Result<Self, Self::Error> {
        let factors = json
            .factors
            .unwrap_or_default()
            .into_iter()
            .map(MfaFactor::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { factors })
    }
}

// =============================================================================
// Error Conversions
// =============================================================================

impl From<GenListProjectError> for ListProjectUserMfaFactorsError {
    fn from(error: GenListProjectError) -> Self {
        match error {
            GenListProjectError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenListProjectError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListProjectError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListProjectError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenListProjectError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListProjectError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListProjectError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenDeleteProjectError> for DeleteProjectUserMfaFactorError {
    fn from(error: GenDeleteProjectError) -> Self {
        match error {
            GenDeleteProjectError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenDeleteProjectError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteProjectError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteProjectError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenDeleteProjectError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteProjectError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteProjectError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenListTenantError> for ListTenantUserMfaFactorsError {
    fn from(error: GenListTenantError) -> Self {
        match error {
            GenListTenantError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenListTenantError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListTenantError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListTenantError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenListTenantError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListTenantError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListTenantError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenDeleteTenantError> for DeleteTenantUserMfaFactorError {
    fn from(error: GenDeleteTenantError) -> Self {
        match error {
            GenDeleteTenantError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenDeleteTenantError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteTenantError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteTenantError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenDeleteTenantError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteTenantError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteTenantError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

// =============================================================================
// Helper for error handling
// =============================================================================

fn handle_api_error<E, T>(
    e: crate::generated_admin::apis::Error<E>,
    convert_entity: impl Fn(E) -> T,
    fallback_fn: impl Fn(u16, String) -> T,
    reqwest_error_fn: impl Fn(String) -> T,
    default_error_fn: impl Fn() -> T,
) -> T
where
    T: std::fmt::Debug,
{
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            // Try entity parsing if available
            if let Some(entity) = resp.entity {
                return convert_entity(entity);
            }

            // Last resort: status code mapping with raw content
            fallback_fn(status, resp.content)
        }
        crate::generated_admin::apis::Error::Reqwest(e) => reqwest_error_fn(e.to_string()),
        _ => default_error_fn(),
    }
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_list_project_user_mfa_factors(
    config: &Configuration,
    req: ListProjectUserMfaFactorsRequest,
) -> Result<ListProjectUserMfaFactorsResponse, ListProjectUserMfaFactorsError> {
    debug!("Creating OpenAPI request for list_project_user_mfa_factors");

    let project_id = req.project_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    let response = list_project_user_mfa_factors(config, &project_id, &pistachio_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in list_project_user_mfa_factors response");
            handle_api_error(
                e,
                ListProjectUserMfaFactorsError::from,
                |status, content| match status {
                    400 => {
                        ListProjectUserMfaFactorsError::BadRequest(fallback_error_details(content))
                    }
                    401 => ListProjectUserMfaFactorsError::Unauthenticated(content),
                    403 => ListProjectUserMfaFactorsError::PermissionDenied(content),
                    404 => {
                        ListProjectUserMfaFactorsError::NotFound(fallback_error_details(content))
                    }
                    500..=599 => ListProjectUserMfaFactorsError::ServiceError(content),
                    _ => ListProjectUserMfaFactorsError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                ListProjectUserMfaFactorsError::ServiceUnavailable,
                || ListProjectUserMfaFactorsError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    ListProjectUserMfaFactorsResponse::from_json(response)
        .map_err(ListProjectUserMfaFactorsError::ResponseValidationError)
}

pub(crate) async fn handle_delete_project_user_mfa_factor(
    config: &Configuration,
    req: DeleteProjectUserMfaFactorRequest,
) -> Result<DeleteProjectUserMfaFactorResponse, DeleteProjectUserMfaFactorError> {
    debug!("Creating OpenAPI request for delete_project_user_mfa_factor");

    let project_id = req.project_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    delete_project_user_mfa_factor(config, &project_id, &pistachio_id, &req.factor_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_project_user_mfa_factor response");
            handle_api_error(
                e,
                DeleteProjectUserMfaFactorError::from,
                |status, content| match status {
                    400 => {
                        DeleteProjectUserMfaFactorError::BadRequest(fallback_error_details(content))
                    }
                    401 => DeleteProjectUserMfaFactorError::Unauthenticated(content),
                    403 => DeleteProjectUserMfaFactorError::PermissionDenied(content),
                    404 => {
                        DeleteProjectUserMfaFactorError::NotFound(fallback_error_details(content))
                    }
                    500..=599 => DeleteProjectUserMfaFactorError::ServiceError(content),
                    _ => DeleteProjectUserMfaFactorError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                DeleteProjectUserMfaFactorError::ServiceUnavailable,
                || DeleteProjectUserMfaFactorError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    Ok(DeleteProjectUserMfaFactorResponse {})
}

pub(crate) async fn handle_list_tenant_user_mfa_factors(
    config: &Configuration,
    req: ListTenantUserMfaFactorsRequest,
) -> Result<ListTenantUserMfaFactorsResponse, ListTenantUserMfaFactorsError> {
    debug!("Creating OpenAPI request for list_tenant_user_mfa_factors");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    let response = list_tenant_user_mfa_factors(config, &project_id, &tenant_id, &pistachio_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in list_tenant_user_mfa_factors response");
            handle_api_error(
                e,
                ListTenantUserMfaFactorsError::from,
                |status, content| match status {
                    400 => {
                        ListTenantUserMfaFactorsError::BadRequest(fallback_error_details(content))
                    }
                    401 => ListTenantUserMfaFactorsError::Unauthenticated(content),
                    403 => ListTenantUserMfaFactorsError::PermissionDenied(content),
                    404 => ListTenantUserMfaFactorsError::NotFound(fallback_error_details(content)),
                    500..=599 => ListTenantUserMfaFactorsError::ServiceError(content),
                    _ => ListTenantUserMfaFactorsError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                ListTenantUserMfaFactorsError::ServiceUnavailable,
                || ListTenantUserMfaFactorsError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    ListTenantUserMfaFactorsResponse::from_json(response)
        .map_err(ListTenantUserMfaFactorsError::ResponseValidationError)
}

pub(crate) async fn handle_delete_tenant_user_mfa_factor(
    config: &Configuration,
    req: DeleteTenantUserMfaFactorRequest,
) -> Result<DeleteTenantUserMfaFactorResponse, DeleteTenantUserMfaFactorError> {
    debug!("Creating OpenAPI request for delete_tenant_user_mfa_factor");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    delete_tenant_user_mfa_factor(
        config,
        &project_id,
        &tenant_id,
        &pistachio_id,
        &req.factor_id,
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in delete_tenant_user_mfa_factor response");
        handle_api_error(
            e,
            DeleteTenantUserMfaFactorError::from,
            |status, content| match status {
                400 => DeleteTenantUserMfaFactorError::BadRequest(fallback_error_details(content)),
                401 => DeleteTenantUserMfaFactorError::Unauthenticated(content),
                403 => DeleteTenantUserMfaFactorError::PermissionDenied(content),
                404 => DeleteTenantUserMfaFactorError::NotFound(fallback_error_details(content)),
                500..=599 => DeleteTenantUserMfaFactorError::ServiceError(content),
                _ => {
                    DeleteTenantUserMfaFactorError::Unknown(format!("HTTP {}: {}", status, content))
                }
            },
            DeleteTenantUserMfaFactorError::ServiceUnavailable,
            || DeleteTenantUserMfaFactorError::ServiceError("Unknown error occurred".into()),
        )
    })?;

    Ok(DeleteTenantUserMfaFactorResponse {})
}
