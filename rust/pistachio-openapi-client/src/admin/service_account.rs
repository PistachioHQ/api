//! Service account operation handlers for the OpenAPI client.

use pistachio_api_common::admin::service_account::{
    CreateServiceAccountError, CreateServiceAccountRequest, CreateServiceAccountResponse,
    DeleteServiceAccountError, DeleteServiceAccountKeyError, DeleteServiceAccountKeyRequest,
    DeleteServiceAccountKeyResponse, DeleteServiceAccountRequest, DeleteServiceAccountResponse,
    DisableServiceAccountKeyError, DisableServiceAccountKeyRequest,
    DisableServiceAccountKeyResponse, EnableServiceAccountKeyError, EnableServiceAccountKeyRequest,
    EnableServiceAccountKeyResponse, GenerateServiceAccountKeyError,
    GenerateServiceAccountKeyRequest, GenerateServiceAccountKeyResponse, GetServiceAccountError,
    GetServiceAccountKeyError, GetServiceAccountKeyRequest, GetServiceAccountKeyResponse,
    GetServiceAccountRequest, GetServiceAccountResponse, KeyAlgorithm, KeyOrigin,
    ListServiceAccountKeysError, ListServiceAccountKeysRequest, ListServiceAccountKeysResponse,
    ListServiceAccountsError, ListServiceAccountsRequest, ListServiceAccountsResponse,
    SearchServiceAccountsError, SearchServiceAccountsRequest, SearchServiceAccountsResponse,
    ServiceAccount, ServiceAccountKey, UpdateServiceAccountError, UpdateServiceAccountRequest,
    UpdateServiceAccountResponse,
};
use pistachio_api_common::error::ValidationError;
use pistachio_api_common::pagination::{PaginationMeta, format_sort_fields};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::service_accounts_api::{
    CreateServiceAccountError as GenCreateError, DeleteServiceAccountError as GenDeleteError,
    DeleteServiceAccountKeyError as GenDeleteKeyError,
    DisableServiceAccountKeyError as GenDisableKeyError,
    EnableServiceAccountKeyError as GenEnableKeyError,
    GenerateServiceAccountKeyError as GenGenerateKeyError, GetServiceAccountError as GenGetError,
    GetServiceAccountKeyError as GenGetKeyError, ListServiceAccountKeysError as GenListKeysError,
    ListServiceAccountsError as GenListError, SearchServiceAccountsError as GenSearchError,
    UpdateServiceAccountError as GenUpdateError, create_service_account, delete_service_account,
    delete_service_account_key, disable_service_account_key, enable_service_account_key,
    generate_service_account_key, get_service_account, get_service_account_key,
    list_service_account_keys, list_service_accounts, search_service_accounts,
    update_service_account,
};
use crate::generated_admin::models::{
    CreateServiceAccount201Response, CreateServiceAccountRequest as GenCreateRequest,
    DisableServiceAccountKey200Response, EnableServiceAccountKey200Response,
    GenerateServiceAccountKey201Response,
    GenerateServiceAccountKeyRequest as GenGenerateKeyRequest, GetServiceAccount200Response,
    GetServiceAccountKey200Response, ListServiceAccountKeys200Response,
    ListServiceAccountKeys200ResponseKeysInner, ListServiceAccounts200Response,
    ListServiceAccounts200ResponseServiceAccountsInner, SearchServiceAccounts200Response,
    SearchServiceAccountsRequest as GenSearchRequest, UpdateServiceAccount200Response,
    UpdateServiceAccountRequest as GenUpdateRequest,
};
use crate::problem_details::fallback_error_details;
use crate::types::{FromJson, convert_error_details, parse_timestamp};

// =============================================================================
// Error conversions
// =============================================================================

