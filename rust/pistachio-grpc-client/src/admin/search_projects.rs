use libgn::project::Project;
use pistachio_api_common::admin::project::{
    SearchProjectsError, SearchProjectsRequest, SearchProjectsResponse,
};
use pistachio_api_common::pagination::PaginationMeta;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::project_management_client::ProjectManagementClient;

use crate::types::{FromProto, IntoProto};

pub(crate) async fn handle_search_projects<I: Interceptor>(
    client: &mut ProjectManagementClient<InterceptedService<Channel, I>>,
    req: SearchProjectsRequest,
) -> Result<SearchProjectsResponse, SearchProjectsError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .search_projects(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in search_projects response");
            match status.code() {
                Code::InvalidArgument => {
                    SearchProjectsError::BadRequest(status.message().to_string())
                }
                Code::Unauthenticated => {
                    SearchProjectsError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    SearchProjectsError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => SearchProjectsError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    SearchProjectsError::ServiceUnavailable(status.message().to_string())
                }
                _ => SearchProjectsError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    SearchProjectsResponse::from_proto(response)
        .map_err(SearchProjectsError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::SearchProjectsRequest>
    for SearchProjectsRequest
{
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::SearchProjectsRequest {
        pistachio_api::pistachio::admin::v1::SearchProjectsRequest {
            params: Some(self.params.into_proto()),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::SearchProjectsResponse>
    for SearchProjectsResponse
{
    type Error = pistachio_api_common::error::ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::SearchProjectsResponse,
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
