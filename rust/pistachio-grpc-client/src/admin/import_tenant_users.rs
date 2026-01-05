use pistachio_api_common::admin::user::{
    ImportTenantUsersError, ImportTenantUsersRequest, ImportTenantUsersResponse, ImportUserError,
};
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, error_details_from_status};

pub(crate) async fn handle_import_tenant_users<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: ImportTenantUsersRequest,
) -> Result<ImportTenantUsersResponse, ImportTenantUsersError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .import_tenant_users(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in import_tenant_users response");
            match status.code() {
                Code::InvalidArgument => {
                    ImportTenantUsersError::BadRequest(error_details_from_status(&status))
                }
                Code::NotFound => {
                    ImportTenantUsersError::NotFound(error_details_from_status(&status))
                }
                Code::Unauthenticated => {
                    ImportTenantUsersError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ImportTenantUsersError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => {
                    ImportTenantUsersError::ServiceError(status.message().to_string())
                }
                Code::Unavailable => {
                    ImportTenantUsersError::ServiceUnavailable(status.message().to_string())
                }
                _ => ImportTenantUsersError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ImportTenantUsersResponse::from_proto(response)
        .map_err(ImportTenantUsersError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ImportTenantUsersRequest>
    for ImportTenantUsersRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ImportTenantUsersRequest {
        pistachio_api::pistachio::admin::v1::ImportTenantUsersRequest {
            project_id: self.project_id.to_string(),
            tenant_id: self.tenant_id.to_string(),
            users: self.users.into_iter().map(IntoProto::into_proto).collect(),
            hash_algorithm: self
                .hash_algorithm
                .map(|a| {
                    let proto_alg: pistachio_api::pistachio::types::v1::HashAlgorithm =
                        IntoProto::into_proto(a);
                    proto_alg.into()
                })
                .unwrap_or(0),
            hash_config: self.hash_config.map(|c| {
                pistachio_api::pistachio::types::v1::HashConfig {
                    rounds: c.rounds.unwrap_or(0),
                    memory_cost: c.memory_cost.unwrap_or(0),
                    parallelization: c.parallelization.unwrap_or(0),
                    salt_separator: c.salt_separator.unwrap_or_default(),
                    signer_key: c.signer_key.unwrap_or_default(),
                }
            }),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ImportTenantUsersResponse>
    for ImportTenantUsersResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ImportTenantUsersResponse,
    ) -> Result<Self, Self::Error> {
        let errors = proto
            .errors
            .into_iter()
            .map(ImportUserError::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            success_count: proto.success_count,
            failure_count: proto.failure_count,
            errors,
        })
    }
}