impl From<GenCreateError> for CreateServiceAccountError {
    fn from(error: GenCreateError) -> Self {
        match error {
            GenCreateError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenCreateError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenCreateError::Status409(_) => Self::AlreadyExists,
            GenCreateError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenCreateError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenGetError> for GetServiceAccountError {
    fn from(error: GenGetError) -> Self {
        match error {
            GenGetError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenGetError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenGetError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenUpdateError> for UpdateServiceAccountError {
    fn from(error: GenUpdateError) -> Self {
        match error {
            GenUpdateError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenUpdateError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenUpdateError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenUpdateError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenDeleteError> for DeleteServiceAccountError {
    fn from(error: GenDeleteError) -> Self {
        match error {
            GenDeleteError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenDeleteError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenDeleteError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenListError> for ListServiceAccountsError {
    fn from(error: GenListError) -> Self {
        match error {
            GenListError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenListError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenListError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenSearchError> for SearchServiceAccountsError {
    fn from(error: GenSearchError) -> Self {
        match error {
            GenSearchError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenSearchError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenSearchError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenSearchError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenSearchError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenSearchError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenSearchError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenGenerateKeyError> for GenerateServiceAccountKeyError {
    fn from(error: GenGenerateKeyError) -> Self {
        match error {
            GenGenerateKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenGenerateKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGenerateKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGenerateKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenGenerateKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGenerateKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGenerateKeyError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenListKeysError> for ListServiceAccountKeysError {
    fn from(error: GenListKeysError) -> Self {
        match error {
            GenListKeysError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenListKeysError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListKeysError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListKeysError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenListKeysError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListKeysError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenListKeysError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenGetKeyError> for GetServiceAccountKeyError {
    fn from(error: GenGetKeyError) -> Self {
        match error {
            GenGetKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenGetKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenGetKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenGetKeyError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenDeleteKeyError> for DeleteServiceAccountKeyError {
    fn from(error: GenDeleteKeyError) -> Self {
        match error {
            GenDeleteKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenDeleteKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenDeleteKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDeleteKeyError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenDisableKeyError> for DisableServiceAccountKeyError {
    fn from(error: GenDisableKeyError) -> Self {
        match error {
            GenDisableKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenDisableKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDisableKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDisableKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenDisableKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDisableKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenDisableKeyError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

impl From<GenEnableKeyError> for EnableServiceAccountKeyError {
    fn from(error: GenEnableKeyError) -> Self {
        match error {
            GenEnableKeyError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenEnableKeyError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenEnableKeyError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenEnableKeyError::Status404(e) => Self::NotFound(convert_error_details(e)),
            GenEnableKeyError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenEnableKeyError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenEnableKeyError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<ListServiceAccounts200ResponseServiceAccountsInner> for ServiceAccount {
    type Error = ValidationError;

    fn from_json(
        json: ListServiceAccounts200ResponseServiceAccountsInner,
    ) -> Result<Self, Self::Error> {
        let created_at = parse_timestamp(Some(json.created_at)).ok();
        let updated_at = parse_timestamp(Some(json.updated_at)).ok();

        Ok(Self {
            name: json.name,
            service_account_id: json.service_account_id,
            pistachio_id: json.pistachio_id.unwrap_or_default(),
            display_name: json.display_name,
            description: json.description.flatten(),
            email: json.email.unwrap_or_default(),
            disabled: json.disabled,
            created_at,
            updated_at,
        })
    }
}

impl FromJson<CreateServiceAccount201Response> for CreateServiceAccountResponse {
    type Error = ValidationError;

    fn from_json(json: CreateServiceAccount201Response) -> Result<Self, Self::Error> {
        let service_account = json
            .service_account
            .map(|sa| ServiceAccount::from_json(*sa))
            .transpose()?
            .ok_or(ValidationError::MissingField("service_account"))?;

        Ok(Self { service_account })
    }
}

impl FromJson<GetServiceAccount200Response> for GetServiceAccountResponse {
    type Error = ValidationError;

    fn from_json(json: GetServiceAccount200Response) -> Result<Self, Self::Error> {
        let service_account = json
            .service_account
            .map(|sa| ServiceAccount::from_json(*sa))
            .transpose()?
            .ok_or(ValidationError::MissingField("service_account"))?;

        Ok(Self { service_account })
    }
}

impl FromJson<UpdateServiceAccount200Response> for UpdateServiceAccountResponse {
    type Error = ValidationError;

    fn from_json(json: UpdateServiceAccount200Response) -> Result<Self, Self::Error> {
        let service_account = json
            .service_account
            .map(|sa| ServiceAccount::from_json(*sa))
            .transpose()?
            .ok_or(ValidationError::MissingField("service_account"))?;

        Ok(Self { service_account })
    }
}

impl FromJson<ListServiceAccounts200Response> for ListServiceAccountsResponse {
    type Error = ValidationError;

    fn from_json(json: ListServiceAccounts200Response) -> Result<Self, Self::Error> {
        let service_accounts = json
            .service_accounts
            .unwrap_or_default()
            .into_iter()
            .map(ServiceAccount::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = json
            .pagination
            .map(|p| PaginationMeta {
                next_cursor: p.next_cursor,
                total_count: p.total_count,
            })
            .unwrap_or_default();

        Ok(Self {
            service_accounts,
            pagination,
        })
    }
}

impl FromJson<SearchServiceAccounts200Response> for SearchServiceAccountsResponse {
    type Error = ValidationError;

    fn from_json(json: SearchServiceAccounts200Response) -> Result<Self, Self::Error> {
        let service_accounts = json
            .service_accounts
            .unwrap_or_default()
            .into_iter()
            .map(ServiceAccount::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = json
            .pagination
            .map(|p| PaginationMeta {
                next_cursor: p.next_cursor,
                total_count: p.total_count,
            })
            .unwrap_or_default();

        Ok(Self {
            service_accounts,
            pagination,
        })
    }
}

impl FromJson<ListServiceAccountKeys200ResponseKeysInner> for ServiceAccountKey {
    type Error = ValidationError;

    fn from_json(json: ListServiceAccountKeys200ResponseKeysInner) -> Result<Self, Self::Error> {
        use crate::generated_admin::models::list_service_account_keys_200_response_keys_inner::{
            KeyAlgorithm as GenKeyAlgorithm, KeyOrigin as GenKeyOrigin,
        };

        let key_algorithm = match json.key_algorithm {
            GenKeyAlgorithm::Rsa2048 => KeyAlgorithm::Rsa2048,
            GenKeyAlgorithm::Rsa4096 => KeyAlgorithm::Rsa4096,
            GenKeyAlgorithm::EcP256 => KeyAlgorithm::EcP256,
            GenKeyAlgorithm::EcP384 => KeyAlgorithm::EcP384,
            GenKeyAlgorithm::Ed25519 => KeyAlgorithm::Ed25519,
        };

        let key_origin = json
            .key_origin
            .map(|o| match o {
                GenKeyOrigin::SystemProvided => KeyOrigin::SystemProvided,
                GenKeyOrigin::UserProvided => KeyOrigin::UserProvided,
            })
            .unwrap_or(KeyOrigin::SystemProvided);

        let valid_after_time = parse_timestamp(Some(json.valid_after_time)).ok();
        let valid_before_time = json
            .valid_before_time
            .flatten()
            .and_then(|s| parse_timestamp(Some(s)).ok());

        Ok(Self {
            name: json.name,
            key_id: json.key_id,
            key_algorithm,
            key_origin,
            valid_after_time,
            valid_before_time,
            disabled: json.disabled.unwrap_or(false),
        })
    }
}

impl FromJson<GenerateServiceAccountKey201Response> for GenerateServiceAccountKeyResponse {
    type Error = ValidationError;

    fn from_json(json: GenerateServiceAccountKey201Response) -> Result<Self, Self::Error> {
        let key = json
            .key
            .map(|k| ServiceAccountKey::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("key"))?;

        // Pass through the decoded private key bytes
        let private_key_data = json.private_key_data.unwrap_or_default();

        Ok(Self {
            key,
            private_key_data,
            key_file_json: json.key_file_json.unwrap_or_default(),
        })
    }
}

impl FromJson<ListServiceAccountKeys200Response> for ListServiceAccountKeysResponse {
    type Error = ValidationError;

    fn from_json(json: ListServiceAccountKeys200Response) -> Result<Self, Self::Error> {
        let keys = json
            .keys
            .unwrap_or_default()
            .into_iter()
            .map(ServiceAccountKey::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { keys })
    }
}

impl FromJson<GetServiceAccountKey200Response> for GetServiceAccountKeyResponse {
    type Error = ValidationError;

    fn from_json(json: GetServiceAccountKey200Response) -> Result<Self, Self::Error> {
        let key = json
            .key
            .map(|k| ServiceAccountKey::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("key"))?;

        Ok(Self { key })
    }
}

impl FromJson<DisableServiceAccountKey200Response> for DisableServiceAccountKeyResponse {
    type Error = ValidationError;

    fn from_json(json: DisableServiceAccountKey200Response) -> Result<Self, Self::Error> {
        let key = json
            .key
            .map(|k| ServiceAccountKey::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("key"))?;

        Ok(Self { key })
    }
}

impl FromJson<EnableServiceAccountKey200Response> for EnableServiceAccountKeyResponse {
    type Error = ValidationError;

    fn from_json(json: EnableServiceAccountKey200Response) -> Result<Self, Self::Error> {
        let key = json
            .key
            .map(|k| ServiceAccountKey::from_json(*k))
            .transpose()?
            .ok_or(ValidationError::MissingField("key"))?;

        Ok(Self { key })
    }
}

// =============================================================================
// Helper for error handling
// =============================================================================

fn handle_api_error<E, T>(
    e: crate::generated_admin::apis::Error<E>,
    convert_entity: impl Fn(E) -> T,
    fallback_fn: impl Fn(u16, String) -> T,
    reqwest_error_fn: impl Fn(String) -> T,
    default_error_fn: impl Fn() -> T,
) -> T
where
    T: std::fmt::Debug,
{
    match e {
        crate::generated_admin::apis::Error::ResponseError(resp) => {
            let status = resp.status.as_u16();

            // Try entity parsing if available
            if let Some(entity) = resp.entity {
                return convert_entity(entity);
            }

            // Last resort: status code mapping with raw content
            fallback_fn(status, resp.content)
        }
        crate::generated_admin::apis::Error::Reqwest(e) => reqwest_error_fn(e.to_string()),
        _ => default_error_fn(),
    }
}

// =============================================================================
// Handlers
// =============================================================================

pub(crate) async fn handle_create_service_account(
    config: &Configuration,
    req: CreateServiceAccountRequest,
) -> Result<CreateServiceAccountResponse, CreateServiceAccountError> {
    debug!("Creating OpenAPI request for create_service_account");

    let request = GenCreateRequest {
        service_account_id: req.service_account_id,
        display_name: req.display_name,
        description: req.description,
    };

    let project_id = req.project_id.to_string();
    debug!(?request, "Sending create_service_account request");

    let response = create_service_account(config, &project_id, request)
        .await
        .map_err(|e| {
            error!(?e, "Error in create_service_account response");
            handle_api_error(
                e,
                CreateServiceAccountError::from,
                |status, content| match status {
                    400 => CreateServiceAccountError::BadRequest(fallback_error_details(content)),
                    401 => CreateServiceAccountError::Unauthenticated(content),
                    403 => CreateServiceAccountError::PermissionDenied(content),
                    404 => CreateServiceAccountError::NotFound(fallback_error_details(content)),
                    409 => CreateServiceAccountError::AlreadyExists,
                    500..=599 => CreateServiceAccountError::ServiceError(content),
                    _ => {
                        CreateServiceAccountError::Unknown(format!("HTTP {}: {}", status, content))
                    }
                },
                CreateServiceAccountError::ServiceUnavailable,
                || CreateServiceAccountError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    CreateServiceAccountResponse::from_json(response)
        .map_err(CreateServiceAccountError::ResponseValidationError)
}

pub(crate) async fn handle_get_service_account(
    config: &Configuration,
    req: GetServiceAccountRequest,
) -> Result<GetServiceAccountResponse, GetServiceAccountError> {
    debug!("Creating OpenAPI request for get_service_account");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;

    let response = get_service_account(config, &project_id, service_account_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_service_account response");
            handle_api_error(
                e,
                GetServiceAccountError::from,
                |status, content| match status {
                    400 => GetServiceAccountError::BadRequest(fallback_error_details(content)),
                    401 => GetServiceAccountError::Unauthenticated(content),
                    403 => GetServiceAccountError::PermissionDenied(content),
                    404 => GetServiceAccountError::NotFound(fallback_error_details(content)),
                    500..=599 => GetServiceAccountError::ServiceError(content),
                    _ => GetServiceAccountError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                GetServiceAccountError::ServiceUnavailable,
                || GetServiceAccountError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    GetServiceAccountResponse::from_json(response)
        .map_err(GetServiceAccountError::ResponseValidationError)
}

pub(crate) async fn handle_update_service_account(
    config: &Configuration,
    req: UpdateServiceAccountRequest,
) -> Result<UpdateServiceAccountResponse, UpdateServiceAccountError> {
    debug!("Creating OpenAPI request for update_service_account");

    let request = GenUpdateRequest {
        display_name: req.display_name,
        description: req.description.map(Some),
        disabled: req.disabled,
    };

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;

    let response = update_service_account(config, &project_id, service_account_id, request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_service_account response");
            handle_api_error(
                e,
                UpdateServiceAccountError::from,
                |status, content| match status {
                    400 => UpdateServiceAccountError::BadRequest(fallback_error_details(content)),
                    401 => UpdateServiceAccountError::Unauthenticated(content),
                    403 => UpdateServiceAccountError::PermissionDenied(content),
                    404 => UpdateServiceAccountError::NotFound(fallback_error_details(content)),
                    500..=599 => UpdateServiceAccountError::ServiceError(content),
                    _ => {
                        UpdateServiceAccountError::Unknown(format!("HTTP {}: {}", status, content))
                    }
                },
                UpdateServiceAccountError::ServiceUnavailable,
                || UpdateServiceAccountError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    UpdateServiceAccountResponse::from_json(response)
        .map_err(UpdateServiceAccountError::ResponseValidationError)
}

pub(crate) async fn handle_delete_service_account(
    config: &Configuration,
    req: DeleteServiceAccountRequest,
) -> Result<DeleteServiceAccountResponse, DeleteServiceAccountError> {
    debug!("Creating OpenAPI request for delete_service_account");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;

    delete_service_account(config, &project_id, service_account_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_service_account response");
            handle_api_error(
                e,
                DeleteServiceAccountError::from,
                |status, content| match status {
                    400 => DeleteServiceAccountError::BadRequest(fallback_error_details(content)),
                    401 => DeleteServiceAccountError::Unauthenticated(content),
                    403 => DeleteServiceAccountError::PermissionDenied(content),
                    404 => DeleteServiceAccountError::NotFound(fallback_error_details(content)),
                    500..=599 => DeleteServiceAccountError::ServiceError(content),
                    _ => {
                        DeleteServiceAccountError::Unknown(format!("HTTP {}: {}", status, content))
                    }
                },
                DeleteServiceAccountError::ServiceUnavailable,
                || DeleteServiceAccountError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    Ok(DeleteServiceAccountResponse {})
}

pub(crate) async fn handle_list_service_accounts(
    config: &Configuration,
    req: ListServiceAccountsRequest,
) -> Result<ListServiceAccountsResponse, ListServiceAccountsError> {
    debug!("Creating OpenAPI request for list_service_accounts");

    let project_id = req.project_id.to_string();
    let page_size = req.pagination.page_size;
    let cursor = req.pagination.cursor.as_deref();
    let sort = format_sort_fields(&req.pagination.sort);

    let response = list_service_accounts(config, &project_id, page_size, cursor, sort.as_deref())
        .await
        .map_err(|e| {
            error!(?e, "Error in list_service_accounts response");
            handle_api_error(
                e,
                ListServiceAccountsError::from,
                |status, content| match status {
                    400 => ListServiceAccountsError::BadRequest(fallback_error_details(content)),
                    401 => ListServiceAccountsError::Unauthenticated(content),
                    403 => ListServiceAccountsError::PermissionDenied(content),
                    404 => ListServiceAccountsError::NotFound(fallback_error_details(content)),
                    500..=599 => ListServiceAccountsError::ServiceError(content),
                    _ => ListServiceAccountsError::Unknown(format!("HTTP {}: {}", status, content)),
                },
                ListServiceAccountsError::ServiceUnavailable,
                || ListServiceAccountsError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    ListServiceAccountsResponse::from_json(response)
        .map_err(ListServiceAccountsError::ResponseValidationError)
}

pub(crate) async fn handle_search_service_accounts(
    config: &Configuration,
    req: SearchServiceAccountsRequest,
) -> Result<SearchServiceAccountsResponse, SearchServiceAccountsError> {
    debug!("Creating OpenAPI request for search_service_accounts");

    let project_id = req.project_id.to_string();
    let sort = format_sort_fields(&req.params.pagination.sort);

    let request = GenSearchRequest {
        query: Some(req.params.query),
        filter: None, // SearchParams doesn't support filter
        page_size: req.params.pagination.page_size,
        cursor: req.params.pagination.cursor.clone(),
        sort,
    };

    let response = search_service_accounts(config, &project_id, request)
        .await
        .map_err(|e| {
            error!(?e, "Error in search_service_accounts response");
            handle_api_error(
                e,
                SearchServiceAccountsError::from,
                |status, content| match status {
                    400 => SearchServiceAccountsError::BadRequest(fallback_error_details(content)),
                    401 => SearchServiceAccountsError::Unauthenticated(content),
                    403 => SearchServiceAccountsError::PermissionDenied(content),
                    404 => SearchServiceAccountsError::NotFound(fallback_error_details(content)),
                    500..=599 => SearchServiceAccountsError::ServiceError(content),
                    _ => {
                        SearchServiceAccountsError::Unknown(format!("HTTP {}: {}", status, content))
                    }
                },
                SearchServiceAccountsError::ServiceUnavailable,
                || SearchServiceAccountsError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    SearchServiceAccountsResponse::from_json(response)
        .map_err(SearchServiceAccountsError::ResponseValidationError)
}

pub(crate) async fn handle_generate_service_account_key(
    config: &Configuration,
    req: GenerateServiceAccountKeyRequest,
) -> Result<GenerateServiceAccountKeyResponse, GenerateServiceAccountKeyError> {
    use crate::generated_admin::models::generate_service_account_key_request::KeyAlgorithm as GenKeyAlgorithm;

    debug!("Creating OpenAPI request for generate_service_account_key");

    let key_algorithm = match req.key_algorithm {
        KeyAlgorithm::Unspecified | KeyAlgorithm::Rsa2048 => Some(GenKeyAlgorithm::Rsa2048),
        KeyAlgorithm::Rsa4096 => Some(GenKeyAlgorithm::Rsa4096),
        KeyAlgorithm::EcP256 => Some(GenKeyAlgorithm::EcP256),
        KeyAlgorithm::EcP384 => Some(GenKeyAlgorithm::EcP384),
        KeyAlgorithm::Ed25519 => Some(GenKeyAlgorithm::Ed25519),
    };

    let request = GenGenerateKeyRequest {
        key_algorithm,
        valid_before_time: req.valid_before_time.map(|t| t.to_rfc3339()),
    };

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;

    let response =
        generate_service_account_key(config, &project_id, service_account_id, Some(request))
            .await
            .map_err(|e| {
                error!(?e, "Error in generate_service_account_key response");
                handle_api_error(
                    e,
                    GenerateServiceAccountKeyError::from,
                    |status, content| match status {
                        400 => GenerateServiceAccountKeyError::BadRequest(fallback_error_details(
                            content,
                        )),
                        401 => GenerateServiceAccountKeyError::Unauthenticated(content),
                        403 => GenerateServiceAccountKeyError::PermissionDenied(content),
                        404 => GenerateServiceAccountKeyError::NotFound(fallback_error_details(
                            content,
                        )),
                        500..=599 => GenerateServiceAccountKeyError::ServiceError(content),
                        _ => GenerateServiceAccountKeyError::Unknown(format!(
                            "HTTP {}: {}",
                            status, content
                        )),
                    },
                    GenerateServiceAccountKeyError::ServiceUnavailable,
                    || {
                        GenerateServiceAccountKeyError::ServiceError(
                            "Unknown error occurred".into(),
                        )
                    },
                )
            })?;

    GenerateServiceAccountKeyResponse::from_json(response)
        .map_err(GenerateServiceAccountKeyError::ResponseValidationError)
}

pub(crate) async fn handle_list_service_account_keys(
    config: &Configuration,
    req: ListServiceAccountKeysRequest,
) -> Result<ListServiceAccountKeysResponse, ListServiceAccountKeysError> {
    debug!("Creating OpenAPI request for list_service_account_keys");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;

    let response = list_service_account_keys(config, &project_id, service_account_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in list_service_account_keys response");
            handle_api_error(
                e,
                ListServiceAccountKeysError::from,
                |status, content| match status {
                    400 => ListServiceAccountKeysError::BadRequest(fallback_error_details(content)),
                    401 => ListServiceAccountKeysError::Unauthenticated(content),
                    403 => ListServiceAccountKeysError::PermissionDenied(content),
                    404 => ListServiceAccountKeysError::NotFound(fallback_error_details(content)),
                    500..=599 => ListServiceAccountKeysError::ServiceError(content),
                    _ => ListServiceAccountKeysError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                ListServiceAccountKeysError::ServiceUnavailable,
                || ListServiceAccountKeysError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    ListServiceAccountKeysResponse::from_json(response)
        .map_err(ListServiceAccountKeysError::ResponseValidationError)
}

pub(crate) async fn handle_get_service_account_key(
    config: &Configuration,
    req: GetServiceAccountKeyRequest,
) -> Result<GetServiceAccountKeyResponse, GetServiceAccountKeyError> {
    debug!("Creating OpenAPI request for get_service_account_key");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;
    let key_id = &req.key_id;

    let response = get_service_account_key(config, &project_id, service_account_id, key_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in get_service_account_key response");
            handle_api_error(
                e,
                GetServiceAccountKeyError::from,
                |status, content| match status {
                    400 => GetServiceAccountKeyError::BadRequest(fallback_error_details(content)),
                    401 => GetServiceAccountKeyError::Unauthenticated(content),
                    403 => GetServiceAccountKeyError::PermissionDenied(content),
                    404 => GetServiceAccountKeyError::NotFound(fallback_error_details(content)),
                    500..=599 => GetServiceAccountKeyError::ServiceError(content),
                    _ => {
                        GetServiceAccountKeyError::Unknown(format!("HTTP {}: {}", status, content))
                    }
                },
                GetServiceAccountKeyError::ServiceUnavailable,
                || GetServiceAccountKeyError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    GetServiceAccountKeyResponse::from_json(response)
        .map_err(GetServiceAccountKeyError::ResponseValidationError)
}

pub(crate) async fn handle_delete_service_account_key(
    config: &Configuration,
    req: DeleteServiceAccountKeyRequest,
) -> Result<DeleteServiceAccountKeyResponse, DeleteServiceAccountKeyError> {
    debug!("Creating OpenAPI request for delete_service_account_key");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;
    let key_id = &req.key_id;

    delete_service_account_key(config, &project_id, service_account_id, key_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in delete_service_account_key response");
            handle_api_error(
                e,
                DeleteServiceAccountKeyError::from,
                |status, content| match status {
                    400 => {
                        DeleteServiceAccountKeyError::BadRequest(fallback_error_details(content))
                    }
                    401 => DeleteServiceAccountKeyError::Unauthenticated(content),
                    403 => DeleteServiceAccountKeyError::PermissionDenied(content),
                    404 => DeleteServiceAccountKeyError::NotFound(fallback_error_details(content)),
                    500..=599 => DeleteServiceAccountKeyError::ServiceError(content),
                    _ => DeleteServiceAccountKeyError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                DeleteServiceAccountKeyError::ServiceUnavailable,
                || DeleteServiceAccountKeyError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    Ok(DeleteServiceAccountKeyResponse {})
}

pub(crate) async fn handle_disable_service_account_key(
    config: &Configuration,
    req: DisableServiceAccountKeyRequest,
) -> Result<DisableServiceAccountKeyResponse, DisableServiceAccountKeyError> {
    debug!("Creating OpenAPI request for disable_service_account_key");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;
    let key_id = &req.key_id;

    let response = disable_service_account_key(config, &project_id, service_account_id, key_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in disable_service_account_key response");
            handle_api_error(
                e,
                DisableServiceAccountKeyError::from,
                |status, content| match status {
                    400 => {
                        DisableServiceAccountKeyError::BadRequest(fallback_error_details(content))
                    }
                    401 => DisableServiceAccountKeyError::Unauthenticated(content),
                    403 => DisableServiceAccountKeyError::PermissionDenied(content),
                    404 => DisableServiceAccountKeyError::NotFound(fallback_error_details(content)),
                    500..=599 => DisableServiceAccountKeyError::ServiceError(content),
                    _ => DisableServiceAccountKeyError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                DisableServiceAccountKeyError::ServiceUnavailable,
                || DisableServiceAccountKeyError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    DisableServiceAccountKeyResponse::from_json(response)
        .map_err(DisableServiceAccountKeyError::ResponseValidationError)
}

pub(crate) async fn handle_enable_service_account_key(
    config: &Configuration,
    req: EnableServiceAccountKeyRequest,
) -> Result<EnableServiceAccountKeyResponse, EnableServiceAccountKeyError> {
    debug!("Creating OpenAPI request for enable_service_account_key");

    let project_id = req.project_id.to_string();
    let service_account_id = &req.service_account_id;
    let key_id = &req.key_id;

    let response = enable_service_account_key(config, &project_id, service_account_id, key_id)
        .await
        .map_err(|e| {
            error!(?e, "Error in enable_service_account_key response");
            handle_api_error(
                e,
                EnableServiceAccountKeyError::from,
                |status, content| match status {
                    400 => {
                        EnableServiceAccountKeyError::BadRequest(fallback_error_details(content))
                    }
                    401 => EnableServiceAccountKeyError::Unauthenticated(content),
                    403 => EnableServiceAccountKeyError::PermissionDenied(content),
                    404 => EnableServiceAccountKeyError::NotFound(fallback_error_details(content)),
                    500..=599 => EnableServiceAccountKeyError::ServiceError(content),
                    _ => EnableServiceAccountKeyError::Unknown(format!(
                        "HTTP {}: {}",
                        status, content
                    )),
                },
                EnableServiceAccountKeyError::ServiceUnavailable,
                || EnableServiceAccountKeyError::ServiceError("Unknown error occurred".into()),
            )
        })?;

    EnableServiceAccountKeyResponse::from_json(response)
        .map_err(EnableServiceAccountKeyError::ResponseValidationError)
}
