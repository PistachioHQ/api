//! Service account operation handlers for the gRPC client.

use chrono::{DateTime, TimeZone, Utc};
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
use prost_types::Timestamp;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;
use pistachio_api::pistachio::types::v1 as proto_types;

use crate::types::{
    IntoProto, error_details_from_status, pagination_meta_from_proto, pagination_params_to_proto,
};

// =============================================================================
// Proto conversions
// =============================================================================

fn timestamp_to_datetime(ts: Timestamp) -> Option<DateTime<Utc>> {
    let nanos = u32::try_from(ts.nanos).ok()?;
    Utc.timestamp_opt(ts.seconds, nanos).single()
}

fn optional_timestamp_to_datetime(ts: Option<Timestamp>) -> Option<DateTime<Utc>> {
    ts.and_then(timestamp_to_datetime)
}

fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn key_algorithm_from_proto(proto: i32) -> KeyAlgorithm {
    match proto_types::KeyAlgorithm::try_from(proto) {
        Ok(proto_types::KeyAlgorithm::Rsa2048) => KeyAlgorithm::Rsa2048,
        Ok(proto_types::KeyAlgorithm::Rsa4096) => KeyAlgorithm::Rsa4096,
        Ok(proto_types::KeyAlgorithm::EcP256) => KeyAlgorithm::EcP256,
        Ok(proto_types::KeyAlgorithm::EcP384) => KeyAlgorithm::EcP384,
        Ok(proto_types::KeyAlgorithm::Ed25519) => KeyAlgorithm::Ed25519,
        _ => KeyAlgorithm::Unspecified,
    }
}

fn key_algorithm_to_proto(algorithm: KeyAlgorithm) -> i32 {
    match algorithm {
        KeyAlgorithm::Unspecified => proto_types::KeyAlgorithm::Unspecified.into(),
        KeyAlgorithm::Rsa2048 => proto_types::KeyAlgorithm::Rsa2048.into(),
        KeyAlgorithm::Rsa4096 => proto_types::KeyAlgorithm::Rsa4096.into(),
        KeyAlgorithm::EcP256 => proto_types::KeyAlgorithm::EcP256.into(),
        KeyAlgorithm::EcP384 => proto_types::KeyAlgorithm::EcP384.into(),
        KeyAlgorithm::Ed25519 => proto_types::KeyAlgorithm::Ed25519.into(),
    }
}

fn key_origin_from_proto(proto: i32) -> KeyOrigin {
    match proto_types::KeyOrigin::try_from(proto) {
        Ok(proto_types::KeyOrigin::SystemProvided) => KeyOrigin::SystemProvided,
        Ok(proto_types::KeyOrigin::UserProvided) => KeyOrigin::UserProvided,
        _ => KeyOrigin::Unspecified,
    }
}

fn service_account_from_proto(
    proto: proto_types::ServiceAccount,
) -> Result<ServiceAccount, ValidationError> {
    Ok(ServiceAccount {
        name: proto.name,
        service_account_id: proto.service_account_id,
        pistachio_id: proto.pistachio_id,
        display_name: proto.display_name,
        description: if proto.description.is_empty() {
            None
        } else {
            Some(proto.description)
        },
        email: proto.email,
        disabled: proto.disabled,
        created_at: optional_timestamp_to_datetime(proto.created_at),
        updated_at: optional_timestamp_to_datetime(proto.updated_at),
    })
}

fn service_account_key_from_proto(proto: proto_types::ServiceAccountKey) -> ServiceAccountKey {
    ServiceAccountKey {
        name: proto.name,
        key_id: proto.key_id,
        key_algorithm: key_algorithm_from_proto(proto.key_algorithm),
        key_origin: key_origin_from_proto(proto.key_origin),
        valid_after_time: optional_timestamp_to_datetime(proto.valid_after_time),
        valid_before_time: proto.valid_before_time.and_then(timestamp_to_datetime),
        disabled: proto.disabled,
    }
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_create_service_account<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateServiceAccountRequest,
) -> Result<CreateServiceAccountResponse, CreateServiceAccountError> {
    debug!("Creating proto request for create_service_account");

    let request = pistachio_api::pistachio::admin::v1::CreateServiceAccountRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id.unwrap_or_default(),
        display_name: req.display_name,
        description: req.description.unwrap_or_default(),
    };

    let response = client
        .create_service_account(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_service_account response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateServiceAccountError::BadRequest(error_details_from_status(&status))
                }
                Code::AlreadyExists => CreateServiceAccountError::AlreadyExists,
                Code::NotFound => {
                    CreateServiceAccountError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    CreateServiceAccountError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateServiceAccountError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    CreateServiceAccountError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    CreateServiceAccountError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateServiceAccountError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let service_account = response
        .service_account
        .ok_or(ValidationError::MissingField("service_account"))?;
    let service_account = service_account_from_proto(service_account)
        .map_err(CreateServiceAccountError::ResponseValidationError)?;

    Ok(CreateServiceAccountResponse { service_account })
}

