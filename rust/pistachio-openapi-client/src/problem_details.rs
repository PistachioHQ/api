//! RFC 7807 Problem Details parsing.

use pistachio_api_common::error::{InvalidParam, ProblemDetails};
use serde::Deserialize;

/// Create a fallback ProblemDetails when parsing fails.
///
/// Uses appropriate error types based on HTTP status code rather than
/// a hardcoded error type, preserving more context about the error.
pub fn fallback_problem_details(status: u16, content: String) -> ProblemDetails {
    let (problem_type, title) = match status {
        400 => (
            "https://docs.pistachiohq.com/errors/invalid_argument",
            "Invalid Argument",
        ),
        401 => (
            "https://docs.pistachiohq.com/errors/unauthenticated",
            "Unauthenticated",
        ),
        403 => (
            "https://docs.pistachiohq.com/errors/permission_denied",
            "Permission Denied",
        ),
        404 => ("https://docs.pistachiohq.com/errors/not_found", "Not Found"),
        409 => (
            "https://docs.pistachiohq.com/errors/already_exists",
            "Already Exists",
        ),
        429 => (
            "https://docs.pistachiohq.com/errors/resource_exhausted",
            "Resource Exhausted",
        ),
        500 => (
            "https://docs.pistachiohq.com/errors/internal",
            "Internal Server Error",
        ),
        501 => (
            "https://docs.pistachiohq.com/errors/unimplemented",
            "Not Implemented",
        ),
        503 => (
            "https://docs.pistachiohq.com/errors/unavailable",
            "Service Unavailable",
        ),
        504 => (
            "https://docs.pistachiohq.com/errors/deadline_exceeded",
            "Deadline Exceeded",
        ),
        _ => (
            "https://docs.pistachiohq.com/errors/unknown",
            "Unknown Error",
        ),
    };

    ProblemDetails {
        problem_type: problem_type.to_string(),
        title: title.to_string(),
        status,
        detail: Some(content),
        instance: None,
        invalid_params: Vec::new(),
    }
}

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
    status: Option<u16>,
    detail: Option<String>,
    instance: Option<String>,
    #[serde(default)]
    invalid_params: Vec<InvalidParamJson>,
}

/// Parse RFC 7807 Problem Details from a JSON response body.
///
/// Returns `Some(ProblemDetails)` if the content is valid RFC 7807 JSON,
/// or `None` if parsing fails.
pub fn parse_problem_details(content: &str, status: u16) -> Option<ProblemDetails> {
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

    Some(ProblemDetails {
        problem_type,
        title: json.title.unwrap_or_else(|| "Error".to_string()),
        status: json.status.unwrap_or(status),
        detail: json.detail,
        instance: json.instance,
        invalid_params,
    })
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

        let result = parse_problem_details(json, 404);
        assert!(result.is_some());

        let problem = result.unwrap();
        assert_eq!(
            problem.problem_type,
            "https://docs.pistachiohq.com/errors/project_not_found"
        );
        assert_eq!(problem.title, "Not Found");
        assert_eq!(problem.status, 404);
        assert_eq!(
            problem.detail,
            Some("Project not found: my-project".to_string())
        );
    }

    #[test]
    fn test_parse_minimal_problem_details() {
        let json = r#"{"type": "https://docs.pistachiohq.com/errors/unknown"}"#;

        let result = parse_problem_details(json, 500);
        assert!(result.is_some());

        let problem = result.unwrap();
        assert_eq!(
            problem.problem_type,
            "https://docs.pistachiohq.com/errors/unknown"
        );
        assert_eq!(problem.title, "Error"); // default
        assert_eq!(problem.status, 500); // fallback to provided status
        assert_eq!(problem.detail, None);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_problem_details("not json", 500);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_type() {
        let json = r#"{"title": "Error", "status": 500}"#;
        let result = parse_problem_details(json, 500);
        assert!(result.is_none()); // type is required
    }
}
