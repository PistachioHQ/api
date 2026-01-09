//! Usage and quota management types for the Admin API.

use chrono::{DateTime, Utc};
use libgn::project::ProjectId;

use crate::error::{ErrorDetails, PistachioApiClientError, ValidationError};

// =============================================================================
// Usage Domain Types
// =============================================================================

/// Usage statistics for a project.
#[derive(Debug, Clone)]
pub struct ProjectUsage {
    /// The project ID.
    pub project_id: ProjectId,
    /// Monthly active users (MAU) count.
    pub monthly_active_users: i64,
    /// Daily active users (DAU) count.
    pub daily_active_users: i64,
    /// Total number of users.
    pub total_users: i64,
    /// Total number of service accounts.
    pub total_service_accounts: i64,
    /// Total number of apps.
    pub total_apps: i64,
    /// Total number of tenants.
    pub total_tenants: i64,
    /// Authentication statistics for the current billing period.
    pub auth_stats: AuthenticationStats,
    /// Start of the current billing period.
    pub billing_period_start: Option<DateTime<Utc>>,
    /// End of the current billing period.
    pub billing_period_end: Option<DateTime<Utc>>,
    /// Timestamp when the usage data was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

/// Authentication statistics for a billing period.
#[derive(Debug, Clone, Default)]
pub struct AuthenticationStats {
    /// Total number of successful sign-ins.
    pub successful_sign_ins: i64,
    /// Total number of failed sign-in attempts.
    pub failed_sign_ins: i64,
    /// Number of sign-ups (new user registrations).
    pub sign_ups: i64,
    /// Number of password resets.
    pub password_resets: i64,
    /// Number of email verifications sent.
    pub email_verifications_sent: i64,
    /// Number of SMS verifications sent.
    pub sms_verifications_sent: i64,
    /// Number of MFA challenges.
    pub mfa_challenges: i64,
    /// Number of custom token creations.
    pub custom_token_creations: i64,
    /// Number of ID token verifications.
    pub id_token_verifications: i64,
    /// Number of session cookie creations.
    pub session_cookie_creations: i64,
}

/// Quota limits and current usage for a project.
#[derive(Debug, Clone)]
pub struct ProjectQuotas {
    /// The project ID.
    pub project_id: ProjectId,
    /// User-related quotas.
    pub user_quotas: UserQuotas,
    /// Authentication-related quotas.
    pub auth_quotas: AuthQuotas,
    /// Rate limits for various operations.
    pub rate_limits: RateLimits,
}

/// User-related quotas.
#[derive(Debug, Clone)]
pub struct UserQuotas {
    /// Maximum number of users allowed.
    pub max_users: i64,
    /// Current number of users.
    pub current_users: i64,
    /// Maximum number of service accounts allowed.
    pub max_service_accounts: i64,
    /// Current number of service accounts.
    pub current_service_accounts: i64,
    /// Maximum number of tenants allowed.
    pub max_tenants: i64,
    /// Current number of tenants.
    pub current_tenants: i64,
    /// Maximum number of apps allowed.
    pub max_apps: i64,
    /// Current number of apps.
    pub current_apps: i64,
}

/// Authentication-related quotas.
#[derive(Debug, Clone)]
pub struct AuthQuotas {
    /// Maximum monthly active users (MAU) allowed in billing period.
    pub max_monthly_active_users: i64,
    /// Current monthly active users.
    pub current_monthly_active_users: i64,
    /// Maximum email verifications per day.
    pub max_email_verifications_per_day: i64,
    /// Current email verifications today.
    pub current_email_verifications_today: i64,
    /// Maximum SMS verifications per day.
    pub max_sms_verifications_per_day: i64,
    /// Current SMS verifications today.
    pub current_sms_verifications_today: i64,
}

/// Rate limits for various operations.
#[derive(Debug, Clone)]
pub struct RateLimits {
    /// Maximum sign-ups per hour.
    pub max_signups_per_hour: i64,
    /// Maximum sign-in attempts per minute per IP.
    pub max_signin_attempts_per_minute_per_ip: i64,
    /// Maximum password reset requests per hour.
    pub max_password_resets_per_hour: i64,
    /// Maximum API requests per second.
    pub max_api_requests_per_second: i64,
}

// =============================================================================
// GetProjectUsage
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetProjectUsageError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to get usage statistics for a project.
#[derive(Debug, Clone)]
pub struct GetProjectUsageRequest {
    /// The project ID to get usage for.
    pub project_id: ProjectId,
    /// Optional start date for usage data range.
    pub start_date: Option<DateTime<Utc>>,
    /// Optional end date for usage data range.
    pub end_date: Option<DateTime<Utc>>,
}

impl GetProjectUsageRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            start_date: None,
            end_date: None,
        }
    }

    /// Sets the date range for usage data.
    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }
}

