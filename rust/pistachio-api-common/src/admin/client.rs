use crate::credentials::AdminCredentials;
use crate::error::PistachioApiClientError;

use super::app::{
    CreateAppError, CreateAppRequest, CreateAppResponse, DeleteAppError, DeleteAppRequest,
    DeleteAppResponse, GetAppConfigError, GetAppConfigRequest, GetAppConfigResponse, GetAppError,
    GetAppRequest, GetAppResponse, ListAppsError, ListAppsRequest, ListAppsResponse,
    SearchAppsError, SearchAppsRequest, SearchAppsResponse, UndeleteAppError, UndeleteAppRequest,
    UndeleteAppResponse, UpdateAppError, UpdateAppRequest, UpdateAppResponse,
};
use super::auth_provider::{
    DeleteProjectAuthProviderError, DeleteProjectAuthProviderRequest,
    DeleteProjectAuthProviderResponse, DeleteTenantAuthProviderError,
    DeleteTenantAuthProviderRequest, DeleteTenantAuthProviderResponse,
    GetEffectiveTenantAuthProvidersError, GetEffectiveTenantAuthProvidersRequest,
    GetEffectiveTenantAuthProvidersResponse, GetProjectAuthProviderError,
    GetProjectAuthProviderRequest, GetProjectAuthProviderResponse, ListProjectAuthProvidersError,
    ListProjectAuthProvidersRequest, ListProjectAuthProvidersResponse,
    ListTenantAuthProvidersError, ListTenantAuthProvidersRequest, ListTenantAuthProvidersResponse,
    UpdateProjectAuthProviderError, UpdateProjectAuthProviderRequest,
    UpdateProjectAuthProviderResponse, UpdateTenantAuthProviderError,
    UpdateTenantAuthProviderRequest, UpdateTenantAuthProviderResponse,
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
use super::user::{
    CreateProjectUserError, CreateProjectUserRequest, CreateProjectUserResponse,
    CreateTenantUserError, CreateTenantUserRequest, CreateTenantUserResponse,
    DeleteProjectUserError, DeleteProjectUserRequest, DeleteProjectUserResponse,
    DeleteTenantUserError, DeleteTenantUserRequest, DeleteTenantUserResponse, GetProjectUserError,
    GetProjectUserRequest, GetProjectUserResponse, GetTenantUserError, GetTenantUserRequest,
    GetTenantUserResponse, ImportProjectUsersError, ImportProjectUsersRequest,
    ImportProjectUsersResponse, ImportTenantUsersError, ImportTenantUsersRequest,
    ImportTenantUsersResponse, ListProjectUsersError, ListProjectUsersRequest,
    ListProjectUsersResponse, ListTenantUsersError, ListTenantUsersRequest,
    ListTenantUsersResponse, SearchProjectUsersError, SearchProjectUsersRequest,
    SearchProjectUsersResponse, SearchTenantUsersError, SearchTenantUsersRequest,
    SearchTenantUsersResponse, UpdateProjectUserError, UpdateProjectUserRequest,
    UpdateProjectUserResponse, UpdateTenantUserError, UpdateTenantUserRequest,
    UpdateTenantUserResponse,
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

    // =========================================================================
    // Auth Provider Operations
    // =========================================================================

    /// Lists all auth providers for a project.
    ///
    /// Returns all configured authentication providers ordered by display order.
    async fn list_project_auth_providers(
        &mut self,
        req: ListProjectAuthProvidersRequest,
    ) -> Result<ListProjectAuthProvidersResponse, ListProjectAuthProvidersError>;

    /// Retrieves a specific auth provider for a project.
    ///
    /// Returns the provider configuration including type-specific settings.
    async fn get_project_auth_provider(
        &mut self,
        req: GetProjectAuthProviderRequest,
    ) -> Result<GetProjectAuthProviderResponse, GetProjectAuthProviderError>;

    /// Creates or updates an auth provider for a project.
    ///
    /// This is an upsert operation: if the provider exists, it is updated;
    /// otherwise, a new provider is created.
    async fn update_project_auth_provider(
        &mut self,
        req: UpdateProjectAuthProviderRequest,
    ) -> Result<UpdateProjectAuthProviderResponse, UpdateProjectAuthProviderError>;

    /// Permanently deletes an auth provider from a project.
    ///
    /// This operation is irreversible.
    async fn delete_project_auth_provider(
        &mut self,
        req: DeleteProjectAuthProviderRequest,
    ) -> Result<DeleteProjectAuthProviderResponse, DeleteProjectAuthProviderError>;

    /// Lists tenant-level auth provider overrides.
    ///
    /// Returns only overrides that have been explicitly set for this tenant.
    async fn list_tenant_auth_providers(
        &mut self,
        req: ListTenantAuthProvidersRequest,
    ) -> Result<ListTenantAuthProvidersResponse, ListTenantAuthProvidersError>;

    /// Creates or updates a tenant-level auth provider override.
    ///
    /// Tenant overrides allow customizing auth provider settings per tenant.
    /// Fields that are not set inherit from the project-level configuration.
    async fn update_tenant_auth_provider(
        &mut self,
        req: UpdateTenantAuthProviderRequest,
    ) -> Result<UpdateTenantAuthProviderResponse, UpdateTenantAuthProviderError>;

    /// Removes a tenant-level auth provider override.
    ///
    /// After deletion, the tenant will inherit the project-level configuration.
    async fn delete_tenant_auth_provider(
        &mut self,
        req: DeleteTenantAuthProviderRequest,
    ) -> Result<DeleteTenantAuthProviderResponse, DeleteTenantAuthProviderError>;

    /// Retrieves effective auth provider configuration for a tenant.
    ///
    /// Returns the merged configuration after applying tenant-level overrides
    /// to project-level configuration.
    async fn get_effective_tenant_auth_providers(
        &mut self,
        req: GetEffectiveTenantAuthProvidersRequest,
    ) -> Result<GetEffectiveTenantAuthProvidersResponse, GetEffectiveTenantAuthProvidersError>;

    // =========================================================================
    // Project User Operations
    // =========================================================================

    /// Creates a new user within a project.
    ///
    /// Users in project-level scope are not associated with any tenant (single-tenant mode).
    async fn create_project_user(
        &mut self,
        req: CreateProjectUserRequest,
    ) -> Result<CreateProjectUserResponse, CreateProjectUserError>;

    /// Retrieves a user by their pistachio_id within a project.
    async fn get_project_user(
        &mut self,
        req: GetProjectUserRequest,
    ) -> Result<GetProjectUserResponse, GetProjectUserError>;

    /// Updates an existing user within a project.
    async fn update_project_user(
        &mut self,
        req: UpdateProjectUserRequest,
    ) -> Result<UpdateProjectUserResponse, UpdateProjectUserError>;

    /// Permanently deletes a user from a project.
    ///
    /// This operation is irreversible. All user data will be deleted.
    async fn delete_project_user(
        &mut self,
        req: DeleteProjectUserRequest,
    ) -> Result<DeleteProjectUserResponse, DeleteProjectUserError>;

    /// Lists all users within a project with pagination.
    async fn list_project_users(
        &mut self,
        req: ListProjectUsersRequest,
    ) -> Result<ListProjectUsersResponse, ListProjectUsersError>;

    /// Imports users into a project in batch.
    ///
    /// Supports importing users with password hashes from external systems.
    /// Maximum 1000 users per request.
    async fn import_project_users(
        &mut self,
        req: ImportProjectUsersRequest,
    ) -> Result<ImportProjectUsersResponse, ImportProjectUsersError>;

    /// Searches for users within a project using full-text search.
    async fn search_project_users(
        &mut self,
        req: SearchProjectUsersRequest,
    ) -> Result<SearchProjectUsersResponse, SearchProjectUsersError>;

    // =========================================================================
    // Tenant User Operations
    // =========================================================================

    /// Creates a new user within a tenant.
    ///
    /// Users in tenant-level scope are isolated to that specific tenant.
    async fn create_tenant_user(
        &mut self,
        req: CreateTenantUserRequest,
    ) -> Result<CreateTenantUserResponse, CreateTenantUserError>;

    /// Retrieves a user by their pistachio_id within a tenant.
    async fn get_tenant_user(
        &mut self,
        req: GetTenantUserRequest,
    ) -> Result<GetTenantUserResponse, GetTenantUserError>;

    /// Updates an existing user within a tenant.
    async fn update_tenant_user(
        &mut self,
        req: UpdateTenantUserRequest,
    ) -> Result<UpdateTenantUserResponse, UpdateTenantUserError>;

    /// Permanently deletes a user from a tenant.
    ///
    /// This operation is irreversible. All user data will be deleted.
    async fn delete_tenant_user(
        &mut self,
        req: DeleteTenantUserRequest,
    ) -> Result<DeleteTenantUserResponse, DeleteTenantUserError>;

    /// Lists all users within a tenant with pagination.
    async fn list_tenant_users(
        &mut self,
        req: ListTenantUsersRequest,
    ) -> Result<ListTenantUsersResponse, ListTenantUsersError>;

    /// Imports users into a tenant in batch.
    ///
    /// Supports importing users with password hashes from external systems.
    /// Maximum 1000 users per request.
    async fn import_tenant_users(
        &mut self,
        req: ImportTenantUsersRequest,
    ) -> Result<ImportTenantUsersResponse, ImportTenantUsersError>;

    /// Searches for users within a tenant using full-text search.
    async fn search_tenant_users(
        &mut self,
        req: SearchTenantUsersRequest,
    ) -> Result<SearchTenantUsersResponse, SearchTenantUsersError>;
}
