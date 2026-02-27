use crate::db::{
  sea_migrations::Migrator, AccessRepository, AccessRequestRepository, ApiKeyUpdate, ApiToken,
  AppAccessRequestRow, AppInstanceRepository, AppInstanceRow, AppToolsetConfigRow, DbCore, DbError,
  DbSetting, DefaultDbService, DownloadRequest, McpAuthHeaderRow, McpOAuthConfigRow,
  McpOAuthTokenRow, McpRepository, McpRow, McpServerRow, McpWithServerRow, ModelMetadataRow,
  ModelRepository, SettingsRepository, TimeService, TokenRepository, ToolsetRepository, ToolsetRow,
  UserAccessRequest, UserAccessRequestStatus, UserAliasRepository,
};
use chrono::{DateTime, Utc};
use objs::test_utils::temp_dir;
use objs::ApiAlias;
use objs::UserAlias;
use rstest::fixture;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use std::{path::Path, sync::Arc};
use tap::Tap;
use tempfile::TempDir;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[fixture]
#[awt]
pub async fn test_db_service(temp_dir: TempDir) -> TestDbService {
  test_db_service_with_temp_dir(Arc::new(temp_dir)).await
}

pub async fn test_db_service_with_temp_dir(shared_temp_dir: Arc<TempDir>) -> TestDbService {
  let db = Database::connect("sqlite::memory:").await.unwrap();
  Migrator::fresh(&db).await.unwrap();
  let time_service = FrozenTimeService::default();
  let now = time_service.utc_now();
  let encryption_key = b"01234567890123456789012345678901".to_vec();
  let db_service = DefaultDbService::new(db, Arc::new(time_service), encryption_key.clone());
  TestDbService::new(shared_temp_dir, db_service, now, encryption_key)
}

#[derive(Debug)]
pub struct FrozenTimeService(DateTime<Utc>);

impl Default for FrozenTimeService {
  fn default() -> Self {
    FrozenTimeService(
      chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2025, 1, 1, 0, 0, 0).unwrap(),
    )
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
  inner: DefaultDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
  pub encryption_key: Vec<u8>,
}

impl TestDbService {
  pub fn new(
    _temp_dir: Arc<TempDir>,
    inner: DefaultDbService,
    now: DateTime<Utc>,
    encryption_key: Vec<u8>,
  ) -> Self {
    let (event_sender, _) = channel(100);
    TestDbService {
      _temp_dir,
      inner,
      event_sender,
      now,
      encryption_key,
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
impl DbCore for TestDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    self.inner.migrate().await.tap(|_| self.notify("migrate"))
  }

  fn now(&self) -> DateTime<Utc> {
    self.now
  }

  fn encryption_key(&self) -> &[u8] {
    &self.encryption_key
  }

  async fn reset_all_tables(&self) -> Result<(), DbError> {
    self
      .inner
      .reset_all_tables()
      .await
      .tap(|_| self.notify("reset_all_tables"))
  }
}

#[async_trait::async_trait]
impl ModelRepository for TestDbService {
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

  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
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
    api_key_update: ApiKeyUpdate,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_api_model_alias(alias, model, api_key_update)
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

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_api_model_cache(id, models, fetched_at)
      .await
      .tap(|_| self.notify("update_api_model_cache"))
  }

  async fn get_api_key_for_alias(&self, alias: &str) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_api_key_for_alias(alias)
      .await
      .tap(|_| self.notify("get_api_key_for_alias"))
  }

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    self
      .inner
      .check_prefix_exists(prefix, exclude_id)
      .await
      .tap(|_| self.notify("check_prefix_exists"))
  }

  async fn upsert_model_metadata(&self, metadata: &ModelMetadataRow) -> Result<(), DbError> {
    self
      .inner
      .upsert_model_metadata(metadata)
      .await
      .tap(|_| self.notify("upsert_model_metadata"))
  }

  async fn get_model_metadata_by_file(
    &self,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<ModelMetadataRow>, DbError> {
    self
      .inner
      .get_model_metadata_by_file(repo, filename, snapshot)
      .await
      .tap(|_| self.notify("get_model_metadata_by_file"))
  }

  async fn batch_get_metadata_by_files(
    &self,
    files: &[(String, String, String)],
  ) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataRow>, DbError> {
    self
      .inner
      .batch_get_metadata_by_files(files)
      .await
      .tap(|_| self.notify("batch_get_metadata_by_files"))
  }

  async fn list_model_metadata(&self) -> Result<Vec<ModelMetadataRow>, DbError> {
    self
      .inner
      .list_model_metadata()
      .await
      .tap(|_| self.notify("list_model_metadata"))
  }
}

