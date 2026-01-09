//! Token operation handlers for the OpenAPI client.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use libgn::pistachio_id::UserId;
use libgn::project::ProjectId;
use libgn::tenant::TenantId;
use pistachio_api_common::admin::token::{
    CreateCustomTokenError, CreateCustomTokenRequest, CreateCustomTokenResponse,
    CreateSessionCookieError, CreateSessionCookieRequest, CreateSessionCookieResponse,
    DecodedIdToken, RevokeRefreshTokensError, RevokeRefreshTokensRequest,
    RevokeRefreshTokensResponse, VerifyIdTokenError, VerifyIdTokenRequest, VerifyIdTokenResponse,
    VerifySessionCookieError, VerifySessionCookieRequest, VerifySessionCookieResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::tokens_api::{
    CreateCustomTokenError as GenCreateCustomTokenError,
    CreateSessionCookieError as GenCreateSessionCookieError,
    VerifyIdTokenError as GenVerifyIdTokenError,
    VerifySessionCookieError as GenVerifySessionCookieError, create_custom_token,
    create_session_cookie, verify_id_token, verify_session_cookie,
};
use crate::generated_admin::apis::users_api::{
    RevokeProjectUserTokensError as GenRevokeProjectUserTokensError,
    RevokeTenantUserTokensError as GenRevokeTenantUserTokensError, revoke_project_user_tokens,
    revoke_tenant_user_tokens,
};
use crate::generated_admin::models::{
    CreateCustomToken200Response, CreateSessionCookie200Response, VerifyIdToken200Response,
    VerifySessionCookie200Response,
};
use crate::problem_details::fallback_error_details;
use crate::types::{FromJson, convert_error_details, parse_timestamp};

// =============================================================================
// Type Conversions
// =============================================================================

impl FromJson<CreateCustomToken200Response> for CreateCustomTokenResponse {
    type Error = ValidationError;

    fn from_json(json: CreateCustomToken200Response) -> Result<Self, Self::Error> {
        Ok(Self {
            custom_token: json.custom_token.unwrap_or_default(),
            expires_at: None, // Not provided by API
        })
    }
}

impl FromJson<CreateSessionCookie200Response> for CreateSessionCookieResponse {
    type Error = ValidationError;

    fn from_json(json: CreateSessionCookie200Response) -> Result<Self, Self::Error> {
        Ok(Self {
            session_cookie: json.session_cookie.unwrap_or_default(),
            expires_at: json.expires_at.and_then(|s| parse_timestamp(Some(s)).ok()),
        })
    }
}

fn parse_decoded_id_token(
    pistachio_id: Option<String>,
    email: Option<Option<String>>,
    email_verified: Option<bool>,
    claims: Option<HashMap<String, serde_json::Value>>,
) -> Result<DecodedIdToken, ValidationError> {
    let pistachio_id_str = pistachio_id.ok_or(ValidationError::MissingField("pistachioId"))?;
    let pistachio_id = UserId::parse(&pistachio_id_str)
        .map_err(|_| ValidationError::InvalidValue("pistachioId"))?;

    let claims = claims.unwrap_or_default();

    // Extract standard claims from the claims map
    let issuer = claims
        .get("iss")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let subject = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let audience = claims
        .get("aud")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let now = Utc::now();
    let issued_at = claims
        .get("iat")
        .and_then(|v| v.as_i64())
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .unwrap_or(now);
    let expires_at = claims
        .get("exp")
        .and_then(|v| v.as_i64())
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .unwrap_or(now);
    let auth_time = claims
        .get("auth_time")
        .and_then(|v| v.as_i64())
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .unwrap_or(issued_at);

    // Extract project_id from claims if present
    let project_id = claims
        .get("project_id")
        .or_else(|| claims.get("aud"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<ProjectId>().ok())
        .unwrap_or_else(ProjectId::generate);

    // Extract tenant_id from claims if present
    let tenant_id = claims
        .get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<TenantId>().ok());

    // Extract custom claims (excluding standard JWT claims)
    let standard_claims = [
        "iss",
        "sub",
        "aud",
        "exp",
        "iat",
        "nbf",
        "auth_time",
        "project_id",
        "tenant_id",
        "email",
        "email_verified",
        "phone_number",
        "name",
        "picture",
        "sign_in_provider",
    ];
    let custom_claims: HashMap<String, serde_json::Value> = claims
        .into_iter()
        .filter(|(k, _)| !standard_claims.contains(&k.as_str()))
        .collect();

    Ok(DecodedIdToken {
        pistachio_id,
        project_id,
        tenant_id,
        issuer,
        subject,
        audience,
        issued_at,
        expires_at,
        auth_time,
        email: email.flatten(),
        email_verified,
        phone_number: None,     // Not directly in response
        name: None,             // Not directly in response
        picture: None,          // Not directly in response
        sign_in_provider: None, // Not directly in response
        custom_claims,
    })
}

impl FromJson<VerifyIdToken200Response> for VerifyIdTokenResponse {
    type Error = ValidationError;

    fn from_json(json: VerifyIdToken200Response) -> Result<Self, Self::Error> {
        let decoded_token = parse_decoded_id_token(
            json.pistachio_id,
            json.email,
            json.email_verified,
            json.claims,
        )?;

        Ok(Self { decoded_token })
    }
}

impl FromJson<VerifySessionCookie200Response> for VerifySessionCookieResponse {
    type Error = ValidationError;

    fn from_json(json: VerifySessionCookie200Response) -> Result<Self, Self::Error> {
        let decoded_token = parse_decoded_id_token(
            json.pistachio_id,
            json.email,
            json.email_verified,
            json.claims,
        )?;

        Ok(Self { decoded_token })
    }
}

// =============================================================================
// Error Conversions
// =============================================================================

impl From<GenCreateCustomTokenError> for CreateCustomTokenError {
    fn from(error: GenCreateCustomTokenError) -> Self {
        match error {
            GenCreateCustomTokenError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenCreateCustomTokenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateCustomTokenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateCustomTokenError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenCreateCustomTokenError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateCustomTokenError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateCustomTokenError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenVerifyIdTokenError> for VerifyIdTokenError {
    fn from(error: GenVerifyIdTokenError) -> Self {
        match error {
            GenVerifyIdTokenError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenVerifyIdTokenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifyIdTokenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifyIdTokenError::Status404(e) => {
                // 404 for verify could indicate an invalid token
                Self::InvalidToken(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifyIdTokenError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifyIdTokenError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifyIdTokenError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenCreateSessionCookieError> for CreateSessionCookieError {
    fn from(error: GenCreateSessionCookieError) -> Self {
        match error {
            GenCreateSessionCookieError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenCreateSessionCookieError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateSessionCookieError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateSessionCookieError::Status404(e) => {
                Self::InvalidIdToken(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateSessionCookieError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateSessionCookieError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateSessionCookieError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenVerifySessionCookieError> for VerifySessionCookieError {
    fn from(error: GenVerifySessionCookieError) -> Self {
        match error {
            GenVerifySessionCookieError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenVerifySessionCookieError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifySessionCookieError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifySessionCookieError::Status404(e) => {
                Self::InvalidSessionCookie(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifySessionCookieError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifySessionCookieError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenVerifySessionCookieError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenRevokeProjectUserTokensError> for RevokeRefreshTokensError {
    fn from(error: GenRevokeProjectUserTokensError) -> Self {
        match error {
            GenRevokeProjectUserTokensError::Status400(e) => {
                Self::BadRequest(convert_error_details(e))
            }
            GenRevokeProjectUserTokensError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeProjectUserTokensError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeProjectUserTokensError::Status404(e) => {
                Self::NotFound(convert_error_details(e))
            }
            GenRevokeProjectUserTokensError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeProjectUserTokensError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeProjectUserTokensError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenRevokeTenantUserTokensError> for RevokeRefreshTokensError {
    fn from(error: GenRevokeTenantUserTokensError) -> Self {
        match error {
            GenRevokeTenantUserTokensError::Status400(e) => {
                Self::BadRequest(convert_error_details(e))
            }
            GenRevokeTenantUserTokensError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeTenantUserTokensError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeTenantUserTokensError::Status404(e) => {
                Self::NotFound(convert_error_details(e))
            }
            GenRevokeTenantUserTokensError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeTenantUserTokensError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRevokeTenantUserTokensError::UnknownValue(v) => Self::Unknown(v.to_string()),
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

pub(crate) async fn handle_create_custom_token(
    config: &Configuration,
    req: CreateCustomTokenRequest,
) -> Result<CreateCustomTokenResponse, CreateCustomTokenError> {
    debug!("Creating OpenAPI request for create_custom_token");

    let project_id = req.project_id.to_string();

    // Build the generated request
    let mut gen_request =
        crate::generated_admin::models::CreateCustomTokenRequest::new(req.pistachio_id.to_string());

    if let Some(tenant_id) = req.tenant_id {
        gen_request.tenant_id = Some(Some(tenant_id.to_string()));
    }

    // Convert serde_json::Value to HashMap if it's an object
    if let Some(serde_json::Value::Object(map)) = req.custom_claims {
        let additional_claims: HashMap<String, serde_json::Value> = map.into_iter().collect();
        gen_request.additional_claims = Some(additional_claims);
    }

    if let Some(expires_in) = req.expires_in_seconds {
        gen_request.expires_in = Some(expires_in);
    }

    let response = create_custom_token(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in create_custom_token response");
            handle_api_error(
                e,
                CreateCustomTokenError::from,
                |status, content| match status {
                    400 => CreateCustomTokenError::BadRequest(fallback_error_details(content)),
                    401 => CreateCustomTokenError::Unauthenticated(content),
                    403 => CreateCustomTokenError::PermissionDenied(content),
                    404 => CreateCustomTokenError::NotFound(fallback_error_details(content)),
                    500..=599 => CreateCustomTokenError::ServiceError(content),
                    _ => CreateCustomTokenError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                CreateCustomTokenError::ServiceUnavailable,
                || CreateCustomTokenError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    CreateCustomTokenResponse::from_json(response)
        .map_err(CreateCustomTokenError::ResponseValidationError)
}

pub(crate) async fn handle_verify_id_token(
    config: &Configuration,
    req: VerifyIdTokenRequest,
) -> Result<VerifyIdTokenResponse, VerifyIdTokenError> {
    debug!("Creating OpenAPI request for verify_id_token");

    let project_id = req.project_id.to_string();

    // Build the generated request
    // Note: The domain has `check_disabled` but the API has `check_revoked`
    // We pass None for check_revoked as check_disabled has different semantics
    let gen_request = crate::generated_admin::models::VerifyIdTokenRequest::new(req.id_token);

    let response = verify_id_token(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in verify_id_token response");
            handle_api_error(
                e,
                VerifyIdTokenError::from,
                |status, content| match status {
                    400 => VerifyIdTokenError::BadRequest(fallback_error_details(content)),
                    401 => VerifyIdTokenError::Unauthenticated(content),
                    403 => VerifyIdTokenError::PermissionDenied(content),
                    404 => VerifyIdTokenError::InvalidToken(content),
                    500..=599 => VerifyIdTokenError::ServiceError(content),
                    _ => VerifyIdTokenError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                VerifyIdTokenError::ServiceUnavailable,
                || VerifyIdTokenError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    VerifyIdTokenResponse::from_json(response).map_err(VerifyIdTokenError::ResponseValidationError)
}

pub(crate) async fn handle_create_session_cookie(
    config: &Configuration,
    req: CreateSessionCookieRequest,
) -> Result<CreateSessionCookieResponse, CreateSessionCookieError> {
    debug!("Creating OpenAPI request for create_session_cookie");

    let project_id = req.project_id.to_string();

    // Build the generated request
    let mut gen_request =
        crate::generated_admin::models::CreateSessionCookieRequest::new(req.id_token);

    if let Some(expires_in) = req.expires_in_seconds {
        gen_request.expires_in = Some(expires_in);
    }

    let response = create_session_cookie(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in create_session_cookie response");
            handle_api_error(
                e,
                CreateSessionCookieError::from,
                |status, content| match status {
                    400 => CreateSessionCookieError::BadRequest(fallback_error_details(content)),
                    401 => CreateSessionCookieError::Unauthenticated(content),
                    403 => CreateSessionCookieError::PermissionDenied(content),
                    404 => CreateSessionCookieError::InvalidIdToken(content),
                    500..=599 => CreateSessionCookieError::ServiceError(content),
                    _ => CreateSessionCookieError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                CreateSessionCookieError::ServiceUnavailable,
                || CreateSessionCookieError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    CreateSessionCookieResponse::from_json(response)
        .map_err(CreateSessionCookieError::ResponseValidationError)
}

pub(crate) async fn handle_verify_session_cookie(
    config: &Configuration,
    req: VerifySessionCookieRequest,
) -> Result<VerifySessionCookieResponse, VerifySessionCookieError> {
    debug!("Creating OpenAPI request for verify_session_cookie");

    let project_id = req.project_id.to_string();

    // Build the generated request
    let mut gen_request =
        crate::generated_admin::models::VerifySessionCookieRequest::new(req.session_cookie);

    if req.check_revoked {
        gen_request.check_revoked = Some(true);
    }

    let response = verify_session_cookie(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in verify_session_cookie response");
            handle_api_error(
                e,
                VerifySessionCookieError::from,
                |status, content| match status {
                    400 => VerifySessionCookieError::BadRequest(fallback_error_details(content)),
                    401 => VerifySessionCookieError::Unauthenticated(content),
                    403 => VerifySessionCookieError::PermissionDenied(content),
                    404 => VerifySessionCookieError::InvalidSessionCookie(content),
                    500..=599 => VerifySessionCookieError::ServiceError(content),
                    _ => VerifySessionCookieError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                VerifySessionCookieError::ServiceUnavailable,
                || VerifySessionCookieError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    VerifySessionCookieResponse::from_json(response)
        .map_err(VerifySessionCookieError::ResponseValidationError)
}

pub(crate) async fn handle_revoke_refresh_tokens(
    config: &Configuration,
    req: RevokeRefreshTokensRequest,
) -> Result<RevokeRefreshTokensResponse, RevokeRefreshTokensError> {
    debug!("Creating OpenAPI request for revoke_refresh_tokens");

    let project_id = req.project_id.to_string();
    let pistachio_id = req.pistachio_id.to_string();

    // Use tenant-scoped or project-scoped endpoint based on tenant_id
    if let Some(tenant_id) = req.tenant_id {
        let tenant_id = tenant_id.to_string();
        revoke_tenant_user_tokens(config, &project_id, &tenant_id, &pistachio_id)
            .await
            .map_err(|e| {
                error!(?e, "Error in revoke_tenant_user_tokens response");
                handle_api_error(
                    e,
                    RevokeRefreshTokensError::from,
                    |status, content| match status {
                        400 => {
                            RevokeRefreshTokensError::BadRequest(fallback_error_details(content))
                        }
                        401 => RevokeRefreshTokensError::Unauthenticated(content),
                        403 => RevokeRefreshTokensError::PermissionDenied(content),
                        404 => RevokeRefreshTokensError::NotFound(fallback_error_details(content)),
                        500..=599 => RevokeRefreshTokensError::ServiceError(content),
                        _ => RevokeRefreshTokensError::Unknown(format!(
                            "HTTP {}: {}",
                            status, content
                        )),
                    },
                    RevokeRefreshTokensError::ServiceUnavailable,
                    || RevokeRefreshTokensError::ServiceError("Unknown error occurred".into()),
                )
            })?;
    } else {
        revoke_project_user_tokens(config, &project_id, &pistachio_id)
            .await
            .map_err(|e| {
                error!(?e, "Error in revoke_project_user_tokens response");
                handle_api_error(
                    e,
                    RevokeRefreshTokensError::from,
                    |status, content| match status {
                        400 => {
                            RevokeRefreshTokensError::BadRequest(fallback_error_details(content))
                        }
                        401 => RevokeRefreshTokensError::Unauthenticated(content),
                        403 => RevokeRefreshTokensError::PermissionDenied(content),
                        404 => RevokeRefreshTokensError::NotFound(fallback_error_details(content)),
                        500..=599 => RevokeRefreshTokensError::ServiceError(content),
                        _ => RevokeRefreshTokensError::Unknown(format!(
                            "HTTP {}: {}",
                            status, content
                        )),
                    },
                    RevokeRefreshTokensError::ServiceUnavailable,
                    || RevokeRefreshTokensError::ServiceError("Unknown error occurred".into()),
                )
            })?;
    }

    Ok(RevokeRefreshTokensResponse {})
}
