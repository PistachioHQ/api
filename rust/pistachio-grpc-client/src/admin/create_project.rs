use libgn::pistachio_id::ProjectId as PistachioProjectId;
use libgn::project::{
    Project, ProjectDisplayName, ProjectId, ProjectName, ProjectResources, ProjectState,
};
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::project_management_client::ProjectManagementClient;

use crate::types::{FromProto, IntoProto, timestamp_to_datetime};

pub(crate) async fn handle_create_project<I: Interceptor>(
    client: &mut ProjectManagementClient<InterceptedService<Channel, I>>,
    req: CreateProjectRequest,
) -> Result<CreateProjectResponse, CreateProjectError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .create_project(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_project response");
            match status.code() {
                Code::InvalidArgument => {
                    CreateProjectError::BadRequest(status.message().to_string())
                }
                Code::AlreadyExists => CreateProjectError::AlreadyExists,
                Code::Unauthenticated => {
                    CreateProjectError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateProjectError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => CreateProjectError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    CreateProjectError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateProjectError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    CreateProjectResponse::from_proto(response).map_err(CreateProjectError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::CreateProjectRequest> for CreateProjectRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::CreateProjectRequest {
        pistachio_api::pistachio::admin::v1::CreateProjectRequest {
            project_id: self.project_id.map(|id| id.to_string()).unwrap_or_default(),
            display_name: self
                .display_name
                .map(|name| name.to_string())
                .unwrap_or_default(),
            invitation_code: self
                .invitation_code
                .map(|code| code.to_string())
                .unwrap_or_default(),
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::CreateProjectResponse>
    for CreateProjectResponse
{
    type Error = ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::CreateProjectResponse,
    ) -> Result<Self, Self::Error> {
        let project_proto = proto
            .project
            .ok_or(ValidationError::MissingField("project"))?;

        let project = Project::from_proto(project_proto)?;

        Ok(Self { project })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::Project> for Project {
    type Error = ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::Project,
    ) -> Result<Self, Self::Error> {
        let state = ProjectState::from_proto(proto.state)?;
        let resources = proto
            .resources
            .map(ProjectResources::from_proto)
            .transpose()?;
        let pistachio_id = PistachioProjectId::parse(&proto.pistachio_id)?;
        let project_id = ProjectId::parse(&proto.project_id)
            .map_err(|_| ValidationError::InvalidValue("project_id"))?;
        let name =
            ProjectName::parse(&proto.name).map_err(|_| ValidationError::InvalidValue("name"))?;
        let display_name = ProjectDisplayName::parse(&proto.display_name)
            .map_err(|_| ValidationError::InvalidValue("display_name"))?;
        let created_at = timestamp_to_datetime(proto.created_at)?;
        let updated_at = timestamp_to_datetime(proto.updated_at)?;

        Ok(Self {
            project_id,
            name,
            pistachio_id,
            display_name,
            state,
            resources,
            location_id: None,
            created_at,
            updated_at,
        })
    }
}

impl FromProto<i32> for ProjectState {
    type Error = ValidationError;

    fn from_proto(proto: i32) -> Result<Self, Self::Error> {
        use pistachio_api::pistachio::types::v1::ProjectState as ProtoState;

        match ProtoState::try_from(proto) {
            Ok(ProtoState::Unspecified) => Ok(Self::Unspecified),
            Ok(ProtoState::Active) => Ok(Self::Active),
            Ok(ProtoState::Deleted) => Ok(Self::Deleted),
            Err(_) => Err(ValidationError::InvalidValue("project_state")),
        }
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::ProjectResources> for ProjectResources {
    type Error = ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::types::v1::ProjectResources,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            hosting_site: proto.hosting_site,
            realtime_database_instance: proto.realtime_database_instance,
            storage_bucket: proto.storage_bucket,
        })
    }
}
