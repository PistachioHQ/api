use chrono::{DateTime, TimeZone, Utc};
use pistachio_api_common::error::ValidationError;
use prost_types::Timestamp;

pub(crate) trait FromProto<T>: Sized {
    type Error;
    fn from_proto(proto: T) -> Result<Self, Self::Error>;
}

pub(crate) trait IntoProto<T>: Sized {
    fn into_proto(self) -> T;
}

/// Convert a prost Timestamp to chrono `DateTime<Utc>`.
pub(crate) fn timestamp_to_datetime(
    ts: Option<Timestamp>,
) -> Result<DateTime<Utc>, ValidationError> {
    let ts = ts.ok_or(ValidationError::MissingField("timestamp"))?;

    // Nanos must be non-negative for valid timestamps
    let nanos =
        u32::try_from(ts.nanos).map_err(|_| ValidationError::InvalidValue("timestamp nanos"))?;

    Utc.timestamp_opt(ts.seconds, nanos)
        .single()
        .ok_or(ValidationError::InvalidValue("timestamp"))
}
