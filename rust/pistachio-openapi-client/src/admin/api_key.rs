//! API key operation handlers for the OpenAPI client.

use pistachio_api_common::admin::api_key::{
    AndroidApplication, ApiKey, ApiKeyInfo, ApiKeyRestrictions, BrowserKeyRestrictions,
    CreateApiKeyError, CreateApiKeyRequest, CreateApiKeyResponse, DeleteApiKeyError,
    DeleteApiKeyRequest, DeleteApiKeyResponse, GetApiKeyError, GetApiKeyRequest, GetApiKeyResponse,
    IosKeyRestrictions, ListApiKeysError, ListApiKeysRequest, ListApiKeysResponse,
    PlatformRestrictions, RotateApiKeyError, RotateApiKeyRequest, RotateApiKeyResponse,
    ServerKeyRestrictions, UpdateApiKeyError, UpdateApiKeyRequest, UpdateApiKeyResponse,
};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::api_keys_api::{
    CreateApiKeyError as GenCreateApiKeyError, DeleteApiKeyError as GenDeleteApiKeyError,
    GetApiKeyError as GenGetApiKeyError, ListApiKeysError as GenListApiKeysError,
    RotateApiKeyError as GenRotateApiKeyError, UpdateApiKeyError as GenUpdateApiKeyError,
    create_api_key, delete_api_key, get_api_key, list_api_keys, rotate_api_key, update_api_key,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::{
    CreateApiKey201Response, CreateApiKey201ResponseApiKey,
    CreateApiKeyRequest as GenCreateApiKeyRequest, GetApiKey200Response, ListApiKeys200Response,
    ListApiKeys200ResponseApiKeysInner, ListApiKeys200ResponseApiKeysInnerRestrictions,
    ListApiKeys200ResponseApiKeysInnerRestrictionsAndroidKeyRestrictions,
    ListApiKeys200ResponseApiKeysInnerRestrictionsAndroidKeyRestrictionsAllowedApplicationsInner,
    ListApiKeys200ResponseApiKeysInnerRestrictionsBrowserKeyRestrictions,
    ListApiKeys200ResponseApiKeysInnerRestrictionsIosKeyRestrictions,
    ListApiKeys200ResponseApiKeysInnerRestrictionsServerKeyRestrictions, RotateApiKey200Response,
    RotateApiKeyRequest as GenRotateApiKeyRequest, UpdateApiKey200Response,
    UpdateApiKeyRequest as GenUpdateApiKeyRequest,
};
use crate::problem_details::fallback_error_details;
use crate::types::convert_error_details;
use crate::types::{FromJson, parse_timestamp};

// =============================================================================
// Type Conversions
// =============================================================================

impl FromJson<CreateApiKey201ResponseApiKey> for ApiKey {
    type Error = ValidationError;

    fn from_json(json: CreateApiKey201ResponseApiKey) -> Result<Self, Self::Error> {
        Ok(Self {
            name: json.name,
            key_id: json.key_id,
            key_string: json.key_string,
            display_name: json.display_name.flatten(),
            restrictions: json.restrictions.map(|r| convert_restrictions(*r)),
            created_at: parse_timestamp(Some(json.created_at)).ok(),
            updated_at: json
                .updated_at
                .flatten()
                .and_then(|s| parse_timestamp(Some(s)).ok()),
        })
    }
}

impl FromJson<ListApiKeys200ResponseApiKeysInner> for ApiKeyInfo {
    type Error = ValidationError;

    fn from_json(json: ListApiKeys200ResponseApiKeysInner) -> Result<Self, Self::Error> {
        Ok(Self {
            name: json.name,
            key_id: json.key_id,
            key_prefix: json.key_prefix,
            display_name: json.display_name.flatten(),
            restrictions: json.restrictions.map(|r| convert_restrictions(*r)),
            created_at: parse_timestamp(Some(json.created_at)).ok(),
            updated_at: json
                .updated_at
                .flatten()
                .and_then(|s| parse_timestamp(Some(s)).ok()),
        })
    }
}

fn convert_restrictions(r: ListApiKeys200ResponseApiKeysInnerRestrictions) -> ApiKeyRestrictions {
    let platform = if let Some(browser) = r.browser_key_restrictions {
        Some(PlatformRestrictions::Browser(BrowserKeyRestrictions {
            allowed_referrers: browser.allowed_referrers.unwrap_or_default(),
        }))
    } else if let Some(server) = r.server_key_restrictions {
        Some(PlatformRestrictions::Server(ServerKeyRestrictions {
            allowed_ips: server.allowed_ips.unwrap_or_default(),
        }))
    } else if let Some(android) = r.android_key_restrictions {
        Some(PlatformRestrictions::Android(
            pistachio_api_common::admin::api_key::AndroidKeyRestrictions {
                allowed_applications: android
                    .allowed_applications
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| AndroidApplication {
                        package_name: a.package_name.unwrap_or_default(),
                        sha256_cert_fingerprint: a.sha256_cert_fingerprint.unwrap_or_default(),
                    })
                    .collect(),
            },
        ))
    } else {
        r.ios_key_restrictions.map(|ios| {
            PlatformRestrictions::Ios(IosKeyRestrictions {
                allowed_bundle_ids: ios.allowed_bundle_ids.unwrap_or_default(),
            })
        })
    };

    ApiKeyRestrictions {
        platform_restrictions: platform,
    }
}

