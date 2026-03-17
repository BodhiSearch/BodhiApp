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
        // Use DO block to truncate only tables that exist (migration/reset timing mismatch safe)
        // Note: tenants table is NOT reset - it holds tenant (client credentials, status)
        sea_orm::ConnectionTrait::execute_unprepared(
          &self.db,
          "DO $$
           DECLARE
             _tables text[];
             _existing text[];
           BEGIN
             _tables := ARRAY[
               'settings', 'app_access_requests', 'toolsets',
               'mcp_auth_params', 'mcp_oauth_tokens', 'mcp_oauth_config_details',
               'mcp_auth_config_params', 'mcps', 'mcp_auth_configs', 'mcp_servers',
               'app_toolset_configs', 'user_aliases', 'model_metadata',
               'api_model_aliases', 'api_tokens', 'user_access_requests',
               'download_requests'
             ];
             SELECT array_agg(t) INTO _existing
               FROM unnest(_tables) AS t
               WHERE EXISTS (
                 SELECT 1 FROM information_schema.tables
                 WHERE table_schema = 'public' AND table_name = t
               );
             IF _existing IS NOT NULL AND array_length(_existing, 1) > 0 THEN
               EXECUTE 'TRUNCATE TABLE ' || array_to_string(_existing, ', ') || ' CASCADE';
             END IF;
           END $$;",
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
        // Use DELETE FROM for SQLite (safe if table doesn't exist yet)
        // Note: tenants table is NOT reset - it holds tenant (client credentials, status)
        let tables = [
          "settings",
          "app_access_requests",
          "toolsets",
          "mcp_auth_params",
          "mcp_oauth_tokens",
          "mcp_oauth_config_details",
          "mcp_auth_config_params",
          "mcps",
          "mcp_auth_configs",
          "mcp_servers",
          "app_toolset_configs",
          "user_aliases",
          "model_metadata",
          "api_model_aliases",
          "api_tokens",
          "user_access_requests",
          "download_requests",
        ];
        for table in tables {
          let sql = format!(
            "DELETE FROM {table} WHERE EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='{table}')"
          );
          let _ = sea_orm::ConnectionTrait::execute_unprepared(&self.db, &sql).await;
        }
        sea_orm::ConnectionTrait::execute_unprepared(
          &self.db,
          "DELETE FROM tenants_users
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
