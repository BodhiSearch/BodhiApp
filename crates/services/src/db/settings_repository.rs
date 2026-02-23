use super::DbError;

#[derive(Debug, Clone)]
pub struct DbSetting {
  pub key: String,
  pub value: String,
  pub value_type: String,
  pub created_at: i64,
  pub updated_at: i64,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SettingsRepository: Send + Sync {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError>;
  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError>;
  async fn delete_setting(&self, key: &str) -> Result<(), DbError>;
  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError>;
}
