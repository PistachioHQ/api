use chrono::{DateTime, Utc};
use pistachio_api_common::error::ValidationError;

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
