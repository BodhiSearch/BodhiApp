use sqlx::PgPool;
use tower_sessions_sqlx_store::PostgresStore;

use super::{SessionResult, SessionServiceError};

pub(crate) fn create_postgres_store(pool: PgPool) -> SessionResult<PostgresStore> {
  Ok(
    PostgresStore::new(pool)
      .with_schema_name("public")
      .map_err(|e| SessionServiceError::DbSetup(format!("invalid schema name: {e}")))?
      .with_table_name("tower_sessions")
      .map_err(|e| SessionServiceError::DbSetup(format!("invalid table name: {e}")))?,
  )
}
