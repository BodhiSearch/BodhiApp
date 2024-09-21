use crate::db::{MockTimeService, SqliteDbService};
use chrono::{DateTime, Timelike, Utc};
use rstest::fixture;
use sqlx::SqlitePool;
use std::{fs::File, sync::Arc};
use tempfile::TempDir;

#[fixture]
pub async fn testdb() -> (TempDir, SqlitePool) {
  let tempdir = tempfile::tempdir().unwrap();
  let dbpath = tempdir
    .path()
    .to_path_buf()
    .join("testdb.sqlite")
    .display()
    .to_string();
  File::create(&dbpath).unwrap();
  let pool = SqlitePool::connect(&format!("sqlite:{dbpath}"))
    .await
    .unwrap();
  sqlx::migrate!("./migrations").run(&pool).await.unwrap();
  (tempdir, pool)
}

#[fixture]
#[awt]
pub async fn db_service(
  #[future] testdb: (TempDir, SqlitePool),
) -> (TempDir, DateTime<Utc>, SqliteDbService) {
  let (_tempdir, pool) = testdb;
  let now = chrono::Utc::now().with_nanosecond(0).unwrap();
  let mut mock_time_service = MockTimeService::new();
  mock_time_service.expect_utc_now().returning(move || now);
  let service = SqliteDbService::new(pool, Arc::new(mock_time_service));
  (_tempdir, now, service)
}
