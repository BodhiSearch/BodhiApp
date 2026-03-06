use super::temp_dir;
use crate::app_access_requests::{AccessRequestRepository, AppAccessRequest};
use crate::db::{sea_migrations::Migrator, DbCore, DbError, DefaultDbService, TimeService};
use crate::tenants::{TenantRepository, TenantRow};

pub const TEST_TENANT_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
pub const TEST_TENANT_B_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBB";
pub const TEST_USER_ID: &str = "test-user";
pub const TEST_TENANT_A_USER_B_ID: &str = "test-tenant-a-user-b";

use crate::mcps::{
  McpAuthHeaderRow, McpAuthRepository, McpInstanceRepository, McpOAuthConfigRow, McpOAuthTokenRow,
  McpRow, McpServerRepository, McpServerRow, McpWithServerRow,
};
use crate::models::{
  ApiAlias, ApiAliasRepository, DownloadRepository, DownloadRequestEntity, ModelMetadataEntity,
  ModelMetadataRepository, UserAlias, UserAliasRepository,
};
use crate::settings::{DbSetting, SettingsRepository};
use crate::tokens::{TokenEntity, TokenRepository};
use crate::toolsets::{AppToolsetConfigRow, ToolsetEntity, ToolsetRepository};
use crate::users::{AccessRepository, UserAccessRequestEntity};
use crate::RawApiKeyUpdate;
use crate::UserAccessRequestStatus;
use chrono::{DateTime, Utc};
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

  async fn begin_tenant_txn(
    &self,
    tenant_id: &str,
  ) -> Result<sea_orm::DatabaseTransaction, DbError> {
    self.inner.begin_tenant_txn(tenant_id).await
  }
}

#[async_trait::async_trait]
impl DownloadRepository for TestDbService {
  async fn get_download_request(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<DownloadRequestEntity>, DbError> {
    self
      .inner
      .get_download_request(tenant_id, id)
      .await
      .tap(|_| self.notify("get_download_request"))
  }

  async fn create_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError> {
    self
      .inner
      .create_download_request(request)
      .await
      .tap(|_| self.notify("create_download_request"))
  }

  async fn update_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError> {
    self
      .inner
      .update_download_request(request)
      .await
      .tap(|_| self.notify("update_download_request"))
  }

  async fn list_download_requests(
    &self,
    tenant_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequestEntity>, usize), DbError> {
    self
      .inner
      .list_download_requests(tenant_id, page, page_size)
      .await
      .tap(|_| self.notify("list_download_requests"))
  }

  async fn find_download_request_by_repo_filename(
    &self,
    tenant_id: &str,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequestEntity>, DbError> {
    self
      .inner
      .find_download_request_by_repo_filename(tenant_id, repo, filename)
      .await
      .tap(|_| self.notify("find_download_request_by_repo_filename"))
  }
}

#[async_trait::async_trait]
impl ApiAliasRepository for TestDbService {
  async fn create_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    self
      .inner
      .create_api_model_alias(tenant_id, user_id, alias, api_key)
      .await
      .tap(|_| self.notify("create_api_model_alias"))
  }

  async fn get_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Result<Option<ApiAlias>, DbError> {
    self
      .inner
      .get_api_model_alias(tenant_id, user_id, alias)
      .await
      .tap(|_| self.notify("get_api_model_alias"))
  }

  async fn update_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
    model: &ApiAlias,
    api_key_update: RawApiKeyUpdate,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_api_model_alias(tenant_id, user_id, alias, model, api_key_update)
      .await
      .tap(|_| self.notify("update_api_model_alias"))
  }

  async fn delete_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_api_model_alias(tenant_id, user_id, alias)
      .await
      .tap(|_| self.notify("delete_api_model_alias"))
  }

  async fn list_api_model_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ApiAlias>, DbError> {
    self
      .inner
      .list_api_model_aliases(tenant_id, user_id)
      .await
      .tap(|_| self.notify("list_api_model_aliases"))
  }

  async fn update_api_model_cache(
    &self,
    tenant_id: &str,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_api_model_cache(tenant_id, id, models, fetched_at)
      .await
      .tap(|_| self.notify("update_api_model_cache"))
  }

  async fn get_api_key_for_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_api_key_for_alias(tenant_id, user_id, alias)
      .await
      .tap(|_| self.notify("get_api_key_for_alias"))
  }

  async fn check_prefix_exists(
    &self,
    tenant_id: &str,
    user_id: &str,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    self
      .inner
      .check_prefix_exists(tenant_id, user_id, prefix, exclude_id)
      .await
      .tap(|_| self.notify("check_prefix_exists"))
  }
}