pub(crate) async fn handle_get_service_account<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetServiceAccountRequest,
) -> Result<GetServiceAccountResponse, GetServiceAccountError> {
    debug!("Creating proto request for get_service_account");

    let request = pistachio_api::pistachio::admin::v1::GetServiceAccountRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
    };

    let response = client
        .get_service_account(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_service_account response");
            match status.code() {
                Code::InvalidArgument => {
                    GetServiceAccountError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    GetServiceAccountError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    GetServiceAccountError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetServiceAccountError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    GetServiceAccountError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    GetServiceAccountError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetServiceAccountError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let service_account = response
        .service_account
        .ok_or(ValidationError::MissingField("service_account"))?;
    let service_account = service_account_from_proto(service_account)
        .map_err(GetServiceAccountError::ResponseValidationError)?;

    Ok(GetServiceAccountResponse { service_account })
}

pub(crate) async fn handle_update_service_account<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateServiceAccountRequest,
) -> Result<UpdateServiceAccountResponse, UpdateServiceAccountError> {
    debug!("Creating proto request for update_service_account");

    let request = pistachio_api::pistachio::admin::v1::UpdateServiceAccountRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
        display_name: req.display_name,
        description: req.description,
        disabled: req.disabled,
    };

    let response = client
        .update_service_account(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_service_account response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateServiceAccountError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    UpdateServiceAccountError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    UpdateServiceAccountError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateServiceAccountError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    UpdateServiceAccountError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    UpdateServiceAccountError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateServiceAccountError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let service_account = response
        .service_account
        .ok_or(ValidationError::MissingField("service_account"))?;
    let service_account = service_account_from_proto(service_account)
        .map_err(UpdateServiceAccountError::ResponseValidationError)?;

    Ok(UpdateServiceAccountResponse { service_account })
}

pub(crate) async fn handle_delete_service_account<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteServiceAccountRequest,
) -> Result<DeleteServiceAccountResponse, DeleteServiceAccountError> {
    debug!("Creating proto request for delete_service_account");

    let request = pistachio_api::pistachio::admin::v1::DeleteServiceAccountRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
    };

    client
        .delete_service_account(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_service_account response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteServiceAccountError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteServiceAccountError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteServiceAccountError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteServiceAccountError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteServiceAccountError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DeleteServiceAccountError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteServiceAccountError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(DeleteServiceAccountResponse {})
}

