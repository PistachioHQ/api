use chrono::{DateTime, Utc};
use pistachio_api_common::error::{InvalidParam, ProblemDetails, ValidationError};

use crate::generated_admin::models::{ListApps400Response, ListApps400ResponseInvalidParamsInner};

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

/// Convert a generated OpenAPI error response to a domain ProblemDetails.
///
/// This function handles the RFC 7807 Problem Details format.
pub(crate) fn convert_problem_details(e: ListApps400Response) -> ProblemDetails {
    ProblemDetails {
        problem_type: e.r#type,
        title: e.title,
        status: e.status as u16,
        detail: e.detail,
        instance: e.instance,
        invalid_params: e
            .invalid_params
            .map(|params| params.into_iter().map(convert_invalid_param).collect())
            .unwrap_or_default(),
    }
}

/// Convert a generated InvalidParam to a domain InvalidParam.
fn convert_invalid_param(param: ListApps400ResponseInvalidParamsInner) -> InvalidParam {
    InvalidParam {
        name: param.name,
        reason: param.reason,
        value: param.value,
        expected_values: param.expected_values.unwrap_or_default(),
    }
}
