use crate::db::DbError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::result::Result;
use std::str::FromStr;

pub struct DbPool {}

impl DbPool {
  pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
    Ok(SqlitePool::connect_with(options).await?)
  }
}

#[cfg(test)]
mod tests {
  use crate::db::DbPool;
  use tempfile::tempdir;

  #[tokio::test]
  async fn test_db_pool_creates_file_if_missing() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("new_database.sqlite");
    assert!(!db_path.exists());
    let url = format!("sqlite:{}", db_path.display());
    let pool = DbPool::connect(&url).await;
    assert!(
      pool.is_ok(),
      "Expected pool creation to succeed: {:?}",
      pool.err()
    );
    assert!(db_path.exists(), "Expected SQLite file to be created");
    Ok(())
  }
}
