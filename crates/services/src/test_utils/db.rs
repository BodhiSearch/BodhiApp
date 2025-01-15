use crate::db::{
  AccessRequest, ApiToken, Conversation, DbError, DbService, DownloadRequest, Message,
  RequestStatus, SqliteDbService, TimeService,
};
use chrono::{DateTime, Timelike, Utc};
use objs::test_utils::temp_dir;
use rstest::fixture;
use sqlx::SqlitePool;
use std::{fs::File, path::Path, sync::Arc};
use tap::Tap;
use tempfile::TempDir;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[fixture]
pub async fn test_db_service(temp_dir: TempDir) -> TestDbService {
  let dbfile = temp_dir.path().join("testdb.sqlite");
  File::create(&dbfile).unwrap();
  let dbpath = dbfile.to_str().unwrap();
  let pool = SqlitePool::connect(&format!("sqlite:{dbpath}"))
    .await
    .unwrap();
  let time_service = FrozenTimeService::default();
  let now = time_service.utc_now();
  let db_service = SqliteDbService::new(pool, Arc::new(time_service));
  db_service.migrate().await.unwrap();
  TestDbService::new(temp_dir, db_service, now)
}

#[derive(Debug)]
pub struct FrozenTimeService(DateTime<Utc>);

impl Default for FrozenTimeService {
  fn default() -> Self {
    FrozenTimeService(chrono::Utc::now().with_nanosecond(0).unwrap())
  }
}

impl TimeService for FrozenTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    self.0
  }

  fn created_at(&self, _path: &Path) -> u32 {
    0
  }
}

#[derive(Debug)]
pub struct TestDbService {
  _temp_dir: TempDir,
  inner: SqliteDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
}

impl TestDbService {
  pub fn new(_temp_dir: TempDir, inner: SqliteDbService, now: DateTime<Utc>) -> Self {
    let (event_sender, _) = channel(100);
    TestDbService {
      _temp_dir,
      inner,
      event_sender,
      now,
    }
  }

  pub fn subscribe(&self) -> Receiver<String> {
    self.event_sender.subscribe()
  }

  fn notify(&self, event: &str) {
    let _ = self.event_sender.send(event.to_string());
  }

  pub fn now(&self) -> DateTime<Utc> {
    self.now
  }
}

#[async_trait::async_trait]
impl DbService for TestDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    self.inner.migrate().await.tap(|_| self.notify("migrate"))
  }

  async fn save_conversation(&self, conversation: &mut Conversation) -> Result<(), DbError> {
    self.inner.save_conversation(conversation).await
    // .tap(|_| self.notify("save_conversation"))
  }

  async fn save_message(&self, message: &mut Message) -> Result<(), DbError> {
    self
      .inner
      .save_message(message)
      .await
      .tap(|_| self.notify("save_message"))
  }

  async fn list_conversations(&self) -> Result<Vec<Conversation>, DbError> {
    self
      .inner
      .list_conversations()
      .await
      .tap(|_| self.notify("list_conversations"))
  }

  async fn delete_conversations(&self, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_conversations(id)
      .await
      .tap(|_| self.notify("delete_conversations"))
  }

  async fn delete_all_conversations(&self) -> Result<(), DbError> {
    self
      .inner
      .delete_all_conversations()
      .await
      .tap(|_| self.notify("delete_all_conversations"))
  }

  async fn get_conversation_with_messages(&self, id: &str) -> Result<Conversation, DbError> {
    self
      .inner
      .get_conversation_with_messages(id)
      .await
      .tap(|_| self.notify("get_conversation_with_messages"))
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    self
      .inner
      .get_download_request(id)
      .await
      .tap(|_| self.notify("get_download_request"))
  }

  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    self
      .inner
      .create_download_request(request)
      .await
      .tap(|_| self.notify("create_download_request"))
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    self
      .inner
      .update_download_request(request)
      .await
      .tap(|_| self.notify("update_download_request"))
  }

  async fn list_all_downloads(&self) -> Result<Vec<DownloadRequest>, DbError> {
    self
      .inner
      .list_all_downloads()
      .await
      .tap(|_| self.notify("list_all_downloads"))
  }

  async fn insert_pending_request(&self, email: String) -> Result<AccessRequest, DbError> {
    self
      .inner
      .insert_pending_request(email)
      .await
      .tap(|_| self.notify("insert_pending_request"))
  }

  async fn get_pending_request(&self, email: String) -> Result<Option<AccessRequest>, DbError> {
    self
      .inner
      .get_pending_request(email)
      .await
      .tap(|_| self.notify("get_pending_request"))
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<Vec<AccessRequest>, DbError> {
    self
      .inner
      .list_pending_requests(page, per_page)
      .await
      .tap(|_| self.notify("list_pending_requests"))
  }

  async fn update_request_status(&self, id: i64, status: RequestStatus) -> Result<(), DbError> {
    self
      .inner
      .update_request_status(id, status)
      .await
      .tap(|_| self.notify("update_request_status"))
  }

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    self
      .inner
      .create_api_token(token)
      .await
      .tap(|_| self.notify("create_api_token"))
  }

  async fn list_api_tokens(&self, page: u32, per_page: u32) -> Result<Vec<ApiToken>, DbError> {
    self
      .inner
      .list_api_tokens(page, per_page)
      .await
      .tap(|_| self.notify("list_api_tokens"))
  }
}
