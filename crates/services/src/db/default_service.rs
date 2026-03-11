use crate::db::{DbCore, DbError, TimeService};
use crate::EnvType;
use chrono::{DateTime, Utc};
use sea_orm::{
  ConnectionTrait, DatabaseConnection, DatabaseTransaction, Statement, TransactionTrait,
};
use sea_orm_migration::MigratorTrait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DefaultDbService {
  pub(crate) db: DatabaseConnection,
  pub(crate) time_service: Arc<dyn TimeService>,
  pub(crate) encryption_key: Vec<u8>,
  pub(crate) env_type: EnvType,
}

impl DefaultDbService {
  pub fn new(
    db: DatabaseConnection,
    time_service: Arc<dyn TimeService>,
    encryption_key: Vec<u8>,
  ) -> Self {
    DefaultDbService {
      db,
      time_service,
      encryption_key,
      env_type: EnvType::Development,
    }
  }

  pub fn with_env_type(mut self, env_type: EnvType) -> Self {
    self.env_type = env_type;
    self
  }

  pub fn db(&self) -> &DatabaseConnection {
    &self.db
  }

  pub async fn with_tenant_txn<T, F>(&self, tenant_id: &str, f: F) -> Result<T, DbError>
  where
    F: for<'a> FnOnce(
      &'a DatabaseTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<T, DbError>> + Send + 'a>>,
    T: Send,
  {
    let txn = self.begin_tenant_txn(tenant_id).await?;
    let result = f(&txn).await?;
    txn.commit().await.map_err(DbError::from)?;
    Ok(result)
  }
}

#[async_trait::async_trait]
impl DbCore for DefaultDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    crate::db::sea_migrations::Migrator::up(&self.db, None).await?;
    Ok(())
  }

  fn now(&self) -> DateTime<Utc> {
    self.time_service.utc_now()
  }

  fn encryption_key(&self) -> &[u8] {
    &self.encryption_key
  }

  async fn begin_tenant_txn(
    &self,
    tenant_id: &str,
  ) -> Result<sea_orm::DatabaseTransaction, DbError> {
    let txn = self.db.begin().await.map_err(DbError::from)?;
    if self.db.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      txn
        .execute(Statement::from_sql_and_values(
          sea_orm::DatabaseBackend::Postgres,
          "SELECT set_config('app.current_tenant_id', $1, true)",
          [tenant_id.into()],
        ))
        .await
        .map_err(DbError::from)?;
    }
    Ok(txn)
  }

  async fn reset_all_tables(&self) -> Result<(), DbError> {
    if self.env_type == EnvType::Production {
      return Err(DbError::ProductionGuard("reset_all_tables".to_string()));
    }
    let backend = self.db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Postgres => {
        // Use TRUNCATE CASCADE for Postgres
        // Note: tenants table is NOT reset - it holds tenant (client credentials, status)
        sea_orm::ConnectionTrait::execute_unprepared(
          &self.db,
          "TRUNCATE TABLE settings,
             app_access_requests,
             toolsets,
             mcps,
             mcp_oauth_tokens,
             mcp_oauth_configs,
             mcp_auth_headers,
             mcp_servers,
             app_toolset_configs,
             user_aliases,
             model_metadata,
             api_model_aliases,
             api_tokens,
             user_access_requests,
             download_requests
           CASCADE",
        )
        .await?;
        // Remove non-creator tenant memberships (preserves creator's membership)
        sea_orm::ConnectionTrait::execute_unprepared(
          &self.db,
          "DELETE FROM tenants_users tu
           WHERE NOT EXISTS (
             SELECT 1 FROM tenants t
             WHERE t.id = tu.tenant_id AND t.created_by = tu.user_id
           )",
        )
        .await?;
      }
      _ => {
        // Use DELETE FROM for SQLite
        // Note: tenants table is NOT reset - it holds tenant (client credentials, status)
        sea_orm::ConnectionTrait::execute_unprepared(
          &self.db,
          "DELETE FROM settings;
           DELETE FROM app_access_requests;
           DELETE FROM toolsets;
           DELETE FROM mcps;
           DELETE FROM mcp_oauth_tokens;
           DELETE FROM mcp_oauth_configs;
           DELETE FROM mcp_auth_headers;
           DELETE FROM mcp_servers;
           DELETE FROM app_toolset_configs;
           DELETE FROM user_aliases;
           DELETE FROM model_metadata;
           DELETE FROM api_model_aliases;
           DELETE FROM api_tokens;
           DELETE FROM user_access_requests;
           DELETE FROM download_requests;
           DELETE FROM tenants_users
             WHERE NOT EXISTS (
               SELECT 1 FROM tenants t
               WHERE t.id = tenants_users.tenant_id AND t.created_by = tenants_users.user_id
             );",
        )
        .await?;
      }
    }

    Ok(())
  }

  async fn reset_tenants(&self) -> Result<(), DbError> {
    if self.env_type == EnvType::Production {
      return Err(DbError::ProductionGuard("reset_tenants".to_string()));
    }
    let backend = self.db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Postgres => {
        sea_orm::ConnectionTrait::execute_unprepared(&self.db, "TRUNCATE TABLE tenants CASCADE")
          .await?;
      }
      _ => {
        sea_orm::ConnectionTrait::execute_unprepared(
          &self.db,
          "DELETE FROM tenants_users; DELETE FROM tenants",
        )
        .await?;
      }
    }
    Ok(())
  }
}
