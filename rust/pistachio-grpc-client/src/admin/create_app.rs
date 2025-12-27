use libgn::app::{
    AndroidAppConfig, App, AppDisplayName, AppId, AppName, AppState, IosAppConfig, LinuxAppConfig,
    MacosAppConfig, Platform, PlatformConfig, WebAppConfig, WindowsAppConfig,
};
use libgn::pistachio_id::AppId as PistachioAppId;
use libgn::project::ProjectId;
use pistachio_api_common::admin::app::{CreateAppError, CreateAppRequest, CreateAppResponse};
use pistachio_api_common::error::ValidationError;
use tonic::Code;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing::{debug, error};

use pistachio_api::pistachio::admin::v1::pistachio_admin_client::PistachioAdminClient;

use crate::types::{FromProto, IntoProto, timestamp_to_datetime};

pub(crate) async fn handle_create_app<I: Interceptor>(
    client: &mut PistachioAdminClient<InterceptedService<Channel, I>>,
    req: CreateAppRequest,
) -> Result<CreateAppResponse, CreateAppError> {
    debug!("creating proto request");
    let request = req.into_proto();
    debug!(?request, "created proto request");

    let response = client
        .create_app(request)
        .await
        .map_err(|status| {
            error!(?status, "Error in create_app response");
            match status.code() {
                Code::InvalidArgument => CreateAppError::BadRequest(status.message().to_string()),
                Code::AlreadyExists => CreateAppError::AlreadyExists,
                Code::NotFound => CreateAppError::NotFound,
                Code::Unauthenticated => {
                    CreateAppError::Unauthenticated(status.message().to_string())
                }
                Code::PermissionDenied => {
                    CreateAppError::PermissionDenied(status.message().to_string())
                }
                Code::Internal => CreateAppError::ServiceError(status.message().to_string()),
                Code::Unavailable => {
                    CreateAppError::ServiceUnavailable(status.message().to_string())
                }
                _ => CreateAppError::Unknown(status.message().to_string()),
            }
        })?
        .into_inner();

    CreateAppResponse::from_proto(response).map_err(CreateAppError::ResponseValidationError)
}

// =============================================================================
// Proto conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::admin::v1::CreateAppRequest> for CreateAppRequest {
    fn into_proto(self) -> pistachio_api::pistachio::admin::v1::CreateAppRequest {
        use pistachio_api::pistachio::admin::v1::create_app_request::PlatformConfig as ProtoPlatformConfig;

        let platform_config = match self.platform_config {
            PlatformConfig::Ios(c) => Some(ProtoPlatformConfig::Ios(c.into_proto())),
            PlatformConfig::Android(c) => Some(ProtoPlatformConfig::Android(c.into_proto())),
            PlatformConfig::Macos(c) => Some(ProtoPlatformConfig::Macos(c.into_proto())),
            PlatformConfig::Windows(c) => Some(ProtoPlatformConfig::Windows(c.into_proto())),
            PlatformConfig::Linux(c) => Some(ProtoPlatformConfig::Linux(c.into_proto())),
            PlatformConfig::Web(c) => Some(ProtoPlatformConfig::Web(c.into_proto())),
        };

        pistachio_api::pistachio::admin::v1::CreateAppRequest {
            project_id: self.project_id.to_string(),
            display_name: self.display_name.to_string(),
            platform: self.platform.to_i32(),
            platform_config,
        }
    }
}

impl FromProto<pistachio_api::pistachio::admin::v1::CreateAppResponse> for CreateAppResponse {
    type Error = ValidationError;

    fn from_proto(
        proto: pistachio_api::pistachio::admin::v1::CreateAppResponse,
    ) -> Result<Self, Self::Error> {
        let app_proto = proto.app.ok_or(ValidationError::MissingField("app"))?;

        let app = App::from_proto(app_proto)?;

        Ok(Self { app })
    }
}

impl FromProto<pistachio_api::pistachio::types::v1::App> for App {
    type Error = ValidationError;

