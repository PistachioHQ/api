use libgn::app::{
    AndroidAppConfig, App, AppDisplayName, AppId, AppName, AppState, IosAppConfig, LinuxAppConfig,
    MacosAppConfig, Platform, PlatformConfig, WebAppConfig, WindowsAppConfig,
};
use libgn::pistachio_id::AppId as PistachioAppId;
use libgn::project::ProjectId;
use pistachio_api_common::admin::app::{CreateAppError, CreateAppRequest, CreateAppResponse};
use pistachio_api_common::error::ValidationError;
use tracing::{debug, error};

use crate::generated_admin::apis::apps_api::{CreateAppError as GenError, create_app};
use crate::generated_admin::apis::configuration::Configuration;
use crate::generated_admin::models::{
    CreateApp200Response, CreateAppRequest as GenRequest, ListApps200ResponseAppsInner,
};
use crate::types::{FromJson, parse_timestamp};

impl From<GenError> for CreateAppError {
    fn from(error: GenError) -> Self {
        match error {
            GenError::Status400(e) => Self::BadRequest(format!("{}: {}", e.code, e.message)),
            GenError::Status401(e) => Self::Unauthenticated(format!("{}: {}", e.code, e.message)),
            GenError::Status403(e) => Self::PermissionDenied(format!("{}: {}", e.code, e.message)),
            GenError::Status404(_) => Self::NotFound,
            GenError::Status409(_) => Self::AlreadyExists,
            GenError::UnknownValue(v) => {
                Self::Unknown(format!("Server returned an unexpected response: {}.", v))
            }
        }
    }
}

pub(crate) async fn handle_create_app(
    config: &Configuration,
    req: CreateAppRequest,
) -> Result<CreateAppResponse, CreateAppError> {
    debug!("Creating OpenAPI request");

    let project_id = req.project_id.to_string();
    let gen_request = build_create_request(req);

    debug!(?project_id, "Sending create_app request");

    let response = create_app(config, &project_id, gen_request)
        .await
        .map_err(|e| {
            error!(?e, "Error in create_app response");
            match e {
                crate::generated_admin::apis::Error::ResponseError(resp) => {
                    resp.entity.map(Into::into).unwrap_or_else(|| {
                        CreateAppError::Unknown(format!("HTTP {}: {}", resp.status, resp.content))
                    })
                }
                crate::generated_admin::apis::Error::Reqwest(e) => {
                    CreateAppError::ServiceUnavailable(e.to_string())
                }
                _ => CreateAppError::ServiceError("Unknown error occurred".into()),
            }
        })?;

    CreateAppResponse::from_json(response).map_err(CreateAppError::ResponseValidationError)
}