#[async_trait::async_trait]
impl AccessRepository for TestDbService {
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
    id: &str,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_request_status(id, status, reviewer)
      .await
      .tap(|_| self.notify("update_request_status"))
  }

  async fn get_request_by_id(&self, id: &str) -> Result<Option<UserAccessRequest>, DbError> {
    self
      .inner
      .get_request_by_id(id)
      .await
      .tap(|_| self.notify("get_request_by_id"))
  }
}

#[async_trait::async_trait]
impl TokenRepository for TestDbService {
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
}

#[async_trait::async_trait]
impl ToolsetRepository for TestDbService {
  async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError> {
    self
      .inner
      .get_toolset(id)
      .await
      .tap(|_| self.notify("get_toolset"))
  }

  async fn get_toolset_by_slug(
    &self,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<ToolsetRow>, DbError> {
    self
      .inner
      .get_toolset_by_slug(user_id, slug)
      .await
      .tap(|_| self.notify("get_toolset_by_slug"))
  }

  async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError> {
    self
      .inner
      .create_toolset(row)
      .await
      .tap(|_| self.notify("create_toolset"))
  }

  async fn update_toolset(
    &self,
    row: &ToolsetRow,
    api_key_update: ApiKeyUpdate,
  ) -> Result<ToolsetRow, DbError> {
    self
      .inner
      .update_toolset(row, api_key_update)
      .await
      .tap(|_| self.notify("update_toolset"))
  }

  async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError> {
    self
      .inner
      .list_toolsets(user_id)
      .await
      .tap(|_| self.notify("list_toolsets"))
  }

  async fn list_toolsets_by_toolset_type(
    &self,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetRow>, DbError> {
    self
      .inner
      .list_toolsets_by_toolset_type(user_id, toolset_type)
      .await
      .tap(|_| self.notify("list_toolsets_by_toolset_type"))
  }

  async fn delete_toolset(&self, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_toolset(id)
      .await
      .tap(|_| self.notify("delete_toolset"))
  }

  async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_toolset_api_key(id)
      .await
      .tap(|_| self.notify("get_toolset_api_key"))
  }

  async fn set_app_toolset_enabled(
    &self,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfigRow, DbError> {
    self
      .inner
      .set_app_toolset_enabled(toolset_type, enabled, updated_by)
      .await
      .tap(|_| self.notify("set_app_toolset_enabled"))
  }

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError> {
    self
      .inner
      .list_app_toolset_configs()
      .await
      .tap(|_| self.notify("list_app_toolset_configs"))
  }

  async fn get_app_toolset_config(
    &self,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError> {
    self
      .inner
      .get_app_toolset_config(toolset_type)
      .await
      .tap(|_| self.notify("get_app_toolset_config"))
  }
}

#[async_trait::async_trait]
impl McpRepository for TestDbService {
  async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    self
      .inner
      .create_mcp_server(row)
      .await
      .tap(|_| self.notify("create_mcp_server"))
  }

  async fn update_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    self
      .inner
      .update_mcp_server(row)
      .await
      .tap(|_| self.notify("update_mcp_server"))
  }

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError> {
    self
      .inner
      .get_mcp_server(id)
      .await
      .tap(|_| self.notify("get_mcp_server"))
  }

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError> {
    self
      .inner
      .get_mcp_server_by_url(url)
      .await
      .tap(|_| self.notify("get_mcp_server_by_url"))
  }

  async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError> {
    self
      .inner
      .list_mcp_servers(enabled)
      .await
      .tap(|_| self.notify("list_mcp_servers"))
  }

  async fn count_mcps_by_server_id(&self, server_id: &str) -> Result<(i64, i64), DbError> {
    self
      .inner
      .count_mcps_by_server_id(server_id)
      .await
      .tap(|_| self.notify("count_mcps_by_server_id"))
  }

  async fn clear_mcp_tools_by_server_id(&self, server_id: &str) -> Result<u64, DbError> {
    self
      .inner
      .clear_mcp_tools_by_server_id(server_id)
      .await
      .tap(|_| self.notify("clear_mcp_tools_by_server_id"))
  }

  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    self
      .inner
      .create_mcp(row)
      .await
      .tap(|_| self.notify("create_mcp"))
  }