    fn from_proto(proto: pistachio_api::pistachio::types::v1::App) -> Result<Self, Self::Error> {
        let name =
            AppName::parse(&proto.name).map_err(|_| ValidationError::InvalidValue("name"))?;
        let project_id = ProjectId::parse(&proto.project_id)
            .map_err(|_| ValidationError::InvalidValue("project_id"))?;
        let app_id =
            AppId::parse(&proto.app_id).map_err(|_| ValidationError::InvalidValue("app_id"))?;
        let pistachio_id = PistachioAppId::parse(&proto.pistachio_id)?;
        let display_name = AppDisplayName::parse(&proto.display_name)
            .map_err(|_| ValidationError::InvalidValue("display_name"))?;
        let platform =
            Platform::from_i32(proto.platform).ok_or(ValidationError::InvalidValue("platform"))?;
        let state =
            AppState::from_i32(proto.state).ok_or(ValidationError::InvalidValue("state"))?;
        let created_at = timestamp_to_datetime(proto.created_at)?;
        let updated_at = timestamp_to_datetime(proto.updated_at)?;

        let platform_config = platform_config_from_proto(&proto)?;

        Ok(Self {
            project_id,
            app_id,
            name,
            pistachio_id,
            display_name,
            platform,
            platform_config,
            api_key: if proto.api_key.is_empty() {
                None
            } else {
                Some(proto.api_key)
            },
            state,
            created_at,
            updated_at,
        })
    }
}

fn platform_config_from_proto(
    proto: &pistachio_api::pistachio::types::v1::App,
) -> Result<PlatformConfig, ValidationError> {
    use pistachio_api::pistachio::types::v1::app::PlatformConfig as ProtoPlatformConfig;

    match &proto.platform_config {
        Some(ProtoPlatformConfig::Ios(c)) => Ok(PlatformConfig::Ios(IosAppConfig::from_proto(c)?)),
        Some(ProtoPlatformConfig::Android(c)) => {
            Ok(PlatformConfig::Android(AndroidAppConfig::from_proto(c)?))
        }
        Some(ProtoPlatformConfig::Macos(c)) => {
            Ok(PlatformConfig::Macos(MacosAppConfig::from_proto(c)?))
        }
        Some(ProtoPlatformConfig::Windows(c)) => {
            Ok(PlatformConfig::Windows(WindowsAppConfig::from_proto(c)?))
        }
        Some(ProtoPlatformConfig::Linux(c)) => {
            Ok(PlatformConfig::Linux(LinuxAppConfig::from_proto(c)?))
        }
        Some(ProtoPlatformConfig::Web(c)) => Ok(PlatformConfig::Web(WebAppConfig::from_proto(c)?)),
        None => Err(ValidationError::MissingField("platform_config")),
    }
}

// =============================================================================
// Platform Config Conversions
// =============================================================================

impl IntoProto<pistachio_api::pistachio::types::v1::IosAppConfig> for IosAppConfig {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::IosAppConfig {
        pistachio_api::pistachio::types::v1::IosAppConfig {
            bundle_id: self.bundle_id,
            app_store_id: self.app_store_id.unwrap_or_default(),
            team_id: self.team_id.unwrap_or_default(),
        }
    }
}

impl FromProto<&pistachio_api::pistachio::types::v1::IosAppConfig> for IosAppConfig {
    type Error = ValidationError;

    fn from_proto(
        proto: &pistachio_api::pistachio::types::v1::IosAppConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            bundle_id: proto.bundle_id.clone(),
            app_store_id: if proto.app_store_id.is_empty() {
                None
            } else {
                Some(proto.app_store_id.clone())
            },
            team_id: if proto.team_id.is_empty() {
                None
            } else {
                Some(proto.team_id.clone())
            },
        })
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::AndroidAppConfig> for AndroidAppConfig {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::AndroidAppConfig {
        pistachio_api::pistachio::types::v1::AndroidAppConfig {
            package_name: self.package_name,
            sha1_hashes: self.sha1_hashes,
            sha256_hashes: self.sha256_hashes,
        }
    }
}

impl FromProto<&pistachio_api::pistachio::types::v1::AndroidAppConfig> for AndroidAppConfig {
    type Error = ValidationError;

