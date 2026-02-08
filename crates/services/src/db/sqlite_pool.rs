use crate::db::DbError;
use sqlx::SqlitePool;
use std::result::Result;

pub struct DbPool {}

impl DbPool {
  pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    Ok(SqlitePool::connect(url).await?)
  }
}

#[cfg(test)]
mod tests {
  use crate::db::{DbError, DbPool};
  use anyhow_trace::anyhow_trace;
  use pretty_assertions::assert_eq;
  use std::error::Error;

  #[tokio::test]
  async fn test_db_pool_raises_error() -> anyhow::Result<()> {
    let pool = DbPool::connect("sqlite:non-existing-db.sqlite").await;
    assert!(matches!(pool, Err(DbError::SqlxError(_))));
    assert_eq!(
      "error returned from database: (code: 14) unable to open database file",
      pool.unwrap_err().source().unwrap().to_string()
    );
    Ok(())
  }
}
