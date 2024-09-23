use crate::db::{
  Conversation, DbError, DbService, DefaultTimeService, DownloadRequest, Message, MockTimeService,
  SqliteDbService,
};
use chrono::{DateTime, Timelike, Utc};
use rstest::fixture;
use sqlx::SqlitePool;
use std::{fs::File, sync::Arc};
use tempfile::TempDir;
use tokio::sync::broadcast::{channel, Receiver, Sender};

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
  service.migrate().await.unwrap();
  (_tempdir, now, service)
}

pub async fn db_service_with_events(temp_home: &TempDir) -> TestDbService {
  let db_service = db_service_in(temp_home).await;
  TestDbService::new(db_service)
}

pub async fn db_service_in(temp_home: &TempDir) -> SqliteDbService {
  let dbfile = temp_home.path().join("testdb.sqlite");
  File::create(&dbfile).unwrap();
  let dbpath = dbfile.to_str().unwrap();
  let pool = SqlitePool::connect(&format!("sqlite:{dbpath}"))
    .await
    .unwrap();
  let db_service = SqliteDbService::new(pool, Arc::new(DefaultTimeService::default()));
  db_service.migrate().await.unwrap();
  db_service
}

#[derive(Debug)]
pub struct TestDbService {
  inner: SqliteDbService,
  event_sender: Sender<String>,
}

impl TestDbService {
  pub fn new(inner: SqliteDbService) -> Self {
    let (event_sender, _) = channel(100);
    TestDbService {
      inner,
      event_sender,
    }
  }

  pub fn subscribe(&self) -> Receiver<String> {
    self.event_sender.subscribe()
  }

  async fn notify(&self, event: &str) {
    let _ = self.event_sender.send(event.to_string());
  }
}

#[async_trait::async_trait]
impl DbService for TestDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    self.inner.migrate().await
  }

  async fn save_conversation(&self, _conversation: &mut Conversation) -> Result<(), DbError> {
    todo!()
  }

  async fn save_message(&self, _message: &mut Message) -> Result<(), DbError> {
    todo!()
  }

  async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError> {
    todo!()
  }

  async fn delete_conversations(&self, _id: &str) -> Result<(), DbError> {
    todo!()
  }

  async fn delete_all_conversations(&self) -> Result<(), DbError> {
    todo!()
  }

  async fn get_conversation_with_messages(&self, _id: &str) -> Result<Conversation, DbError> {
    todo!()
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    let result = self.inner.get_download_request(id).await;
    self.notify("get_download_request").await;
    result
  }

  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let result = self.inner.create_download_request(request).await;
    self.notify("create_download_request").await;
    result
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let result = self.inner.update_download_request(request).await;
    self.notify("update_download_request").await;
    result
  }

  async fn list_pending_downloads(&self) -> Result<Vec<DownloadRequest>, DbError> {
    let result = self.inner.list_pending_downloads().await;
    self.notify("list_pending_downloads").await;
    result
  }
}
