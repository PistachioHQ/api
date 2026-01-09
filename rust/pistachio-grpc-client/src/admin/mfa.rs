//! MFA operation handlers for the gRPC client.

use chrono::{DateTime, TimeZone, Utc};
use pistachio_api_common::admin::mfa::{
    DeleteProjectUserMfaFactorError, DeleteProjectUserMfaFactorRequest,
    DeleteProjectUserMfaFactorResponse, DeleteTenantUserMfaFactorError,
    DeleteTenantUserMfaFactorRequest, DeleteTenantUserMfaFactorResponse,
    ListProjectUserMfaFactorsError, ListProjectUserMfaFactorsRequest,
    ListProjectUserMfaFactorsResponse, ListTenantUserMfaFactorsError,
    ListTenantUserMfaFactorsRequest, ListTenantUserMfaFactorsResponse, MfaFactor, MfaFactorType,
};
use prost_types::Timestamp;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;
use pistachio_api::pistachio::types::v1 as proto_types;

use crate::types::error_details_from_status;

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

fn mfa_factor_type_from_proto(proto: i32) -> MfaFactorType {
    match proto_types::MfaFactorType::try_from(proto) {
        Ok(proto_types::MfaFactorType::Totp) => MfaFactorType::Totp,
        Ok(proto_types::MfaFactorType::Sms) => MfaFactorType::Sms,
        Ok(proto_types::MfaFactorType::Email) => MfaFactorType::Email,
        _ => MfaFactorType::Unspecified,
    }
}

fn mfa_factor_from_proto(proto: proto_types::MfaFactor) -> MfaFactor {
    MfaFactor {
        factor_id: proto.factor_id,
        factor_type: mfa_factor_type_from_proto(proto.factor_type),
        display_name: if proto.display_name.is_empty() {
            None
        } else {
            Some(proto.display_name)
        },
        phone_number: if proto.phone_number.is_empty() {
            None
        } else {
            Some(proto.phone_number)
        },
        email: if proto.email.is_empty() {
            None
        } else {
            Some(proto.email)
        },
        verified: proto.verified,
        created_at: optional_timestamp_to_datetime(proto.created_at),
        last_used_at: proto.last_used_at.and_then(timestamp_to_datetime),
    }
}

// =============================================================================
// Handler Implementations
// =============================================================================

pub(crate) async fn handle_list_project_user_mfa_factors<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListProjectUserMfaFactorsRequest,
) -> Result<ListProjectUserMfaFactorsResponse, ListProjectUserMfaFactorsError> {
    debug!("Creating proto request for list_project_user_mfa_factors");

    let request = pistachio_api::pistachio::admin::v1::ListProjectUserMfaFactorsRequest {
        project_id: req.project_id.to_string(),
        pistachio_id: req.pistachio_id.to_string(),
    };

    let response = client
        .list_project_user_mfa_factors(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_project_user_mfa_factors response");
            match status.code() {
                Code::InvalidArgument => {
                    ListProjectUserMfaFactorsError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListProjectUserMfaFactorsError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListProjectUserMfaFactorsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListProjectUserMfaFactorsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ListProjectUserMfaFactorsError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ListProjectUserMfaFactorsError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListProjectUserMfaFactorsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let factors = response
        .factors
        .into_iter()
        .map(mfa_factor_from_proto)
        .collect();

    Ok(ListProjectUserMfaFactorsResponse { factors })
}

pub(crate) async fn handle_delete_project_user_mfa_factor<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteProjectUserMfaFactorRequest,
) -> Result<DeleteProjectUserMfaFactorResponse, DeleteProjectUserMfaFactorError> {
    debug!("Creating proto request for delete_project_user_mfa_factor");

    let request = pistachio_api::pistachio::admin::v1::DeleteProjectUserMfaFactorRequest {
        project_id: req.project_id.to_string(),
        pistachio_id: req.pistachio_id.to_string(),
        factor_id: req.factor_id,
    };

    client
        .delete_project_user_mfa_factor(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_project_user_mfa_factor response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteProjectUserMfaFactorError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteProjectUserMfaFactorError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteProjectUserMfaFactorError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteProjectUserMfaFactorError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteProjectUserMfaFactorError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => DeleteProjectUserMfaFactorError::ServiceUnavailable(
                    status.message().to_string(),
                ),
                _ => DeleteProjectUserMfaFactorError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(DeleteProjectUserMfaFactorResponse {})
}

pub(crate) async fn handle_list_tenant_user_mfa_factors<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ListTenantUserMfaFactorsRequest,
) -> Result<ListTenantUserMfaFactorsResponse, ListTenantUserMfaFactorsError> {
    debug!("Creating proto request for list_tenant_user_mfa_factors");

    let request = pistachio_api::pistachio::admin::v1::ListTenantUserMfaFactorsRequest {
        project_id: req.project_id.to_string(),
        tenant_id: req.tenant_id.to_string(),
        pistachio_id: req.pistachio_id.to_string(),
    };

    let response = client
        .list_tenant_user_mfa_factors(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_tenant_user_mfa_factors response");
            match status.code() {
                Code::InvalidArgument => {
                    ListTenantUserMfaFactorsError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ListTenantUserMfaFactorsError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ListTenantUserMfaFactorsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListTenantUserMfaFactorsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ListTenantUserMfaFactorsError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ListTenantUserMfaFactorsError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListTenantUserMfaFactorsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    let factors = response
        .factors
        .into_iter()
        .map(mfa_factor_from_proto)
        .collect();

    Ok(ListTenantUserMfaFactorsResponse { factors })
}

pub(crate) async fn handle_delete_tenant_user_mfa_factor<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: DeleteTenantUserMfaFactorRequest,
) -> Result<DeleteTenantUserMfaFactorResponse, DeleteTenantUserMfaFactorError> {
    debug!("Creating proto request for delete_tenant_user_mfa_factor");

    let request = pistachio_api::pistachio::admin::v1::DeleteTenantUserMfaFactorRequest {
        project_id: req.project_id.to_string(),
        tenant_id: req.tenant_id.to_string(),
        pistachio_id: req.pistachio_id.to_string(),
        factor_id: req.factor_id,
    };

    client
        .delete_tenant_user_mfa_factor(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in delete_tenant_user_mfa_factor response");
            match status.code() {
                Code::InvalidArgument => {
                    DeleteTenantUserMfaFactorError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    DeleteTenantUserMfaFactorError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    DeleteTenantUserMfaFactorError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    DeleteTenantUserMfaFactorError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    DeleteTenantUserMfaFactorError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    DeleteTenantUserMfaFactorError::ServiceUnavailable(status.message().to_string())
                }
                _ => DeleteTenantUserMfaFactorError::Unknown(status.message().to_string()),
            }
        })?;

    Ok(DeleteTenantUserMfaFactorResponse {})
}
