//! Usage operation handlers for the OpenAPI client.

use libgn::project::ProjectId;
use pistachio_api_common::admin::usage::{
    AuthQuotas, AuthenticationStats, GetProjectQuotasError, GetProjectQuotasRequest,
    GetProjectQuotasResponse, GetProjectUsageError, GetProjectUsageRequest,
    GetProjectUsageResponse, ProjectQuotas, ProjectUsage, RateLimits, UpdateProjectQuotasError,
    UpdateProjectQuotasRequest, UpdateProjectQuotasResponse, UserQuotas,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::usage_api::{
    GetProjectQuotasError as GenGetProjectQuotasError,
    GetProjectUsageError as GenGetProjectUsageError,
    UpdateProjectQuotasError as GenUpdateProjectQuotasError, get_project_quotas, get_project_usage,
    update_project_quotas,
};
use crate::generated_admin::models::{
    GetProjectQuotas200Response, GetProjectQuotas200ResponseQuotas,
    GetProjectQuotas200ResponseQuotasUsers, GetProjectUsage200Response,
    UpdateProjectQuotas200Response,
};
use crate::problem_details::fallback_error_details;
use crate::types::{FromJson, convert_error_details, parse_timestamp};

// =============================================================================
// Type Conversions
// =============================================================================

fn quota_value_from_response(
    quota: Option<Box<GetProjectQuotas200ResponseQuotasUsers>>,
) -> (i64, i64) {
    quota
        .map(|q| (q.limit.unwrap_or(0), q.current.unwrap_or(0)))
        .unwrap_or((0, 0))
}

impl FromJson<GetProjectUsage200Response> for GetProjectUsageResponse {
    type Error = ValidationError;

    fn from_json(json: GetProjectUsage200Response) -> Result<Self, Self::Error> {
        let usage = json.usage.unwrap_or_default();

        let active_users = usage.active_users.unwrap_or_default();
        let authentications = usage.authentications.unwrap_or_default();
        let period = usage.period.unwrap_or_default();

        // Parse project_id from resource name if available
        let project_id = usage
            .name
            .as_ref()
            .and_then(|name| {
                // Format: "projects/{project_id}/usage"
                name.strip_prefix("projects/")
                    .and_then(|rest| rest.strip_suffix("/usage"))
            })
            .and_then(|id| id.parse::<ProjectId>().ok())
            .unwrap_or_else(ProjectId::generate);

        let auth_stats = AuthenticationStats {
            successful_sign_ins: authentications.logins.unwrap_or(0),
            failed_sign_ins: authentications.failed_attempts.unwrap_or(0),
            sign_ups: authentications.signups.unwrap_or(0),
            password_resets: 0,          // Not in API response
            email_verifications_sent: 0, // Not in API response
            sms_verifications_sent: 0,   // Not in API response
            mfa_challenges: 0,           // Not in API response
            custom_token_creations: 0,   // Not in API response
            id_token_verifications: 0,   // Not in API response
            session_cookie_creations: 0, // Not in API response
        };

        let project_usage = ProjectUsage {
            project_id,
            monthly_active_users: active_users.monthly.unwrap_or(0),
            daily_active_users: active_users.daily.unwrap_or(0),
            total_users: 0,            // Not in API response
            total_service_accounts: 0, // Not in API response
            total_apps: 0,             // Not in API response
            total_tenants: 0,          // Not in API response
            auth_stats,
            billing_period_start: period
                .start_time
                .as_ref()
                .and_then(|s| parse_timestamp(Some(s.clone())).ok()),
            billing_period_end: period
                .end_time
                .as_ref()
                .and_then(|s| parse_timestamp(Some(s.clone())).ok()),
            updated_at: None, // Not in API response
        };

        Ok(Self {
            usage: project_usage,
        })
    }
}

fn parse_project_quotas(
    quotas: Option<Box<GetProjectQuotas200ResponseQuotas>>,
) -> Result<ProjectQuotas, ValidationError> {
    let quotas = quotas.unwrap_or_default();

    // Parse project_id from resource name if available
    let project_id = quotas
        .name
        .as_ref()
        .and_then(|name| {
            // Format: "projects/{project_id}/quotas"
            name.strip_prefix("projects/")
                .and_then(|rest| rest.strip_suffix("/quotas"))
        })
        .and_then(|id| id.parse::<ProjectId>().ok())
        .unwrap_or_else(ProjectId::generate);

    let (max_users, current_users) = quota_value_from_response(quotas.users);
    let (max_service_accounts, current_service_accounts) =
        quota_value_from_response(quotas.service_accounts);
    let (max_tenants, current_tenants) = quota_value_from_response(quotas.tenants);
    let (max_apps, current_apps) = quota_value_from_response(quotas.apps);
    let (max_api_requests_per_second, _) = quota_value_from_response(quotas.requests_per_second);
    let (max_signups_per_hour, _) = quota_value_from_response(quotas.requests_per_minute);
    let (max_password_resets_per_hour, _) = quota_value_from_response(quotas.requests_per_day);

    Ok(ProjectQuotas {
        project_id,
        user_quotas: UserQuotas {
            max_users,
            current_users,
            max_service_accounts,
            current_service_accounts,
            max_tenants,
            current_tenants,
            max_apps,
            current_apps,
        },
        auth_quotas: AuthQuotas {
            max_monthly_active_users: max_users,         // Approximation
            current_monthly_active_users: current_users, // Approximation
            max_email_verifications_per_day: 0,          // Not in API response
            current_email_verifications_today: 0,        // Not in API response
            max_sms_verifications_per_day: 0,            // Not in API response
            current_sms_verifications_today: 0,          // Not in API response
        },
        rate_limits: RateLimits {
            max_signups_per_hour,
            max_signin_attempts_per_minute_per_ip: 0, // Not in API response
            max_password_resets_per_hour,
            max_api_requests_per_second,
        },
    })
}

impl FromJson<GetProjectQuotas200Response> for GetProjectQuotasResponse {
    type Error = ValidationError;

    fn from_json(json: GetProjectQuotas200Response) -> Result<Self, Self::Error> {
        let quotas = parse_project_quotas(json.quotas)?;
        Ok(Self { quotas })
    }
}

impl FromJson<UpdateProjectQuotas200Response> for UpdateProjectQuotasResponse {
    type Error = ValidationError;

    fn from_json(json: UpdateProjectQuotas200Response) -> Result<Self, Self::Error> {
        let quotas = parse_project_quotas(json.quotas)?;
        Ok(Self { quotas })
    }
}

// =============================================================================
// Error Conversions
// =============================================================================

impl From<GenGetProjectUsageError> for GetProjectUsageError {
    fn from(error: GenGetProjectUsageError) -> Self {
        match error {
            GenGetProjectUsageError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenGetProjectUsageError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectUsageError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectUsageError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenGetProjectUsageError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectUsageError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectUsageError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenGetProjectQuotasError> for GetProjectQuotasError {
    fn from(error: GenGetProjectQuotasError) -> Self {
        match error {
            GenGetProjectQuotasError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenGetProjectQuotasError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectQuotasError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectQuotasError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenGetProjectQuotasError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectQuotasError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetProjectQuotasError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenUpdateProjectQuotasError> for UpdateProjectQuotasError {
    fn from(error: GenUpdateProjectQuotasError) -> Self {
        match error {
            GenUpdateProjectQuotasError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenUpdateProjectQuotasError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateProjectQuotasError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateProjectQuotasError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenUpdateProjectQuotasError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateProjectQuotasError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateProjectQuotasError::UnknownValue(v) => Self::Unknown(v.to_string()),
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

pub(crate) async fn handle_get_project_usage(
    config: &Configuration,
    req: GetProjectUsageRequest,
) -> Result<GetProjectUsageResponse, GetProjectUsageError> {
    debug!("Creating OpenAPI request for get_project_usage");

    let project_id = req.project_id.to_string();

    // Convert optional date params to ISO strings
    let start_time = req.start_date.map(|dt| dt.to_rfc3339());
    let end_time = req.end_date.map(|dt| dt.to_rfc3339());

    let response = get_project_usage(config, &project_id, start_time, end_time)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_project_usage response");
            handle_api_error(
                e,
                GetProjectUsageError::from,
                |status, content| match status {
                    400 => GetProjectUsageError::BadRequest(fallback_error_details(content)),
                    401 => GetProjectUsageError::Unauthenticated(content),
                    403 => GetProjectUsageError::PermissionDenied(content),
                    404 => GetProjectUsageError::NotFound(fallback_error_details(content)),
                    500..=599 => GetProjectUsageError::ServiceError(content),
                    _ => GetProjectUsageError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                GetProjectUsageError::ServiceUnavailable,
                || GetProjectUsageError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    GetProjectUsageResponse::from_json(response)
        .map_err(GetProjectUsageError::ResponseValidationError)
}

pub(crate) async fn handle_get_project_quotas(
    config: &Configuration,
    req: GetProjectQuotasRequest,
) -> Result<GetProjectQuotasResponse, GetProjectQuotasError> {
    debug!("Creating OpenAPI request for get_project_quotas");

    let project_id = req.project_id.to_string();

    let response = get_project_quotas(config, &project_id).await.map_err(|e| {
        error!(?e, "Error in get_project_quotas response");
        handle_api_error(
            e,
            GetProjectQuotasError::from,
            |status, content| match status {
                400 => GetProjectQuotasError::BadRequest(fallback_error_details(content)),
                401 => GetProjectQuotasError::Unauthenticated(content),
                403 => GetProjectQuotasError::PermissionDenied(content),
                404 => GetProjectQuotasError::NotFound(fallback_error_details(content)),
                500..=599 => GetProjectQuotasError::ServiceError(content),
                _ => GetProjectQuotasError::Unknown(format!("HTTP {}: {}", status, content)),
            },
            GetProjectQuotasError::ServiceUnavailable,
            || GetProjectQuotasError::ServiceError("Unknown error occurred".into()),
        )
    })?;

    GetProjectQuotasResponse::from_json(response)
        .map_err(GetProjectQuotasError::ResponseValidationError)
}

pub(crate) async fn handle_update_project_quotas(
    config: &Configuration,
    req: UpdateProjectQuotasRequest,
) -> Result<UpdateProjectQuotasResponse, UpdateProjectQuotasError> {
    debug!("Creating OpenAPI request for update_project_quotas");

    let project_id = req.project_id.to_string();

    // Build the generated request
    let mut gen_request = crate::generated_admin::models::UpdateProjectQuotasRequest::new();

    if let Some(max_users) = req.max_users {
        gen_request.users = Some(max_users);
    }
    if let Some(max_service_accounts) = req.max_service_accounts {
        gen_request.service_accounts = Some(max_service_accounts);
    }
    if let Some(max_tenants) = req.max_tenants {
        gen_request.tenants = Some(max_tenants);
    }
    if let Some(max_apps) = req.max_apps {
        gen_request.apps = Some(max_apps);
    }
    // Note: max_monthly_active_users doesn't have a direct mapping in the API request

    let response = update_project_quotas(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_project_quotas response");
            handle_api_error(
                e,
                UpdateProjectQuotasError::from,
                |status, content| match status {
                    400 => UpdateProjectQuotasError::BadRequest(fallback_error_details(content)),
                    401 => UpdateProjectQuotasError::Unauthenticated(content),
                    403 => UpdateProjectQuotasError::PermissionDenied(content),
                    404 => UpdateProjectQuotasError::NotFound(fallback_error_details(content)),
                    500..=599 => UpdateProjectQuotasError::ServiceError(content),
                    _ => UpdateProjectQuotasError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                UpdateProjectQuotasError::ServiceUnavailable,
                || UpdateProjectQuotasError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    UpdateProjectQuotasResponse::from_json(response)
        .map_err(UpdateProjectQuotasError::ResponseValidationError)
}
