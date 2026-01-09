use chrono::{DateTime, Utc};
use pistachio_api_common::error::{ErrorDetails, InvalidParam, ValidationError};

use crate::generated_admin::models::{
    ListTenants400Response, ListTenants400ResponseInvalidParamsInner,
};

/// Base URL for error type documentation.
const ERROR_TYPE_BASE_URL: &str = "https://docs.pistachiohq.com/errors/";

/// Trait for converting JSON response types to domain types.
pub(crate) trait FromJson<T>: Sized {
    type Error;
    fn from_json(json: T) -> Result<Self, Self::Error>;
}

/// Parse an RFC3339 timestamp string to chrono `DateTime<Utc>`.
pub(crate) fn parse_timestamp(s: Option<String>) -> Result<DateTime<Utc>, ValidationError> {
    let s = s.ok_or(ValidationError::MissingField("timestamp"))?;
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| ValidationError::InvalidValue("timestamp"))
}

/// Extract the error type slug from a full error type URL.
///
/// Example: "https://docs.pistachiohq.com/errors/not_found" -> "not_found"
fn extract_error_type_slug(url: &str) -> String {
    url.strip_prefix(ERROR_TYPE_BASE_URL)
        .unwrap_or(url)
        .to_string()
}

/// Convert a generated OpenAPI error response to protocol-agnostic ErrorDetails.
///
/// This function handles the RFC 7807 Problem Details format and converts it
/// to a protocol-agnostic representation.
pub(crate) fn convert_error_details(e: ListTenants400Response) -> ErrorDetails {
    ErrorDetails {
        error_type: extract_error_type_slug(&e.r#type),
        title: e.title,
        message: e.detail,
        invalid_params: e
            .invalid_params
            .map(|params: Vec<ListTenants400ResponseInvalidParamsInner>| {
                params.into_iter().map(convert_invalid_param).collect()
            })
            .unwrap_or_default(),
    }
}

/// Convert a generated InvalidParam to a domain InvalidParam.
fn convert_invalid_param(param: ListTenants400ResponseInvalidParamsInner) -> InvalidParam {
    InvalidParam {
        name: param.name,
        reason: param.reason,
        value: param.value,
        expected_values: param.expected_values.unwrap_or_default(),
    }
}