#[async_trait::async_trait]
impl ModelMetadataRepository for TestDbService {
  async fn upsert_model_metadata(&self, metadata: &ModelMetadataEntity) -> Result<(), DbError> {
    self
      .inner
      .upsert_model_metadata(metadata)
      .await
      .tap(|_| self.notify("upsert_model_metadata"))
  }

  async fn get_model_metadata_by_file(
    &self,
    tenant_id: &str,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<ModelMetadataEntity>, DbError> {
    self
      .inner
      .get_model_metadata_by_file(tenant_id, repo, filename, snapshot)
      .await
      .tap(|_| self.notify("get_model_metadata_by_file"))
  }

  async fn batch_get_metadata_by_files(
    &self,
    tenant_id: &str,
    files: &[(String, String, String)],
  ) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataEntity>, DbError> {
    self
      .inner
      .batch_get_metadata_by_files(tenant_id, files)
      .await
      .tap(|_| self.notify("batch_get_metadata_by_files"))
  }

  async fn list_model_metadata(
    &self,
    tenant_id: &str,
  ) -> Result<Vec<ModelMetadataEntity>, DbError> {
    self
      .inner
      .list_model_metadata(tenant_id)
      .await
      .tap(|_| self.notify("list_model_metadata"))
  }
}

#[async_trait::async_trait]
impl AccessRepository for TestDbService {
  async fn insert_pending_request(
    &self,
    tenant_id: &str,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequestEntity, DbError> {
    self
      .inner
      .insert_pending_request(tenant_id, username, user_id)
      .await
      .tap(|_| self.notify("insert_pending_request"))
  }

  async fn get_pending_request(
    &self,
    tenant_id: &str,
    user_id: String,
  ) -> Result<Option<UserAccessRequestEntity>, DbError> {
    self
      .inner
      .get_pending_request(tenant_id, user_id)
      .await
      .tap(|_| self.notify("get_pending_request"))
  }

  async fn list_pending_requests(
    &self,
    tenant_id: &str,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError> {
    self
      .inner
      .list_pending_requests(tenant_id, page, per_page)
      .await
      .tap(|_| self.notify("list_pending_requests"))
  }

  async fn list_all_requests(
    &self,
    tenant_id: &str,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError> {
    self
      .inner
      .list_all_requests(tenant_id, page, per_page)
      .await
      .tap(|_| self.notify("list_all_requests"))
  }

  async fn update_request_status(
    &self,
    tenant_id: &str,
    id: &str,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_request_status(tenant_id, id, status, reviewer)
      .await
      .tap(|_| self.notify("update_request_status"))
  }

  async fn get_request_by_id(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<UserAccessRequestEntity>, DbError> {
    self
      .inner
      .get_request_by_id(tenant_id, id)
      .await
      .tap(|_| self.notify("get_request_by_id"))
  }
}

#[async_trait::async_trait]
impl TokenRepository for TestDbService {
  async fn create_api_token(
    &self,
    tenant_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), DbError> {
    self
      .inner
      .create_api_token(tenant_id, token)
      .await
      .tap(|_| self.notify("create_api_token"))
  }

  async fn list_api_tokens(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<TokenEntity>, usize), DbError> {
    self
      .inner
      .list_api_tokens(tenant_id, user_id, page, per_page)
      .await
      .tap(|_| self.notify("list_api_tokens"))
  }

  async fn get_api_token_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<TokenEntity>, DbError> {
    self
      .inner
      .get_api_token_by_id(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("get_api_token_by_id"))
  }

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<TokenEntity>, DbError> {
    self
      .inner
      .get_api_token_by_prefix(prefix)
      .await
      .tap(|_| self.notify("get_api_token_by_prefix"))
  }

  async fn update_api_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_api_token(tenant_id, user_id, token)
      .await
      .tap(|_| self.notify("update_api_token"))
  }
}

#[async_trait::async_trait]
impl ToolsetRepository for TestDbService {
  async fn get_toolset(&self, tenant_id: &str, id: &str) -> Result<Option<ToolsetEntity>, DbError> {
    self
      .inner
      .get_toolset(tenant_id, id)
      .await
      .tap(|_| self.notify("get_toolset"))
  }

  async fn get_toolset_by_slug(
    &self,
    tenant_id: &str,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<ToolsetEntity>, DbError> {
    self
      .inner
      .get_toolset_by_slug(tenant_id, user_id, slug)
      .await
      .tap(|_| self.notify("get_toolset_by_slug"))
  }

  async fn create_toolset(
    &self,
    tenant_id: &str,
    row: &ToolsetEntity,
  ) -> Result<ToolsetEntity, DbError> {
    self
      .inner
      .create_toolset(tenant_id, row)
      .await
      .tap(|_| self.notify("create_toolset"))
  }

  async fn update_toolset(
    &self,
    tenant_id: &str,
    row: &ToolsetEntity,
    api_key_update: RawApiKeyUpdate,
  ) -> Result<ToolsetEntity, DbError> {
    self
      .inner
      .update_toolset(tenant_id, row, api_key_update)
      .await
      .tap(|_| self.notify("update_toolset"))
  }

  async fn list_toolsets(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ToolsetEntity>, DbError> {
    self
      .inner
      .list_toolsets(tenant_id, user_id)
      .await
      .tap(|_| self.notify("list_toolsets"))
  }

  async fn list_toolsets_by_toolset_type(
    &self,
    tenant_id: &str,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetEntity>, DbError> {
    self
      .inner
      .list_toolsets_by_toolset_type(tenant_id, user_id, toolset_type)
      .await
      .tap(|_| self.notify("list_toolsets_by_toolset_type"))
  }

  async fn delete_toolset(&self, tenant_id: &str, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_toolset(tenant_id, id)
      .await
      .tap(|_| self.notify("delete_toolset"))
  }

  async fn get_toolset_api_key(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_toolset_api_key(tenant_id, id)
      .await
      .tap(|_| self.notify("get_toolset_api_key"))
  }

  async fn set_app_toolset_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfigRow, DbError> {
    self
      .inner
      .set_app_toolset_enabled(tenant_id, toolset_type, enabled, updated_by)
      .await
      .tap(|_| self.notify("set_app_toolset_enabled"))
  }

  async fn list_app_toolset_configs(
    &self,
    tenant_id: &str,
  ) -> Result<Vec<AppToolsetConfigRow>, DbError> {
    self
      .inner
      .list_app_toolset_configs(tenant_id)
      .await
      .tap(|_| self.notify("list_app_toolset_configs"))
  }

  async fn get_app_toolset_config(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError> {
    self
      .inner
      .get_app_toolset_config(tenant_id, toolset_type)
      .await
      .tap(|_| self.notify("get_app_toolset_config"))
  }
}

#[async_trait::async_trait]
impl McpServerRepository for TestDbService {
  async fn create_mcp_server(
    &self,
    tenant_id: &str,
    row: &McpServerRow,
  ) -> Result<McpServerRow, DbError> {
    self
      .inner
      .create_mcp_server(tenant_id, row)
      .await
      .tap(|_| self.notify("create_mcp_server"))
  }

  async fn update_mcp_server(
    &self,
    tenant_id: &str,
    row: &McpServerRow,
  ) -> Result<McpServerRow, DbError> {
    self
      .inner
      .update_mcp_server(tenant_id, row)
      .await
      .tap(|_| self.notify("update_mcp_server"))
  }

  async fn get_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpServerRow>, DbError> {
    self
      .inner
      .get_mcp_server(tenant_id, id)
      .await
      .tap(|_| self.notify("get_mcp_server"))
  }

  async fn get_mcp_server_by_url(
    &self,
    tenant_id: &str,
    url: &str,
  ) -> Result<Option<McpServerRow>, DbError> {
    self
      .inner
      .get_mcp_server_by_url(tenant_id, url)
      .await
      .tap(|_| self.notify("get_mcp_server_by_url"))
  }

  async fn list_mcp_servers(
    &self,
    tenant_id: &str,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServerRow>, DbError> {
    self
      .inner
      .list_mcp_servers(tenant_id, enabled)
      .await
      .tap(|_| self.notify("list_mcp_servers"))
  }

  async fn count_mcps_by_server_id(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<(i64, i64), DbError> {
    self
      .inner
      .count_mcps_by_server_id(tenant_id, server_id)
      .await
      .tap(|_| self.notify("count_mcps_by_server_id"))
  }

  async fn clear_mcp_tools_by_server_id(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<u64, DbError> {
    self
      .inner
      .clear_mcp_tools_by_server_id(tenant_id, server_id)
      .await
      .tap(|_| self.notify("clear_mcp_tools_by_server_id"))
  }
}

#[async_trait::async_trait]
impl McpInstanceRepository for TestDbService {
  async fn create_mcp(&self, tenant_id: &str, row: &McpRow) -> Result<McpRow, DbError> {
    self
      .inner
      .create_mcp(tenant_id, row)
      .await
      .tap(|_| self.notify("create_mcp"))
  }

  async fn get_mcp(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpRow>, DbError> {
    self
      .inner
      .get_mcp(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("get_mcp"))
  }

  async fn get_mcp_by_slug(
    &self,
    tenant_id: &str,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<McpRow>, DbError> {
    self
      .inner
      .get_mcp_by_slug(tenant_id, user_id, slug)
      .await
      .tap(|_| self.notify("get_mcp_by_slug"))
  }

  async fn list_mcps_with_server(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpWithServerRow>, DbError> {
    self
      .inner
      .list_mcps_with_server(tenant_id, user_id)
      .await
      .tap(|_| self.notify("list_mcps_with_server"))
  }

  async fn update_mcp(&self, tenant_id: &str, row: &McpRow) -> Result<McpRow, DbError> {
    self
      .inner
      .update_mcp(tenant_id, row)
      .await
      .tap(|_| self.notify("update_mcp"))
  }

  async fn delete_mcp(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("delete_mcp"))
  }
}

#[async_trait::async_trait]
impl McpAuthRepository for TestDbService {
  async fn get_mcp_auth_header(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<crate::mcps::McpAuthHeader>, DbError> {
    self
      .inner
      .get_mcp_auth_header(tenant_id, id)
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

  async fn delete_mcp_auth_header(&self, tenant_id: &str, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp_auth_header(tenant_id, id)
      .await
      .tap(|_| self.notify("delete_mcp_auth_header"))
  }

  async fn list_mcp_auth_headers_by_server(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<Vec<crate::mcps::McpAuthHeader>, DbError> {
    self
      .inner
      .list_mcp_auth_headers_by_server(tenant_id, mcp_server_id)
      .await
      .tap(|_| self.notify("list_mcp_auth_headers_by_server"))
  }

  async fn get_decrypted_auth_header(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    self
      .inner
      .get_decrypted_auth_header(tenant_id, id)
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

  async fn get_mcp_oauth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<crate::mcps::McpOAuthConfig>, DbError> {
    self
      .inner
      .get_mcp_oauth_config(tenant_id, id)
      .await
      .tap(|_| self.notify("get_mcp_oauth_config"))
  }

  async fn list_mcp_oauth_configs_by_server(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<Vec<crate::mcps::McpOAuthConfig>, DbError> {
    self
      .inner
      .list_mcp_oauth_configs_by_server(tenant_id, mcp_server_id)
      .await
      .tap(|_| self.notify("list_mcp_oauth_configs_by_server"))
  }

  async fn delete_mcp_oauth_config(&self, tenant_id: &str, id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp_oauth_config(tenant_id, id)
      .await
      .tap(|_| self.notify("delete_mcp_oauth_config"))
  }

  async fn delete_oauth_config_cascade(
    &self,
    tenant_id: &str,
    config_id: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_oauth_config_cascade(tenant_id, config_id)
      .await
      .tap(|_| self.notify("delete_oauth_config_cascade"))
  }

  async fn get_decrypted_client_secret(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    self
      .inner
      .get_decrypted_client_secret(tenant_id, id)
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
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<crate::mcps::McpOAuthToken>, DbError> {
    self
      .inner
      .get_mcp_oauth_token(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("get_mcp_oauth_token"))
  }

  async fn get_latest_oauth_token_by_config(
    &self,
    tenant_id: &str,
    config_id: &str,
  ) -> Result<Option<crate::mcps::McpOAuthToken>, DbError> {
    self
      .inner
      .get_latest_oauth_token_by_config(tenant_id, config_id)
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

  async fn delete_mcp_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_mcp_oauth_token(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("delete_mcp_oauth_token"))
  }

  async fn delete_oauth_tokens_by_config(
    &self,
    tenant_id: &str,
    config_id: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_oauth_tokens_by_config(tenant_id, config_id)
      .await
      .tap(|_| self.notify("delete_oauth_tokens_by_config"))
  }

  async fn delete_oauth_tokens_by_config_and_user(
    &self,
    tenant_id: &str,
    config_id: &str,
    user_id: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_oauth_tokens_by_config_and_user(tenant_id, config_id, user_id)
      .await
      .tap(|_| self.notify("delete_oauth_tokens_by_config_and_user"))
  }

  async fn get_decrypted_oauth_bearer(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    self
      .inner
      .get_decrypted_oauth_bearer(tenant_id, id)
      .await
      .tap(|_| self.notify("get_decrypted_oauth_bearer"))
  }

  async fn get_decrypted_refresh_token(
    &self,
    tenant_id: &str,
    token_id: &str,
  ) -> Result<Option<String>, DbError> {
    self
      .inner
      .get_decrypted_refresh_token(tenant_id, token_id)
      .await
      .tap(|_| self.notify("get_decrypted_refresh_token"))
  }
}

#[async_trait::async_trait]
impl UserAliasRepository for TestDbService {
  async fn create_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &UserAlias,
  ) -> Result<(), DbError> {
    self
      .inner
      .create_user_alias(tenant_id, user_id, alias)
      .await
      .tap(|_| self.notify("create_user_alias"))
  }

  async fn get_user_alias_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<UserAlias>, DbError> {
    self
      .inner
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("get_user_alias_by_id"))
  }

  async fn get_user_alias_by_name(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
  ) -> Result<Option<UserAlias>, DbError> {
    self
      .inner
      .get_user_alias_by_name(tenant_id, user_id, alias)
      .await
      .tap(|_| self.notify("get_user_alias_by_name"))
  }

  async fn update_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    alias: &UserAlias,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_user_alias(tenant_id, user_id, id, alias)
      .await
      .tap(|_| self.notify("update_user_alias"))
  }

  async fn delete_user_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError> {
    self
      .inner
      .delete_user_alias(tenant_id, user_id, id)
      .await
      .tap(|_| self.notify("delete_user_alias"))
  }

  async fn list_user_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<UserAlias>, DbError> {
    self
      .inner
      .list_user_aliases(tenant_id, user_id)
      .await
      .tap(|_| self.notify("list_user_aliases"))
  }
}

#[async_trait::async_trait]
impl TenantRepository for TestDbService {
  async fn get_tenant(&self) -> Result<Option<TenantRow>, DbError> {
    self
      .inner
      .get_tenant()
      .await
      .tap(|_| self.notify("get_tenant"))
  }

  async fn get_tenant_by_id(&self, id: &str) -> Result<Option<TenantRow>, DbError> {
    self
      .inner
      .get_tenant_by_id(id)
      .await
      .tap(|_| self.notify("get_tenant_by_id"))
  }

  async fn get_tenant_by_client_id(&self, client_id: &str) -> Result<Option<TenantRow>, DbError> {
    self
      .inner
      .get_tenant_by_client_id(client_id)
      .await
      .tap(|_| self.notify("get_tenant_by_client_id"))
  }

  async fn create_tenant(
    &self,
    client_id: &str,
    client_secret: &str,
    status: &crate::AppStatus,
  ) -> Result<TenantRow, DbError> {
    self
      .inner
      .create_tenant(client_id, client_secret, status)
      .await
      .tap(|_| self.notify("create_tenant"))
  }

  async fn update_tenant_status(
    &self,
    client_id: &str,
    status: &crate::AppStatus,
  ) -> Result<(), DbError> {
    self
      .inner
      .update_tenant_status(client_id, status)
      .await
      .tap(|_| self.notify("update_tenant_status"))
  }

  async fn delete_tenant(&self, client_id: &str) -> Result<(), DbError> {
    self
      .inner
      .delete_tenant(client_id)
      .await
      .tap(|_| self.notify("delete_tenant"))
  }

  async fn create_tenant_test(&self, tenant: &crate::Tenant) -> Result<TenantRow, DbError> {
    self
      .inner
      .create_tenant_test(tenant)
      .await
      .tap(|_| self.notify("create_tenant_test"))
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
  async fn create(&self, row: &AppAccessRequest) -> Result<AppAccessRequest, DbError> {
    self
      .inner
      .create(row)
      .await
      .tap(|_| self.notify("access_request_create"))
  }

  async fn get(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>, DbError> {
    self
      .inner
      .get(tenant_id, id)
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
  ) -> Result<AppAccessRequest, DbError> {
    self
      .inner
      .update_approval(id, user_id, approved, approved_role, access_request_scope)
      .await
      .tap(|_| self.notify("access_request_update_approval"))
  }

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequest, DbError> {
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
  ) -> Result<AppAccessRequest, DbError> {
    self
      .inner
      .update_failure(id, error_message)
      .await
      .tap(|_| self.notify("access_request_update_failure"))
  }

  async fn get_by_access_request_scope(
    &self,
    tenant_id: &str,
    scope: &str,
  ) -> Result<Option<AppAccessRequest>, DbError> {
    self
      .inner
      .get_by_access_request_scope(tenant_id, scope)
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
    async fn begin_tenant_txn(&self, tenant_id: &str) -> Result<sea_orm::DatabaseTransaction, DbError>;
  }

  #[async_trait::async_trait]
  impl DownloadRepository for DbService {
    async fn create_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError>;
    async fn get_download_request(&self, tenant_id: &str, id: &str) -> Result<Option<DownloadRequestEntity>, DbError>;
    async fn update_download_request(&self, request: &DownloadRequestEntity) -> Result<(), DbError>;
    async fn list_download_requests(&self, tenant_id: &str, page: usize, page_size: usize) -> Result<(Vec<DownloadRequestEntity>, usize), DbError>;
    async fn find_download_request_by_repo_filename(&self, tenant_id: &str, repo: &str, filename: &str) -> Result<Vec<DownloadRequestEntity>, DbError>;
  }

  #[async_trait::async_trait]
  impl ApiAliasRepository for DbService {
    async fn create_api_model_alias(&self, tenant_id: &str, user_id: &str, alias: &ApiAlias, api_key: Option<String>) -> Result<(), DbError>;
    async fn get_api_model_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<Option<ApiAlias>, DbError>;
    async fn update_api_model_alias(&self, tenant_id: &str, user_id: &str, id: &str, model: &ApiAlias, api_key: RawApiKeyUpdate) -> Result<(), DbError>;
    async fn update_api_model_cache(&self, tenant_id: &str, id: &str, models: Vec<String>, fetched_at: DateTime<Utc>) -> Result<(), DbError>;
    async fn delete_api_model_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError>;
    async fn list_api_model_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<ApiAlias>, DbError>;
    async fn get_api_key_for_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<Option<String>, DbError>;
    async fn check_prefix_exists(&self, tenant_id: &str, user_id: &str, prefix: &str, exclude_id: Option<String>) -> Result<bool, DbError>;
  }

  #[async_trait::async_trait]
  impl ModelMetadataRepository for DbService {
    async fn upsert_model_metadata(&self, metadata: &ModelMetadataEntity) -> Result<(), DbError>;
    async fn get_model_metadata_by_file(&self, tenant_id: &str, repo: &str, filename: &str, snapshot: &str) -> Result<Option<ModelMetadataEntity>, DbError>;
    async fn batch_get_metadata_by_files(&self, tenant_id: &str, files: &[(String, String, String)]) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataEntity>, DbError>;
    async fn list_model_metadata(&self, tenant_id: &str) -> Result<Vec<ModelMetadataEntity>, DbError>;
  }

  #[async_trait::async_trait]
  impl AccessRepository for DbService {
    async fn insert_pending_request(&self, tenant_id: &str, username: String, user_id: String) -> Result<UserAccessRequestEntity, DbError>;
    async fn get_pending_request(&self, tenant_id: &str, user_id: String) -> Result<Option<UserAccessRequestEntity>, DbError>;
    async fn list_pending_requests(&self, tenant_id: &str, page: u32, per_page: u32) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError>;
    async fn list_all_requests(&self, tenant_id: &str, page: u32, per_page: u32) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError>;
    async fn update_request_status(&self, tenant_id: &str, id: &str, status: UserAccessRequestStatus, reviewer: String) -> Result<(), DbError>;
    async fn get_request_by_id(&self, tenant_id: &str, id: &str) -> Result<Option<UserAccessRequestEntity>, DbError>;
  }

  #[async_trait::async_trait]
  impl TenantRepository for DbService {
    async fn get_tenant(&self) -> Result<Option<TenantRow>, DbError>;
    async fn get_tenant_by_id(&self, id: &str) -> Result<Option<TenantRow>, DbError>;
    async fn get_tenant_by_client_id(&self, client_id: &str) -> Result<Option<TenantRow>, DbError>;
    async fn create_tenant(
      &self,
      client_id: &str,
      client_secret: &str,
      status: &crate::AppStatus,
    ) -> Result<TenantRow, DbError>;
    async fn update_tenant_status(
      &self,
      client_id: &str,
      status: &crate::AppStatus,
    ) -> Result<(), DbError>;
    async fn delete_tenant(&self, client_id: &str) -> Result<(), DbError>;
    async fn create_tenant_test(&self, tenant: &crate::Tenant) -> Result<TenantRow, DbError>;
  }

  #[async_trait::async_trait]
  impl TokenRepository for DbService {
    async fn create_api_token(&self, tenant_id: &str, token: &mut TokenEntity) -> Result<(), DbError>;
    async fn list_api_tokens(&self, tenant_id: &str, user_id: &str, page: usize, per_page: usize) -> Result<(Vec<TokenEntity>, usize), DbError>;
    async fn get_api_token_by_id(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<Option<TokenEntity>, DbError>;
    async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<TokenEntity>, DbError>;
    async fn update_api_token(&self, tenant_id: &str, user_id: &str, token: &mut TokenEntity) -> Result<(), DbError>;
  }

  #[async_trait::async_trait]
  impl UserAliasRepository for DbService {
    async fn create_user_alias(&self, tenant_id: &str, user_id: &str, alias: &UserAlias) -> Result<(), DbError>;
    async fn get_user_alias_by_id(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<Option<UserAlias>, DbError>;
    async fn get_user_alias_by_name(&self, tenant_id: &str, user_id: &str, alias: &str) -> Result<Option<UserAlias>, DbError>;
    async fn update_user_alias(&self, tenant_id: &str, user_id: &str, id: &str, alias: &UserAlias) -> Result<(), DbError>;
    async fn delete_user_alias(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError>;
    async fn list_user_aliases(&self, tenant_id: &str, user_id: &str) -> Result<Vec<UserAlias>, DbError>;
  }

  #[async_trait::async_trait]
  impl ToolsetRepository for DbService {
    async fn get_toolset(&self, tenant_id: &str, id: &str) -> Result<Option<ToolsetEntity>, DbError>;
    async fn get_toolset_by_slug(&self, tenant_id: &str, user_id: &str, slug: &str) -> Result<Option<ToolsetEntity>, DbError>;
    async fn create_toolset(&self, tenant_id: &str, row: &ToolsetEntity) -> Result<ToolsetEntity, DbError>;
    async fn update_toolset(&self, tenant_id: &str, row: &ToolsetEntity, api_key_update: RawApiKeyUpdate) -> Result<ToolsetEntity, DbError>;
    async fn list_toolsets(&self, tenant_id: &str, user_id: &str) -> Result<Vec<ToolsetEntity>, DbError>;
    async fn list_toolsets_by_toolset_type(&self, tenant_id: &str, user_id: &str, toolset_type: &str) -> Result<Vec<ToolsetEntity>, DbError>;
    async fn delete_toolset(&self, tenant_id: &str, id: &str) -> Result<(), DbError>;
    async fn get_toolset_api_key(&self, tenant_id: &str, id: &str) -> Result<Option<String>, DbError>;
    async fn set_app_toolset_enabled(&self, tenant_id: &str, toolset_type: &str, enabled: bool, updated_by: &str) -> Result<AppToolsetConfigRow, DbError>;
    async fn list_app_toolset_configs(&self, tenant_id: &str) -> Result<Vec<AppToolsetConfigRow>, DbError>;
    async fn get_app_toolset_config(&self, tenant_id: &str, toolset_type: &str) -> Result<Option<AppToolsetConfigRow>, DbError>;
  }

  #[async_trait::async_trait]
  impl McpServerRepository for DbService {
    async fn create_mcp_server(&self, tenant_id: &str, row: &McpServerRow) -> Result<McpServerRow, DbError>;
    async fn update_mcp_server(&self, tenant_id: &str, row: &McpServerRow) -> Result<McpServerRow, DbError>;
    async fn get_mcp_server(&self, tenant_id: &str, id: &str) -> Result<Option<McpServerRow>, DbError>;
    async fn get_mcp_server_by_url(&self, tenant_id: &str, url: &str) -> Result<Option<McpServerRow>, DbError>;
    async fn list_mcp_servers(&self, tenant_id: &str, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError>;
    async fn count_mcps_by_server_id(&self, tenant_id: &str, server_id: &str) -> Result<(i64, i64), DbError>;
    async fn clear_mcp_tools_by_server_id(&self, tenant_id: &str, server_id: &str) -> Result<u64, DbError>;
  }

  #[async_trait::async_trait]
  impl McpInstanceRepository for DbService {
    async fn create_mcp(&self, tenant_id: &str, row: &McpRow) -> Result<McpRow, DbError>;
    async fn get_mcp(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError>;
    async fn get_mcp_by_slug(&self, tenant_id: &str, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError>;
    async fn list_mcps_with_server(&self, tenant_id: &str, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError>;
    async fn update_mcp(&self, tenant_id: &str, row: &McpRow) -> Result<McpRow, DbError>;
    async fn delete_mcp(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError>;
  }

  #[async_trait::async_trait]
  impl McpAuthRepository for DbService {
    async fn create_mcp_auth_header(&self, row: &McpAuthHeaderRow) -> Result<McpAuthHeaderRow, DbError>;
    async fn get_mcp_auth_header(&self, tenant_id: &str, id: &str) -> Result<Option<crate::mcps::McpAuthHeader>, DbError>;
    async fn update_mcp_auth_header(&self, row: &McpAuthHeaderRow) -> Result<McpAuthHeaderRow, DbError>;
    async fn delete_mcp_auth_header(&self, tenant_id: &str, id: &str) -> Result<(), DbError>;
    async fn list_mcp_auth_headers_by_server(&self, tenant_id: &str, mcp_server_id: &str) -> Result<Vec<crate::mcps::McpAuthHeader>, DbError>;
    async fn get_decrypted_auth_header(&self, tenant_id: &str, id: &str) -> Result<Option<(String, String)>, DbError>;
    async fn create_mcp_oauth_config(&self, row: &McpOAuthConfigRow) -> Result<McpOAuthConfigRow, DbError>;
    async fn get_mcp_oauth_config(&self, tenant_id: &str, id: &str) -> Result<Option<crate::mcps::McpOAuthConfig>, DbError>;
    async fn list_mcp_oauth_configs_by_server(&self, tenant_id: &str, mcp_server_id: &str) -> Result<Vec<crate::mcps::McpOAuthConfig>, DbError>;
    async fn delete_mcp_oauth_config(&self, tenant_id: &str, id: &str) -> Result<(), DbError>;
    async fn delete_oauth_config_cascade(&self, tenant_id: &str, config_id: &str) -> Result<(), DbError>;
    async fn get_decrypted_client_secret(&self, tenant_id: &str, id: &str) -> Result<Option<(String, String)>, DbError>;
    async fn create_mcp_oauth_token(&self, row: &McpOAuthTokenRow) -> Result<McpOAuthTokenRow, DbError>;
    async fn get_mcp_oauth_token(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<Option<crate::mcps::McpOAuthToken>, DbError>;
    async fn get_latest_oauth_token_by_config(&self, tenant_id: &str, config_id: &str) -> Result<Option<crate::mcps::McpOAuthToken>, DbError>;
    async fn update_mcp_oauth_token(&self, row: &McpOAuthTokenRow) -> Result<McpOAuthTokenRow, DbError>;
    async fn delete_mcp_oauth_token(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), DbError>;
    async fn delete_oauth_tokens_by_config(&self, tenant_id: &str, config_id: &str) -> Result<(), DbError>;
    async fn delete_oauth_tokens_by_config_and_user(&self, tenant_id: &str, config_id: &str, user_id: &str) -> Result<(), DbError>;
    async fn get_decrypted_oauth_bearer(&self, tenant_id: &str, id: &str) -> Result<Option<(String, String)>, DbError>;
    async fn get_decrypted_refresh_token(&self, tenant_id: &str, token_id: &str) -> Result<Option<String>, DbError>;
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
    async fn create(&self, row: &AppAccessRequest) -> Result<AppAccessRequest, DbError>;
    async fn get(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>, DbError>;
    async fn update_approval(
      &self,
      id: &str,
      user_id: &str,
      approved: &str,
      approved_role: &str,
      access_request_scope: &str,
    ) -> Result<AppAccessRequest, DbError>;
    async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequest, DbError>;
    async fn update_failure(&self, id: &str, error_message: &str) -> Result<AppAccessRequest, DbError>;
    async fn get_by_access_request_scope(
      &self,
      tenant_id: &str,
      scope: &str,
    ) -> Result<Option<AppAccessRequest>, DbError>;
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
