//! Usage operation handlers for the gRPC client.

use chrono::{DateTime, TimeZone, Utc};
use libgn::project::ProjectId;
use pistachio_api_common::admin::usage::{
    AuthQuotas, AuthenticationStats, GetProjectQuotasError, GetProjectQuotasRequest,
    GetProjectQuotasResponse, GetProjectUsageError, GetProjectUsageRequest,
    GetProjectUsageResponse, ProjectQuotas, ProjectUsage, RateLimits, UpdateProjectQuotasError,
    UpdateProjectQuotasRequest, UpdateProjectQuotasResponse, UserQuotas,
};
use pistachio_api_common::error::ValidationError;
use prost_types::Timestamp;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;
use pistachio_api::pistachio::types::v1 as proto_types;

use crate::types::error_details_from_status;

// =============================================================================
// Proto conversions
// =============================================================================

fn timestamp_to_datetime(ts: Timestamp) -> Option<DateTime<Utc>> {
    let nanos = u32::try_from(ts.nanos).ok()?;
    Utc.timestamp_opt(ts.seconds, nanos).single()
}

fn optional_timestamp_to_datetime(ts: Option<Timestamp>) -> Option<DateTime<Utc>> {
    ts.and_then(timestamp_to_datetime)
}

fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn project_id_from_resource_name(name: &str) -> Result<ProjectId, ValidationError> {
    // Format: "projects/{project_id}/usage" or "projects/{project_id}/quotas"
    let parts: Vec<&str> = name.split('/').collect();
    if parts.len() >= 2 && parts[0] == "projects" {
        parts[1]
            .parse::<ProjectId>()
            .map_err(|_| ValidationError::InvalidValue("project_id"))
    } else {
        Err(ValidationError::InvalidValue("resource_name"))
    }
}

fn project_usage_from_proto(
    proto: proto_types::ProjectUsage,
) -> Result<ProjectUsage, ValidationError> {
    let project_id = project_id_from_resource_name(&proto.name)?;

    let period = proto.period.unwrap_or_default();
    let active_users = proto.active_users.unwrap_or_default();
    let authentications = proto.authentications.unwrap_or_default();
    let storage = proto.storage.unwrap_or_default();

    Ok(ProjectUsage {
        project_id,
        monthly_active_users: active_users.monthly,
        daily_active_users: active_users.daily,
        total_users: storage.total_users,
        total_service_accounts: storage.total_service_accounts,
        total_apps: storage.total_apps,
        total_tenants: storage.total_tenants,
        auth_stats: AuthenticationStats {
            successful_sign_ins: authentications.logins,
            failed_sign_ins: authentications.failed_attempts,
            sign_ups: authentications.signups,
            password_resets: 0,          // Not in proto
            email_verifications_sent: 0, // Not in proto
            sms_verifications_sent: 0,   // Not in proto
            mfa_challenges: 0,           // Not in proto
            custom_token_creations: 0,   // Not in proto
            id_token_verifications: 0,   // Not in proto
            session_cookie_creations: 0, // Not in proto
        },
        billing_period_start: optional_timestamp_to_datetime(period.start_time),
        billing_period_end: optional_timestamp_to_datetime(period.end_time),
        updated_at: None, // Not in proto
    })
}

fn quota_limit_value(quota: Option<proto_types::QuotaLimit>) -> (i64, i64) {
    quota.map(|q| (q.limit, q.current)).unwrap_or((0, 0))
}

