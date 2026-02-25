use super::DbError;

#[derive(Debug, Clone)]
pub struct AppInstanceRow {
  pub client_id: String,
  pub client_secret: String,
  pub salt_client_secret: String,
  pub nonce_client_secret: String,
  pub scope: String,
  pub app_status: String,
  pub created_at: i64,
  pub updated_at: i64,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait AppInstanceRepository: Send + Sync {
  async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError>;
  async fn upsert_app_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    scope: &str,
    status: &str,
  ) -> Result<(), DbError>;
  async fn update_app_instance_status(&self, client_id: &str, status: &str) -> Result<(), DbError>;
  async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError>;
}