  async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError> {
    self
      .inner
      .get_mcp(user_id, id)
      .await
      .tap(|_| self.notify("get_mcp"))
  }

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError> {
    self
      .inner
      .get_mcp_by_slug(user_id, slug)
      .await
      .tap(|_| self.notify("get_mcp_by_slug"))
  }

  async fn list_mcps_with_server(&self, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError> {
    self
      .inner
      .list_mcps_with_server(user_id)
      .await
      .tap(|_| self.notify("list_mcps_with_server"))
  }

  async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    self
      .inner
      .update_mcp(row)
      .await
      .tap(|_| self.notify("update_mcp"))
  }

  async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp(user_id, id)
      .await
      .tap(|_| self.notify("delete_mcp"))
  }

  async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<objs::McpAuthHeader>, DbError> {
    self
      .inner
      .get_mcp_auth_header(id)
      .await
      .tap(|_| self.notify("get_mcp_auth_header"))
  }

  async fn create_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    self
      .inner
      .create_mcp_auth_header(row)
      .await
      .tap(|_| self.notify("create_mcp_auth_header"))
  }

  async fn update_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    self
      .inner
      .update_mcp_auth_header(row)
      .await
      .tap(|_| self.notify("update_mcp_auth_header"))
  }

  async fn delete_mcp_auth_header(&self, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp_auth_header(id)
      .await
      .tap(|_| self.notify("delete_mcp_auth_header"))
  }

  async fn list_mcp_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<objs::McpAuthHeader>, DbError> {
    self
      .inner
      .list_mcp_auth_headers_by_server(mcp_server_id)
      .await
      .tap(|_| self.notify("list_mcp_auth_headers_by_server"))
  }

  async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError> {
    self
      .inner
      .get_decrypted_auth_header(id)
      .await
      .tap(|_| self.notify("get_decrypted_auth_header"))
  }

  async fn create_mcp_oauth_config(
    &self,
    row: &McpOAuthConfigRow,
  ) -> Result<McpOAuthConfigRow, DbError> {
    self
      .inner
      .create_mcp_oauth_config(row)
      .await
      .tap(|_| self.notify("create_mcp_oauth_config"))
  }

  async fn get_mcp_oauth_config(&self, id: &str) -> Result<Option<objs::McpOAuthConfig>, DbError> {
    self
      .inner
      .get_mcp_oauth_config(id)
      .await
      .tap(|_| self.notify("get_mcp_oauth_config"))
  }

  async fn list_mcp_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<objs::McpOAuthConfig>, DbError> {
    self
      .inner
      .list_mcp_oauth_configs_by_server(mcp_server_id)
      .await
      .tap(|_| self.notify("list_mcp_oauth_configs_by_server"))
  }

  async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp_oauth_config(id)
      .await
      .tap(|_| self.notify("delete_mcp_oauth_config"))
  }

  async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_oauth_config_cascade(config_id)
      .await
      .tap(|_| self.notify("delete_oauth_config_cascade"))
  }

  async fn get_decrypted_client_secret(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    self
      .inner
      .get_decrypted_client_secret(id)
      .await
      .tap(|_| self.notify("get_decrypted_client_secret"))
  }

  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError> {
    self
      .inner
      .create_mcp_oauth_token(row)
      .await
      .tap(|_| self.notify("create_mcp_oauth_token"))
  }

  async fn get_mcp_oauth_token(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<objs::McpOAuthToken>, DbError> {
    self
      .inner
      .get_mcp_oauth_token(user_id, id)
      .await
      .tap(|_| self.notify("get_mcp_oauth_token"))
  }

  async fn get_latest_oauth_token_by_config(
    &self,
    config_id: &str,
  ) -> Result<Option<objs::McpOAuthToken>, DbError> {
    self
      .inner
      .get_latest_oauth_token_by_config(config_id)
      .await
      .tap(|_| self.notify("get_latest_oauth_token_by_config"))
  }

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError> {
    self
      .inner
      .update_mcp_oauth_token(row)
      .await
      .tap(|_| self.notify("update_mcp_oauth_token"))
  }

  async fn delete_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp_oauth_token(user_id, id)
      .await
      .tap(|_| self.notify("delete_mcp_oauth_token"))
  }

  async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_oauth_tokens_by_config(config_id)
      .await
      .tap(|_| self.notify("delete_oauth_tokens_by_config"))
  }

  async fn delete_oauth_tokens_by_config_and_user(
    &self,
    config_id: &str,
    user_id: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_oauth_tokens_by_config_and_user(config_id, user_id)
      .await
      .tap(|_| self.notify("delete_oauth_tokens_by_config_and_user"))
  }

  async fn get_decrypted_oauth_bearer(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    self
      .inner
      .get_decrypted_oauth_bearer(id)
      .await
      .tap(|_| self.notify("get_decrypted_oauth_bearer"))
  }

  async fn get_decrypted_refresh_token(&self, token_id: &str) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_decrypted_refresh_token(token_id)
      .await
      .tap(|_| self.notify("get_decrypted_refresh_token"))
  }
}

