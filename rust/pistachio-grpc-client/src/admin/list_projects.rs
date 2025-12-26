use libgn::project::Project;
use pistachio_api_common::admin::project::{
    ListProjectsError, ListProjectsRequest, ListProjectsResponse,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::project_management_client::ProjectManagementClient;

use crate::types::{FromProto, IntoProto};

pub(crate) async fn handle_list_projects<I: Interceptor>(
    client: &mut ProjectManagementClient<InterceptedService<Channel, I>>,
    req: ListProjectsRequest,
) -> Result<ListProjectsResponse, ListProjectsError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .list_projects(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in list_projects response");
            match status.code() {
                Code::InvalidArgument => {
                    ListProjectsError::BadRequest(status.message().to_string())
                }
                Code::Unauthenticated => {
                    ListProjectsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    ListProjectsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => ListProjectsError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    ListProjectsError::ServiceUnavailable(status.message().to_string())
                }
                _ => ListProjectsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    ListProjectsResponse::from_proto(response).map_err(ListProjectsError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::ListProjectsRequest> for ListProjectsRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::ListProjectsRequest {
        pistachio_api::pistachio::admin::v1::ListProjectsRequest {
            pagination: Some(self.pagination.into_proto()),
            show_deleted: self.show_deleted,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::ListProjectsResponse> for ListProjectsResponse {
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::ListProjectsResponse,
    ) -> Result<Self, Self::Error> {
        let projects = proto
            .projects
            .into_iter()
            .map(Project::from_proto)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = proto
            .pagination
            .map(PaginationMeta::from_proto)
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            projects,
            pagination,
        })
    }
}
