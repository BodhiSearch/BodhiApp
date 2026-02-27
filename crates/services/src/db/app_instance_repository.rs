use super::DbError;
use chrono::{DateTime, Utc};
use objs::AppStatus;

#[derive(Debug, Clone)]
pub struct AppInstanceRow {
  pub client_id: String,
  pub client_secret: String,
  pub app_status: AppStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait AppInstanceRepository: Send + Sync {
  async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError>;
  async fn upsert_app_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &AppStatus,
  ) -> Result<(), DbError>;
  async fn update_app_instance_status(
    &self,
    client_id: &str,
    status: &AppStatus,
  ) -> Result<(), DbError>;
  async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError>;
}