/// Response from getting project usage.
#[derive(Debug, Clone)]
pub struct GetProjectUsageResponse {
    /// The usage statistics.
    pub usage: ProjectUsage,
}

// =============================================================================
// GetProjectQuotas
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum GetProjectQuotasError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to get quotas for a project.
#[derive(Debug, Clone)]
pub struct GetProjectQuotasRequest {
    /// The project ID to get quotas for.
    pub project_id: ProjectId,
}

impl GetProjectQuotasRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

/// Response from getting project quotas.
#[derive(Debug, Clone)]
pub struct GetProjectQuotasResponse {
    /// The quota limits and current usage.
    pub quotas: ProjectQuotas,
}

// =============================================================================
// UpdateProjectQuotas
// =============================================================================

#[derive(Debug, thiserror::Error)]
pub enum UpdateProjectQuotasError {
    #[error("Bad request: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    BadRequest(ErrorDetails),
    #[error("Project not found: {}", .0.message.as_deref().unwrap_or(&.0.title))]
    NotFound(ErrorDetails),
    #[error("Quota limit exceeded: {0}")]
    QuotaExceeded(String),
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client error: {0}")]
    PistachioApiClientError(#[from] PistachioApiClientError),
    #[error("Response validation error: {0}")]
    ResponseValidationError(#[from] ValidationError),
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Request to update quotas for a project.
///
/// Note: This operation may require elevated permissions depending on the
/// requested quota increases and billing plan.
#[derive(Debug, Clone)]
pub struct UpdateProjectQuotasRequest {
    /// The project ID to update quotas for.
    pub project_id: ProjectId,
    /// New maximum number of users (requires quota increase approval).
    pub max_users: Option<i64>,
    /// New maximum number of service accounts.
    pub max_service_accounts: Option<i64>,
    /// New maximum number of tenants.
    pub max_tenants: Option<i64>,
    /// New maximum number of apps.
    pub max_apps: Option<i64>,
    /// New maximum monthly active users.
    pub max_monthly_active_users: Option<i64>,
}

impl UpdateProjectQuotasRequest {
    /// Creates a new request.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            max_users: None,
            max_service_accounts: None,
            max_tenants: None,
            max_apps: None,
            max_monthly_active_users: None,
        }
    }

    /// Sets the maximum number of users.
    pub fn with_max_users(mut self, max: i64) -> Self {
        self.max_users = Some(max);
        self
    }

    /// Sets the maximum number of service accounts.
    pub fn with_max_service_accounts(mut self, max: i64) -> Self {
        self.max_service_accounts = Some(max);
        self
    }

    /// Sets the maximum number of tenants.
    pub fn with_max_tenants(mut self, max: i64) -> Self {
        self.max_tenants = Some(max);
        self
    }

    /// Sets the maximum number of apps.
    pub fn with_max_apps(mut self, max: i64) -> Self {
        self.max_apps = Some(max);
        self
    }

    /// Sets the maximum monthly active users.
    pub fn with_max_monthly_active_users(mut self, max: i64) -> Self {
        self.max_monthly_active_users = Some(max);
        self
    }
}

/// Response from updating project quotas.
#[derive(Debug, Clone)]
pub struct UpdateProjectQuotasResponse {
    /// The updated quota limits and current usage.
    pub quotas: ProjectQuotas,
}
