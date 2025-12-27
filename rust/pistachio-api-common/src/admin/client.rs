use crate::credentials::AdminCredentials;
use crate::error::PistachioApiClientError;

use super::app::{
    CreateAppError, CreateAppRequest, CreateAppResponse, DeleteAppError, DeleteAppRequest,
    DeleteAppResponse, GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse, GetAppError,
    GetAppRequest, GetAppResponse, ListAppsError, ListAppsRequest, ListAppsResponse,
    SearchAppsError, SearchAppsRequest, SearchAppsResponse, UndeleteAppError, UndeleteAppRequest,
    UndeleteAppResponse, UpdateAppError, UpdateAppRequest, UpdateAppResponse,
};
use super::project::{
    CreateProjectError, CreateProjectRequest, CreateProjectResponse, DeleteProjectError,
    DeleteProjectRequest, DeleteProjectResponse, GetAdminSdkConfigError, GetAdminSdkConfigRequest,
    GetAdminSdkConfigResponse, GetProjectError, GetProjectRequest, GetProjectResponse,
    ListProjectsError, ListProjectsRequest, ListProjectsResponse, SearchProjectsError,
    SearchProjectsRequest, SearchProjectsResponse, UndeleteProjectError, UndeleteProjectRequest,
    UndeleteProjectResponse, UpdateProjectError, UpdateProjectRequest, UpdateProjectResponse,
};
use super::tenant::{
    CreateTenantError, CreateTenantRequest, CreateTenantResponse, DeleteTenantError,
    DeleteTenantRequest, DeleteTenantResponse, GetTenantError, GetTenantRequest, GetTenantResponse,
    ListTenantsError, ListTenantsRequest, ListTenantsResponse, SearchTenantsError,
    SearchTenantsRequest, SearchTenantsResponse, UpdateTenantError, UpdateTenantRequest,
    UpdateTenantResponse,
};

/// Trait for Pistachio Admin API clients.
///
/// This trait defines the operations available for administrative tasks
/// such as project management. All operations require service account
/// authentication.
#[cfg_attr(
    any(feature = "single-threaded", target_arch = "wasm32"),
    async_trait::async_trait(?Send)
)]
#[cfg_attr(
    not(any(feature = "single-threaded", target_arch = "wasm32")),
    async_trait::async_trait
)]
pub trait PistachioAdminClient: Sized {
    /// Creates a new client with the given endpoint and credentials.
    fn new(
        endpoint: impl AsRef<str>,
        credentials: AdminCredentials,
    ) -> Result<Self, PistachioApiClientError>;

    /// Connects to the API server.
    async fn connect(self) -> Result<Self, PistachioApiClientError>;

    /// Creates a new project.
    ///
    /// Projects are the top-level container for apps, users, and resources.
    /// Each project can have multiple apps across different platforms
    /// that share the same user pool.
    async fn create_project(
        &mut self,
        req: CreateProjectRequest,
    ) -> Result<CreateProjectResponse, CreateProjectError>;

    /// Retrieves a project by its ID.
    ///
    /// Returns the project details including its current state and resources.
    async fn get_project(
        &mut self,
        req: GetProjectRequest,
    ) -> Result<GetProjectResponse, GetProjectError>;

    /// Updates an existing project.
    ///
    /// Currently only the display_name can be updated.
    async fn update_project(
        &mut self,
        req: UpdateProjectRequest,
    ) -> Result<UpdateProjectResponse, UpdateProjectError>;

    /// Soft-deletes a project.
    ///
    /// The project will be marked as DELETED and will be permanently removed
    /// after 30 days. During this period, the project can be restored.
    async fn delete_project(
        &mut self,
        req: DeleteProjectRequest,
    ) -> Result<DeleteProjectResponse, DeleteProjectError>;

    /// Restores a soft-deleted project.
    ///
    /// The project must be in the DELETED state and within the 30-day
    /// grace period. The project will be restored to ACTIVE state.
    async fn undelete_project(
        &mut self,
        req: UndeleteProjectRequest,
    ) -> Result<UndeleteProjectResponse, UndeleteProjectError>;

    /// Lists all projects accessible to the service account.
    ///
    /// Results are paginated and can optionally include deleted projects.
    async fn list_projects(
        &mut self,
        req: ListProjectsRequest,
    ) -> Result<ListProjectsResponse, ListProjectsError>;

    /// Searches for projects using full-text search.
    ///
    /// Provides advanced search capabilities including field-specific queries,
    /// boolean operators, and flexible sorting.
    async fn search_projects(
        &mut self,
        req: SearchProjectsRequest,
    ) -> Result<SearchProjectsResponse, SearchProjectsError>;

