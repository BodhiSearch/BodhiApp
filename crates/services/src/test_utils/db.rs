use crate::db::{
  ApiToken, DbError, DbService, DownloadRequest, SqliteDbService, TimeService, UserAccessRequest,
  UserAccessRequestStatus,
};
use chrono::{DateTime, Timelike, Utc};
use objs::test_utils::temp_dir;
use objs::ApiAlias;
use rstest::fixture;
use sqlx::SqlitePool;
use std::{fs::File, path::Path, sync::Arc};
use tap::Tap;
use tempfile::TempDir;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[fixture]
#[awt]
pub async fn test_db_service(temp_dir: TempDir) -> TestDbService {
  test_db_service_with_temp_dir(Arc::new(temp_dir)).await
}

pub async fn test_db_service_with_temp_dir(shared_temp_dir: Arc<TempDir>) -> TestDbService {
  let dbfile = shared_temp_dir.path().join("testdb.sqlite");
  File::create(&dbfile).unwrap();
  let dbpath = dbfile.to_str().unwrap();
  let pool = SqlitePool::connect(&format!("sqlite:{dbpath}"))
    .await
    .unwrap();
  let time_service = FrozenTimeService::default();
  let now = time_service.utc_now();
  let encryption_key = b"test_encryption_key_1234567890123456".to_vec();
  let db_service = SqliteDbService::new(pool, Arc::new(time_service), encryption_key);
  db_service.migrate().await.unwrap();
  TestDbService::new(shared_temp_dir, db_service, now)
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
  _temp_dir: Arc<TempDir>,
  inner: SqliteDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
}

impl TestDbService {
  pub fn new(_temp_dir: Arc<TempDir>, inner: SqliteDbService, now: DateTime<Utc>) -> Self {
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

  async fn list_download_requests(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequest>, usize), DbError> {
    self
      .inner
      .list_download_requests(page, page_size)
      .await
      .tap(|_| self.notify("list_download_requests"))
  }

  async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequest, DbError> {
    self
      .inner
      .insert_pending_request(username, user_id)
      .await
      .tap(|_| self.notify("insert_pending_request"))
  }

  async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequest>, DbError> {
    self
      .inner
      .get_pending_request(user_id)
      .await
      .tap(|_| self.notify("get_pending_request"))
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    self
      .inner
      .list_pending_requests(page, per_page)
      .await
      .tap(|_| self.notify("list_pending_requests"))
  }

  async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    self
      .inner
      .list_all_requests(page, per_page)
      .await
      .tap(|_| self.notify("list_all_requests"))
  }

  async fn update_request_status(
    &self,
    id: i64,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_request_status(id, status, reviewer)
      .await
      .tap(|_| self.notify("update_request_status"))
  }

  async fn get_request_by_id(&self, id: i64) -> Result<Option<UserAccessRequest>, DbError> {
    self
      .inner
      .get_request_by_id(id)
      .await
      .tap(|_| self.notify("get_request_by_id"))
  }

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    self
      .inner
      .create_api_token(token)
      .await
      .tap(|_| self.notify("create_api_token"))
  }

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError> {
    self
      .inner
      .list_api_tokens(user_id, page, per_page)
      .await
      .tap(|_| self.notify("list_api_tokens"))
  }

  async fn get_api_token_by_id(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, DbError> {
    self
      .inner
      .get_api_token_by_id(user_id, id)
      .await
      .tap(|_| self.notify("get_api_token_by_id"))
  }

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError> {
    self
      .inner
      .get_api_token_by_prefix(prefix)
      .await
      .tap(|_| self.notify("get_api_token_by_prefix"))
  }

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError> {
    self
      .inner
      .update_api_token(user_id, token)
      .await
      .tap(|_| self.notify("update_api_token"))
  }

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError> {
    self
      .inner
      .find_download_request_by_repo_filename(repo, filename)
      .await
      .tap(|_| self.notify("find_download_request_by_repo_filename"))
  }

  async fn create_api_model_alias(&self, alias: &ApiAlias, api_key: &str) -> Result<(), DbError> {
    self
      .inner
      .create_api_model_alias(alias, api_key)
      .await
      .tap(|_| self.notify("create_api_model_alias"))
  }

  async fn get_api_model_alias(&self, alias: &str) -> Result<Option<ApiAlias>, DbError> {
    self
      .inner
      .get_api_model_alias(alias)
      .await
      .tap(|_| self.notify("get_api_model_alias"))
  }

  async fn update_api_model_alias(
    &self,
    alias: &str,
    model: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_api_model_alias(alias, model, api_key)
      .await
      .tap(|_| self.notify("update_api_model_alias"))
  }

  async fn delete_api_model_alias(&self, alias: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_api_model_alias(alias)
      .await
      .tap(|_| self.notify("delete_api_model_alias"))
  }

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError> {
    self
      .inner
      .list_api_model_aliases()
      .await
      .tap(|_| self.notify("list_api_model_aliases"))
  }

  async fn get_api_key_for_alias(&self, alias: &str) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_api_key_for_alias(alias)
      .await
      .tap(|_| self.notify("get_api_key_for_alias"))
  }

  fn now(&self) -> DateTime<Utc> {
    self.now
  }
}
