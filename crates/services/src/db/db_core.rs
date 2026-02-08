use crate::db::DbError;
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait DbCore: Send + Sync {
  async fn migrate(&self) -> Result<(), DbError>;

  fn now(&self) -> DateTime<Utc>;

  fn encryption_key(&self) -> &[u8];
}
