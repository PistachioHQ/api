use pistachio_api_common::admin::user::{
    ImportProjectUsersError, ImportProjectUsersRequest, ImportProjectUsersResponse, ImportUserError,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    ImportProjectUsersError as GenError, import_project_users,
};
use crate::generated_admin::models::{
    ImportProjectUsersRequest as GenRequest, ImportProjectUsersRequestHashConfig as GenHashConfig,
    ImportProjectUsersRequestUsersInner as GenUserRecord,
};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, hash_algorithm_to_openapi};

impl From<GenError> for ImportProjectUsersError {
    fn from(error: GenError) -> Self {
        use crate::types::convert_error_details;
        match error {
            GenError::Status400(e) => Self::BadRequest(convert_error_details(e)),
            GenError::Status401(e) => {
                Self::Unauthenticated(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status403(e) => {
                Self::PermissionDenied(e.detail.unwrap_or_else(|| e.title.clone()))
            }
            GenError::Status404(e) => Self::NotFound(convert_error_details(e)),
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

pub(crate) async fn handle_import_project_users(
    config: &Configuration,
    req: ImportProjectUsersRequest,
) -> Result<ImportProjectUsersResponse, ImportProjectUsersError> {
    debug!("Creating OpenAPI request for import_project_users");

    let project_id = req.project_id.to_string();

    let users: Vec<GenUserRecord> = req
        .users
        .into_iter()
        .map(|u| GenUserRecord {
            email: u.email.map(Some),
            email_verified: Some(u.email_verified),
            phone_number: u.phone_number.map(Some),
            display_name: u.display_name.map(Some),
            photo_url: u.photo_url.map(Some),
            disabled: Some(u.disabled),
            custom_claims: u.custom_claims.map(|c| {
                c.into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect()
            }),
            password_hash: u.password_hash.map(Some),
            password_salt: u.password_salt.map(Some),
        })
        .collect();

    let hash_config = req.hash_config.map(|c| {
        Box::new(GenHashConfig {
            rounds: c.rounds.map(Some),
            memory_cost: c.memory_cost.map(Some),
            parallelization: c.parallelization.map(Some),
            salt_separator: c.salt_separator.map(Some),
            signer_key: c.signer_key.map(Some),
        })
    });

    let gen_request = GenRequest {
        users,
        hash_algorithm: req.hash_algorithm.map(hash_algorithm_to_openapi),
        hash_config,
    };

    debug!(?project_id, "Sending import_project_users request");

    let response = import_project_users(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in import_project_users response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => ImportProjectUsersError::BadRequest(problem),
                            401 => ImportProjectUsersError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => ImportProjectUsersError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => ImportProjectUsersError::NotFound(problem),
                            500..=599 => ImportProjectUsersError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => ImportProjectUsersError::Unknown(format!(
                                "HTTP {}: {}",
                                status,
                                problem.message.unwrap_or(problem.title)
                            )),
                        };
                    }
                    if let Some(entity) = resp.entity
                        && !matches!(entity, GenError::UnknownValue(_))
                    {
                        return entity.into();
                    }
                    match status {
                        400 => ImportProjectUsersError::BadRequest(fallback_error_details(
                            resp.content,
                        )),
                        401 => ImportProjectUsersError::Unauthenticated(resp.content),
                        403 => ImportProjectUsersError::PermissionDenied(resp.content),
                        404 => {
                            ImportProjectUsersError::NotFound(fallback_error_details(resp.content))
                        }
                        500..=599 => ImportProjectUsersError::ServiceError(resp.content),
                        _ => ImportProjectUsersError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    ImportProjectUsersError::ServiceUnavailable(e.to_string())
                }
                _ => ImportProjectUsersError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let errors: Vec<ImportUserError> = response
        .errors
        .unwrap_or_default()
        .into_iter()
        .map(ImportUserError::from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ImportProjectUsersError::ResponseValidationError)?;

    Ok(ImportProjectUsersResponse {
        success_count: response.success_count.unwrap_or(0),
        failure_count: response.failure_count.unwrap_or(0),
        errors,
    })
}