fn build_create_request(req: CreateAppRequest) -> GenRequest {
    use crate::generated_admin::models::create_app_request::Platform as GenPlatform;

    let (platform, ios, android, macos, windows, linux, web) = match req.platform_config {
        PlatformConfig::Ios(c) => (
            GenPlatform::Ios,
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
        PlatformConfig::Android(c) => (
            GenPlatform::Android,
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
        PlatformConfig::Macos(c) => (
            GenPlatform::Macos,
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
        PlatformConfig::Windows(c) => (
            GenPlatform::Windows,
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
        PlatformConfig::Linux(c) => (
            GenPlatform::Linux,
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
        PlatformConfig::Web(c) => (
            GenPlatform::Web,
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
    };

    GenRequest {
        display_name: req.display_name.to_string(),
        platform,
        ios,
        android,
        macos,
        windows,
        linux,
        web,
    }
}

// =============================================================================
// JSON conversions
// =============================================================================

impl FromJson<CreateApp200Response> for CreateAppResponse {
    type Error = ValidationError;

    fn from_json(json: CreateApp200Response) -> Result<Self, Self::Error> {
        let app = json
            .app
            .map(|a| App::from_json(*a))
            .transpose()?
            .ok_or(ValidationError::MissingField("app"))?;

        Ok(Self { app })
    }
}

impl FromJson<ListApps200ResponseAppsInner> for App {
    type Error = ValidationError;

    fn from_json(json: ListApps200ResponseAppsInner) -> Result<Self, Self::Error> {
        let name_str = json
            .name
            .clone()
            .ok_or(ValidationError::MissingField("name"))?;
        let name = AppName::parse(&name_str).map_err(|_| ValidationError::InvalidValue("name"))?;

        let project_id_str = json
            .project_id
            .clone()
            .ok_or(ValidationError::MissingField("project_id"))?;
        let project_id = ProjectId::parse(&project_id_str)
            .map_err(|_| ValidationError::InvalidValue("project_id"))?;

        let app_id_str = json
            .app_id
            .clone()
            .ok_or(ValidationError::MissingField("app_id"))?;
        let app_id =
            AppId::parse(&app_id_str).map_err(|_| ValidationError::InvalidValue("app_id"))?;

        let pistachio_id_str = json
            .pistachio_id
            .clone()
            .ok_or(ValidationError::MissingField("pistachio_id"))?;
        let pistachio_id = PistachioAppId::parse(&pistachio_id_str)?;

        let display_name_str = json
            .display_name
            .clone()
            .ok_or(ValidationError::MissingField("display_name"))?;
        let display_name = AppDisplayName::parse(&display_name_str)
            .map_err(|_| ValidationError::InvalidValue("display_name"))?;

        let platform = match json.platform.ok_or(ValidationError::MissingField("platform"))? {
            crate::generated_admin::models::list_apps_200_response_apps_inner::Platform::Ios => {
                Platform::Ios
            }
            crate::generated_admin::models::list_apps_200_response_apps_inner::Platform::Android => {
                Platform::Android
            }
            crate::generated_admin::models::list_apps_200_response_apps_inner::Platform::Macos => {
                Platform::Macos
            }
            crate::generated_admin::models::list_apps_200_response_apps_inner::Platform::Windows => {
                Platform::Windows
            }
            crate::generated_admin::models::list_apps_200_response_apps_inner::Platform::Linux => {
                Platform::Linux
            }
            crate::generated_admin::models::list_apps_200_response_apps_inner::Platform::Web => {
                Platform::Web
            }
        };

        let state = match json.state.ok_or(ValidationError::MissingField("state"))? {
            crate::generated_admin::models::list_apps_200_response_apps_inner::State::Active => {
                AppState::Active
            }
            crate::generated_admin::models::list_apps_200_response_apps_inner::State::Deleted => {
                AppState::Deleted
            }
        };

        let platform_config = parse_platform_config(&json, platform)?;

        let created_at = parse_timestamp(json.created_at)?;
        let updated_at = parse_timestamp(json.updated_at)?;

        Ok(Self {
            project_id,
            app_id,
            name,
            pistachio_id,
            display_name,
            platform,
            platform_config,
            api_key: json.api_key,
            state,
            created_at,
            updated_at,
        })
    }
}

fn parse_platform_config(
    json: &ListApps200ResponseAppsInner,
    platform: Platform,
) -> Result<PlatformConfig, ValidationError> {
    match platform {
        Platform::Ios => {
            let ios = json
                .ios
                .as_ref()
                .ok_or(ValidationError::MissingField("ios"))?;
            Ok(PlatformConfig::Ios(IosAppConfig {
                bundle_id: ios.bundle_id.clone(),
                app_store_id: ios.app_store_id.clone(),
                team_id: ios.team_id.clone(),
            }))
        }
        Platform::Android => {
            let android = json
                .android
                .as_ref()
                .ok_or(ValidationError::MissingField("android"))?;
            Ok(PlatformConfig::Android(AndroidAppConfig {
                package_name: android.package_name.clone(),
                sha1_hashes: android.sha1_hashes.clone().unwrap_or_default(),
                sha256_hashes: android.sha256_hashes.clone().unwrap_or_default(),
            }))
        }
        Platform::Macos => {
            let macos = json
                .macos
                .as_ref()
                .ok_or(ValidationError::MissingField("macos"))?;
            Ok(PlatformConfig::Macos(MacosAppConfig {
                bundle_id: macos.bundle_id.clone(),
                team_id: macos.team_id.clone(),
                mac_app_store_id: macos.mac_app_store_id.clone(),
            }))
        }
        Platform::Windows => {
            let windows = json
                .windows
                .as_ref()
                .ok_or(ValidationError::MissingField("windows"))?;
            Ok(PlatformConfig::Windows(WindowsAppConfig {
                package_family_name: windows.package_family_name.clone(),
                publisher_id: windows.publisher_id.clone(),
                cert_thumbprints: windows.cert_thumbprints.clone().unwrap_or_default(),
                microsoft_store_id: windows.microsoft_store_id.clone(),
            }))
        }
        Platform::Linux => {
            let linux = json
                .linux
                .as_ref()
                .ok_or(ValidationError::MissingField("linux"))?;
            Ok(PlatformConfig::Linux(LinuxAppConfig {
                app_name: linux.app_name.clone(),
                flatpak_id: linux.flatpak_id.clone(),
                snap_name: linux.snap_name.clone(),
            }))
        }
        Platform::Web => {
            let web = json
                .web
                .as_ref()
                .ok_or(ValidationError::MissingField("web"))?;
            Ok(PlatformConfig::Web(WebAppConfig {
                authorized_domains: web.authorized_domains.clone().unwrap_or_default(),
                auth_domain: web.auth_domain.clone(),
            }))
        }
    }
}
