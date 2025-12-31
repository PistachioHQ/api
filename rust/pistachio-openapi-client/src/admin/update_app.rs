use libgn::app::{App, PlatformConfig};
use pistachio_api_common::admin::app::{UpdateAppError, UpdateAppRequest, UpdateAppResponse};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{UpdateAppError as GenError, update_app};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::{UpdateApp200Response, UpdateAppRequest as GenRequest};
use crate::problem_details::{fallback_error_details, parse_error_details};
use crate::types::{FromJson, convert_error_details};

impl From<GenError> for UpdateAppError {
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

pub(crate) async fn handle_update_app(
    config: &Configuration,
    req: UpdateAppRequest,
) -> Result<UpdateAppResponse, UpdateAppError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let app_id = req.app_id.to_string();
    let gen_request = build_update_request(req);

    debug!(?project_id, ?app_id, "Sending update_app request");

    let response = update_app(config, &project_id, &app_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in update_app response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    let status = resp.status.as_u16();
                    if let Some(problem) = parse_error_details(&resp.content) {
                        return match status {
                            400 => UpdateAppError::BadRequest(problem),
                            401 => UpdateAppError::Unauthenticated(
                                problem.message.unwrap_or(problem.title),
                            ),
                            403 => UpdateAppError::PermissionDenied(
                                problem.message.unwrap_or(problem.title),
                            ),
                            404 => UpdateAppError::NotFound(problem),
                            500..=599 => UpdateAppError::ServiceError(
                                problem.message.unwrap_or(problem.title),
                            ),
                            _ => UpdateAppError::Unknown(format!(
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
                        400 => UpdateAppError::BadRequest(fallback_error_details(resp.content)),
                        401 => UpdateAppError::Unauthenticated(resp.content),
                        403 => UpdateAppError::PermissionDenied(resp.content),
                        404 => UpdateAppError::NotFound(fallback_error_details(resp.content)),
                        500..=599 => UpdateAppError::ServiceError(resp.content),
                        _ => UpdateAppError::Unknown(format!("HTTP {}: {}", status, resp.content)),
                    }
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    UpdateAppError::ServiceUnavailable(e.to_string())
                }
                _ => UpdateAppError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    UpdateAppResponse::from_json(response).map_err(UpdateAppError::ResponseValidationError)
}

fn build_update_request(req: UpdateAppRequest) -> GenRequest {
    let (ios, android, macos, windows, linux, web) = match req.platform_config {
        Some(PlatformConfig::Ios(c)) => (
            Some(Box::new(
                crate::generated_admin::models::ListApps200ResponseAppsInnerIos {
                    bundle_id: c.bundle_id,
                    app_store_id: c.app_store_id,
                    team_id: c.team_id,
                },
            )),
            None,
            None,
            None,
            None,
            None,
        ),
        Some(PlatformConfig::Android(c)) => (
            None,
            Some(Box::new(
                crate::generated_admin::models::ListApps200ResponseAppsInnerAndroid {
                    package_name: c.package_name,
                    sha1_hashes: if c.sha1_hashes.is_empty() {
                        None
                    } else {
                        Some(c.sha1_hashes)
                    },
                    sha256_hashes: if c.sha256_hashes.is_empty() {
                        None
                    } else {
                        Some(c.sha256_hashes)
                    },
                },
            )),
            None,
            None,
            None,
            None,
        ),
        Some(PlatformConfig::Macos(c)) => (
            None,
            None,
            Some(Box::new(
                crate::generated_admin::models::ListApps200ResponseAppsInnerMacos {
                    bundle_id: c.bundle_id,
                    team_id: c.team_id,
                    mac_app_store_id: c.mac_app_store_id,
                },
            )),
            None,
            None,
            None,
        ),
        Some(PlatformConfig::Windows(c)) => (
            None,
            None,
            None,
            Some(Box::new(
                crate::generated_admin::models::ListApps200ResponseAppsInnerWindows {
                    package_family_name: c.package_family_name,
                    publisher_id: c.publisher_id,
                    cert_thumbprints: if c.cert_thumbprints.is_empty() {
                        None
                    } else {
                        Some(c.cert_thumbprints)
                    },
                    microsoft_store_id: c.microsoft_store_id,
                },
            )),
            None,
            None,
        ),
        Some(PlatformConfig::Linux(c)) => (
            None,
            None,
            None,
            None,
            Some(Box::new(
                crate::generated_admin::models::ListApps200ResponseAppsInnerLinux {
                    app_name: c.app_name,
                    flatpak_id: c.flatpak_id,
                    snap_name: c.snap_name,
                },
            )),
            None,
        ),
        Some(PlatformConfig::Web(c)) => (
            None,
            None,
            None,
            None,
            None,
            Some(Box::new(
                crate::generated_admin::models::ListApps200ResponseAppsInnerWeb {
                    authorized_domains: if c.authorized_domains.is_empty() {
                        None
                    } else {
                        Some(c.authorized_domains)
                    },
                    auth_domain: c.auth_domain,
                },
            )),
        ),
        None => (None, None, None, None, None, None),
    };

    GenRequest {
        display_name: req.display_name.map(|d| d.to_string()),
        ios,
        android,
        macos,
        windows,
        linux,
        web,
    }
}

impl FromJson<UpdateApp200Response> for UpdateAppResponse {
    type Error = ValidationError;

    fn from_json(json: UpdateApp200Response) -> Result<Self, Self::Error> {
        let app = json
            .app
            .map(|a| App::from_json(*a))
            .transpose()?
            .ok_or(ValidationError::MissingField("app"))?;

        Ok(Self { app })
    }
}
