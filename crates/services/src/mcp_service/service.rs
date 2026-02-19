use super::{McpError, McpServerError};
use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  DbService, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow, McpRow, McpServerRow,
  McpWithServerRow, TimeService,
};
use chrono::DateTime;
use mcp_client::McpClient;
use objs::{
  Mcp, McpAuthHeader, McpAuthType, McpExecutionRequest, McpExecutionResponse, McpOAuthConfig,
  McpOAuthToken, McpServer, McpServerInfo, McpTool,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait McpService: Debug + Send + Sync {
  // ---- MCP Server admin operations ----

  async fn create_mcp_server(
    &self,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    created_by: &str,
  ) -> Result<McpServer, McpServerError>;

  async fn update_mcp_server(
    &self,
    id: &str,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServer, McpServerError>;

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServer>, McpServerError>;

  async fn list_mcp_servers(&self, enabled: Option<bool>)
    -> Result<Vec<McpServer>, McpServerError>;

  async fn count_mcps_for_server(&self, server_id: &str) -> Result<(i64, i64), McpServerError>;

  // ---- MCP user instance operations ----

  async fn list(&self, user_id: &str) -> Result<Vec<Mcp>, McpError>;

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Mcp>, McpError>;

  async fn create(
    &self,
    user_id: &str,
    name: &str,
    slug: &str,
    mcp_server_id: &str,
    description: Option<String>,
    enabled: bool,
    tools_cache: Option<Vec<McpTool>>,
    tools_filter: Option<Vec<String>>,
    auth_type: McpAuthType,
    auth_uuid: Option<String>,
  ) -> Result<Mcp, McpError>;

  async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    slug: &str,
    description: Option<String>,
    enabled: bool,
    tools_filter: Option<Vec<String>>,
    tools_cache: Option<Vec<McpTool>>,
    auth_type: Option<McpAuthType>,
    auth_uuid: Option<String>,
  ) -> Result<Mcp, McpError>;

  async fn delete(&self, user_id: &str, id: &str) -> Result<(), McpError>;

  async fn fetch_tools(&self, user_id: &str, id: &str) -> Result<Vec<McpTool>, McpError>;

  async fn fetch_tools_for_server(
    &self,
    server_id: &str,
    auth_header_key: Option<String>,
    auth_header_value: Option<String>,
    auth_uuid: Option<String>,
  ) -> Result<Vec<McpTool>, McpError>;

  async fn execute(
    &self,
    user_id: &str,
    id: &str,
    tool_name: &str,
    request: McpExecutionRequest,
  ) -> Result<McpExecutionResponse, McpError>;

  // ---- MCP auth header config operations ----

  async fn create_auth_header(
    &self,
    user_id: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError>;

  async fn get_auth_header(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthHeader>, McpError>;

  async fn update_auth_header(
    &self,
    user_id: &str,
    id: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError>;

  async fn delete_auth_header(&self, user_id: &str, id: &str) -> Result<(), McpError>;

  // ---- MCP OAuth config operations ----

  async fn create_oauth_config(
    &self,
    user_id: &str,
    mcp_server_id: &str,
    client_id: &str,
    client_secret: &str,
    authorization_endpoint: &str,
    token_endpoint: &str,
    scopes: Option<String>,
  ) -> Result<McpOAuthConfig, McpError>;

  async fn get_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfig>, McpError>;

  async fn list_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpOAuthConfig>, McpError>;

  async fn delete_oauth_config(&self, id: &str) -> Result<(), McpError>;

  // ---- MCP OAuth token operations ----

  async fn store_oauth_token(
    &self,
    user_id: &str,
    config_id: &str,
    access_token: &str,
    refresh_token: Option<String>,
    scopes_granted: Option<String>,
    expires_in: Option<i64>,
  ) -> Result<McpOAuthToken, McpError>;

  async fn get_oauth_token(
    &self,
    user_id: &str,
    token_id: &str,
  ) -> Result<Option<McpOAuthToken>, McpError>;

  // ---- OAuth discovery ----

  async fn discover_oauth_metadata(&self, url: &str) -> Result<serde_json::Value, McpError>;
}

pub struct DefaultMcpService {
  db_service: Arc<dyn DbService>,
  mcp_client: Arc<dyn McpClient>,
  time_service: Arc<dyn TimeService>,
  http_client: reqwest::Client,
  refresh_locks: RwLock<HashMap<String, Arc<Mutex<()>>>>,
}

impl Debug for DefaultMcpService {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DefaultMcpService")
      .field("db_service", &self.db_service)
      .field("mcp_client", &self.mcp_client)
      .field("time_service", &self.time_service)
      .finish()
  }
}

impl DefaultMcpService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    mcp_client: Arc<dyn McpClient>,
    time_service: Arc<dyn TimeService>,
  ) -> Self {
    Self {
      db_service,
      mcp_client,
      time_service,
      http_client: reqwest::Client::new(),
      refresh_locks: RwLock::new(HashMap::new()),
    }
  }

  async fn get_refresh_lock(&self, key: &str) -> Arc<Mutex<()>> {
    {
      let locks = self.refresh_locks.read().await;
      if let Some(lock) = locks.get(key) {
        return Arc::clone(lock);
      }
    }
    let mut locks = self.refresh_locks.write().await;
    if let Some(lock) = locks.get(key) {
      return Arc::clone(lock);
    }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(key.to_string(), Arc::clone(&lock));
    lock
  }

  fn mcp_server_row_to_model(&self, row: McpServerRow) -> McpServer {
    McpServer {
      id: row.id,
      url: row.url,
      name: row.name,
      description: row.description,
      enabled: row.enabled,
      created_by: row.created_by,
      updated_by: row.updated_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn mcp_with_server_to_model(&self, row: McpWithServerRow) -> Mcp {
    let tools_cache: Option<Vec<McpTool>> = row
      .tools_cache
      .as_ref()
      .and_then(|tc| serde_json::from_str(tc).ok());
    let tools_filter: Option<Vec<String>> = row
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok());
    let auth_type = row.auth_type.parse::<McpAuthType>().unwrap_or_default();

    Mcp {
      id: row.id,
      mcp_server: McpServerInfo {
        id: row.mcp_server_id,
        url: row.server_url,
        name: row.server_name,
        enabled: row.server_enabled,
      },
      slug: row.slug,
      name: row.name,
      description: row.description,
      enabled: row.enabled,
      tools_cache,
      tools_filter,
      auth_type,
      auth_uuid: row.auth_uuid,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn mcp_row_to_model(&self, row: McpRow, server: &McpServerRow) -> Mcp {
    let tools_cache: Option<Vec<McpTool>> = row
      .tools_cache
      .as_ref()
      .and_then(|tc| serde_json::from_str(tc).ok());
    let tools_filter: Option<Vec<String>> = row
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok());
    let auth_type = row.auth_type.parse::<McpAuthType>().unwrap_or_default();

    Mcp {
      id: row.id,
      mcp_server: McpServerInfo {
        id: server.id.clone(),
        url: server.url.clone(),
        name: server.name.clone(),
        enabled: server.enabled,
      },
      slug: row.slug,
      name: row.name,
      description: row.description,
      enabled: row.enabled,
      tools_cache,
      tools_filter,
      auth_type,
      auth_uuid: row.auth_uuid,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn auth_header_row_to_model(&self, row: McpAuthHeaderRow) -> McpAuthHeader {
    McpAuthHeader {
      id: row.id,
      header_key: row.header_key,
      has_header_value: true,
      created_by: row.created_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn oauth_config_row_to_model(&self, row: McpOAuthConfigRow) -> McpOAuthConfig {
    McpOAuthConfig {
      id: row.id,
      mcp_server_id: row.mcp_server_id,
      client_id: row.client_id,
      authorization_endpoint: row.authorization_endpoint,
      token_endpoint: row.token_endpoint,
      scopes: row.scopes,
      has_client_secret: true,
      created_by: row.created_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn oauth_token_row_to_model(&self, row: McpOAuthTokenRow) -> McpOAuthToken {
    McpOAuthToken {
      id: row.id,
      mcp_oauth_config_id: row.mcp_oauth_config_id,
      scopes_granted: row.scopes_granted,
      expires_at: row.expires_at,
      has_access_token: true,
      has_refresh_token: row.encrypted_refresh_token.is_some(),
      created_by: row.created_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  async fn get_mcp_with_server(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<(McpRow, McpServerRow)>, McpError> {
    let row = self.db_service.get_mcp(user_id, id).await?;
    match row {
      Some(mcp_row) => {
        let server = self
          .db_service
          .get_mcp_server(&mcp_row.mcp_server_id)
          .await?;
        match server {
          Some(s) => Ok(Some((mcp_row, s))),
          None => Ok(None),
        }
      }
      None => Ok(None),
    }
  }

  async fn resolve_auth_header_for_mcp(
    &self,
    mcp_row: &McpRow,
  ) -> Result<Option<(String, String)>, McpError> {
    let auth_type = mcp_row.auth_type.parse::<McpAuthType>().unwrap_or_default();
    let user_id = &mcp_row.created_by;

    match auth_type {
      McpAuthType::Header => {
        if let Some(ref auth_uuid) = mcp_row.auth_uuid {
          return Ok(self.db_service.get_decrypted_auth_header(auth_uuid).await?);
        }
      }
      McpAuthType::OauthPreRegistered => {
        if let Some(ref auth_uuid) = mcp_row.auth_uuid {
          let lock = self
            .get_refresh_lock(&format!("oauth_refresh:{}", auth_uuid))
            .await;
          let _guard = lock.lock().await;

          let token_row = self
            .db_service
            .get_mcp_oauth_token(user_id, auth_uuid)
            .await?
            .ok_or_else(|| McpError::OAuthTokenNotFound(auth_uuid.clone()))?;

          let now = self.time_service.utc_now().timestamp();
          let is_expired = token_row
            .expires_at
            .map(|exp| now >= (exp - 60))
            .unwrap_or(false);

          if is_expired {
            if let (Some(ref enc_refresh), Some(ref salt), Some(ref nonce)) = (
              &token_row.encrypted_refresh_token,
              &token_row.refresh_token_salt,
              &token_row.refresh_token_nonce,
            ) {
              let refresh_token =
                decrypt_api_key(self.db_service.encryption_key(), enc_refresh, salt, nonce)?;

              let config = self
                .db_service
                .get_mcp_oauth_config(&token_row.mcp_oauth_config_id)
                .await?
                .ok_or_else(|| McpError::McpNotFound(token_row.mcp_oauth_config_id.clone()))?;

              let (client_id, client_secret) = self
                .db_service
                .get_decrypted_client_secret(&config.id)
                .await?
                .ok_or_else(|| McpError::McpNotFound(config.id.clone()))?;

              let resp = self
                .http_client
                .post(&config.token_endpoint)
                .form(&[
                  ("grant_type", "refresh_token"),
                  ("refresh_token", &refresh_token),
                  ("client_id", &client_id),
                  ("client_secret", &client_secret),
                ])
                .send()
                .await
                .map_err(|e| McpError::OAuthRefreshFailed(e.to_string()))?;

              if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(McpError::OAuthRefreshFailed(body));
              }

              let token_resp: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| McpError::OAuthRefreshFailed(e.to_string()))?;

              let new_access_token = token_resp["access_token"]
                .as_str()
                .ok_or_else(|| {
                  McpError::OAuthRefreshFailed("missing access_token in response".to_string())
                })?
                .to_string();

              let new_refresh = token_resp["refresh_token"].as_str().map(|s| s.to_string());
              let new_expires_in = token_resp["expires_in"].as_i64();

              let (enc_at, salt_at, nonce_at) =
                encrypt_api_key(self.db_service.encryption_key(), &new_access_token)?;

              let (enc_rt, salt_rt, nonce_rt) = if let Some(ref rt) = new_refresh {
                let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), rt)?;
                (Some(e), Some(s), Some(n))
              } else {
                (
                  token_row.encrypted_refresh_token.clone(),
                  token_row.refresh_token_salt.clone(),
                  token_row.refresh_token_nonce.clone(),
                )
              };

              let new_expires_at = new_expires_in.map(|ei| now + ei);

              let updated_row = McpOAuthTokenRow {
                encrypted_access_token: enc_at,
                access_token_salt: salt_at,
                access_token_nonce: nonce_at,
                encrypted_refresh_token: enc_rt,
                refresh_token_salt: salt_rt,
                refresh_token_nonce: nonce_rt,
                expires_at: new_expires_at,
                updated_at: now,
                ..token_row
              };

              self.db_service.update_mcp_oauth_token(&updated_row).await?;

              return Ok(Some((
                "Authorization".to_string(),
                format!("Bearer {}", new_access_token),
              )));
            } else {
              return Err(McpError::OAuthTokenExpired(auth_uuid.clone()));
            }
          }

          let access_token = decrypt_api_key(
            self.db_service.encryption_key(),
            &token_row.encrypted_access_token,
            &token_row.access_token_salt,
            &token_row.access_token_nonce,
          )?;

          return Ok(Some((
            "Authorization".to_string(),
            format!("Bearer {}", access_token),
          )));
        }
      }
      McpAuthType::Public => {}
    }
    Ok(None)
  }
}

#[async_trait::async_trait]
impl McpService for DefaultMcpService {
  // ---- MCP Server admin operations ----

  async fn create_mcp_server(
    &self,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    created_by: &str,
  ) -> Result<McpServer, McpServerError> {
    let trimmed_url = url.trim();

    objs::validate_mcp_server_name(name).map_err(|_| {
      if name.is_empty() {
        McpServerError::NameRequired
      } else {
        McpServerError::NameTooLong
      }
    })?;

    objs::validate_mcp_server_url(trimmed_url).map_err(|e| {
      if trimmed_url.is_empty() {
        McpServerError::UrlRequired
      } else if trimmed_url.len() > objs::MAX_MCP_SERVER_URL_LEN {
        McpServerError::UrlTooLong
      } else {
        McpServerError::UrlInvalid(e)
      }
    })?;

    if let Some(ref desc) = description {
      objs::validate_mcp_server_description(desc)
        .map_err(|_| McpServerError::DescriptionTooLong)?;
    }

    if let Some(existing) = self.db_service.get_mcp_server_by_url(trimmed_url).await? {
      return Err(McpServerError::UrlAlreadyExists(existing.url));
    }

    let now = self.time_service.utc_now().timestamp();
    let row = McpServerRow {
      id: Uuid::new_v4().to_string(),
      url: trimmed_url.to_string(),
      name: name.to_string(),
      description,
      enabled,
      created_by: created_by.to_string(),
      updated_by: created_by.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_server(&row).await?;
    Ok(self.mcp_server_row_to_model(result))
  }

  async fn update_mcp_server(
    &self,
    id: &str,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServer, McpServerError> {
    let trimmed_url = url.trim();

    objs::validate_mcp_server_name(name).map_err(|_| {
      if name.is_empty() {
        McpServerError::NameRequired
      } else {
        McpServerError::NameTooLong
      }
    })?;

    objs::validate_mcp_server_url(trimmed_url).map_err(|e| {
      if trimmed_url.is_empty() {
        McpServerError::UrlRequired
      } else if trimmed_url.len() > objs::MAX_MCP_SERVER_URL_LEN {
        McpServerError::UrlTooLong
      } else {
        McpServerError::UrlInvalid(e)
      }
    })?;

    if let Some(ref desc) = description {
      objs::validate_mcp_server_description(desc)
        .map_err(|_| McpServerError::DescriptionTooLong)?;
    }

    let existing = self
      .db_service
      .get_mcp_server(id)
      .await?
      .ok_or_else(|| McpServerError::McpServerNotFound(id.to_string()))?;

    if let Some(dup) = self.db_service.get_mcp_server_by_url(trimmed_url).await? {
      if dup.id != existing.id {
        return Err(McpServerError::UrlAlreadyExists(dup.url));
      }
    }

    let url_changed = existing.url.to_lowercase() != trimmed_url.to_lowercase();
    if url_changed {
      self.db_service.clear_mcp_tools_by_server_id(id).await?;
    }

    let now = self.time_service.utc_now().timestamp();
    let row = McpServerRow {
      id: existing.id,
      url: trimmed_url.to_string(),
      name: name.to_string(),
      description,
      enabled,
      created_by: existing.created_by,
      updated_by: updated_by.to_string(),
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp_server(&row).await?;
    Ok(self.mcp_server_row_to_model(result))
  }

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServer>, McpServerError> {
    let row = self.db_service.get_mcp_server(id).await?;
    Ok(row.map(|r| self.mcp_server_row_to_model(r)))
  }

  async fn list_mcp_servers(
    &self,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServer>, McpServerError> {
    let rows = self.db_service.list_mcp_servers(enabled).await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.mcp_server_row_to_model(r))
        .collect(),
    )
  }

  async fn count_mcps_for_server(&self, server_id: &str) -> Result<(i64, i64), McpServerError> {
    Ok(self.db_service.count_mcps_by_server_id(server_id).await?)
  }

  // ---- MCP user instance operations ----

  async fn list(&self, user_id: &str) -> Result<Vec<Mcp>, McpError> {
    let rows = self.db_service.list_mcps_with_server(user_id).await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.mcp_with_server_to_model(r))
        .collect(),
    )
  }

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Mcp>, McpError> {
    match self.get_mcp_with_server(user_id, id).await? {
      Some((row, server)) => Ok(Some(self.mcp_row_to_model(row, &server))),
      None => Ok(None),
    }
  }

  async fn create(
    &self,
    user_id: &str,
    name: &str,
    slug: &str,
    mcp_server_id: &str,
    description: Option<String>,
    enabled: bool,
    tools_cache: Option<Vec<McpTool>>,
    tools_filter: Option<Vec<String>>,
    auth_type: McpAuthType,
    auth_uuid: Option<String>,
  ) -> Result<Mcp, McpError> {
    if name.is_empty() {
      return Err(McpError::NameRequired);
    }

    objs::validate_mcp_slug(slug).map_err(McpError::InvalidSlug)?;

    if let Some(ref desc) = description {
      objs::validate_mcp_description(desc).map_err(McpError::InvalidDescription)?;
    }

    let mcp_server = self
      .db_service
      .get_mcp_server(mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(mcp_server_id.to_string()))?;

    if !mcp_server.enabled {
      return Err(McpError::McpDisabled);
    }

    if self
      .db_service
      .get_mcp_by_slug(user_id, slug)
      .await?
      .is_some()
    {
      return Err(McpError::SlugExists(slug.to_string()));
    }

    let tools_cache_json = tools_cache
      .as_ref()
      .map(|tc| serde_json::to_string(tc).expect("Vec<McpTool> serialization cannot fail"));
    let tools_filter_json = tools_filter
      .as_ref()
      .map(|tf| serde_json::to_string(tf).expect("Vec<String> serialization cannot fail"));

    let now = self.time_service.utc_now().timestamp();
    let row = McpRow {
      id: Uuid::new_v4().to_string(),
      created_by: user_id.to_string(),
      mcp_server_id: mcp_server.id.clone(),
      name: name.to_string(),
      slug: slug.to_string(),
      description,
      enabled,
      tools_cache: tools_cache_json,
      tools_filter: tools_filter_json,
      auth_type: auth_type.as_str().to_string(),
      auth_uuid,
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp(&row).await?;
    Ok(self.mcp_row_to_model(result, &mcp_server))
  }

  async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    slug: &str,
    description: Option<String>,
    enabled: bool,
    tools_filter: Option<Vec<String>>,
    tools_cache: Option<Vec<McpTool>>,
    auth_type: Option<McpAuthType>,
    auth_uuid: Option<String>,
  ) -> Result<Mcp, McpError> {
    if name.is_empty() {
      return Err(McpError::NameRequired);
    }

    objs::validate_mcp_slug(slug).map_err(McpError::InvalidSlug)?;

    if let Some(ref desc) = description {
      objs::validate_mcp_description(desc).map_err(McpError::InvalidDescription)?;
    }

    let (existing, server) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if slug.to_lowercase() != existing.slug.to_lowercase()
      && self
        .db_service
        .get_mcp_by_slug(user_id, slug)
        .await?
        .is_some()
    {
      return Err(McpError::SlugExists(slug.to_string()));
    }

    let resolved_filter = if let Some(filter) = tools_filter {
      Some(serde_json::to_string(&filter).expect("Vec<String> serialization cannot fail"))
    } else {
      existing.tools_filter
    };

    let resolved_cache = if let Some(cache) = tools_cache {
      Some(serde_json::to_string(&cache).expect("Vec<McpTool> serialization cannot fail"))
    } else {
      existing.tools_cache
    };

    let (resolved_auth_type, resolved_auth_uuid) = if let Some(new_auth_type) = auth_type {
      let existing_auth_type = existing
        .auth_type
        .parse::<McpAuthType>()
        .unwrap_or_default();
      if existing_auth_type != new_auth_type {
        if let Some(ref old_uuid) = existing.auth_uuid {
          match existing_auth_type {
            McpAuthType::Header => {
              let _ = self
                .db_service
                .delete_mcp_auth_header(user_id, old_uuid)
                .await;
            }
            McpAuthType::OauthPreRegistered => {
              let _ = self
                .db_service
                .delete_mcp_oauth_token(user_id, old_uuid)
                .await;
            }
            McpAuthType::Public => {}
          }
        }
      }
      (new_auth_type.as_str().to_string(), auth_uuid)
    } else {
      (existing.auth_type, existing.auth_uuid)
    };

    let now = self.time_service.utc_now().timestamp();
    let row = McpRow {
      id: id.to_string(),
      created_by: user_id.to_string(),
      mcp_server_id: existing.mcp_server_id,
      name: name.to_string(),
      slug: slug.to_string(),
      description,
      enabled,
      tools_cache: resolved_cache,
      tools_filter: resolved_filter,
      auth_type: resolved_auth_type,
      auth_uuid: resolved_auth_uuid,
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp(&row).await?;
    Ok(self.mcp_row_to_model(result, &server))
  }

  async fn delete(&self, user_id: &str, id: &str) -> Result<(), McpError> {
    let (existing, _) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    let auth_type = existing
      .auth_type
      .parse::<McpAuthType>()
      .unwrap_or_default();
    if let Some(ref auth_uuid) = existing.auth_uuid {
      match auth_type {
        McpAuthType::Header => {
          let _ = self
            .db_service
            .delete_mcp_auth_header(user_id, auth_uuid)
            .await;
        }
        McpAuthType::OauthPreRegistered => {
          let _ = self
            .db_service
            .delete_mcp_oauth_token(user_id, auth_uuid)
            .await;
        }
        McpAuthType::Public => {}
      }
    }

    self.db_service.delete_mcp(user_id, id).await?;
    Ok(())
  }

  async fn fetch_tools(&self, user_id: &str, id: &str) -> Result<Vec<McpTool>, McpError> {
    let (existing, server) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    let auth_header = self.resolve_auth_header_for_mcp(&existing).await?;
    let tools = self
      .mcp_client
      .fetch_tools(&server.url, auth_header)
      .await?;

    let tools_cache_json = serde_json::to_string(&tools).unwrap_or_default();
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    let tools_filter_json = if existing.tools_filter.is_none() {
      Some(serde_json::to_string(&tool_names).unwrap_or_default())
    } else {
      existing.tools_filter
    };

    let now = self.time_service.utc_now().timestamp();
    let updated_row = McpRow {
      tools_cache: Some(tools_cache_json),
      tools_filter: tools_filter_json,
      updated_at: now,
      ..existing
    };

    self.db_service.update_mcp(&updated_row).await?;
    Ok(tools)
  }

  async fn fetch_tools_for_server(
    &self,
    server_id: &str,
    auth_header_key: Option<String>,
    auth_header_value: Option<String>,
    auth_uuid: Option<String>,
  ) -> Result<Vec<McpTool>, McpError> {
    let server = self
      .db_service
      .get_mcp_server(server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(server_id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    let auth_header = if let Some(uuid) = auth_uuid {
      let header = self.db_service.get_decrypted_auth_header(&uuid).await?;
      if header.is_some() {
        header
      } else {
        self.db_service.get_decrypted_oauth_bearer(&uuid).await?
      }
    } else {
      match (auth_header_key, auth_header_value) {
        (Some(key), Some(value)) => Some((key, value)),
        _ => None,
      }
    };

    let tools = self
      .mcp_client
      .fetch_tools(&server.url, auth_header)
      .await?;
    Ok(tools)
  }

  async fn execute(
    &self,
    user_id: &str,
    id: &str,
    tool_name: &str,
    request: McpExecutionRequest,
  ) -> Result<McpExecutionResponse, McpError> {
    let (existing, server) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    if !existing.enabled {
      return Err(McpError::McpDisabled);
    }

    let tools_filter: Vec<String> = existing
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok())
      .unwrap_or_default();

    if !tools_filter.iter().any(|t| t == tool_name) {
      return Err(McpError::ToolNotAllowed(tool_name.to_string()));
    }

    let auth_header = self.resolve_auth_header_for_mcp(&existing).await?;
    match self
      .mcp_client
      .call_tool(&server.url, tool_name, request.params, auth_header)
      .await
    {
      Ok(result) => Ok(McpExecutionResponse {
        result: Some(result),
        error: None,
      }),
      Err(e) => Ok(McpExecutionResponse {
        result: None,
        error: Some(e.to_string()),
      }),
    }
  }

  // ---- MCP auth header config operations ----

  async fn create_auth_header(
    &self,
    user_id: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError> {
    let (encrypted, salt, nonce) = encrypt_api_key(self.db_service.encryption_key(), header_value)?;

    let now = self.time_service.utc_now().timestamp();
    let row = McpAuthHeaderRow {
      id: Uuid::new_v4().to_string(),
      header_key: header_key.to_string(),
      encrypted_header_value: encrypted,
      header_value_salt: salt,
      header_value_nonce: nonce,
      created_by: user_id.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_auth_header(&row).await?;
    Ok(self.auth_header_row_to_model(result))
  }

  async fn get_auth_header(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthHeader>, McpError> {
    let row = self.db_service.get_mcp_auth_header(user_id, id).await?;
    Ok(row.map(|r| self.auth_header_row_to_model(r)))
  }

  async fn update_auth_header(
    &self,
    user_id: &str,
    id: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError> {
    let existing = self
      .db_service
      .get_mcp_auth_header(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    let (encrypted, salt, nonce) = encrypt_api_key(self.db_service.encryption_key(), header_value)?;

    let now = self.time_service.utc_now().timestamp();
    let row = McpAuthHeaderRow {
      id: existing.id,
      header_key: header_key.to_string(),
      encrypted_header_value: encrypted,
      header_value_salt: salt,
      header_value_nonce: nonce,
      created_by: user_id.to_string(),
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp_auth_header(&row).await?;
    Ok(self.auth_header_row_to_model(result))
  }

  async fn delete_auth_header(&self, user_id: &str, id: &str) -> Result<(), McpError> {
    self.db_service.delete_mcp_auth_header(user_id, id).await?;
    Ok(())
  }

  // ---- MCP OAuth config operations ----

  async fn create_oauth_config(
    &self,
    user_id: &str,
    mcp_server_id: &str,
    client_id: &str,
    client_secret: &str,
    authorization_endpoint: &str,
    token_endpoint: &str,
    scopes: Option<String>,
  ) -> Result<McpOAuthConfig, McpError> {
    let (encrypted, salt, nonce) =
      encrypt_api_key(self.db_service.encryption_key(), client_secret)?;

    let now = self.time_service.utc_now().timestamp();
    let row = McpOAuthConfigRow {
      id: Uuid::new_v4().to_string(),
      mcp_server_id: mcp_server_id.to_string(),
      client_id: client_id.to_string(),
      encrypted_client_secret: encrypted,
      client_secret_salt: salt,
      client_secret_nonce: nonce,
      authorization_endpoint: authorization_endpoint.to_string(),
      token_endpoint: token_endpoint.to_string(),
      scopes,
      created_by: user_id.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_oauth_config(&row).await?;
    Ok(self.oauth_config_row_to_model(result))
  }

  async fn get_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfig>, McpError> {
    let row = self.db_service.get_mcp_oauth_config(id).await?;
    Ok(row.map(|r| self.oauth_config_row_to_model(r)))
  }

  async fn list_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpOAuthConfig>, McpError> {
    let rows = self
      .db_service
      .list_mcp_oauth_configs_by_server(mcp_server_id, user_id)
      .await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.oauth_config_row_to_model(r))
        .collect(),
    )
  }

  async fn delete_oauth_config(&self, id: &str) -> Result<(), McpError> {
    self.db_service.delete_oauth_tokens_by_config(id).await?;
    self.db_service.delete_mcp_oauth_config(id).await?;
    Ok(())
  }

  // ---- MCP OAuth token operations ----

  async fn store_oauth_token(
    &self,
    user_id: &str,
    config_id: &str,
    access_token: &str,
    refresh_token: Option<String>,
    scopes_granted: Option<String>,
    expires_in: Option<i64>,
  ) -> Result<McpOAuthToken, McpError> {
    let (enc_at, salt_at, nonce_at) =
      encrypt_api_key(self.db_service.encryption_key(), access_token)?;

    let (enc_rt, salt_rt, nonce_rt) = if let Some(ref rt) = refresh_token {
      let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), rt)?;
      (Some(e), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let now = self.time_service.utc_now().timestamp();
    let expires_at = expires_in.map(|ei| now + ei);

    let row = McpOAuthTokenRow {
      id: Uuid::new_v4().to_string(),
      mcp_oauth_config_id: config_id.to_string(),
      encrypted_access_token: enc_at,
      access_token_salt: salt_at,
      access_token_nonce: nonce_at,
      encrypted_refresh_token: enc_rt,
      refresh_token_salt: salt_rt,
      refresh_token_nonce: nonce_rt,
      scopes_granted,
      expires_at,
      created_by: user_id.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_oauth_token(&row).await?;
    Ok(self.oauth_token_row_to_model(result))
  }

  async fn get_oauth_token(
    &self,
    user_id: &str,
    token_id: &str,
  ) -> Result<Option<McpOAuthToken>, McpError> {
    let row = self
      .db_service
      .get_mcp_oauth_token(user_id, token_id)
      .await?;
    Ok(row.map(|r| self.oauth_token_row_to_model(r)))
  }

  // ---- OAuth discovery ----

  async fn discover_oauth_metadata(&self, url: &str) -> Result<serde_json::Value, McpError> {
    let discovery_url = format!(
      "{}/.well-known/oauth-authorization-server",
      url.trim_end_matches('/')
    );
    let resp = self
      .http_client
      .get(&discovery_url)
      .send()
      .await
      .map_err(|e| McpError::OAuthDiscoveryFailed(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(McpError::OAuthDiscoveryFailed(format!(
        "HTTP {}: {}",
        status.as_u16(),
        body
      )));
    }

    resp
      .json()
      .await
      .map_err(|e| McpError::OAuthDiscoveryFailed(e.to_string()))
  }
}