    /// Retrieves the Admin SDK configuration for a project.
    ///
    /// Returns configuration needed to initialize the Admin SDK.
    async fn get_admin_sdk_config(
        &mut self,
        req: GetAdminSdkConfigRequest,
    ) -> Result<GetAdminSdkConfigResponse, GetAdminSdkConfigError>;

    // =========================================================================
    // Tenant Operations
    // =========================================================================

    /// Creates a new tenant within a project.
    ///
    /// Tenants provide multi-tenant isolation within a project, allowing
    /// each tenant to have its own user pool and authentication configuration.
    async fn create_tenant(
        &mut self,
        req: CreateTenantRequest,
    ) -> Result<CreateTenantResponse, CreateTenantError>;

    /// Retrieves a tenant by its ID.
    ///
    /// Returns the tenant details including its authentication configuration.
    async fn get_tenant(
        &mut self,
        req: GetTenantRequest,
    ) -> Result<GetTenantResponse, GetTenantError>;

    /// Updates an existing tenant.
    ///
    /// Updates the tenant's display name, authentication settings, or MFA configuration.
    async fn update_tenant(
        &mut self,
        req: UpdateTenantRequest,
    ) -> Result<UpdateTenantResponse, UpdateTenantError>;

    /// Permanently deletes a tenant.
    ///
    /// This operation is irreversible. All users and data associated with
    /// the tenant will be permanently deleted.
    async fn delete_tenant(
        &mut self,
        req: DeleteTenantRequest,
    ) -> Result<DeleteTenantResponse, DeleteTenantError>;

    /// Lists all tenants within a project.
    ///
    /// Results are paginated and sorted by creation time by default.
    async fn list_tenants(
        &mut self,
        req: ListTenantsRequest,
    ) -> Result<ListTenantsResponse, ListTenantsError>;

    /// Searches for tenants within a project using full-text search.
    ///
    /// Provides advanced search capabilities including field-specific queries,
    /// boolean operators, and flexible sorting.
    async fn search_tenants(
        &mut self,
        req: SearchTenantsRequest,
    ) -> Result<SearchTenantsResponse, SearchTenantsError>;

    // =========================================================================
    // App Operations
    // =========================================================================

    /// Registers a new app in the project.
    ///
    /// API keys are auto-generated for each app.
    /// Platform-specific configuration is provided via the platform_config field.
    async fn create_app(
        &mut self,
        req: CreateAppRequest,
    ) -> Result<CreateAppResponse, CreateAppError>;

    /// Retrieves an app by its ID.
    ///
    /// Returns the app details including its platform configuration.
    async fn get_app(&mut self, req: GetAppRequest) -> Result<GetAppResponse, GetAppError>;

    /// Updates an existing app.
    ///
    /// Updates the app's display name or platform-specific configuration.
    async fn update_app(
        &mut self,
        req: UpdateAppRequest,
    ) -> Result<UpdateAppResponse, UpdateAppError>;

    /// Soft-deletes an app.
    ///
    /// The app enters DELETED state and will be permanently removed after 30 days.
    /// During this period, the app can be restored using undelete_app.
    async fn delete_app(
        &mut self,
        req: DeleteAppRequest,
    ) -> Result<DeleteAppResponse, DeleteAppError>;

    /// Restores a soft-deleted app.
    ///
    /// Only works for apps in DELETED state within the 30-day grace period.
    /// The app will be restored to ACTIVE state.
    async fn undelete_app(
        &mut self,
        req: UndeleteAppRequest,
    ) -> Result<UndeleteAppResponse, UndeleteAppError>;

    /// Lists all apps within a project.
    ///
    /// Results are paginated and can optionally include deleted apps.
    async fn list_apps(&mut self, req: ListAppsRequest) -> Result<ListAppsResponse, ListAppsError>;

    /// Searches for apps within a project using full-text search.
    ///
    /// Provides advanced search capabilities including field-specific queries,
    /// boolean operators, and flexible sorting.
    async fn search_apps(
        &mut self,
        req: SearchAppsRequest,
    ) -> Result<SearchAppsResponse, SearchAppsError>;

    /// Retrieves the SDK configuration for an app.
    ///
    /// Returns platform-specific configuration files:
    /// - iOS: GoogleService-Info.plist
    /// - Android: google-services.json
    /// - macOS/Windows/Linux: pistachio-config.json
    /// - Web: JavaScript config object
    async fn get_app_config(
        &mut self,
        req: GetAppConfigRequest,
    ) -> Result<GetAppConfigResponse, GetAppConfigError>;
}
