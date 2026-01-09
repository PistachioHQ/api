//! API key operation handlers for the gRPC client.

use chrono::{DateTime, TimeZone, Utc};
use pistachio_api_common::admin::api_key::{
    AndroidApplication, AndroidKeyRestrictions, ApiKey, ApiKeyInfo, ApiKeyRestrictions,
    BrowserKeyRestrictions, CreateApiKeyError, CreateApiKeyRequest, CreateApiKeyResponse,
    DeleteApiKeyError, DeleteApiKeyRequest, DeleteApiKeyResponse, GetApiKeyError, GetApiKeyRequest,
    GetApiKeyResponse, IosKeyRestrictions, ListApiKeysError, ListApiKeysRequest,
    ListApiKeysResponse, PlatformRestrictions, RotateApiKeyError, RotateApiKeyRequest,
    RotateApiKeyResponse, ServerKeyRestrictions, UpdateApiKeyError, UpdateApiKeyRequest,
    UpdateApiKeyResponse,
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
    error_details_from_status, pagination_meta_from_proto, pagination_params_to_proto,
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

fn restrictions_from_proto(
    proto: Option<proto_types::ApiKeyRestrictions>,
) -> Option<ApiKeyRestrictions> {
    let proto = proto?;
    let platform_restrictions = proto.platform_restrictions.map(|pr| {
        use proto_types::api_key_restrictions::PlatformRestrictions as ProtoPR;
        match pr {
            ProtoPR::BrowserKeyRestrictions(b) => {
                PlatformRestrictions::Browser(BrowserKeyRestrictions {
                    allowed_referrers: b.allowed_referrers,
                })
            }
            ProtoPR::ServerKeyRestrictions(s) => {
                PlatformRestrictions::Server(ServerKeyRestrictions {
                    allowed_ips: s.allowed_ips,
                })
            }
            ProtoPR::AndroidKeyRestrictions(a) => {
                PlatformRestrictions::Android(AndroidKeyRestrictions {
                    allowed_applications: a
                        .allowed_applications
                        .into_iter()
                        .map(|app| AndroidApplication {
                            package_name: app.package_name,
                            sha256_cert_fingerprint: app.sha256_cert_fingerprint,
                        })
                        .collect(),
                })
            }
            ProtoPR::IosKeyRestrictions(i) => PlatformRestrictions::Ios(IosKeyRestrictions {
                allowed_bundle_ids: i.allowed_bundle_ids,
            }),
        }
    });
    Some(ApiKeyRestrictions {
        platform_restrictions,
    })
}

fn restrictions_to_proto(
    restrictions: Option<ApiKeyRestrictions>,
) -> Option<proto_types::ApiKeyRestrictions> {
    let restrictions = restrictions?;
    let platform_restrictions = restrictions.platform_restrictions.map(|pr| {
        use proto_types::api_key_restrictions::PlatformRestrictions as ProtoPR;
        match pr {
            PlatformRestrictions::Browser(b) => {
                ProtoPR::BrowserKeyRestrictions(proto_types::BrowserKeyRestrictions {
                    allowed_referrers: b.allowed_referrers,
                })
            }
            PlatformRestrictions::Server(s) => {
                ProtoPR::ServerKeyRestrictions(proto_types::ServerKeyRestrictions {
                    allowed_ips: s.allowed_ips,
                })
            }
            PlatformRestrictions::Android(a) => {
                ProtoPR::AndroidKeyRestrictions(proto_types::AndroidKeyRestrictions {
                    allowed_applications: a
                        .allowed_applications
                        .into_iter()
                        .map(|app| proto_types::AndroidApplication {
                            package_name: app.package_name,
                            sha256_cert_fingerprint: app.sha256_cert_fingerprint,
                        })
                        .collect(),
                })
            }
            PlatformRestrictions::Ios(i) => {
                ProtoPR::IosKeyRestrictions(proto_types::IosKeyRestrictions {
                    allowed_bundle_ids: i.allowed_bundle_ids,
                })
            }
        }
    });
    Some(proto_types::ApiKeyRestrictions {
        platform_restrictions,
    })
}

fn api_key_from_proto(proto: proto_types::ApiKey) -> Result<ApiKey, ValidationError> {
    Ok(ApiKey {
        name: proto.name,
        key_id: proto.key_id,
        key_string: proto.key_string,
        display_name: if proto.display_name.is_empty() {
            None
        } else {
            Some(proto.display_name)
        },
        restrictions: restrictions_from_proto(proto.restrictions),
        created_at: optional_timestamp_to_datetime(proto.created_at),
        updated_at: proto.updated_at.and_then(timestamp_to_datetime),
    })
}

fn api_key_info_from_proto(proto: proto_types::ApiKeyInfo) -> Result<ApiKeyInfo, ValidationError> {
    Ok(ApiKeyInfo {
        name: proto.name,
        key_id: proto.key_id,
        key_prefix: proto.key_prefix,
        display_name: if proto.display_name.is_empty() {
            None
        } else {
            Some(proto.display_name)
        },
        restrictions: restrictions_from_proto(proto.restrictions),
        created_at: optional_timestamp_to_datetime(proto.created_at),
        updated_at: proto.updated_at.and_then(timestamp_to_datetime),
    })
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_create_api_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateApiKeyRequest,
) -> Result<CreateApiKeyResponse, CreateApiKeyError> {
    debug!("Creating proto request for create_api_key");

    let request = pistachio_api::pistachio::admin::v1::CreateApiKeyRequest {
        project_id: req.project_id.to_string(),
        app_id: req.app_id.to_string(),
        display_name: req.display_name.unwrap_or_default(),
        restrictions: restrictions_to_proto(req.restrictions),
    };

    let response = client
        .create_api_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_api_key response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateApiKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => CreateApiKeyError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    CreateApiKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateApiKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => CreateApiKeyError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    CreateApiKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateApiKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let api_key = response
        .api_key
        .ok_or(ValidationError::MissingField("api_key"))?;
    let api_key =
        api_key_from_proto(api_key).map_err(CreateApiKeyError::ResponseValidationError)?;

    Ok(CreateApiKeyResponse { api_key })
}

pub(crate) async fn handle_get_api_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: GetApiKeyRequest,
) -> Result<GetApiKeyResponse, GetApiKeyError> {
    debug!("Creating proto request for get_api_key");

    let request = pistachio_api::pistachio::admin::v1::GetApiKeyRequest {
        project_id: req.project_id.to_string(),
        app_id: req.app_id.to_string(),
        key_id: req.key_id,
    };

    let response = client
        .get_api_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in get_api_key response");
            match status.code() {
                Code::InvalidArgument => {
                    GetApiKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => GetApiKeyError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    GetApiKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    GetApiKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => GetApiKeyError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    GetApiKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => GetApiKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let api_key = response
        .api_key
        .ok_or(ValidationError::MissingField("api_key"))?;
    let api_key =
        api_key_info_from_proto(api_key).map_err(GetApiKeyError::ResponseValidationError)?;

    Ok(GetApiKeyResponse { api_key })
}

pub(crate) async fn handle_update_api_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: UpdateApiKeyRequest,
) -> Result<UpdateApiKeyResponse, UpdateApiKeyError> {
    debug!("Creating proto request for update_api_key");

    let request = pistachio_api::pistachio::admin::v1::UpdateApiKeyRequest {
        project_id: req.project_id.to_string(),
        app_id: req.app_id.to_string(),
        key_id: req.key_id,
        display_name: req.display_name,
        restrictions: req
            .restrictions
            .and_then(|r| restrictions_to_proto(Some(r))),
    };

    let response = client
        .update_api_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in update_api_key response");
            match status.code() {
                Code::InvalidArgument => {
                    UpdateApiKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => UpdateApiKeyError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    UpdateApiKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    UpdateApiKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => UpdateApiKeyError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    UpdateApiKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => UpdateApiKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let api_key = response
        .api_key
        .ok_or(ValidationError::MissingField("api_key"))?;
    let api_key =
        api_key_info_from_proto(api_key).map_err(UpdateApiKeyError::ResponseValidationError)?;

    Ok(UpdateApiKeyResponse { api_key })
}

pub(crate) async fn handle_delete_api_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteApiKeyRequest,
) -> Result<DeleteApiKeyResponse, DeleteApiKeyError> {
    debug!("Creating proto request for delete_api_key");

    let request = pistachio_api::pistachio::admin::v1::DeleteApiKeyRequest {
        project_id: req.project_id.to_string(),
        app_id: req.app_id.to_string(),
        key_id: req.key_id,
    };

    client.delete_api_key(request).await.map_err(|status| {
        error!(?status, "Error in delete_api_key response");
        match status.code() {
            Code::InvalidArgument => {
                DeleteApiKeyError::BadRequest(error_details_from_status(&status))
            }
            Code::NotFound => DeleteApiKeyError::NotFound(error_details_from_status(&status)),
            Code::Unauthenticated => {
                DeleteApiKeyError::Unauthenticated(status.message().to_string())
            }
            Code::PermissionDenied => {
                DeleteApiKeyError::PermissionDenied(status.message().to_string())
            }
            Code::Internal => DeleteApiKeyError::ServiceError(status.message().to_string()),
            Code::Unavailable => {
                DeleteApiKeyError::ServiceUnavailable(status.message().to_string())
            }
            _ => DeleteApiKeyError::Unknown(status.message().to_string()),
        }
    })?;

    Ok(DeleteApiKeyResponse {})
}

pub(crate) async fn handle_list_api_keys<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListApiKeysRequest,
) -> Result<ListApiKeysResponse, ListApiKeysError> {
    debug!("Creating proto request for list_api_keys");

    let request = pistachio_api::pistachio::admin::v1::ListApiKeysRequest {
        project_id: req.project_id.to_string(),
        app_id: req.app_id.to_string(),
        pagination: Some(pagination_params_to_proto(req.pagination)),
    };

    let response = client
        .list_api_keys(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_api_keys response");
            match status.code() {
                Code::InvalidArgument => {
                    ListApiKeysError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => ListApiKeysError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    ListApiKeysError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListApiKeysError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => ListApiKeysError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    ListApiKeysError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListApiKeysError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let api_keys = response
        .api_keys
        .into_iter()
        .map(api_key_info_from_proto)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ListApiKeysError::ResponseValidationError)?;

    let pagination = response
        .pagination
        .map(pagination_meta_from_proto)
        .unwrap_or_default();

    Ok(ListApiKeysResponse {
        api_keys,
        pagination,
    })
}

pub(crate) async fn handle_rotate_api_key<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: RotateApiKeyRequest,
) -> Result<RotateApiKeyResponse, RotateApiKeyError> {
    debug!("Creating proto request for rotate_api_key");

    let request = pistachio_api::pistachio::admin::v1::RotateApiKeyRequest {
        project_id: req.project_id.to_string(),
        app_id: req.app_id.to_string(),
        key_id: req.key_id,
        grace_period_seconds: req.grace_period_seconds,
    };

    let response = client
        .rotate_api_key(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in rotate_api_key response");
            match status.code() {
                Code::InvalidArgument => {
                    RotateApiKeyError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => RotateApiKeyError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    RotateApiKeyError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    RotateApiKeyError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => RotateApiKeyError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    RotateApiKeyError::ServiceUnavailable(status.message().to_string())
                }
                _ => RotateApiKeyError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let api_key = response
        .api_key
        .ok_or(ValidationError::MissingField("api_key"))?;
    let api_key =
        api_key_from_proto(api_key).map_err(RotateApiKeyError::ResponseValidationError)?;

    let grace_period_expires_at = response
        .grace_period_expires_at
        .and_then(timestamp_to_datetime);

    Ok(RotateApiKeyResponse {
        api_key,
        previous_key_string: response.previous_key_string,
        grace_period_expires_at,
    })
}