fn convert_restrictions_to_openapi(
    r: &ApiKeyRestrictions,
) -> ListApiKeys200ResponseApiKeysInnerRestrictions {
    let mut result = ListApiKeys200ResponseApiKeysInnerRestrictions::new();

    if let Some(ref platform) = r.platform_restrictions {
        match platform {
            PlatformRestrictions::Browser(browser) => {
                let mut br =
                    ListApiKeys200ResponseApiKeysInnerRestrictionsBrowserKeyRestrictions::new();
                br.allowed_referrers = Some(browser.allowed_referrers.clone());
                result.browser_key_restrictions = Some(Box::new(br));
            }
            PlatformRestrictions::Server(server) => {
                let mut sr =
                    ListApiKeys200ResponseApiKeysInnerRestrictionsServerKeyRestrictions::new();
                sr.allowed_ips = Some(server.allowed_ips.clone());
                result.server_key_restrictions = Some(Box::new(sr));
            }
            PlatformRestrictions::Android(android) => {
                let mut ar =
                    ListApiKeys200ResponseApiKeysInnerRestrictionsAndroidKeyRestrictions::new();
                ar.allowed_applications = Some(
                    android
                        .allowed_applications
                        .iter()
                        .map(|a| {
                            let mut app = ListApiKeys200ResponseApiKeysInnerRestrictionsAndroidKeyRestrictionsAllowedApplicationsInner::new();
                            app.package_name = Some(a.package_name.clone());
                            app.sha256_cert_fingerprint = Some(a.sha256_cert_fingerprint.clone());
                            app
                        })
                        .collect(),
                );
                result.android_key_restrictions = Some(Box::new(ar));
            }
            PlatformRestrictions::Ios(ios) => {
                let mut ir =
                    ListApiKeys200ResponseApiKeysInnerRestrictionsIosKeyRestrictions::new();
                ir.allowed_bundle_ids = Some(ios.allowed_bundle_ids.clone());
                result.ios_key_restrictions = Some(Box::new(ir));
            }
        }
    }

    result
}

impl FromJson<CreateApiKey201Response> for CreateApiKeyResponse {
    type Error = ValidationError;

    fn from_json(json: CreateApiKey201Response) -> Result<Self, Self::Error> {
        let api_key = json
            .api_key
            .map(|k| ApiKey::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("api_key"))?;

        Ok(Self { api_key })
    }
}

impl FromJson<GetApiKey200Response> for GetApiKeyResponse {
    type Error = ValidationError;

    fn from_json(json: GetApiKey200Response) -> Result<Self, Self::Error> {
        let api_key = json
            .api_key
            .map(|k| ApiKeyInfo::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("api_key"))?;

        Ok(Self { api_key })
    }
}

impl FromJson<UpdateApiKey200Response> for UpdateApiKeyResponse {
    type Error = ValidationError;

    fn from_json(json: UpdateApiKey200Response) -> Result<Self, Self::Error> {
        let api_key = json
            .api_key
            .map(|k| ApiKeyInfo::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("api_key"))?;

        Ok(Self { api_key })
    }
}

impl FromJson<ListApiKeys200Response> for ListApiKeysResponse {
    type Error = ValidationError;

