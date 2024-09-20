use super::DbError;
use sqlx::SqlitePool;
use std::result::Result;

pub struct DbPool {}

impl DbPool {
  pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    let pool = SqlitePool::connect(url)
      .await
      .map_err(|source| DbError::SqlxConnect {
        source,
        url: url.to_string(),
      })?;
    Ok(pool)
  }
}

#[cfg(test)]
mod test {
  use super::DbPool;

  #[tokio::test]
  async fn test_db_pool_raises_error() -> anyhow::Result<()> {
    let pool = DbPool::connect("sqlite:non-existing-db.sqlite").await;
    assert!(pool.is_err());
    assert_eq!("sqlx_connect: error returned from database: (code: 14) unable to open database file\nurl: sqlite:non-existing-db.sqlite", pool.unwrap_err().to_string());
    Ok(())
  }
}
