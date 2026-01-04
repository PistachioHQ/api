use chrono::{DateTime, TimeZone, Utc};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{
    PaginationMeta, PaginationParams, SortDirection, SortField,
};
use pistachio_api_common::search::SearchParams;
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

// =============================================================================
// Pagination conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::types::v1::SortField> for SortField {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::SortField {
        pistachio_api::pistachio::types::v1::SortField {
            field: self.field,
            direction: match self.direction {
                SortDirection::Asc => {
                    pistachio_api::pistachio::types::v1::SortDirection::Asc.into()
                }
                SortDirection::Desc => {
                    pistachio_api::pistachio::types::v1::SortDirection::Desc.into()
                }
            },
        }
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::PaginationParams> for PaginationParams {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::PaginationParams {
        pistachio_api::pistachio::types::v1::PaginationParams {
            page_size: self.page_size.unwrap_or(0),
            cursor: self.cursor.unwrap_or_default(),
            sort: self.sort.into_iter().map(|s| s.into_proto()).collect(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::PaginationMeta> for PaginationMeta {
    type Error = ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::PaginationMeta,
    ) -> Result<Self, Self::Error> {
        let next_cursor = if proto.next_cursor.is_empty() {
            None
        } else {
            Some(proto.next_cursor)
        };

        Ok(Self {
            next_cursor,
            total_count: proto.total_count,
        })
    }
}

/// Convert `PaginationParams` to proto format.
pub(crate) fn pagination_params_to_proto(
    params: PaginationParams,
) -> pistachio_api::pistachio::types::v1::PaginationParams {
    params.into_proto()
}

/// Convert proto `PaginationMeta` to domain type.
pub(crate) fn pagination_meta_from_proto(
    proto: pistachio_api::pistachio::types::v1::PaginationMeta,
) -> PaginationMeta {
    // unwrap is safe here as we control the conversion and it can't fail
    // for the simple fields we're converting
    PaginationMeta {
        next_cursor: if proto.next_cursor.is_empty() {
            None
        } else {
            Some(proto.next_cursor)
        },
        total_count: proto.total_count,
    }
}

// =============================================================================
// Search conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::types::v1::SearchParams> for SearchParams {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::SearchParams {
        pistachio_api::pistachio::types::v1::SearchParams {
            query: self.query,
            pagination: Some(self.pagination.into_proto()),
        }
    }
}