pub(crate) async fn handle_list_service_accounts<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListServiceAccountsRequest,
) -> Result<ListServiceAccountsResponse, ListServiceAccountsError> {
    debug!("Creating proto request for list_service_accounts");

    let request = pistachio_api::pistachio::admin::v1::ListServiceAccountsRequest {
        project_id: req.project_id.to_string(),
        pagination: Some(pagination_params_to_proto(req.pagination)),
    };

    let response = client
        .list_service_accounts(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_service_accounts response");
            match status.code() {
                Code::InvalidArgument => {
                    ListServiceAccountsError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListServiceAccountsError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListServiceAccountsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListServiceAccountsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ListServiceAccountsError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ListServiceAccountsError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListServiceAccountsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let service_accounts = response
        .service_accounts
        .into_iter()
        .map(service_account_from_proto)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ListServiceAccountsError::ResponseValidationError)?;

    let pagination = response
        .pagination
        .map(pagination_meta_from_proto)
        .unwrap_or_default();

    Ok(ListServiceAccountsResponse {
        service_accounts,
        pagination,
    })
}

pub(crate) async fn handle_search_service_accounts<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: SearchServiceAccountsRequest,
) -> Result<SearchServiceAccountsResponse, SearchServiceAccountsError> {
    debug!("Creating proto request for search_service_accounts");

    let request = pistachio_api::pistachio::admin::v1::SearchServiceAccountsRequest {
        project_id: req.project_id.to_string(),
        params: Some(req.params.into_proto()),
    };

    let response = client
        .search_service_accounts(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in search_service_accounts response");
            match status.code() {
                Code::InvalidArgument => {
                    SearchServiceAccountsError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    SearchServiceAccountsError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    SearchServiceAccountsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    SearchServiceAccountsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    SearchServiceAccountsError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    SearchServiceAccountsError::ServiceUnavailable(status.message().to_string())
                }
                _ => SearchServiceAccountsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let service_accounts = response
        .service_accounts
        .into_iter()
        .map(service_account_from_proto)
        .collect::<Result<Vec<_>, _>>()
        .map_err(SearchServiceAccountsError::ResponseValidationError)?;

    let pagination = response
        .pagination
        .map(pagination_meta_from_proto)
        .unwrap_or_default();

    Ok(SearchServiceAccountsResponse {
        service_accounts,
        pagination,
    })
}

pub(crate) async fn handle_generate_service_account_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GenerateServiceAccountKeyRequest,
) -> Result<GenerateServiceAccountKeyResponse, GenerateServiceAccountKeyError> {
    debug!("Creating proto request for generate_service_account_key");

    let request = pistachio_api::pistachio::admin::v1::GenerateServiceAccountKeyRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
        key_algorithm: key_algorithm_to_proto(req.key_algorithm),
        valid_before_time: req.valid_before_time.map(datetime_to_timestamp),
    };

    let response = client
        .generate_service_account_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in generate_service_account_key response");
            match status.code() {
                Code::InvalidArgument => {
                    GenerateServiceAccountKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    GenerateServiceAccountKeyError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    GenerateServiceAccountKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GenerateServiceAccountKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    GenerateServiceAccountKeyError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    GenerateServiceAccountKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => GenerateServiceAccountKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let key = response.key.ok_or(ValidationError::MissingField("key"))?;
    let key = service_account_key_from_proto(key);

    Ok(GenerateServiceAccountKeyResponse {
        key,
        private_key_data: response.private_key_data.to_vec(),
        key_file_json: response.key_file_json,
    })
}

pub(crate) async fn handle_list_service_account_keys<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListServiceAccountKeysRequest,
) -> Result<ListServiceAccountKeysResponse, ListServiceAccountKeysError> {
    debug!("Creating proto request for list_service_account_keys");

    let request = pistachio_api::pistachio::admin::v1::ListServiceAccountKeysRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
    };

    let response = client
        .list_service_account_keys(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_service_account_keys response");
            match status.code() {
                Code::InvalidArgument => {
                    ListServiceAccountKeysError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListServiceAccountKeysError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListServiceAccountKeysError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListServiceAccountKeysError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ListServiceAccountKeysError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ListServiceAccountKeysError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListServiceAccountKeysError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let keys = response
        .keys
        .into_iter()
        .map(service_account_key_from_proto)
        .collect();

    Ok(ListServiceAccountKeysResponse { keys })
}

pub(crate) async fn handle_get_service_account_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetServiceAccountKeyRequest,
) -> Result<GetServiceAccountKeyResponse, GetServiceAccountKeyError> {
    debug!("Creating proto request for get_service_account_key");

    let request = pistachio_api::pistachio::admin::v1::GetServiceAccountKeyRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
        key_id: req.key_id,
    };

    let response = client
        .get_service_account_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_service_account_key response");
            match status.code() {
                Code::InvalidArgument => {
                    GetServiceAccountKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    GetServiceAccountKeyError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    GetServiceAccountKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetServiceAccountKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    GetServiceAccountKeyError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    GetServiceAccountKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetServiceAccountKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let key = response.key.ok_or(ValidationError::MissingField("key"))?;
    let key = service_account_key_from_proto(key);

    Ok(GetServiceAccountKeyResponse { key })
}

pub(crate) async fn handle_delete_service_account_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteServiceAccountKeyRequest,
) -> Result<DeleteServiceAccountKeyResponse, DeleteServiceAccountKeyError> {
    debug!("Creating proto request for delete_service_account_key");

    let request = pistachio_api::pistachio::admin::v1::DeleteServiceAccountKeyRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
        key_id: req.key_id,
    };

    client
        .delete_service_account_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_service_account_key response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteServiceAccountKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteServiceAccountKeyError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteServiceAccountKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteServiceAccountKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteServiceAccountKeyError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DeleteServiceAccountKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteServiceAccountKeyError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(DeleteServiceAccountKeyResponse {})
}

pub(crate) async fn handle_disable_service_account_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DisableServiceAccountKeyRequest,
) -> Result<DisableServiceAccountKeyResponse, DisableServiceAccountKeyError> {
    debug!("Creating proto request for disable_service_account_key");

    let request = pistachio_api::pistachio::admin::v1::DisableServiceAccountKeyRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
        key_id: req.key_id,
    };

    let response = client
        .disable_service_account_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in disable_service_account_key response");
            match status.code() {
                Code::InvalidArgument => {
                    DisableServiceAccountKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DisableServiceAccountKeyError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DisableServiceAccountKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DisableServiceAccountKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DisableServiceAccountKeyError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DisableServiceAccountKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => DisableServiceAccountKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let key = response.key.ok_or(ValidationError::MissingField("key"))?;
    let key = service_account_key_from_proto(key);

    Ok(DisableServiceAccountKeyResponse { key })
}

pub(crate) async fn handle_enable_service_account_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: EnableServiceAccountKeyRequest,
) -> Result<EnableServiceAccountKeyResponse, EnableServiceAccountKeyError> {
    debug!("Creating proto request for enable_service_account_key");

    let request = pistachio_api::pistachio::admin::v1::EnableServiceAccountKeyRequest {
        project_id: req.project_id.to_string(),
        service_account_id: req.service_account_id,
        key_id: req.key_id,
    };

    let response = client
        .enable_service_account_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in enable_service_account_key response");
            match status.code() {
                Code::InvalidArgument => {
                    EnableServiceAccountKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    EnableServiceAccountKeyError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    EnableServiceAccountKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    EnableServiceAccountKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    EnableServiceAccountKeyError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    EnableServiceAccountKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => EnableServiceAccountKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let key = response.key.ok_or(ValidationError::MissingField("key"))?;
    let key = service_account_key_from_proto(key);

    Ok(EnableServiceAccountKeyResponse { key })
}
