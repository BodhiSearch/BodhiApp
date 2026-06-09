use crate::db::DbError;
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait DbCore: Send + Sync {
  async fn migrate(&self) -> Result<(), DbError>;

  fn now(&self) -> DateTime<Utc>;

  fn encryption_key(&self) -> &[u8];

  async fn reset_all_tables(&self) -> Result<(), DbError>;

  /// On PostgreSQL sets `app.current_tenant_id` so RLS policies filter to the tenant
  /// for the transaction; on SQLite (no RLS) returns a plain transaction. Caller commits.
  async fn begin_tenant_txn(
    &self,
    tenant_id: &str,
  ) -> Result<sea_orm::DatabaseTransaction, DbError>;

  async fn reset_tenants(&self) -> Result<(), DbError>;
}
