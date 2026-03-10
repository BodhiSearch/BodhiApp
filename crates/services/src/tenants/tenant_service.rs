use super::error::Result;
use super::{AppStatus, Tenant};
use crate::db::DbService;
use std::sync::Arc;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TenantService: Send + Sync + std::fmt::Debug {
  /// Look up a tenant by its ID.
  async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>>;
  /// Look up a tenant by its OAuth2 client_id.
  async fn get_tenant_by_client_id(&self, client_id: &str) -> Result<Option<Tenant>>;
  /// Get the status of a tenant by its ID.
  async fn get_status(&self, tenant_id: &str) -> Result<AppStatus>;
  /// In standalone mode, returns the single registered app (or None if not yet registered).
  async fn get_standalone_app(&self) -> Result<Option<Tenant>>;
  async fn create_tenant(
    &self,
    client_id: &str,
    client_secret: &str,
    name: &str,
    description: Option<String>,
    status: AppStatus,
    created_by: Option<String>,
  ) -> Result<Tenant>;
  /// Atomically set tenant status to Ready, set created_by, and upsert tenant-user membership.
  async fn set_tenant_ready(&self, tenant_id: &str, user_id: &str) -> Result<()>;
  /// Upsert a tenant-user membership (idempotent).
  async fn upsert_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<()>;
  /// Delete a tenant-user membership (idempotent).
  async fn delete_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<()>;
  /// List all tenants a user has membership in, regardless of status.
  async fn list_user_tenants(&self, user_id: &str) -> Result<Vec<Tenant>>;
  /// Check if a user has any tenant memberships.
  async fn has_tenant_memberships(&self, user_id: &str) -> Result<bool>;
}

#[derive(Debug, derive_new::new)]
pub struct DefaultTenantService {
  db_service: Arc<dyn DbService>,
}

#[async_trait::async_trait]
impl TenantService for DefaultTenantService {
  async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
    let row = self.db_service.get_tenant_by_id(tenant_id).await?;
    Ok(row.map(Tenant::from))
  }

  async fn get_tenant_by_client_id(&self, client_id: &str) -> Result<Option<Tenant>> {
    let row = self.db_service.get_tenant_by_client_id(client_id).await?;
    Ok(row.map(Tenant::from))
  }

  async fn get_status(&self, tenant_id: &str) -> Result<AppStatus> {
    let row = self.db_service.get_tenant_by_id(tenant_id).await?;
    match row {
      None => Ok(AppStatus::default()),
      Some(r) => Ok(r.app_status),
    }
  }

  async fn get_standalone_app(&self) -> Result<Option<Tenant>> {
    let row = self.db_service.get_tenant().await?;
    Ok(row.map(Tenant::from))
  }

  async fn create_tenant(
    &self,
    client_id: &str,
    client_secret: &str,
    name: &str,
    description: Option<String>,
    status: AppStatus,
    created_by: Option<String>,
  ) -> Result<Tenant> {
    let row = self
      .db_service
      .create_tenant(
        client_id,
        client_secret,
        name,
        description,
        &status,
        created_by,
      )
      .await?;
    Ok(Tenant::from(row))
  }

  async fn set_tenant_ready(&self, tenant_id: &str, user_id: &str) -> Result<()> {
    self.db_service.set_tenant_ready(tenant_id, user_id).await?;
    Ok(())
  }

  async fn upsert_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<()> {
    self
      .db_service
      .upsert_tenant_user(tenant_id, user_id)
      .await?;
    Ok(())
  }

  async fn delete_tenant_user(&self, tenant_id: &str, user_id: &str) -> Result<()> {
    self
      .db_service
      .delete_tenant_user(tenant_id, user_id)
      .await?;
    Ok(())
  }

  async fn list_user_tenants(&self, user_id: &str) -> Result<Vec<Tenant>> {
    let rows = self.db_service.list_user_tenants(user_id).await?;
    Ok(rows.into_iter().map(Tenant::from).collect())
  }

  async fn has_tenant_memberships(&self, user_id: &str) -> Result<bool> {
    Ok(self.db_service.has_tenant_memberships(user_id).await?)
  }
}

#[cfg(test)]
#[path = "test_tenant_service.rs"]
mod test_tenant_service;
