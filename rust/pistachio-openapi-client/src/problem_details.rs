//! RFC 7807 Problem Details parsing.
//!
//! This module parses RFC 7807 Problem Details from HTTP responses
//! and converts them to protocol-agnostic ErrorDetails for the public API.

use pistachio_api_common::error::{ErrorDetails, InvalidParam};
use serde::Deserialize;

/// Base URL for error type documentation.
const ERROR_TYPE_BASE_URL: &str = "https://docs.pistachiohq.com/errors/";

/// JSON structure for an invalid parameter in RFC 7807 Problem Details.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvalidParamJson {
    name: String,
    reason: String,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    expected_values: Vec<String>,
}

/// JSON structure for RFC 7807 Problem Details.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDetailsJson {
    #[serde(rename = "type")]
    problem_type: Option<String>,
    title: Option<String>,
    #[allow(dead_code)]
    status: Option<u16>,
    detail: Option<String>,
    #[allow(dead_code)]
    instance: Option<String>,
    #[serde(default)]
    invalid_params: Vec<InvalidParamJson>,
}

/// Extract the error type slug from a full error type URL.
///
/// Example: "https://docs.pistachiohq.com/errors/not_found" -> "not_found"
fn extract_error_type_slug(url: &str) -> String {
    url.strip_prefix(ERROR_TYPE_BASE_URL)
        .unwrap_or(url)
        .to_string()
}

/// Parse RFC 7807 Problem Details from a JSON response body
/// into protocol-agnostic ErrorDetails.
///
/// Returns `Some(ErrorDetails)` if the content is valid RFC 7807 JSON,
/// or `None` if parsing fails.
pub fn parse_error_details(content: &str) -> Option<ErrorDetails> {
    let json: ProblemDetailsJson = serde_json::from_str(content).ok()?;

    // RFC 7807 requires at least a "type" field (though it can be "about:blank")
    let problem_type = json.problem_type?;

    let invalid_params = json
        .invalid_params
        .into_iter()
        .map(|p| InvalidParam {
            name: p.name,
            reason: p.reason,
            value: p.value,
            expected_values: p.expected_values,
        })
        .collect();

    Some(ErrorDetails {
        error_type: extract_error_type_slug(&problem_type),
        title: json.title.unwrap_or_else(|| "Error".to_string()),
        message: json.detail,
        invalid_params,
    })
}

/// Create fallback ErrorDetails when parsing fails.
///
/// Used when the response body is not valid RFC 7807 JSON.
pub fn fallback_error_details(message: impl Into<String>) -> ErrorDetails {
    ErrorDetails {
        error_type: "unknown".to_string(),
        title: "Error".to_string(),
        message: Some(message.into()),
        invalid_params: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_problem_details() {
        let json = r#"{
            "type": "https://docs.pistachiohq.com/errors/project_not_found",
            "title": "Not Found",
            "status": 404,
            "detail": "Project not found: my-project"
        }"#;

        let result = parse_error_details(json);
        assert!(result.is_some());

        let details = result.unwrap();
        assert_eq!(details.error_type, "project_not_found");
        assert_eq!(details.title, "Not Found");
        assert_eq!(
            details.message,
            Some("Project not found: my-project".to_string())
        );
    }

    #[test]
    fn test_parse_minimal_problem_details() {
        let json = r#"{"type": "https://docs.pistachiohq.com/errors/unknown"}"#;

        let result = parse_error_details(json);
        assert!(result.is_some());

        let details = result.unwrap();
        assert_eq!(details.error_type, "unknown");
        assert_eq!(details.title, "Error"); // default
        assert_eq!(details.message, None);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_error_details("not json");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_type() {
        let json = r#"{"title": "Error", "status": 500}"#;
        let result = parse_error_details(json);
        assert!(result.is_none()); // type is required
    }

    #[test]
    fn test_extract_error_type_slug() {
        assert_eq!(
            extract_error_type_slug("https://docs.pistachiohq.com/errors/not_found"),
            "not_found"
        );
        assert_eq!(
            extract_error_type_slug("https://docs.pistachiohq.com/errors/already_exists"),
            "already_exists"
        );
        // If not a valid URL, return as-is
        assert_eq!(extract_error_type_slug("some_error"), "some_error");
    }
}
