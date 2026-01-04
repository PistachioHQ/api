use pistachio_api_common::admin::auth_provider::{
    UpdateTenantAuthProviderError, UpdateTenantAuthProviderRequest,
    UpdateTenantAuthProviderResponse,
};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::auth_providers_api::{
    UpdateTenantAuthProviderError as GenError, update_tenant_auth_provider,
};
use crate::generated_admin::apis::configuration::Configuration;
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{build_update_tenant_request, convert_error_details, override_from_json};

impl From<GenError> for UpdateTenantAuthProviderError {
    fn from(error: GenError) -> Self {
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

pub(crate) async fn handle_update_tenant_auth_provider(
    config: &Configuration,
    req: UpdateTenantAuthProviderRequest,
) -> Result<UpdateTenantAuthProviderResponse, UpdateTenantAuthProviderError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let tenant_id = req.tenant_id.to_string();
    let provider_id = req.provider_id.to_string();

    let request_body = build_update_tenant_request(
        req.enabled,
        req.display_order,
        req.config.as_ref(),
        req.client_secret,
    );

    debug!(
        ?project_id,
        ?tenant_id,
        ?provider_id,
        "Sending update_tenant_auth_provider request"
    );

    let response =
        update_tenant_auth_provider(config, &project_id, &tenant_id, &provider_id, request_body)
            .await
            .map_err(|e| {
                error!(?e, "Error in update_tenant_auth_provider response");
                match e {
                    crate::generated_admin::apis::Error::ResponseError(resp) => {
                        let status = resp.status.as_u16();

                        if let Some(problem) = parse_error_details(&resp.content) {
                            return match status {
                                400 => UpdateTenantAuthProviderError::BadRequest(problem),
                                401 => UpdateTenantAuthProviderError::Unauthenticated(
                                    problem.message.unwrap_or(problem.title),
                                ),
                                403 => UpdateTenantAuthProviderError::PermissionDenied(
                                    problem.message.unwrap_or(problem.title),
                                ),
                                404 => UpdateTenantAuthProviderError::NotFound(problem),
                                500..=599 => UpdateTenantAuthProviderError::ServiceError(
                                    problem.message.unwrap_or(problem.title),
                                ),
                                _ => UpdateTenantAuthProviderError::Unknown(format!(
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
                            400 => UpdateTenantAuthProviderError::BadRequest(
                                fallback_error_details(resp.content),
                            ),
                            401 => UpdateTenantAuthProviderError::Unauthenticated(resp.content),
                            403 => UpdateTenantAuthProviderError::PermissionDenied(resp.content),
                            404 => UpdateTenantAuthProviderError::NotFound(fallback_error_details(
                                resp.content,
                            )),
                            500..=599 => UpdateTenantAuthProviderError::ServiceError(resp.content),
                            _ => UpdateTenantAuthProviderError::Unknown(format!(
                                "HTTP {}: {}",
                                status, resp.content
                            )),
                        }
                    }
                    crate::generated_admin::apis::Error::Reqwest(e) => {
                        UpdateTenantAuthProviderError::ServiceUnavailable(e.to_string())
                    }
                    _ => {
                        UpdateTenantAuthProviderError::ServiceError("Unknown error occurred".into())
                    }
                }
            })?;

    let override_ = response
        .r#override
        .map(|o| override_from_json(*o))
        .transpose()
        .map_err(UpdateTenantAuthProviderError::ResponseValidationError)?
        .ok_or(UpdateTenantAuthProviderError::ResponseValidationError(
            ValidationError::MissingField("override"),
        ))?;

    Ok(UpdateTenantAuthProviderResponse { override_ })
}
