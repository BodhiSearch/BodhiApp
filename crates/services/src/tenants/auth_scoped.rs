use super::error::TenantError;
use crate::{AppService, AppStatus, AuthContext, Tenant};
use std::sync::Arc;

pub struct AuthScopedTenantService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedTenantService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  pub fn auth_context(&self) -> &AuthContext {
    &self.auth_context
  }

  /// List all tenants the authenticated user has membership in, regardless of status.
  pub async fn list_my_tenants(&self) -> Result<Vec<Tenant>, TenantError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .tenant_service()
      .list_user_tenants(user_id)
      .await
  }

  /// Check if the authenticated user has any tenant memberships.
  pub async fn has_memberships(&self) -> Result<bool, TenantError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .tenant_service()
      .has_tenant_memberships(user_id)
      .await
  }

  // --- Passthrough methods ---
  // These methods don't need user_id scoping because they operate on explicit
  // tenant_id/user_id parameters or global lookups (not user-specific queries).

  pub async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, TenantError> {
    self
      .app_service
      .tenant_service()
      .get_tenant(tenant_id)
      .await
  }

  pub async fn get_tenant_by_client_id(
    &self,
    client_id: &str,
  ) -> Result<Option<Tenant>, TenantError> {
    self
      .app_service
      .tenant_service()
      .get_tenant_by_client_id(client_id)
      .await
  }

  pub async fn get_status(&self, tenant_id: &str) -> Result<AppStatus, TenantError> {
    self
      .app_service
      .tenant_service()
      .get_status(tenant_id)
      .await
  }

  pub async fn get_standalone_app(&self) -> Result<Option<Tenant>, TenantError> {
    self.app_service.tenant_service().get_standalone_app().await
  }

  /// Passthrough: takes explicit params, no user_id scoping needed.
  pub async fn create_tenant(
    &self,
    client_id: &str,
    client_secret: &str,
    name: &str,
    description: Option<String>,
    status: AppStatus,
    created_by: Option<String>,
  ) -> Result<Tenant, TenantError> {
    self
      .app_service
      .tenant_service()
      .create_tenant(
        client_id,
        client_secret,
        name,
        description,
        status,
        created_by,
      )
      .await
  }

  /// Passthrough: takes explicit tenant_id and user_id params, no scoping needed.
  pub async fn set_tenant_ready(&self, tenant_id: &str, user_id: &str) -> Result<(), TenantError> {
    self
      .app_service
      .tenant_service()
      .set_tenant_ready(tenant_id, user_id)
      .await
  }

  /// Passthrough: takes explicit tenant_id and user_id params, no scoping needed.
  pub async fn upsert_tenant_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<(), TenantError> {
    self
      .app_service
      .tenant_service()
      .upsert_tenant_user(tenant_id, user_id)
      .await
  }
}
