use crate::db::{DbCore, DbError, TimeService};
use crate::toolsets::app_toolset_config_entity as app_toolset_config;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, Set};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DefaultDbService {
  pub(crate) db: DatabaseConnection,
  pub(crate) time_service: Arc<dyn TimeService>,
  pub(crate) encryption_key: Vec<u8>,
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
    }
  }

  pub fn db(&self) -> &DatabaseConnection {
    &self.db
  }

  async fn seed_toolset_configs(&self) -> Result<(), DbError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    let existing = app_toolset_config::Entity::find()
      .filter(app_toolset_config::Column::ToolsetType.eq("builtin-exa-search"))
      .one(&self.db)
      .await?;
    if existing.is_none() {
      let now = self.time_service.utc_now();
      let model = app_toolset_config::ActiveModel {
        id: Set(ulid::Ulid::new().to_string()),
        toolset_type: Set("builtin-exa-search".to_string()),
        enabled: Set(false),
        updated_by: Set("system".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      };
      model.insert(&self.db).await?;
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl DbCore for DefaultDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    crate::db::sea_migrations::Migrator::up(&self.db, None).await?;
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
    let backend = self.db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Postgres => {
        // Use TRUNCATE CASCADE for Postgres
        // Note: apps table is NOT reset - it holds app instance (client credentials, status)
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
             access_requests,
             download_requests
           CASCADE",
        )
        .await?;
      }
      _ => {
        // Use DELETE FROM for SQLite
        // Note: apps table is NOT reset - it holds app instance (client credentials, status)
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
           DELETE FROM access_requests;
           DELETE FROM download_requests;",
        )
        .await?;
      }
    }

    // Re-seed default config
    self.seed_toolset_configs().await?;
    Ok(())
  }
}
