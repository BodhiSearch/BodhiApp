use chrono::{DateTime, Utc};

use super::DbError;

#[derive(Debug, Clone, PartialEq)]
pub struct DbSetting {
  pub key: String,
  pub value: String,
  pub value_type: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SettingsRepository: Send + Sync {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError>;
  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError>;
  async fn delete_setting(&self, key: &str) -> Result<(), DbError>;
  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError>;
}