    fn from_json(json: ListApiKeys200Response) -> Result<Self, Self::Error> {
        let api_keys = json
            .api_keys
            .unwrap_or_default()
            .into_iter()
            .map(ApiKeyInfo::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = json.pagination.map_or_else(
            || PaginationMeta {
                next_cursor: None,
                total_count: None,
            },
            |p| PaginationMeta {
                next_cursor: p.next_cursor.clone(),
                total_count: p.total_count,
            },
        );

        Ok(Self {
            api_keys,
            pagination,
        })
    }
}

impl FromJson<RotateApiKey200Response> for RotateApiKeyResponse {
    type Error = ValidationError;

    fn from_json(json: RotateApiKey200Response) -> Result<Self, Self::Error> {
        let api_key = json
            .api_key
            .map(|k| ApiKey::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("api_key"))?;

        Ok(Self {
            api_key,
            grace_period_expires_at: json
                .grace_period_expires_at
                .and_then(|s| parse_timestamp(Some(s)).ok()),
        })
    }
}

// =============================================================================
// Error Conversions
// =============================================================================

impl From<GenCreateApiKeyError> for CreateApiKeyError {
    fn from(error: GenCreateApiKeyError) -> Self {
        match error {
            GenCreateApiKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenCreateApiKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateApiKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateApiKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenCreateApiKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateApiKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateApiKeyError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenGetApiKeyError> for GetApiKeyError {
    fn from(error: GenGetApiKeyError) -> Self {
        match error {
            GenGetApiKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenGetApiKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetApiKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetApiKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenGetApiKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetApiKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetApiKeyError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenUpdateApiKeyError> for UpdateApiKeyError {
    fn from(error: GenUpdateApiKeyError) -> Self {
        match error {
            GenUpdateApiKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenUpdateApiKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateApiKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateApiKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenUpdateApiKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateApiKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateApiKeyError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenDeleteApiKeyError> for DeleteApiKeyError {
    fn from(error: GenDeleteApiKeyError) -> Self {
        match error {
            GenDeleteApiKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenDeleteApiKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteApiKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteApiKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenDeleteApiKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteApiKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteApiKeyError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenListApiKeysError> for ListApiKeysError {
    fn from(error: GenListApiKeysError) -> Self {
        match error {
            GenListApiKeysError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenListApiKeysError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListApiKeysError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListApiKeysError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenListApiKeysError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListApiKeysError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListApiKeysError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

impl From<GenRotateApiKeyError> for RotateApiKeyError {
    fn from(error: GenRotateApiKeyError) -> Self {
        match error {
            GenRotateApiKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenRotateApiKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRotateApiKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRotateApiKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenRotateApiKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRotateApiKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenRotateApiKeyError::UnknownValue(v) => Self::Unknown(v.to_string()),
        }
    }
}

// =============================================================================
// Helper for error handling
// =============================================================================

/// Extracts `ListTenants400Response` from `GenGetApiKeyError`.
fn extract_get_api_key_error_response(
    entity: GenGetApiKeyError,
) -> Option<crate::generated_admin::models::ListTenants400Response> {
    match entity {
        GenGetApiKeyError::Status400(e)
        | GenGetApiKeyError::Status401(e)
        | GenGetApiKeyError::Status403(e)
        | GenGetApiKeyError::Status404(e)
        | GenGetApiKeyError::Status500(e)
        | GenGetApiKeyError::Status503(e) => Some(e),
        GenGetApiKeyError::UnknownValue(_) => None,
    }
}

fn extract_create_api_key_error_response(
    entity: GenCreateApiKeyError,
) -> Option<crate::generated_admin::models::ListTenants400Response> {
    match entity {
        GenCreateApiKeyError::Status400(e)
        | GenCreateApiKeyError::Status401(e)
        | GenCreateApiKeyError::Status403(e)
        | GenCreateApiKeyError::Status404(e)
        | GenCreateApiKeyError::Status500(e)
        | GenCreateApiKeyError::Status503(e) => Some(e),
        GenCreateApiKeyError::UnknownValue(_) => None,
    }
}

fn extract_update_api_key_error_response(
    entity: GenUpdateApiKeyError,
) -> Option<crate::generated_admin::models::ListTenants400Response> {
    match entity {
        GenUpdateApiKeyError::Status400(e)
        | GenUpdateApiKeyError::Status401(e)
        | GenUpdateApiKeyError::Status403(e)
        | GenUpdateApiKeyError::Status404(e)
        | GenUpdateApiKeyError::Status500(e)
        | GenUpdateApiKeyError::Status503(e) => Some(e),
        GenUpdateApiKeyError::UnknownValue(_) => None,
    }
}

fn extract_delete_api_key_error_response(
    entity: GenDeleteApiKeyError,
) -> Option<crate::generated_admin::models::ListTenants400Response> {
    match entity {
        GenDeleteApiKeyError::Status400(e)
        | GenDeleteApiKeyError::Status401(e)
        | GenDeleteApiKeyError::Status403(e)
        | GenDeleteApiKeyError::Status404(e)
        | GenDeleteApiKeyError::Status500(e)
        | GenDeleteApiKeyError::Status503(e) => Some(e),
        GenDeleteApiKeyError::UnknownValue(_) => None,
    }
}

fn extract_list_api_keys_error_response(
    entity: GenListApiKeysError,
) -> Option<crate::generated_admin::models::ListTenants400Response> {
    match entity {
        GenListApiKeysError::Status400(e)
        | GenListApiKeysError::Status401(e)
        | GenListApiKeysError::Status403(e)
        | GenListApiKeysError::Status404(e)
        | GenListApiKeysError::Status500(e)
        | GenListApiKeysError::Status503(e) => Some(e),
        GenListApiKeysError::UnknownValue(_) => None,
    }
}

fn extract_rotate_api_key_error_response(
    entity: GenRotateApiKeyError,
) -> Option<crate::generated_admin::models::ListTenants400Response> {
    match entity {
        GenRotateApiKeyError::Status400(e)
        | GenRotateApiKeyError::Status401(e)
        | GenRotateApiKeyError::Status403(e)
        | GenRotateApiKeyError::Status404(e)
        | GenRotateApiKeyError::Status500(e)
        | GenRotateApiKeyError::Status503(e) => Some(e),
        GenRotateApiKeyError::UnknownValue(_) => None,
    }
}

fn handle_get_api_key_error(
    e: crate::generated_admin::apis::Error<GenGetApiKeyError>,
) -> GetApiKeyError {
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            // Extract error details from parsed entity, if available
            let error_details = resp
                .entity
                .and_then(extract_get_api_key_error_response)
                .map(convert_error_details)
                .unwrap_or_else(|| fallback_error_details(resp.content.clone()));

            // Use actual HTTP status code to determine error type
            match status {
                400 => GetApiKeyError::BadRequest(error_details),
                401 => GetApiKeyError::Unauthenticated(
                    error_details.message.unwrap_or(error_details.title),
                ),
                403 => GetApiKeyError::PermissionDenied(
                    error_details.message.unwrap_or(error_details.title),
                ),
                404 => GetApiKeyError::NotFound(error_details),
                500..=599 => GetApiKeyError::ServiceError(
                    error_details.message.unwrap_or(error_details.title),
                ),
                _ => GetApiKeyError::Unknown(format!("HTTP {}: {:?}", status, error_details)),
            }
        }
        crate::generated_admin::apis::Error::Reqwest(e) => {
            GetApiKeyError::ServiceUnavailable(e.to_string())
        }
        _ => GetApiKeyError::ServiceError("Unknown error occurred".into()),
    }
}

fn handle_create_api_key_error(
    e: crate::generated_admin::apis::Error<GenCreateApiKeyError>,
) -> CreateApiKeyError {
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            let error_details = resp
                .entity
                .and_then(extract_create_api_key_error_response)
                .map(convert_error_details)
                .unwrap_or_else(|| fallback_error_details(resp.content.clone()));

            match status {
                400 => CreateApiKeyError::BadRequest(error_details),
                401 => CreateApiKeyError::Unauthenticated(
                    error_details.message.unwrap_or(error_details.title),
                ),
                403 => CreateApiKeyError::PermissionDenied(
                    error_details.message.unwrap_or(error_details.title),
                ),
                404 => CreateApiKeyError::NotFound(error_details),
                500..=599 => CreateApiKeyError::ServiceError(
                    error_details.message.unwrap_or(error_details.title),
                ),
                _ => CreateApiKeyError::Unknown(format!("HTTP {}: {:?}", status, error_details)),
            }
        }
        crate::generated_admin::apis::Error::Reqwest(e) => {
            CreateApiKeyError::ServiceUnavailable(e.to_string())
        }
        _ => CreateApiKeyError::ServiceError("Unknown error occurred".into()),
    }
}

fn handle_update_api_key_error(
    e: crate::generated_admin::apis::Error<GenUpdateApiKeyError>,
) -> UpdateApiKeyError {
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            let error_details = resp
                .entity
                .and_then(extract_update_api_key_error_response)
                .map(convert_error_details)
                .unwrap_or_else(|| fallback_error_details(resp.content.clone()));

            match status {
                400 => UpdateApiKeyError::BadRequest(error_details),
                401 => UpdateApiKeyError::Unauthenticated(
                    error_details.message.unwrap_or(error_details.title),
                ),
                403 => UpdateApiKeyError::PermissionDenied(
                    error_details.message.unwrap_or(error_details.title),
                ),
                404 => UpdateApiKeyError::NotFound(error_details),
                500..=599 => UpdateApiKeyError::ServiceError(
                    error_details.message.unwrap_or(error_details.title),
                ),
                _ => UpdateApiKeyError::Unknown(format!("HTTP {}: {:?}", status, error_details)),
            }
        }
        crate::generated_admin::apis::Error::Reqwest(e) => {
            UpdateApiKeyError::ServiceUnavailable(e.to_string())
        }
        _ => UpdateApiKeyError::ServiceError("Unknown error occurred".into()),
    }
}

fn handle_delete_api_key_error(
    e: crate::generated_admin::apis::Error<GenDeleteApiKeyError>,
) -> DeleteApiKeyError {
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            let error_details = resp
                .entity
                .and_then(extract_delete_api_key_error_response)
                .map(convert_error_details)
                .unwrap_or_else(|| fallback_error_details(resp.content.clone()));

            match status {
                400 => DeleteApiKeyError::BadRequest(error_details),
                401 => DeleteApiKeyError::Unauthenticated(
                    error_details.message.unwrap_or(error_details.title),
                ),
                403 => DeleteApiKeyError::PermissionDenied(
                    error_details.message.unwrap_or(error_details.title),
                ),
                404 => DeleteApiKeyError::NotFound(error_details),
                500..=599 => DeleteApiKeyError::ServiceError(
                    error_details.message.unwrap_or(error_details.title),
                ),
                _ => DeleteApiKeyError::Unknown(format!("HTTP {}: {:?}", status, error_details)),
            }
        }
        crate::generated_admin::apis::Error::Reqwest(e) => {
            DeleteApiKeyError::ServiceUnavailable(e.to_string())
        }
        _ => DeleteApiKeyError::ServiceError("Unknown error occurred".into()),
    }
}

fn handle_list_api_keys_error(
    e: crate::generated_admin::apis::Error<GenListApiKeysError>,
) -> ListApiKeysError {
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            let error_details = resp
                .entity
                .and_then(extract_list_api_keys_error_response)
                .map(convert_error_details)
                .unwrap_or_else(|| fallback_error_details(resp.content.clone()));

            match status {
                400 => ListApiKeysError::BadRequest(error_details),
                401 => ListApiKeysError::Unauthenticated(
                    error_details.message.unwrap_or(error_details.title),
                ),
                403 => ListApiKeysError::PermissionDenied(
                    error_details.message.unwrap_or(error_details.title),
                ),
                404 => ListApiKeysError::NotFound(error_details),
                500..=599 => ListApiKeysError::ServiceError(
                    error_details.message.unwrap_or(error_details.title),
                ),
                _ => ListApiKeysError::Unknown(format!("HTTP {}: {:?}", status, error_details)),
            }
        }
        crate::generated_admin::apis::Error::Reqwest(e) => {
            ListApiKeysError::ServiceUnavailable(e.to_string())
        }
        _ => ListApiKeysError::ServiceError("Unknown error occurred".into()),
    }
}

fn handle_rotate_api_key_error(
    e: crate::generated_admin::apis::Error<GenRotateApiKeyError>,
) -> RotateApiKeyError {
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            let error_details = resp
                .entity
                .and_then(extract_rotate_api_key_error_response)
                .map(convert_error_details)
                .unwrap_or_else(|| fallback_error_details(resp.content.clone()));

            match status {
                400 => RotateApiKeyError::BadRequest(error_details),
                401 => RotateApiKeyError::Unauthenticated(
                    error_details.message.unwrap_or(error_details.title),
                ),
                403 => RotateApiKeyError::PermissionDenied(
                    error_details.message.unwrap_or(error_details.title),
                ),
                404 => RotateApiKeyError::NotFound(error_details),
                500..=599 => RotateApiKeyError::ServiceError(
                    error_details.message.unwrap_or(error_details.title),
                ),
                _ => RotateApiKeyError::Unknown(format!("HTTP {}: {:?}", status, error_details)),
            }
        }
        crate::generated_admin::apis::Error::Reqwest(e) => {
            RotateApiKeyError::ServiceUnavailable(e.to_string())
        }
        _ => RotateApiKeyError::ServiceError("Unknown error occurred".into()),
    }
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_create_api_key(
    config: &Configuration,
    req: CreateApiKeyRequest,
) -> Result<CreateApiKeyResponse, CreateApiKeyError> {
    debug!("Creating OpenAPI request for create_api_key");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    let request = GenCreateApiKeyRequest {
        display_name: req.display_name,
        restrictions: req
            .restrictions
            .map(|r| Box::new(convert_restrictions_to_openapi(&r))),
    };

    let response = create_api_key(config, &project_id, &app_id, Some(request))
        .await
        .map_err(|e| {
            error!(?e, "Error in create_api_key response");
            handle_create_api_key_error(e)
        })?;

    CreateApiKeyResponse::from_json(response).map_err(CreateApiKeyError::ResponseValidationError)
}

pub(crate) async fn handle_get_api_key(
    config: &Configuration,
    req: GetApiKeyRequest,
) -> Result<GetApiKeyResponse, GetApiKeyError> {
    debug!("Creating OpenAPI request for get_api_key");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    let response = get_api_key(config, &project_id, &app_id, &req.key_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_api_key response");
            handle_get_api_key_error(e)
        })?;

    GetApiKeyResponse::from_json(response).map_err(GetApiKeyError::ResponseValidationError)
}

pub(crate) async fn handle_update_api_key(
    config: &Configuration,
    req: UpdateApiKeyRequest,
) -> Result<UpdateApiKeyResponse, UpdateApiKeyError> {
    debug!("Creating OpenAPI request for update_api_key");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    let request = GenUpdateApiKeyRequest {
        display_name: req.display_name,
        restrictions: req
            .restrictions
            .map(|r| Box::new(convert_restrictions_to_openapi(&r))),
    };

    let response = update_api_key(config, &project_id, &app_id, &req.key_id, request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_api_key response");
            handle_update_api_key_error(e)
        })?;

    UpdateApiKeyResponse::from_json(response).map_err(UpdateApiKeyError::ResponseValidationError)
}

pub(crate) async fn handle_delete_api_key(
    config: &Configuration,
    req: DeleteApiKeyRequest,
) -> Result<DeleteApiKeyResponse, DeleteApiKeyError> {
    debug!("Creating OpenAPI request for delete_api_key");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    delete_api_key(config, &project_id, &app_id, &req.key_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_api_key response");
            handle_delete_api_key_error(e)
        })?;

    Ok(DeleteApiKeyResponse {})
}

pub(crate) async fn handle_list_api_keys(
    config: &Configuration,
    req: ListApiKeysRequest,
) -> Result<ListApiKeysResponse, ListApiKeysError> {
    debug!("Creating OpenAPI request for list_api_keys");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();
    let sort = format_sort_fields(&req.pagination.sort);

    let response = list_api_keys(
        config,
        &project_id,
        &app_id,
        req.pagination.page_size,
        req.pagination.cursor.as_deref(),
        sort.as_deref(),
    )
    .await
    .map_err(|e| {
        error!(?e, "Error in list_api_keys response");
        handle_list_api_keys_error(e)
    })?;

    ListApiKeysResponse::from_json(response).map_err(ListApiKeysError::ResponseValidationError)
}

pub(crate) async fn handle_rotate_api_key(
    config: &Configuration,
    req: RotateApiKeyRequest,
) -> Result<RotateApiKeyResponse, RotateApiKeyError> {
    debug!("Creating OpenAPI request for rotate_api_key");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();

    let request = GenRotateApiKeyRequest {
        grace_period_seconds: req.grace_period_seconds,
    };

    let response = rotate_api_key(config, &project_id, &app_id, &req.key_id, Some(request))
        .await
        .map_err(|e| {
            error!(?e, "Error in rotate_api_key response");
            handle_rotate_api_key_error(e)
        })?;

    RotateApiKeyResponse::from_json(response).map_err(RotateApiKeyError::ResponseValidationError)
}
