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

// =============================================================================
// User conversions
// =============================================================================

use libgn::claims::ClaimValue;
use libgn::email::Email;
use libgn::pistachio_id::UserId;
use libgn::tenant::TenantId;
use libgn::user::{DisplayName, PhoneNumber, PhotoUrl, UserResourceName};
use pistachio_api_common::admin::user::{
    CustomClaims, HashAlgorithm, ImportUserError, ImportUserRecord, User,
};

/// Convert an optional prost Timestamp to an optional chrono `DateTime<Utc>`.
pub(crate) fn optional_timestamp_to_datetime(
    ts: Option<Timestamp>,
) -> Result<Option<DateTime<Utc>>, ValidationError> {
    match ts {
        Some(ts) => {
            let nanos = u32::try_from(ts.nanos)
                .map_err(|_| ValidationError::InvalidValue("timestamp nanos"))?;

            Ok(Utc.timestamp_opt(ts.seconds, nanos).single())
        }
        None => Ok(None),
    }
}

/// Convert a prost Value to a ClaimValue.
fn prost_value_to_claim(value: &prost_types::Value) -> ClaimValue {
    match &value.kind {
        Some(prost_types::value::Kind::NullValue(_)) => ClaimValue::Null,
        Some(prost_types::value::Kind::BoolValue(b)) => ClaimValue::Bool(*b),
        Some(prost_types::value::Kind::NumberValue(n)) => ClaimValue::Number(*n),
        Some(prost_types::value::Kind::StringValue(s)) => ClaimValue::String(s.clone()),
        Some(prost_types::value::Kind::ListValue(list)) => {
            ClaimValue::Array(list.values.iter().map(prost_value_to_claim).collect())
        }
        Some(prost_types::value::Kind::StructValue(s)) => ClaimValue::Object(
            s.fields
                .iter()
                .map(|(k, v)| (k.clone(), prost_value_to_claim(v)))
                .collect(),
        ),
        None => ClaimValue::Null,
    }
}

/// Convert a prost Struct to CustomClaims (HashMap<String, ClaimValue>).
fn struct_to_custom_claims(s: Option<prost_types::Struct>) -> Option<CustomClaims> {
    s.map(|s| {
        s.fields
            .iter()
            .map(|(k, v)| (k.clone(), prost_value_to_claim(v)))
            .collect()
    })
}

/// Convert a ClaimValue to a prost Value.
fn claim_to_prost_value(value: &ClaimValue) -> prost_types::Value {
    prost_types::Value {
        kind: Some(match value {
            ClaimValue::Null => prost_types::value::Kind::NullValue(0),
            ClaimValue::Bool(b) => prost_types::value::Kind::BoolValue(*b),
            ClaimValue::Number(n) => prost_types::value::Kind::NumberValue(*n),
            ClaimValue::String(s) => prost_types::value::Kind::StringValue(s.clone()),
            ClaimValue::Array(arr) => prost_types::value::Kind::ListValue(prost_types::ListValue {
                values: arr.iter().map(claim_to_prost_value).collect(),
            }),
            ClaimValue::Object(map) => prost_types::value::Kind::StructValue(prost_types::Struct {
                fields: map
                    .iter()
                    .map(|(k, v)| (k.clone(), claim_to_prost_value(v)))
                    .collect(),
            }),
        }),
    }
}

/// Convert CustomClaims to a prost Struct.
pub(crate) fn custom_claims_to_struct(claims: &CustomClaims) -> prost_types::Struct {
    prost_types::Struct {
        fields: claims
            .iter()
            .map(|(k, v)| (k.clone(), claim_to_prost_value(v)))
            .collect(),
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::User> for User {
    type Error = ValidationError;

    fn from_proto(proto: pistachio_api::pistachio::types::v1::User) -> Result<Self, Self::Error> {
        let name = UserResourceName::parse(&proto.name)
            .map_err(|_| ValidationError::InvalidValue("name"))?;

        let pistachio_id = UserId::parse(&proto.pistachio_id)
            .map_err(|_| ValidationError::InvalidValue("pistachio_id"))?;

        let tenant_id = if proto.tenant_id.is_empty() {
            None
        } else {
            Some(
                TenantId::parse(&proto.tenant_id)
                    .map_err(|_| ValidationError::InvalidValue("tenant_id"))?,
            )
        };

        let email = if proto.email.is_empty() {
            None
        } else {
            Some(Email::parse(&proto.email).map_err(|_| ValidationError::InvalidValue("email"))?)
        };

        let phone_number = if proto.phone_number.is_empty() {
            None
        } else {
            Some(
                PhoneNumber::parse(&proto.phone_number)
                    .map_err(|_| ValidationError::InvalidValue("phone_number"))?,
            )
        };

        let display_name = if proto.display_name.is_empty() {
            None
        } else {
            Some(
                DisplayName::parse(&proto.display_name)
                    .map_err(|_| ValidationError::InvalidValue("display_name"))?,
            )
        };

        let photo_url = if proto.photo_url.is_empty() {
            None
        } else {
            Some(
                PhotoUrl::parse(&proto.photo_url)
                    .map_err(|_| ValidationError::InvalidValue("photo_url"))?,
            )
        };

        let custom_claims = struct_to_custom_claims(proto.custom_claims);

        let created_at = optional_timestamp_to_datetime(proto.created_at)?;
        let last_sign_in_at = optional_timestamp_to_datetime(proto.last_sign_in_at)?;
        let last_refresh_at = optional_timestamp_to_datetime(proto.last_refresh_at)?;
        let updated_at = optional_timestamp_to_datetime(proto.updated_at)?;

        Ok(Self {
            name,
            pistachio_id,
            tenant_id,
            email,
            email_verified: proto.email_verified,
            phone_number,
            display_name,
            photo_url,
            disabled: proto.disabled,
            custom_claims,
            created_at,
            last_sign_in_at,
            last_refresh_at,
            updated_at,
        })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::ImportUserError> for ImportUserError {
    type Error = ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::ImportUserError,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            index: proto.index,
            message: proto.message,
            field: if proto.field.is_empty() {
                None
            } else {
                Some(proto.field)
            },
        })
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::ImportUserRecord> for ImportUserRecord {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::ImportUserRecord {
        pistachio_api::pistachio::types::v1::ImportUserRecord {
            email: self.email.unwrap_or_default(),
            email_verified: self.email_verified,
            phone_number: self.phone_number.unwrap_or_default(),
            display_name: self.display_name.unwrap_or_default(),
            photo_url: self.photo_url.unwrap_or_default(),
            disabled: self.disabled,
            custom_claims: self
                .custom_claims
                .map(|claims| custom_claims_to_struct(&claims)),
            password_hash: self.password_hash.unwrap_or_default(),
            password_salt: self.password_salt.unwrap_or_default(),
        }
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::HashAlgorithm> for HashAlgorithm {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::HashAlgorithm {
        match self {
            HashAlgorithm::Scrypt => pistachio_api::pistachio::types::v1::HashAlgorithm::Scrypt,
            HashAlgorithm::Bcrypt => pistachio_api::pistachio::types::v1::HashAlgorithm::Bcrypt,
            HashAlgorithm::Argon2 => pistachio_api::pistachio::types::v1::HashAlgorithm::Argon2,
            HashAlgorithm::Pbkdf2Sha256 => {
                pistachio_api::pistachio::types::v1::HashAlgorithm::Pbkdf2Sha256
            }
        }
    }
}
