use sqlx::SqlitePool;
use tower_sessions_sqlx_store::SqliteStore;

pub(crate) fn create_sqlite_store(pool: SqlitePool) -> SqliteStore {
  SqliteStore::new(pool)
}
