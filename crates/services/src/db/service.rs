use crate::db::{
  AccessRepository, AccessRequestRepository, DbCore, DbError, McpRepository, ModelRepository,
  TimeService, TokenRepository, ToolsetRepository, UserAliasRepository,
};
use chrono::{DateTime, Utc};
use derive_new::new;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Super-trait that combines all repository sub-traits.
/// Any type implementing all sub-traits automatically implements DbService
/// via the blanket impl below.
pub trait DbService:
  ModelRepository
  + AccessRepository
  + AccessRequestRepository
  + TokenRepository
  + ToolsetRepository
  + McpRepository
  + UserAliasRepository
  + DbCore
  + Send
  + Sync
  + std::fmt::Debug
{
}

impl<T> DbService for T where
  T: ModelRepository
    + AccessRepository
    + AccessRequestRepository
    + TokenRepository
    + ToolsetRepository
    + McpRepository
    + UserAliasRepository
    + DbCore
    + Send
    + Sync
    + std::fmt::Debug
{
}

#[derive(Debug, Clone, new)]
pub struct SqliteDbService {
  pub(crate) pool: SqlitePool,
  pub(crate) time_service: Arc<dyn TimeService>,
  pub(crate) encryption_key: Vec<u8>,
}

impl SqliteDbService {
  async fn seed_toolset_configs(&self) -> Result<(), DbError> {
    sqlx::query(
      "INSERT OR IGNORE INTO app_toolset_configs
       (toolset_type, enabled, updated_by, created_at, updated_at)
       VALUES (?, 0, 'system', strftime('%s', 'now'), strftime('%s', 'now'))",
    )
    .bind("builtin-exa-search")
    .execute(&self.pool)
    .await?;

    Ok(())
  }
}

#[async_trait::async_trait]
impl DbCore for SqliteDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(&self.pool).await?;
    self.seed_toolset_configs().await?;
    Ok(())
  }

  fn now(&self) -> DateTime<Utc> {
    self.time_service.utc_now()
  }

  fn encryption_key(&self) -> &[u8] {
    &self.encryption_key
  }

  async fn reset_all_tables(&self) -> Result<(), DbError> {
    // Delete in order to respect any future FK constraints
    sqlx::query(
      "DELETE FROM app_access_requests;
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
       DELETE FROM access_requests;
       DELETE FROM download_requests;",
    )
    .execute(&self.pool)
    .await?;

    // Re-seed default config
    self.seed_toolset_configs().await?;
    Ok(())
  }
}
