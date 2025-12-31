use libgn::pistachio_id::TenantId as PistachioTenantId;
use libgn::tenant::{Tenant, TenantDisplayName, TenantId, TenantName};
use pistachio_api_common::admin::tenant::{
    CreateTenantError, CreateTenantRequest, CreateTenantResponse,
};
use pistachio_api_common::error::ValidationError;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status, timestamp_to_datetime};

pub(crate) async fn handle_create_tenant<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateTenantRequest,
) -> Result<CreateTenantResponse, CreateTenantError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .create_tenant(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_tenant response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateTenantError::BadRequest(error_details_from_status(&status))
                }
                Code::AlreadyExists => CreateTenantError::AlreadyExists,
                Code::NotFound => CreateTenantError::NotFound(error_details_from_status(&status)),
                Code::Unauthenticated => {
                    CreateTenantError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateTenantError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => CreateTenantError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    CreateTenantError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateTenantError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    CreateTenantResponse::from_proto(response).map_err(CreateTenantError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::CreateTenantRequest> for CreateTenantRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::CreateTenantRequest {
        pistachio_api::pistachio::admin::v1::CreateTenantRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.map(|id| id.to_string()).unwrap_or_default(),
            display_name: self.display_name.to_string(),
            allow_pdpka_signup: self.allow_pdpka_signup,
            disable_auth: self.disable_auth,
            mfa_config: self.mfa_config,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::CreateTenantResponse> for CreateTenantResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::CreateTenantResponse,
    ) -> Result<Self, Self::Error> {
        let tenant_proto =
            proto
                .tenant
                .ok_or(pistachio_api_common::error::ValidationError::MissingField(
                    "tenant",
                ))?;

        let tenant = Tenant::from_proto(tenant_proto)?;

        Ok(Self { tenant })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::Tenant> for Tenant {
    type Error = ValidationError;

    fn from_proto(proto: pistachio_api::pistachio::types::v1::Tenant) -> Result<Self, Self::Error> {
        let name =
            TenantName::parse(&proto.name).map_err(|_| ValidationError::InvalidValue("name"))?;
        let project_id = name.project_id();
        let tenant_id = TenantId::parse(&proto.tenant_id)
            .map_err(|_| ValidationError::InvalidValue("tenant_id"))?;
        let pistachio_id = PistachioTenantId::parse(&proto.pistachio_id)?;
        let display_name = TenantDisplayName::parse(&proto.display_name)
            .map_err(|_| ValidationError::InvalidValue("display_name"))?;
        let created_at = timestamp_to_datetime(proto.created_at)?;
        let updated_at = timestamp_to_datetime(proto.updated_at)?;

        Ok(Self {
            project_id,
            tenant_id,
            name,
            pistachio_id,
            display_name,
            allow_pdpka_signup: proto.allow_pdpka_signup,
            disable_auth: proto.disable_auth,
            mfa_config: proto.mfa_config,
            created_at,
            updated_at,
        })
    }
}
