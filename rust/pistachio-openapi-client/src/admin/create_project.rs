use libgn::pistachio_id::ProjectId as PistachioProjectId;
use libgn::project::{
    Project, ProjectDisplayName, ProjectId, ProjectName, ProjectResources, ProjectState,
};
use pistachio_api_common::admin::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::projects_api::{CreateProjectError as GenError, create_project};
use crate::generated_admin::models::{
    CreateProject200Response, CreateProject200ResponseProject,
    CreateProject200ResponseProjectResources, CreateProjectRequest as GenCreateProjectRequest,
};
use crate::types::{FromJson, parse_timestamp};

impl From<GenError> for CreateProjectError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status409(_) => Self::AlreadyExists,
            GenError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

pub(crate) async fn handle_create_project(
    config: &Configuration,
    req: CreateProjectRequest,
) -> Result<CreateProjectResponse, CreateProjectError> {
    debug!("Creating OpenAPI request");

    let request = GenCreateProjectRequest {
        project_id: req.project_id.map(|id| id.to_string()),
        display_name: req.display_name.map(|name| name.to_string()),
    };

    debug!(?request, "Sending create_project request");

    let response = create_project(config, request).await.map_err(|e| {
        error!(?e, "Error in create_project response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                resp.entity.map(Into::into).unwrap_or_else(|| {
                    CreateProjectError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                })
            }
            crate::generated_admin::apis::Error::Reqwest(e) => {
                CreateProjectError::ServiceUnavailable(e.to_string())
            }
            _ => CreateProjectError::ServiceError("Unknown error occurred".into()),
        }
    })?;

    CreateProjectResponse::from_json(response).map_err(CreateProjectError::ResponseValidationError)
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<CreateProject200Response> for CreateProjectResponse {
    type Error = ValidationError;

    fn from_json(json: CreateProject200Response) -> Result<Self, Self::Error> {
        let project = json
            .project
            .map(|p| Project::from_json(*p))
            .transpose()?
            .ok_or(ValidationError::MissingField("project"))?;

        Ok(Self { project })
    }
}

impl FromJson<CreateProject200ResponseProject> for Project {
    type Error = ValidationError;

    fn from_json(json: CreateProject200ResponseProject) -> Result<Self, Self::Error> {
        use crate::generated_admin::models::create_project_200_response_project::State;

        let state = match json.state {
            Some(State::Active) => ProjectState::Active,
            Some(State::Deleted) => ProjectState::Deleted,
            None => ProjectState::Unspecified,
        };

        let resources = json
            .resources
            .map(|r| ProjectResources::from_json(*r))
            .transpose()?;

        let pistachio_id_str = json
            .pistachio_id
            .ok_or(ValidationError::MissingField("pistachio_id"))?;
        let pistachio_id = PistachioProjectId::parse(&pistachio_id_str)?;

        let project_id_str = json
            .project_id
            .ok_or(ValidationError::MissingField("project_id"))?;
        let project_id = ProjectId::parse(&project_id_str)
            .map_err(|_| ValidationError::InvalidValue("project_id"))?;

        let name_str = json.name.ok_or(ValidationError::MissingField("name"))?;
        let name =
            ProjectName::parse(&name_str).map_err(|_| ValidationError::InvalidValue("name"))?;

        let display_name_str = json
            .display_name
            .ok_or(ValidationError::MissingField("display_name"))?;
        let display_name = ProjectDisplayName::parse(&display_name_str)
            .map_err(|_| ValidationError::InvalidValue("display_name"))?;

        let created_at = parse_timestamp(json.created_at)?;
        let updated_at = parse_timestamp(json.updated_at)?;

        Ok(Self {
            project_id,
            name,
            pistachio_id,
            display_name,
            state,
            resources,
            created_at,
            updated_at,
        })
    }
}

impl FromJson<CreateProject200ResponseProjectResources> for ProjectResources {
    type Error = ValidationError;

    fn from_json(json: CreateProject200ResponseProjectResources) -> Result<Self, Self::Error> {
        Ok(Self {
            hosting_site: json.hosting_site.unwrap_or_default(),
            realtime_database_instance: json.realtime_database_instance.unwrap_or_default(),
            storage_bucket: json.storage_bucket.unwrap_or_default(),
        })
    }
}
