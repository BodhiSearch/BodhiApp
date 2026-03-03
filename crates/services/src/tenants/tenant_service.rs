use super::error::{Result, TenantError};
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
    status: AppStatus,
  ) -> Result<Tenant>;
  async fn update_status(&self, status: &AppStatus) -> Result<()>;
  /// Update the status of a specific tenant by its ID.
  async fn update_status_by_id(&self, tenant_id: &str, status: &AppStatus) -> Result<()>;
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
    status: AppStatus,
  ) -> Result<Tenant> {
    let row = self
      .db_service
      .create_tenant(client_id, client_secret, &status)
      .await?;
    Ok(Tenant::from(row))
  }

  async fn update_status(&self, status: &AppStatus) -> Result<()> {
    let tenant = self
      .get_standalone_app()
      .await?
      .ok_or(TenantError::NotFound)?;
    self
      .db_service
      .update_tenant_status(&tenant.client_id, status)
      .await?;
    Ok(())
  }

  async fn update_status_by_id(&self, tenant_id: &str, status: &AppStatus) -> Result<()> {
    let tenant = self
      .get_tenant(tenant_id)
      .await?
      .ok_or(TenantError::NotFound)?;
    self
      .db_service
      .update_tenant_status(&tenant.client_id, status)
      .await?;
    Ok(())
  }
}

#[cfg(test)]
#[path = "test_tenant_service.rs"]
mod test_tenant_service;
