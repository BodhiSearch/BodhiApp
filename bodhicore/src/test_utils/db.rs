use crate::db::{
  objs::{Conversation, Message},
  DbError, DbService, DbServiceFn, TimeServiceFn,
};
use chrono::{DateTime, Timelike, Utc};
use rstest::fixture;
use sqlx::SqlitePool;
use std::{
  fmt::{self, Formatter},
  fs::File,
  sync::Arc,
};
use tempfile::TempDir;

mockall::mock! {
  pub TimeService {}

  impl TimeServiceFn for TimeService {
    fn utc_now(&self) -> DateTime<Utc>;
  }

  impl std::fmt::Debug for TimeService {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> fmt::Result;
  }

  unsafe impl Send for TimeService {}

  unsafe impl Sync for TimeService {}
}

mockall::mock! {
  pub DbService {}

  #[async_trait::async_trait]
  impl DbServiceFn for DbService {
    async fn save_conversation(&self, conversation: &mut Conversation) -> Result<(), DbError>;

    async fn save_message(&self, message: &mut Message) -> Result<(), DbError>;

    async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError>;

    async fn delete_conversations(&self, id: &str) -> Result<(), DbError>;

    async fn delete_all_conversations(&self) -> Result<(), DbError>;

    async fn get_conversation_with_messages(&self, id: &str) -> Result<Conversation, DbError>;
  }

  impl std::fmt::Debug for DbService {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
  }

  unsafe impl Send for DbService {}
  unsafe impl Sync for DbService {}
}

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
) -> (TempDir, DateTime<Utc>, DbService) {
  let (_tempdir, pool) = testdb;
  let now = chrono::Utc::now().with_nanosecond(0).unwrap();
  let mut mock_time_service = MockTimeService::new();
  mock_time_service.expect_utc_now().returning(move || now);
  let service = DbService::new(pool, Arc::new(mock_time_service));
  (_tempdir, now, service)
}
