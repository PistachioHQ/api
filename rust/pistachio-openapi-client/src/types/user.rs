//! User type conversions for OpenAPI responses.

use libgn::claims::ClaimValue;
use libgn::email::Email;
use libgn::pistachio_id::UserId;
use libgn::tenant::TenantId as HumanTenantId;
use libgn::user::{DisplayName, PhoneNumber, PhotoUrl, UserResourceName};
use pistachio_api_common::admin::user::{CustomClaims, HashAlgorithm, ImportUserError, User};
use pistachio_api_common::error::ValidationError;

use super::{FromJson, parse_timestamp};
use crate::generated_admin::models::{
    CreateProjectUser201ResponseUser, ImportProjectUsers200ResponseErrorsInner,
    ListProjectUsers200ResponseUsersInner,
};

impl FromJson<CreateProjectUser201ResponseUser> for User {
    type Error = ValidationError;

    fn from_json(json: CreateProjectUser201ResponseUser) -> Result<Self, Self::Error> {
        user_from_json_inner(
            json.name,
            json.pistachio_id,
            json.tenant_id,
            json.email,
            json.email_verified,
            json.phone_number,
            json.display_name,
            json.photo_url,
            json.disabled,
            json.custom_claims,
            Some(json.created_at),
            json.last_sign_in_at,
            json.last_refresh_at,
            Some(json.updated_at),
        )
    }
}

impl FromJson<ListProjectUsers200ResponseUsersInner> for User {
    type Error = ValidationError;

    fn from_json(json: ListProjectUsers200ResponseUsersInner) -> Result<Self, Self::Error> {
        user_from_json_inner(
            json.name,
            json.pistachio_id,
            json.tenant_id,
            json.email,
            json.email_verified,
            json.phone_number,
            json.display_name,
            json.photo_url,
            json.disabled,
            json.custom_claims.map(Some),
            Some(json.created_at),
            json.last_sign_in_at,
            json.last_refresh_at,
            Some(json.updated_at),
        )
    }
}

/// Shared conversion logic for User types from various OpenAPI response models.
#[allow(clippy::too_many_arguments)]
fn user_from_json_inner(
    name: String,
    pistachio_id: String,
    tenant_id: Option<Option<String>>,
    email: Option<Option<String>>,
    email_verified: bool,
    phone_number: Option<Option<String>>,
    display_name: Option<Option<String>>,
    photo_url: Option<Option<String>>,
    disabled: bool,
    custom_claims: Option<Option<std::collections::HashMap<String, serde_json::Value>>>,
    created_at: Option<String>,
    last_sign_in_at: Option<Option<String>>,
    last_refresh_at: Option<Option<String>>,
    updated_at: Option<String>,
) -> Result<User, ValidationError> {
    let name = UserResourceName::parse(&name).map_err(|_| ValidationError::InvalidValue("name"))?;

    let pistachio_id =
        UserId::parse(&pistachio_id).map_err(|_| ValidationError::InvalidValue("pistachio_id"))?;

    // Handle Option<Option<String>> pattern from serde_with::double_option
    let tenant_id = match tenant_id.flatten() {
        Some(s) if !s.is_empty() => {
            Some(HumanTenantId::parse(&s).map_err(|_| ValidationError::InvalidValue("tenant_id"))?)
        }
        _ => None,
    };

    let email = match email.flatten() {
        Some(s) if !s.is_empty() => {
            Some(Email::parse(&s).map_err(|_| ValidationError::InvalidValue("email"))?)
        }
        _ => None,
    };

    let phone_number = match phone_number.flatten() {
        Some(s) if !s.is_empty() => Some(
            PhoneNumber::parse(&s).map_err(|_| ValidationError::InvalidValue("phone_number"))?,
        ),
        _ => None,
    };

    let display_name = match display_name.flatten() {
        Some(s) if !s.is_empty() => Some(
            DisplayName::parse(&s).map_err(|_| ValidationError::InvalidValue("display_name"))?,
        ),
        _ => None,
    };

    let photo_url = match photo_url.flatten() {
        Some(s) if !s.is_empty() => {
            Some(PhotoUrl::parse(&s).map_err(|_| ValidationError::InvalidValue("photo_url"))?)
        }
        _ => None,
    };

    // Convert HashMap<String, serde_json::Value> to HashMap<String, ClaimValue>
    let custom_claims = custom_claims.flatten().map(|claims| {
        claims
            .into_iter()
            .map(|(k, v)| (k, ClaimValue::from(v)))
            .collect::<CustomClaims>()
    });

    let created_at = parse_timestamp(created_at).ok();
    let last_sign_in_at = parse_timestamp(last_sign_in_at.flatten()).ok();
    let last_refresh_at = parse_timestamp(last_refresh_at.flatten()).ok();
    let updated_at = parse_timestamp(updated_at).ok();

    Ok(User {
        name,
        pistachio_id,
        tenant_id,
        email,
        email_verified,
        phone_number,
        display_name,
        photo_url,
        disabled,
        custom_claims,
        created_at,
        last_sign_in_at,
        last_refresh_at,
        updated_at,
    })
}

impl FromJson<ImportProjectUsers200ResponseErrorsInner> for ImportUserError {
    type Error = ValidationError;

    fn from_json(json: ImportProjectUsers200ResponseErrorsInner) -> Result<Self, Self::Error> {
        Ok(ImportUserError {
            index: json.index,
            message: json.message,
            field: json.field.flatten(),
        })
    }
}

/// Convert domain HashAlgorithm to OpenAPI enum.
pub(crate) fn hash_algorithm_to_openapi(
    alg: HashAlgorithm,
) -> crate::generated_admin::models::import_project_users_request::HashAlgorithm {
    use crate::generated_admin::models::import_project_users_request::HashAlgorithm as GenHashAlgorithm;
    match alg {
        HashAlgorithm::Scrypt => GenHashAlgorithm::Scrypt,
        HashAlgorithm::Bcrypt => GenHashAlgorithm::Bcrypt,
        HashAlgorithm::Argon2 => GenHashAlgorithm::Argon2,
        HashAlgorithm::Pbkdf2Sha256 => GenHashAlgorithm::Pbkdf2Sha256,
    }
}
