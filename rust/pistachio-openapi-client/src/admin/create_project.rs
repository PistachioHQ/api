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
    CreateProject201Response, CreateProjectRequest as GenCreateProjectRequest,
    ListProjects200ResponseProjectsInner, ListProjects200ResponseProjectsInnerResources,
};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details, parse_timestamp};

impl From<GenError> for CreateProjectError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status409(_) => Self::AlreadyExists,
            GenError::Status500(e) => {
                Self::ServiceError(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status503(e) => {
                Self::ServiceUnavailable(e.detail.unwrap_or_else(|| e.title.clone()))
            }
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
        invitation_code: req.invitation_code.map(|code| code.to_string()),
    };

    debug!(?request, "Sending create_project request");

    let response = create_project(config, request).await.map_err(|e| {
        error!(?e, "Error in create_project response");
        match e {
            crate::generated_admin::apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();

                // Try to parse RFC 7807 Problem Details from the response content
                if let Some(problem) = parse_error_details(&resp.content) {
                    return match status {
                        400 => CreateProjectError::BadRequest(problem),
                        401 => CreateProjectError::Unauthenticated(
                            problem.message.unwrap_or(problem.title),
                        ),
                        403 => CreateProjectError::PermissionDenied(
                            problem.message.unwrap_or(problem.title),
                        ),
                        409 => CreateProjectError::AlreadyExists,
                        500..=599 => CreateProjectError::ServiceError(
                            problem.message.unwrap_or(problem.title),
                        ),
                        _ => CreateProjectError::Unknown(format!(
                            "HTTP {}: {}",
                            status,
                            problem.message.unwrap_or(problem.title)
                        )),
                    };
                }

                // Fall back to entity parsing if RFC 7807 parsing failed
                if let Some(entity) = resp.entity
                    && !matches!(entity, GenError::UnknownValue(_))
                {
                    return entity.into();
                }

                // Last resort: status code mapping with raw content
                match status {
                    400 => CreateProjectError::BadRequest(fallback_error_details(resp.content)),
                    401 => CreateProjectError::Unauthenticated(resp.content),
                    403 => CreateProjectError::PermissionDenied(resp.content),
                    409 => CreateProjectError::AlreadyExists,
                    500..=599 => CreateProjectError::ServiceError(resp.content),
                    _ => CreateProjectError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                }
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

impl FromJson<CreateProject201Response> for CreateProjectResponse {
    type Error = ValidationError;

    fn from_json(json: CreateProject201Response) -> Result<Self, Self::Error> {
        let project = json
            .project
            .map(|p| Project::from_json(*p))
            .transpose()?
            .ok_or(ValidationError::MissingField("project"))?;

        Ok(Self { project })
    }
}

impl FromJson<ListProjects200ResponseProjectsInner> for Project {
    type Error = ValidationError;

    fn from_json(json: ListProjects200ResponseProjectsInner) -> Result<Self, Self::Error> {
        use crate::generated_admin::models::list_projects_200_response_projects_inner::State;

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
            location_id: None,
            created_at,
            updated_at,
        })
    }
}

impl FromJson<ListProjects200ResponseProjectsInnerResources> for ProjectResources {
    type Error = ValidationError;

    fn from_json(json: ListProjects200ResponseProjectsInnerResources) -> Result<Self, Self::Error> {
        Ok(Self {
            hosting_site: json.hosting_site.unwrap_or_default(),
            realtime_database_instance: json.realtime_database_instance.unwrap_or_default(),
            storage_bucket: json.storage_bucket.unwrap_or_default(),
        })
    }
}
