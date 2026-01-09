//! Token operation handlers for the gRPC client.

use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use libgn::pistachio_id::UserId;
use libgn::project::ProjectId;
use pistachio_api_common::admin::token::{
    CreateCustomTokenError, CreateCustomTokenRequest, CreateCustomTokenResponse,
    CreateSessionCookieError, CreateSessionCookieRequest, CreateSessionCookieResponse,
    DecodedIdToken, RevokeRefreshTokensError, RevokeRefreshTokensRequest,
    RevokeRefreshTokensResponse, VerifyIdTokenError, VerifyIdTokenRequest, VerifyIdTokenResponse,
    VerifySessionCookieError, VerifySessionCookieRequest, VerifySessionCookieResponse,
};
use pistachio_api_common::error::ValidationError;
use prost_types::{Duration, Struct, Timestamp, Value};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

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

fn seconds_to_duration(seconds: i32) -> Option<Duration> {
    if seconds > 0 {
        Some(Duration {
            seconds: i64::from(seconds),
            nanos: 0,
        })
    } else {
        None
    }
}

fn seconds_i64_to_duration(seconds: i64) -> Option<Duration> {
    if seconds > 0 {
        Some(Duration { seconds, nanos: 0 })
    } else {
        None
    }
}

fn json_to_struct(value: &serde_json::Value) -> Option<Struct> {
    if let serde_json::Value::Object(map) = value {
        let fields = map
            .iter()
            .map(|(k, v)| (k.clone(), json_value_to_proto_value(v)))
            .collect();
        Some(Struct { fields })
    } else {
        None
    }
}

fn json_value_to_proto_value(value: &serde_json::Value) -> Value {
    use prost_types::value::Kind;

    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Kind::NumberValue(f)
            } else {
                Kind::NumberValue(0.0)
            }
        }
        serde_json::Value::String(s) => Kind::StringValue(s.clone()),
        serde_json::Value::Array(arr) => {
            let values = arr.iter().map(json_value_to_proto_value).collect();
            Kind::ListValue(prost_types::ListValue { values })
        }
        serde_json::Value::Object(map) => {
            let fields = map
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_proto_value(v)))
                .collect();
            Kind::StructValue(Struct { fields })
        }
    };

    Value { kind: Some(kind) }
}

fn struct_to_hashmap(s: Option<Struct>) -> HashMap<String, serde_json::Value> {
    s.map(|st| {
        st.fields
            .into_iter()
            .map(|(k, v)| (k, proto_value_to_json_value(v)))
            .collect()
    })
    .unwrap_or_default()
}