fn project_quotas_from_proto(
    proto: proto_types::ProjectQuotas,
) -> Result<ProjectQuotas, ValidationError> {
    let project_id = project_id_from_resource_name(&proto.name)?;

    let (max_users, current_users) = quota_limit_value(proto.users);
    let (max_service_accounts, current_service_accounts) =
        quota_limit_value(proto.service_accounts);
    let (max_tenants, current_tenants) = quota_limit_value(proto.tenants);
    let (max_apps, current_apps) = quota_limit_value(proto.apps);
    let (max_api_requests_per_second, _) = quota_limit_value(proto.requests_per_second);
    let (max_signups_per_hour, _) = quota_limit_value(proto.requests_per_minute);
    let (max_password_resets_per_hour, _) = quota_limit_value(proto.requests_per_day);

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
            max_email_verifications_per_day: 0,          // Not in proto
            current_email_verifications_today: 0,        // Not in proto
            max_sms_verifications_per_day: 0,            // Not in proto
            current_sms_verifications_today: 0,          // Not in proto
        },
        rate_limits: RateLimits {
            max_signups_per_hour,
            max_signin_attempts_per_minute_per_ip: 0, // Not in proto
            max_password_resets_per_hour,
            max_api_requests_per_second,
        },
    })
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_get_project_usage<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetProjectUsageRequest,
) -> Result<GetProjectUsageResponse, GetProjectUsageError> {
    debug!("Creating proto request for get_project_usage");

    let request = pistachio_api::pistachio::admin::v1::GetProjectUsageRequest {
        project_id: req.project_id.to_string(),
        start_time: req.start_date.map(datetime_to_timestamp),
        end_time: req.end_date.map(datetime_to_timestamp),
    };

    let response = client
        .get_project_usage(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_project_usage response");
            match status.code() {
                Code::InvalidArgument => {
                    GetProjectUsageError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    GetProjectUsageError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    GetProjectUsageError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetProjectUsageError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetProjectUsageError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetProjectUsageError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetProjectUsageError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let usage = response
        .usage
        .ok_or(ValidationError::MissingField("usage"))?;
    let usage =
        project_usage_from_proto(usage).map_err(GetProjectUsageError::ResponseValidationError)?;

    Ok(GetProjectUsageResponse { usage })
}

pub(crate) async fn handle_get_project_quotas<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetProjectQuotasRequest,
) -> Result<GetProjectQuotasResponse, GetProjectQuotasError> {
    debug!("Creating proto request for get_project_quotas");

    let request = pistachio_api::pistachio::admin::v1::GetProjectQuotasRequest {
        project_id: req.project_id.to_string(),
    };

    let response = client
        .get_project_quotas(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_project_quotas response");
            match status.code() {
                Code::InvalidArgument => {
                    GetProjectQuotasError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    GetProjectQuotasError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    GetProjectQuotasError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetProjectQuotasError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetProjectQuotasError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetProjectQuotasError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetProjectQuotasError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let quotas = response
        .quotas
        .ok_or(ValidationError::MissingField("quotas"))?;
    let quotas = project_quotas_from_proto(quotas)
        .map_err(GetProjectQuotasError::ResponseValidationError)?;

    Ok(GetProjectQuotasResponse { quotas })
}

pub(crate) async fn handle_update_project_quotas<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateProjectQuotasRequest,
) -> Result<UpdateProjectQuotasResponse, UpdateProjectQuotasError> {
    debug!("Creating proto request for update_project_quotas");

    let request = pistachio_api::pistachio::admin::v1::UpdateProjectQuotasRequest {
        project_id: req.project_id.to_string(),
        users: req.max_users,
        service_accounts: req.max_service_accounts,
        tenants: req.max_tenants,
        apps: req.max_apps,
        api_keys_per_app: None,    // Not in common types request
        requests_per_minute: None, // Not in common types request
        requests_per_day: None,    // Not in common types request
    };

    let response = client
        .update_project_quotas(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_project_quotas response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateProjectQuotasError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    UpdateProjectQuotasError::NotFound(error_details_from_status(&status))
                }
                Code::ResourceExhausted => {
                    UpdateProjectQuotasError::QuotaExceeded(status.message().to_string())
                }
                Code::Unauthenticated => {
                    UpdateProjectQuotasError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateProjectQuotasError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    UpdateProjectQuotasError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    UpdateProjectQuotasError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateProjectQuotasError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let quotas = response
        .quotas
        .ok_or(ValidationError::MissingField("quotas"))?;
    let quotas = project_quotas_from_proto(quotas)
        .map_err(UpdateProjectQuotasError::ResponseValidationError)?;

    Ok(UpdateProjectQuotasResponse { quotas })
}