#[async_trait::async_trait]
impl UserAliasRepository for TestDbService {
  async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError> {
    self
      .inner
      .create_user_alias(alias)
      .await
      .tap(|_| self.notify("create_user_alias"))
  }

  async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError> {
    self
      .inner
      .get_user_alias_by_id(id)
      .await
      .tap(|_| self.notify("get_user_alias_by_id"))
  }

  async fn get_user_alias_by_name(&self, alias: &str) -> Result<Option<UserAlias>, DbError> {
    self
      .inner
      .get_user_alias_by_name(alias)
      .await
      .tap(|_| self.notify("get_user_alias_by_name"))
  }

  async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError> {
    self
      .inner
      .update_user_alias(id, alias)
      .await
      .tap(|_| self.notify("update_user_alias"))
  }

  async fn delete_user_alias(&self, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_user_alias(id)
      .await
      .tap(|_| self.notify("delete_user_alias"))
  }

  async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError> {
    self
      .inner
      .list_user_aliases()
      .await
      .tap(|_| self.notify("list_user_aliases"))
  }
}

#[async_trait::async_trait]
impl AppInstanceRepository for TestDbService {
  async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError> {
    self
      .inner
      .get_app_instance()
      .await
      .tap(|_| self.notify("get_app_instance"))
  }

  async fn upsert_app_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &objs::AppStatus,
  ) -> Result<(), DbError> {
    self
      .inner
      .upsert_app_instance(client_id, client_secret, status)
      .await
      .tap(|_| self.notify("upsert_app_instance"))
  }

  async fn update_app_instance_status(
    &self,
    client_id: &str,
    status: &objs::AppStatus,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_app_instance_status(client_id, status)
      .await
      .tap(|_| self.notify("update_app_instance_status"))
  }

  async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_app_instance(client_id)
      .await
      .tap(|_| self.notify("delete_app_instance"))
  }
}

#[async_trait::async_trait]
impl SettingsRepository for TestDbService {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError> {
    self
      .inner
      .get_setting(key)
      .await
      .tap(|_| self.notify("get_setting"))
  }

  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError> {
    self
      .inner
      .upsert_setting(setting)
      .await
      .tap(|_| self.notify("upsert_setting"))
  }

  async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_setting(key)
      .await
      .tap(|_| self.notify("delete_setting"))
  }

  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError> {
    self
      .inner
      .list_settings()
      .await
      .tap(|_| self.notify("list_settings"))
  }
}

#[async_trait::async_trait]
impl AccessRequestRepository for TestDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    self
      .inner
      .create(row)
      .await
      .tap(|_| self.notify("access_request_create"))
  }

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError> {
    self
      .inner
      .get(id)
      .await
      .tap(|_| self.notify("access_request_get"))
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    self
      .inner
      .update_approval(id, user_id, approved, approved_role, access_request_scope)
      .await
      .tap(|_| self.notify("access_request_update_approval"))
  }

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow, DbError> {
    self
      .inner
      .update_denial(id, user_id)
      .await
      .tap(|_| self.notify("access_request_update_denial"))
  }

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    self
      .inner
      .update_failure(id, error_message)
      .await
      .tap(|_| self.notify("access_request_update_failure"))
  }

  async fn get_by_access_request_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppAccessRequestRow>, DbError> {
    self
      .inner
      .get_by_access_request_scope(scope)
      .await
      .tap(|_| self.notify("access_request_get_by_scope"))
  }
}

