use crate::db::DbError;
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait DbCore: Send + Sync {
  async fn migrate(&self) -> Result<(), DbError>;

  fn now(&self) -> DateTime<Utc>;

  fn encryption_key(&self) -> &[u8];

  async fn reset_all_tables(&self) -> Result<(), DbError>;

  /// Begin a transaction with the tenant context pre-configured.
  ///
  /// On PostgreSQL this runs `SET LOCAL app.current_tenant_id = '<tenant_id>'`
  /// so that RLS policies automatically filter rows to that tenant for the
  /// duration of the transaction.  The caller is responsible for calling
  /// `txn.commit()` (or letting it roll back on drop).
  ///
  /// On SQLite (dev / desktop) a plain transaction is returned with no
  /// additional configuration, because SQLite has no RLS support.
  async fn begin_tenant_txn(
    &self,
    tenant_id: &str,
  ) -> Result<sea_orm::DatabaseTransaction, DbError>;

  async fn reset_tenants(&self) -> Result<(), DbError>;
}
