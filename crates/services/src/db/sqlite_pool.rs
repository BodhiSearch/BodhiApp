use crate::db::DbError;
use sqlx::SqlitePool;
use std::result::Result;

pub struct DbPool {}

impl DbPool {
  pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    let pool = SqlitePool::connect(url).await?;
    Ok(pool)
  }
}

#[cfg(test)]
mod test {
  use crate::db::{DbError, DbPool, SqlxError};

  #[tokio::test]
  async fn test_db_pool_raises_error() -> anyhow::Result<()> {
    let pool = DbPool::connect("sqlite:non-existing-db.sqlite").await;
    assert!(pool.is_err());
    match pool.unwrap_err() {
      DbError::SqlxError(SqlxError { source }) => {
        assert_eq!(
          source.to_string(),
          "error returned from database: (code: 14) unable to open database file"
        );
      }
      err => {
        panic!("expected DbError::SqlxError, found {:?}", err);
      }
    }
    Ok(())
  }
}