// Composite mock using mockall::mock! that preserves MockDbService name
mockall::mock! {
  pub DbService {}

  impl std::fmt::Debug for DbService {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
  }

  #[async_trait::async_trait]
  impl DbCore for DbService {
    async fn migrate(&self) -> Result<(), DbError>;
    fn now(&self) -> DateTime<Utc>;
    fn encryption_key(&self) -> &[u8];
    async fn reset_all_tables(&self) -> Result<(), DbError>;
  }

  #[async_trait::async_trait]
  impl ModelRepository for DbService {
    async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;
    async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError>;
    async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;
    async fn list_download_requests(&self, page: usize, page_size: usize) -> Result<(Vec<DownloadRequest>, usize), DbError>;
    async fn find_download_request_by_repo_filename(&self, repo: &str, filename: &str) -> Result<Vec<DownloadRequest>, DbError>;
    async fn create_api_model_alias(&self, alias: &ApiAlias, api_key: Option<String>) -> Result<(), DbError>;
    async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError>;
    async fn update_api_model_alias(&self, id: &str, model: &ApiAlias, api_key: ApiKeyUpdate) -> Result<(), DbError>;
    async fn update_api_model_cache(&self, id: &str, models: Vec<String>, fetched_at: DateTime<Utc>) -> Result<(), DbError>;
    async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError>;
    async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError>;
    async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError>;
    async fn check_prefix_exists(&self, prefix: &str, exclude_id: Option<String>) -> Result<bool, DbError>;
    async fn upsert_model_metadata(&self, metadata: &ModelMetadataRow) -> Result<(), DbError>;
    async fn get_model_metadata_by_file(&self, repo: &str, filename: &str, snapshot: &str) -> Result<Option<ModelMetadataRow>, DbError>;
    async fn batch_get_metadata_by_files(&self, files: &[(String, String, String)]) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataRow>, DbError>;
    async fn list_model_metadata(&self) -> Result<Vec<ModelMetadataRow>, DbError>;
  }

  #[async_trait::async_trait]
  impl AccessRepository for DbService {
    async fn insert_pending_request(&self, username: String, user_id: String) -> Result<UserAccessRequest, DbError>;
    async fn get_pending_request(&self, user_id: String) -> Result<Option<UserAccessRequest>, DbError>;
    async fn list_pending_requests(&self, page: u32, per_page: u32) -> Result<(Vec<UserAccessRequest>, usize), DbError>;
    async fn list_all_requests(&self, page: u32, per_page: u32) -> Result<(Vec<UserAccessRequest>, usize), DbError>;
    async fn update_request_status(&self, id: &str, status: UserAccessRequestStatus, reviewer: String) -> Result<(), DbError>;
    async fn get_request_by_id(&self, id: &str) -> Result<Option<UserAccessRequest>, DbError>;
  }

  #[async_trait::async_trait]
  impl AppInstanceRepository for DbService {
    async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError>;
    async fn upsert_app_instance(
      &self,
      client_id: &str,
      client_secret: &str,
      status: &objs::AppStatus,
    ) -> Result<(), DbError>;
    async fn update_app_instance_status(
      &self,
      client_id: &str,
      status: &objs::AppStatus,
    ) -> Result<(), DbError>;
    async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError>;
  }

  #[async_trait::async_trait]
  impl TokenRepository for DbService {
    async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError>;
    async fn list_api_tokens(&self, user_id: &str, page: usize, per_page: usize) -> Result<(Vec<ApiToken>, usize), DbError>;
    async fn get_api_token_by_id(&self, user_id: &str, id: &str) -> Result<Option<ApiToken>, DbError>;
    async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError>;
    async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError>;
  }

  #[async_trait::async_trait]
  impl UserAliasRepository for DbService {
    async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError>;
    async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError>;
    async fn get_user_alias_by_name(&self, alias: &str) -> Result<Option<UserAlias>, DbError>;
    async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError>;
    async fn delete_user_alias(&self, id: &str) -> Result<(), DbError>;
    async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError>;
  }

  #[async_trait::async_trait]
  impl ToolsetRepository for DbService {
    async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError>;
    async fn get_toolset_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<ToolsetRow>, DbError>;
    async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError>;
    async fn update_toolset(&self, row: &ToolsetRow, api_key_update: ApiKeyUpdate) -> Result<ToolsetRow, DbError>;
    async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError>;
    async fn list_toolsets_by_toolset_type(&self, user_id: &str, toolset_type: &str) -> Result<Vec<ToolsetRow>, DbError>;
    async fn delete_toolset(&self, id: &str) -> Result<(), DbError>;
    async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError>;
    async fn set_app_toolset_enabled(&self, toolset_type: &str, enabled: bool, updated_by: &str) -> Result<AppToolsetConfigRow, DbError>;
    async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError>;
    async fn get_app_toolset_config(&self, toolset_type: &str) -> Result<Option<AppToolsetConfigRow>, DbError>;
  }

  #[async_trait::async_trait]
  impl McpRepository for DbService {
    async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError>;
    async fn update_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError>;
    async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError>;
    async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError>;
    async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError>;
    async fn count_mcps_by_server_id(&self, server_id: &str) -> Result<(i64, i64), DbError>;
    async fn clear_mcp_tools_by_server_id(&self, server_id: &str) -> Result<u64, DbError>;
    async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;
    async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError>;
    async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError>;
    async fn list_mcps_with_server(&self, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError>;
    async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;
    async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError>;
    async fn create_mcp_auth_header(&self, row: &McpAuthHeaderRow) -> Result<McpAuthHeaderRow, DbError>;
    async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<objs::McpAuthHeader>, DbError>;
    async fn update_mcp_auth_header(&self, row: &McpAuthHeaderRow) -> Result<McpAuthHeaderRow, DbError>;
    async fn delete_mcp_auth_header(&self, id: &str) -> Result<(), DbError>;
    async fn list_mcp_auth_headers_by_server(&self, mcp_server_id: &str) -> Result<Vec<objs::McpAuthHeader>, DbError>;
    async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError>;
    async fn create_mcp_oauth_config(&self, row: &McpOAuthConfigRow) -> Result<McpOAuthConfigRow, DbError>;
    async fn get_mcp_oauth_config(&self, id: &str) -> Result<Option<objs::McpOAuthConfig>, DbError>;
    async fn list_mcp_oauth_configs_by_server(&self, mcp_server_id: &str) -> Result<Vec<objs::McpOAuthConfig>, DbError>;
    async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError>;
    async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError>;
    async fn get_decrypted_client_secret(&self, id: &str) -> Result<Option<(String, String)>, DbError>;
    async fn create_mcp_oauth_token(&self, row: &McpOAuthTokenRow) -> Result<McpOAuthTokenRow, DbError>;
    async fn get_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<Option<objs::McpOAuthToken>, DbError>;
    async fn get_latest_oauth_token_by_config(&self, config_id: &str) -> Result<Option<objs::McpOAuthToken>, DbError>;
    async fn update_mcp_oauth_token(&self, row: &McpOAuthTokenRow) -> Result<McpOAuthTokenRow, DbError>;
    async fn delete_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<(), DbError>;
    async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError>;
    async fn delete_oauth_tokens_by_config_and_user(&self, config_id: &str, user_id: &str) -> Result<(), DbError>;
    async fn get_decrypted_oauth_bearer(&self, id: &str) -> Result<Option<(String, String)>, DbError>;
    async fn get_decrypted_refresh_token(&self, token_id: &str) -> Result<Option<String>, DbError>;
  }

  #[async_trait::async_trait]
  impl SettingsRepository for DbService {
    async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError>;
    async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError>;
    async fn delete_setting(&self, key: &str) -> Result<(), DbError>;
    async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError>;
  }

  #[async_trait::async_trait]
  impl AccessRequestRepository for DbService {
    async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError>;
    async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError>;
    async fn update_approval(
      &self,
      id: &str,
      user_id: &str,
      approved: &str,
      approved_role: &str,
      access_request_scope: &str,
    ) -> Result<AppAccessRequestRow, DbError>;
    async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow, DbError>;
    async fn update_failure(&self, id: &str, error_message: &str) -> Result<AppAccessRequestRow, DbError>;
    async fn get_by_access_request_scope(
      &self,
      scope: &str,
    ) -> Result<Option<AppAccessRequestRow>, DbError>;
  }
}

#[derive(Debug)]
pub struct InMemorySettingsRepository {
  store: std::sync::RwLock<std::collections::HashMap<String, DbSetting>>,
}

impl InMemorySettingsRepository {
  pub fn new() -> Self {
    Self {
      store: std::sync::RwLock::new(std::collections::HashMap::new()),
    }
  }
}

#[async_trait::async_trait]
impl SettingsRepository for InMemorySettingsRepository {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError> {
    Ok(self.store.read().unwrap().get(key).cloned())
  }

  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError> {
    let mut store = self.store.write().unwrap();
    store.insert(setting.key.clone(), setting.clone());
    Ok(setting.clone())
  }

  async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
    self.store.write().unwrap().remove(key);
    Ok(())
  }

  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError> {
    Ok(self.store.read().unwrap().values().cloned().collect())
  }
}
