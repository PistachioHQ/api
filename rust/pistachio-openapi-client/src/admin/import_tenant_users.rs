use pistachio_api_common::admin::user::{
    ImportTenantUsersError, ImportTenantUsersRequest, ImportTenantUsersResponse, ImportUserError,
};
use tracing::{debug, error};

use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::apis::users_api::{
    ImportTenantUsersError as GenError, import_tenant_users,
};
use crate::generated_admin::models::{
    ImportProjectUsersRequest as GenRequest, ImportProjectUsersRequestHashConfig as GenHashConfig,
    ImportProjectUsersRequestUsersInner as GenUserRecord,
};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, hash_algorithm_to_openapi};

impl From<GenError> for ImportTenantUsersError {
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

pub(crate) async fn handle_import_tenant_users(
    config: &Configuration,
    req: ImportTenantUsersRequest,
) -> Result<ImportTenantUsersResponse, ImportTenantUsersError> {
    debug!("Creating OpenAPI request for import_tenant_users");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();

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
            custom_claims: u
                .custom_claims
                .map(|c| c.into_iter().map(|(k, v)| (k, v.to_json())).collect()),
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

    debug!(
        ?project_id,
        ?tenant_id,
        "Sending import_tenant_users request"
    );

    let response = import_tenant_users(config, &project_id, &tenant_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in import_tenant_users response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => ImportTenantUsersError::BadRequest(problem),
                            401 => ImportTenantUsersError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => ImportTenantUsersError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => ImportTenantUsersError::NotFound(problem),
                            500..=599 => ImportTenantUsersError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => ImportTenantUsersError::Unknown(format!(
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
                        400 => {
                            ImportTenantUsersError::BadRequest(fallback_error_details(resp.content))
                        }
                        401 => ImportTenantUsersError::Unauthenticated(resp.content),
                        403 => ImportTenantUsersError::PermissionDenied(resp.content),
                        404 => {
                            ImportTenantUsersError::NotFound(fallback_error_details(resp.content))
                        }
                        500..=599 => ImportTenantUsersError::ServiceError(resp.content),
                        _ => ImportTenantUsersError::Unknown(format!(
                            "HTTP {}: {}",
                            status, resp.content
                        )),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    ImportTenantUsersError::ServiceUnavailable(e.to_string())
                }
                _ => ImportTenantUsersError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    let errors: Vec<ImportUserError> = response
        .errors
        .unwrap_or_default()
        .into_iter()
        .map(ImportUserError::from_json)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ImportTenantUsersError::ResponseValidationError)?;

    Ok(ImportTenantUsersResponse {
        success_count: response.success_count.unwrap_or(0),
        failure_count: response.failure_count.unwrap_or(0),
        errors,
    })
}
