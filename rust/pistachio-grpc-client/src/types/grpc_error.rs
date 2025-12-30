//! gRPC error to RFC 7807 ProblemDetails conversion utilities.
//!
//! This module provides shared functionality for converting gRPC Status errors
//! into RFC 7807 ProblemDetails responses, including parsing of structured
//! error details from gRPC ErrorInfo metadata.

use std::collections::HashMap;

use pistachio_api_common::error::{InvalidParam, ProblemDetails};
use tonic::Code;
use tonic_types::StatusExt;

/// Base URL for error type documentation.
const ERROR_TYPE_BASE_URL: &str = "https://docs.pistachiohq.com/errors/";

/// Map an error slug to a human-readable title.
///
/// This is a fallback mapping used when the server doesn't provide an explicit
/// title in the error metadata. The server-provided title always takes precedence.
fn error_slug_to_title(slug: &str) -> &'static str {
    match slug {
        // Resource errors
        "not_found" => "Not found",
        "project_not_found" => "Project not found",
        "tenant_not_found" => "Tenant not found",
        "app_not_found" => "App not found",
        "already_exists" => "Already exists",
        "not_deleted" => "Resource not deleted",
        // Validation errors
        "invalid_request" => "Invalid request",
        "invalid_parameter" => "Invalid parameter",
        "invalid_sort_field" => "Invalid sort field",
        // Auth errors
        "unauthenticated" => "Authentication required",
        "permission_denied" => "Permission denied",
        // Infrastructure errors
        "service_unavailable" => "Service unavailable",
        "internal_error" => "Internal server error",
        _ => "Error",
    }
}

/// Fallback title based on gRPC status code.
///
/// Used when no ErrorInfo is available in the gRPC status.
fn fallback_title_from_code(code: Code) -> &'static str {
    match code {
        Code::NotFound => "Not found",
        Code::InvalidArgument => "Bad request",
        Code::Unauthenticated => "Authentication required",
        Code::PermissionDenied => "Permission denied",
        Code::Internal => "Internal server error",
        Code::Unavailable => "Service unavailable",
        Code::AlreadyExists => "Already exists",
        Code::FailedPrecondition => "Failed precondition",
        _ => "Error",
    }
}

/// Extract a single RFC 7807 `invalidParam` from `ErrorInfo.metadata`.
///
/// This performs a mechanical transformation of the structured metadata format
/// used by api-gateway into an RFC 7807 invalid param. The metadata format uses
/// generic `field.*` keys (transformed from `grpc.field.*` by api-gateway):
///
/// - `field.key` - The field path with the violation
/// - `field.reason` - Human-readable reason for the violation
/// - `field.value` - The invalid value that was provided (optional)
/// - `field.expected_values` - Comma-separated valid values (optional)
fn extract_invalid_param(metadata: &HashMap<String, String>) -> Option<InvalidParam> {
    let field_key = metadata.get("field.key")?;

    let reason = metadata.get("field.reason").cloned().unwrap_or_default();
    let value = metadata.get("field.value").cloned();
    let expected_values = metadata
        .get("field.expected_values")
        .map(|v| v.split(',').map(str::to_string).collect())
        .unwrap_or_default();

    Some(InvalidParam {
        name: field_key.clone(),
        reason,
        value,
        expected_values,
    })
}

/// Create a ProblemDetails from a gRPC Status.
///
/// This function extracts structured error information from gRPC status details
/// and converts it to an RFC 7807 ProblemDetails response. It handles:
///
/// - Error type URL from ErrorInfo.reason
/// - Title from ErrorInfo.metadata["title"] or fallback slug mapping
/// - Invalid parameters from ErrorInfo.metadata["field.*"] entries
///
/// # Arguments
///
/// * `status` - The gRPC status containing error details
/// * `http_status` - The HTTP status code to use in the ProblemDetails
pub(crate) fn problem_details_from_status(
    status: &tonic::Status,
    http_status: u16,
) -> ProblemDetails {
    // Get all error details from the status (supports multiple ErrorInfo entries)
    let error_details = status.get_error_details_vec();

    // Collect all ErrorInfo entries
    let error_infos: Vec<&tonic_types::ErrorInfo> = error_details
        .iter()
        .filter_map(|d| {
            if let tonic_types::ErrorDetail::ErrorInfo(info) = d {
                Some(info)
            } else {
                None
            }
        })
        .collect();

    // Extract problem type and title from the first ErrorInfo
    let (problem_type, title) = error_infos
        .first()
        .filter(|info| !info.reason.is_empty())
        .map_or_else(
            || {
                (
                    format!("{ERROR_TYPE_BASE_URL}unknown"),
                    fallback_title_from_code(status.code()).to_string(),
                )
            },
            |info| {
                let slug = &info.reason;
                // Check for explicit title in metadata first, fall back to slug mapping
                let title = info
                    .metadata
                    .get("title")
                    .map_or_else(|| error_slug_to_title(slug).to_string(), Clone::clone);
                (format!("{ERROR_TYPE_BASE_URL}{slug}"), title)
            },
        );

    // Extract invalid params from each ErrorInfo.metadata
    let invalid_params: Vec<InvalidParam> = error_infos
        .iter()
        .filter_map(|info| extract_invalid_param(&info.metadata))
        .collect();

    ProblemDetails {
        problem_type,
        title,
        status: http_status,
        detail: Some(status.message().to_string()),
        instance: None,
        invalid_params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_slug_to_title_known_slugs() {
        assert_eq!(error_slug_to_title("not_found"), "Not found");
        assert_eq!(
            error_slug_to_title("project_not_found"),
            "Project not found"
        );
        assert_eq!(error_slug_to_title("invalid_request"), "Invalid request");
    }

    #[test]
    fn test_error_slug_to_title_unknown_slug() {
        assert_eq!(error_slug_to_title("some_unknown_error"), "Error");
    }

    #[test]
    fn test_fallback_title_from_code() {
        assert_eq!(fallback_title_from_code(Code::NotFound), "Not found");
        assert_eq!(
            fallback_title_from_code(Code::InvalidArgument),
            "Bad request"
        );
        assert_eq!(
            fallback_title_from_code(Code::Unauthenticated),
            "Authentication required"
        );
    }

    #[test]
    fn test_extract_invalid_param_complete() {
        let mut metadata = HashMap::new();
        metadata.insert("field.key".to_string(), "sort".to_string());
        metadata.insert("field.reason".to_string(), "Invalid value".to_string());
        metadata.insert("field.value".to_string(), "foo".to_string());
        metadata.insert(
            "field.expected_values".to_string(),
            "project_id,created_at,display_name".to_string(),
        );

        let param = extract_invalid_param(&metadata).unwrap();
        assert_eq!(param.name, "sort");
        assert_eq!(param.reason, "Invalid value");
        assert_eq!(param.value, Some("foo".to_string()));
        assert_eq!(
            param.expected_values,
            vec!["project_id", "created_at", "display_name"]
        );
    }

    #[test]
    fn test_extract_invalid_param_minimal() {
        let mut metadata = HashMap::new();
        metadata.insert("field.key".to_string(), "page_size".to_string());

        let param = extract_invalid_param(&metadata).unwrap();
        assert_eq!(param.name, "page_size");
        assert_eq!(param.reason, "");
        assert_eq!(param.value, None);
        assert!(param.expected_values.is_empty());
    }

    #[test]
    fn test_extract_invalid_param_missing_key() {
        let metadata = HashMap::new();
        assert!(extract_invalid_param(&metadata).is_none());
    }
}