fn proto_value_to_json_value(value: Value) -> serde_json::Value {
    use prost_types::value::Kind;

    match value.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(Kind::NumberValue(n)) => serde_json::json!(n),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(Kind::ListValue(list)) => {
            let arr: Vec<serde_json::Value> = list
                .values
                .into_iter()
                .map(proto_value_to_json_value)
                .collect();
            serde_json::Value::Array(arr)
        }
        Some(Kind::StructValue(st)) => {
            let map: serde_json::Map<String, serde_json::Value> = st
                .fields
                .into_iter()
                .map(|(k, v)| (k, proto_value_to_json_value(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        None => serde_json::Value::Null,
    }
}

fn decoded_id_token_from_proto(
    pistachio_id: String,
    email: String,
    email_verified: bool,
    claims: Option<Struct>,
    project_id: &ProjectId,
) -> Result<DecodedIdToken, ValidationError> {
    let user_id =
        UserId::parse(&pistachio_id).map_err(|_| ValidationError::InvalidValue("pistachio_id"))?;

    // Extract optional fields from claims
    let custom_claims = struct_to_hashmap(claims);

    // Get subject from claims or use pistachio_id
    let subject = custom_claims
        .get("sub")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| user_id.to_string());

    // Build decoded token with available information
    // Note: Some fields like issuer, audience, etc. may need to come from the claims
    Ok(DecodedIdToken {
        pistachio_id: user_id,
        project_id: project_id.clone(),
        tenant_id: None, // Could be extracted from claims if present
        issuer: custom_claims
            .get("iss")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        subject,
        audience: custom_claims
            .get("aud")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        issued_at: Utc::now(),  // Could be extracted from claims if present
        expires_at: Utc::now(), // Could be extracted from claims if present
        auth_time: Utc::now(),  // Could be extracted from claims if present
        email: if email.is_empty() { None } else { Some(email) },
        email_verified: Some(email_verified),
        phone_number: custom_claims
            .get("phone_number")
            .and_then(|v| v.as_str())
            .map(String::from),
        name: custom_claims
            .get("name")
            .and_then(|v| v.as_str())
            .map(String::from),
        picture: custom_claims
            .get("picture")
            .and_then(|v| v.as_str())
            .map(String::from),
        sign_in_provider: custom_claims
            .get("sign_in_provider")
            .and_then(|v| v.as_str())
            .map(String::from),
        custom_claims,
    })
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_create_custom_token<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateCustomTokenRequest,
) -> Result<CreateCustomTokenResponse, CreateCustomTokenError> {
    debug!("Creating proto request for create_custom_token");

    let request = pistachio_api::pistachio::admin::v1::CreateCustomTokenRequest {
        project_id: req.project_id.to_string(),
        pistachio_id: req.pistachio_id.to_string(),
        tenant_id: req.tenant_id.map(|t| t.to_string()).unwrap_or_default(),
        additional_claims: req.custom_claims.as_ref().and_then(json_to_struct),
        expires_in: req.expires_in_seconds.and_then(seconds_to_duration),
    };

    let response = client
        .create_custom_token(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_custom_token response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateCustomTokenError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    CreateCustomTokenError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    CreateCustomTokenError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateCustomTokenError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    CreateCustomTokenError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    CreateCustomTokenError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateCustomTokenError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(CreateCustomTokenResponse {
        custom_token: response.custom_token,
        expires_at: None, // Proto response doesn't include expiration time
    })
}

pub(crate) async fn handle_verify_id_token<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: VerifyIdTokenRequest,
) -> Result<VerifyIdTokenResponse, VerifyIdTokenError> {
    debug!("Creating proto request for verify_id_token");

    let request = pistachio_api::pistachio::admin::v1::VerifyIdTokenRequest {
        project_id: req.project_id.to_string(),
        id_token: req.id_token,
        check_revoked: req.check_disabled, // Map check_disabled to check_revoked
    };

    let response = client
        .verify_id_token(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in verify_id_token response");
            match status.code() {
                Code::InvalidArgument => {
                    VerifyIdTokenError::BadRequest(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    // Check if this is a token validation error
                    let message = status.message();
                    if message.contains("expired") {
                        VerifyIdTokenError::TokenExpired
                    } else {
                        VerifyIdTokenError::InvalidToken(message.to_string())
                    }
                }
                Code::PermissionDenied => {
                    VerifyIdTokenError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => VerifyIdTokenError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    VerifyIdTokenError::ServiceUnavailable(status.message().to_string())
                }
                _ => VerifyIdTokenError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    if !response.valid {
        return Err(VerifyIdTokenError::InvalidToken(
            "Token validation failed".to_string(),
        ));
    }

    let decoded_token = decoded_id_token_from_proto(
        response.pistachio_id,
        response.email,
        response.email_verified,
        response.claims,
        &req.project_id,
    )
    .map_err(VerifyIdTokenError::ResponseValidationError)?;

    Ok(VerifyIdTokenResponse { decoded_token })
}

pub(crate) async fn handle_create_session_cookie<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateSessionCookieRequest,
) -> Result<CreateSessionCookieResponse, CreateSessionCookieError> {
    debug!("Creating proto request for create_session_cookie");

    let request = pistachio_api::pistachio::admin::v1::CreateSessionCookieRequest {
        project_id: req.project_id.to_string(),
        id_token: req.id_token,
        expires_in: req.expires_in_seconds.and_then(seconds_i64_to_duration),
    };

    let response = client
        .create_session_cookie(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_session_cookie response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateSessionCookieError::BadRequest(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    let message = status.message();
                    if message.contains("expired") {
                        CreateSessionCookieError::TokenExpired
                    } else {
                        CreateSessionCookieError::InvalidIdToken(message.to_string())
                    }
                }
                Code::PermissionDenied => {
                    CreateSessionCookieError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    CreateSessionCookieError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    CreateSessionCookieError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateSessionCookieError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    Ok(CreateSessionCookieResponse {
        session_cookie: response.session_cookie,
        expires_at: optional_timestamp_to_datetime(response.expires_at),
    })
}

pub(crate) async fn handle_verify_session_cookie<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: VerifySessionCookieRequest,
) -> Result<VerifySessionCookieResponse, VerifySessionCookieError> {
    debug!("Creating proto request for verify_session_cookie");

    let request = pistachio_api::pistachio::admin::v1::VerifySessionCookieRequest {
        project_id: req.project_id.to_string(),
        session_cookie: req.session_cookie,
        check_revoked: req.check_revoked,
    };

    let response = client
        .verify_session_cookie(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in verify_session_cookie response");
            match status.code() {
                Code::InvalidArgument => {
                    VerifySessionCookieError::BadRequest(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    let message = status.message();
                    if message.contains("expired") {
                        VerifySessionCookieError::SessionExpired
                    } else if message.contains("revoked") {
                        VerifySessionCookieError::SessionRevoked
                    } else {
                        VerifySessionCookieError::InvalidSessionCookie(message.to_string())
                    }
                }
                Code::PermissionDenied => {
                    VerifySessionCookieError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    VerifySessionCookieError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    VerifySessionCookieError::ServiceUnavailable(status.message().to_string())
                }
                _ => VerifySessionCookieError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    if !response.valid {
        return Err(VerifySessionCookieError::InvalidSessionCookie(
            "Session cookie validation failed".to_string(),
        ));
    }

    let decoded_token = decoded_id_token_from_proto(
        response.pistachio_id,
        response.email,
        response.email_verified,
        response.claims,
        &req.project_id,
    )
    .map_err(VerifySessionCookieError::ResponseValidationError)?;

    Ok(VerifySessionCookieResponse { decoded_token })
}

pub(crate) async fn handle_revoke_refresh_tokens<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: RevokeRefreshTokensRequest,
) -> Result<RevokeRefreshTokensResponse, RevokeRefreshTokensError> {
    debug!("Creating proto request for revoke_refresh_tokens");

    let request = pistachio_api::pistachio::admin::v1::RevokeRefreshTokensRequest {
        project_id: req.project_id.to_string(),
        pistachio_id: req.pistachio_id.to_string(),
        tenant_id: req.tenant_id.map(|t| t.to_string()).unwrap_or_default(),
    };

    client
        .revoke_refresh_tokens(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in revoke_refresh_tokens response");
            match status.code() {
                Code::InvalidArgument => {
                    RevokeRefreshTokensError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    RevokeRefreshTokensError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    RevokeRefreshTokensError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    RevokeRefreshTokensError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    RevokeRefreshTokensError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    RevokeRefreshTokensError::ServiceUnavailable(status.message().to_string())
                }
                _ => RevokeRefreshTokensError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(RevokeRefreshTokensResponse {})
}