    fn from_proto(
        proto: &pistachio_api::pistachio::types::v1::AndroidAppConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            package_name: proto.package_name.clone(),
            sha1_hashes: proto.sha1_hashes.clone(),
            sha256_hashes: proto.sha256_hashes.clone(),
        })
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::MacosAppConfig> for MacosAppConfig {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::MacosAppConfig {
        pistachio_api::pistachio::types::v1::MacosAppConfig {
            bundle_id: self.bundle_id,
            team_id: self.team_id.unwrap_or_default(),
            mac_app_store_id: self.mac_app_store_id.unwrap_or_default(),
        }
    }
}

impl FromProto<&pistachio_api::pistachio::types::v1::MacosAppConfig> for MacosAppConfig {
    type Error = ValidationError;

    fn from_proto(
        proto: &pistachio_api::pistachio::types::v1::MacosAppConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            bundle_id: proto.bundle_id.clone(),
            team_id: if proto.team_id.is_empty() {
                None
            } else {
                Some(proto.team_id.clone())
            },
            mac_app_store_id: if proto.mac_app_store_id.is_empty() {
                None
            } else {
                Some(proto.mac_app_store_id.clone())
            },
        })
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::WindowsAppConfig> for WindowsAppConfig {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::WindowsAppConfig {
        pistachio_api::pistachio::types::v1::WindowsAppConfig {
            package_family_name: self.package_family_name.unwrap_or_default(),
            publisher_id: self.publisher_id.unwrap_or_default(),
            cert_thumbprints: self.cert_thumbprints,
            microsoft_store_id: self.microsoft_store_id.unwrap_or_default(),
        }
    }
}

impl FromProto<&pistachio_api::pistachio::types::v1::WindowsAppConfig> for WindowsAppConfig {
    type Error = ValidationError;

    fn from_proto(
        proto: &pistachio_api::pistachio::types::v1::WindowsAppConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            package_family_name: if proto.package_family_name.is_empty() {
                None
            } else {
                Some(proto.package_family_name.clone())
            },
            publisher_id: if proto.publisher_id.is_empty() {
                None
            } else {
                Some(proto.publisher_id.clone())
            },
            cert_thumbprints: proto.cert_thumbprints.clone(),
            microsoft_store_id: if proto.microsoft_store_id.is_empty() {
                None
            } else {
                Some(proto.microsoft_store_id.clone())
            },
        })
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::LinuxAppConfig> for LinuxAppConfig {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::LinuxAppConfig {
        pistachio_api::pistachio::types::v1::LinuxAppConfig {
            app_name: self.app_name,
            flatpak_id: self.flatpak_id.unwrap_or_default(),
            snap_name: self.snap_name.unwrap_or_default(),
        }
    }
}

impl FromProto<&pistachio_api::pistachio::types::v1::LinuxAppConfig> for LinuxAppConfig {
    type Error = ValidationError;

    fn from_proto(
        proto: &pistachio_api::pistachio::types::v1::LinuxAppConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            app_name: proto.app_name.clone(),
            flatpak_id: if proto.flatpak_id.is_empty() {
                None
            } else {
                Some(proto.flatpak_id.clone())
            },
            snap_name: if proto.snap_name.is_empty() {
                None
            } else {
                Some(proto.snap_name.clone())
            },
        })
    }
}

impl IntoProto<pistachio_api::pistachio::types::v1::WebAppConfig> for WebAppConfig {
    fn into_proto(self) -> pistachio_api::pistachio::types::v1::WebAppConfig {
        pistachio_api::pistachio::types::v1::WebAppConfig {
            authorized_domains: self.authorized_domains,
            auth_domain: self.auth_domain.unwrap_or_default(),
        }
    }
}

impl FromProto<&pistachio_api::pistachio::types::v1::WebAppConfig> for WebAppConfig {
    type Error = ValidationError;

    fn from_proto(
        proto: &pistachio_api::pistachio::types::v1::WebAppConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            authorized_domains: proto.authorized_domains.clone(),
            auth_domain: if proto.auth_domain.is_empty() {
                None
            } else {
                Some(proto.auth_domain.clone())
            },
        })
    }
}
