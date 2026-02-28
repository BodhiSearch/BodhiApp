use super::{McpError, McpServerError};
use crate::db::{
  encryption::encrypt_api_key, DbService, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow,
  McpRow, McpServerRow, McpWithServerRow, TimeService,
};
use mcp_client::McpClient;
use objs::{
  CreateMcpAuthConfigRequest, Mcp, McpAuthConfigResponse, McpAuthHeader, McpAuthType,
  McpExecutionRequest, McpExecutionResponse, McpOAuthConfig, McpOAuthToken, McpServer,
  McpServerInfo, McpTool, RegistrationType,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use ulid::Ulid;

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
    name: &str,
    mcp_server_id: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError>;

  async fn get_auth_header(&self, id: &str) -> Result<Option<McpAuthHeader>, McpError>;

  async fn list_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeader>, McpError>;

  async fn update_auth_header(
    &self,
    id: &str,
    name: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError>;

  async fn delete_auth_header(&self, id: &str) -> Result<(), McpError>;

  // ---- MCP OAuth config operations ----

  async fn create_oauth_config(
    &self,
    user_id: &str,
    name: &str,
    mcp_server_id: &str,
    client_id: &str,
    client_secret: Option<String>,
    authorization_endpoint: &str,
    token_endpoint: &str,
    scopes: Option<String>,
    registration_type: RegistrationType,
    registration_endpoint: Option<String>,
    token_endpoint_auth_method: Option<String>,
    client_id_issued_at: Option<i64>,
    registration_access_token: Option<String>,
  ) -> Result<McpOAuthConfig, McpError>;

  async fn get_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfig>, McpError>;

  async fn list_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
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

  /// Exchange an authorization code for tokens via the OAuth token endpoint.
  /// Handles client credential resolution, HTTP POST, response parsing,
  /// and token storage.
  async fn exchange_oauth_token(
    &self,
    user_id: &str,
    config_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
  ) -> Result<McpOAuthToken, McpError>;

  // ---- OAuth discovery ----

  async fn discover_oauth_metadata(&self, url: &str) -> Result<serde_json::Value, McpError>;

  async fn discover_mcp_oauth_metadata(
    &self,
    mcp_server_url: &str,
  ) -> Result<serde_json::Value, McpError>;

  async fn dynamic_register_client(
    &self,
    registration_endpoint: &str,
    redirect_uri: &str,
    scopes: Option<String>,
  ) -> Result<serde_json::Value, McpError>;

  // ---- Unified auth config operations ----

  async fn create_auth_config(
    &self,
    user_id: &str,
    mcp_server_id: &str,
    request: CreateMcpAuthConfigRequest,
  ) -> Result<McpAuthConfigResponse, McpError>;

  async fn list_auth_configs(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigResponse>, McpError>;

  async fn get_auth_config(&self, id: &str) -> Result<Option<McpAuthConfigResponse>, McpError>;

  async fn delete_auth_config(&self, id: &str) -> Result<(), McpError>;
}

/// Maximum number of concurrent refresh lock entries.
/// Once exceeded, least-recently-inserted entries are evicted.
const MAX_REFRESH_LOCKS: usize = 1000;

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
    // Double-check after acquiring write lock
    if let Some(lock) = locks.get(key) {
      return Arc::clone(lock);
    }
    // Evict entries when exceeding the bound.
    // Only evict entries whose Mutex is not currently held (strong_count == 1).
    if locks.len() >= MAX_REFRESH_LOCKS {
      let evict_keys: Vec<String> = locks
        .iter()
        .filter(|(_, v)| Arc::strong_count(v) == 1)
        .map(|(k, _)| k.clone())
        .collect();
      for k in evict_keys {
        locks.remove(&k);
        if locks.len() < MAX_REFRESH_LOCKS {
          break;
        }
      }
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
      created_at: row.created_at,
      updated_at: row.updated_at,
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
      auth_type: row.auth_type,
      auth_uuid: row.auth_uuid,
      created_at: row.created_at,
      updated_at: row.updated_at,
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
      auth_type: row.auth_type,
      auth_uuid: row.auth_uuid,
      created_at: row.created_at,
      updated_at: row.updated_at,
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

  async fn get_mcp_server_url(&self, mcp_server_id: &str) -> Result<String, McpError> {
    let server = self
      .db_service
      .get_mcp_server(mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(mcp_server_id.to_string()))?;
    Ok(server.url)
  }

  async fn resolve_oauth_token(
    &self,
    user_id: &str,
    auth_uuid: &str,
  ) -> Result<Option<(String, String)>, McpError> {
    let lock = self
      .get_refresh_lock(&format!("oauth_refresh:{}", auth_uuid))
      .await;
    let _guard = lock.lock().await;

    let token = self
      .db_service
      .get_mcp_oauth_token(user_id, auth_uuid)
      .await?
      .ok_or_else(|| McpError::OAuthTokenNotFound(auth_uuid.to_string()))?;

    let now = self.time_service.utc_now();
    let is_expired = token
      .expires_at
      .map(|exp_ts| now.timestamp() >= exp_ts - 60)
      .unwrap_or(false);

    if is_expired {
      if !token.has_refresh_token {
        warn!(
          auth_uuid,
          "OAuth token expired with no refresh token available"
        );
        return Err(McpError::OAuthTokenExpired(auth_uuid.to_string()));
      }

      debug!(auth_uuid, "OAuth token expired, attempting refresh");
      let refresh_token = self
        .db_service
        .get_decrypted_refresh_token(auth_uuid)
        .await?
        .ok_or_else(|| McpError::OAuthTokenExpired(auth_uuid.to_string()))?;

      let config = self
        .db_service
        .get_mcp_oauth_config(&token.mcp_oauth_config_id)
        .await?
        .ok_or_else(|| McpError::McpNotFound(token.mcp_oauth_config_id.clone()))?;

      let client_creds = self
        .db_service
        .get_decrypted_client_secret(&config.id)
        .await?;

      let mut form_params = vec![
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh_token.clone()),
      ];

      if let Some((client_id, client_secret)) = client_creds {
        form_params.push(("client_id".to_string(), client_id));
        form_params.push(("client_secret".to_string(), client_secret));
      } else {
        form_params.push(("client_id".to_string(), config.client_id.clone()));
      }

      let mcp_server_url = self.get_mcp_server_url(&config.mcp_server_id).await?;
      form_params.push(("resource".to_string(), mcp_server_url));

      debug!(
        auth_uuid,
        token_endpoint = config.token_endpoint,
        "Sending token refresh request"
      );
      let resp = self
        .http_client
        .post(&config.token_endpoint)
        .form(&form_params)
        .send()
        .await
        .map_err(|e| McpError::OAuthRefreshFailed(e.to_string()))?;

      if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        warn!(
          auth_uuid,
          status = status.as_u16(),
          "OAuth token refresh failed"
        );
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

      // Use new refresh token if provided, otherwise re-encrypt the old one
      let effective_refresh = new_refresh.as_deref().unwrap_or(&refresh_token);
      let (enc_rt, salt_rt, nonce_rt) = {
        let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), effective_refresh)?;
        (Some(e), Some(s), Some(n))
      };

      let new_expires_at = new_expires_in.map(|ei| now + chrono::Duration::seconds(ei));

      let updated_row = McpOAuthTokenRow {
        id: token.id,
        mcp_oauth_config_id: token.mcp_oauth_config_id,
        encrypted_access_token: enc_at,
        access_token_salt: salt_at,
        access_token_nonce: nonce_at,
        encrypted_refresh_token: enc_rt,
        refresh_token_salt: salt_rt,
        refresh_token_nonce: nonce_rt,
        scopes_granted: token.scopes_granted,
        expires_at: new_expires_at,
        created_by: token.created_by,
        created_at: token.created_at,
        updated_at: now,
      };

      self.db_service.update_mcp_oauth_token(&updated_row).await?;

      info!(auth_uuid, "OAuth token refreshed successfully");
      return Ok(Some((
        "Authorization".to_string(),
        format!("Bearer {}", new_access_token),
      )));
    }

    debug!(auth_uuid, "OAuth token not expired, using cached token");
    Ok(
      self
        .db_service
        .get_decrypted_oauth_bearer(auth_uuid)
        .await?,
    )
  }

  async fn resolve_auth_header_for_mcp(
    &self,
    mcp_row: &McpRow,
  ) -> Result<Option<(String, String)>, McpError> {
    let user_id = &mcp_row.created_by;

    match mcp_row.auth_type {
      McpAuthType::Header => {
        if let Some(ref auth_uuid) = mcp_row.auth_uuid {
          return Ok(self.db_service.get_decrypted_auth_header(auth_uuid).await?);
        }
      }
      McpAuthType::Oauth => {
        if let Some(ref auth_uuid) = mcp_row.auth_uuid {
          return self.resolve_oauth_token(user_id, auth_uuid).await;
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

    let now = self.time_service.utc_now();
    let row = McpServerRow {
      id: Ulid::new().to_string(),
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

    let now = self.time_service.utc_now();
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

    let now = self.time_service.utc_now();
    let row = McpRow {
      id: Ulid::new().to_string(),
      created_by: user_id.to_string(),
      mcp_server_id: mcp_server.id.clone(),
      name: name.to_string(),
      slug: slug.to_string(),
      description,
      enabled,
      tools_cache: tools_cache_json,
      tools_filter: tools_filter_json,
      auth_type,
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
      if existing.auth_type != new_auth_type {
        if let Some(ref old_uuid) = existing.auth_uuid {
          match existing.auth_type {
            // Auth headers are admin-managed resources - don't delete them when
            // an MCP instance stops using them. They can be reused by other instances.
            McpAuthType::Header => {}
            McpAuthType::Oauth => {
              let _ = self
                .db_service
                .delete_mcp_oauth_token(user_id, old_uuid)
                .await;
            }
            McpAuthType::Public => {}
          }
        }
      }
      (new_auth_type, auth_uuid)
    } else {
      (existing.auth_type, existing.auth_uuid)
    };

    let now = self.time_service.utc_now();
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

    if let Some(ref auth_uuid) = existing.auth_uuid {
      match existing.auth_type {
        // Auth headers are admin-managed shared resources - don't delete them
        // when an MCP instance is deleted. They can be reused by other instances.
        McpAuthType::Header => {}
        McpAuthType::Oauth => {
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

    let now = self.time_service.utc_now();
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
    name: &str,
    mcp_server_id: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError> {
    let (encrypted, salt, nonce) = encrypt_api_key(self.db_service.encryption_key(), header_value)?;

    let now = self.time_service.utc_now();
    let row = McpAuthHeaderRow {
      id: Ulid::new().to_string(),
      name: name.to_string(),
      mcp_server_id: mcp_server_id.to_string(),
      header_key: header_key.to_string(),
      encrypted_header_value: encrypted,
      header_value_salt: salt,
      header_value_nonce: nonce,
      created_by: user_id.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_auth_header(&row).await?;
    Ok(McpAuthHeader {
      id: result.id,
      name: result.name,
      mcp_server_id: result.mcp_server_id,
      header_key: result.header_key,
      has_header_value: true,
      created_by: result.created_by,
      created_at: result.created_at,
      updated_at: result.updated_at,
    })
  }

  async fn get_auth_header(&self, id: &str) -> Result<Option<McpAuthHeader>, McpError> {
    Ok(self.db_service.get_mcp_auth_header(id).await?)
  }

  async fn list_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeader>, McpError> {
    Ok(
      self
        .db_service
        .list_mcp_auth_headers_by_server(mcp_server_id)
        .await?,
    )
  }

  async fn update_auth_header(
    &self,
    id: &str,
    name: &str,
    header_key: &str,
    header_value: &str,
  ) -> Result<McpAuthHeader, McpError> {
    let existing = self
      .db_service
      .get_mcp_auth_header(id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    let (encrypted, salt, nonce) = encrypt_api_key(self.db_service.encryption_key(), header_value)?;

    let now = self.time_service.utc_now();
    let row = McpAuthHeaderRow {
      id: existing.id,
      name: name.to_string(),
      mcp_server_id: existing.mcp_server_id,
      header_key: header_key.to_string(),
      encrypted_header_value: encrypted,
      header_value_salt: salt,
      header_value_nonce: nonce,
      created_by: existing.created_by,
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp_auth_header(&row).await?;
    Ok(McpAuthHeader {
      id: result.id,
      name: result.name,
      mcp_server_id: result.mcp_server_id,
      header_key: result.header_key,
      has_header_value: true,
      created_by: result.created_by,
      created_at: result.created_at,
      updated_at: result.updated_at,
    })
  }

  async fn delete_auth_header(&self, id: &str) -> Result<(), McpError> {
    self.db_service.delete_mcp_auth_header(id).await?;
    Ok(())
  }

  // ---- MCP OAuth config operations ----

  async fn create_oauth_config(
    &self,
    user_id: &str,
    name: &str,
    mcp_server_id: &str,
    client_id: &str,
    client_secret: Option<String>,
    authorization_endpoint: &str,
    token_endpoint: &str,
    scopes: Option<String>,
    registration_type: RegistrationType,
    registration_endpoint: Option<String>,
    token_endpoint_auth_method: Option<String>,
    client_id_issued_at: Option<i64>,
    registration_access_token: Option<String>,
  ) -> Result<McpOAuthConfig, McpError> {
    let (enc_secret, salt_secret, nonce_secret) = if let Some(ref secret) = client_secret {
      let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), secret)?;
      (Some(e), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let (enc_rat, salt_rat, nonce_rat) = if let Some(ref rat) = registration_access_token {
      let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), rat)?;
      (Some(e), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let now = self.time_service.utc_now();
    let row = McpOAuthConfigRow {
      id: Ulid::new().to_string(),
      name: name.to_string(),
      mcp_server_id: mcp_server_id.to_string(),
      registration_type,
      client_id: client_id.to_string(),
      encrypted_client_secret: enc_secret,
      client_secret_salt: salt_secret,
      client_secret_nonce: nonce_secret,
      authorization_endpoint: authorization_endpoint.to_string(),
      token_endpoint: token_endpoint.to_string(),
      registration_endpoint,
      encrypted_registration_access_token: enc_rat,
      registration_access_token_salt: salt_rat,
      registration_access_token_nonce: nonce_rat,
      client_id_issued_at: client_id_issued_at
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
      token_endpoint_auth_method,
      scopes,
      created_by: user_id.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_oauth_config(&row).await?;
    Ok(McpOAuthConfig {
      id: result.id,
      name: result.name,
      mcp_server_id: result.mcp_server_id,
      registration_type: result.registration_type,
      client_id: result.client_id,
      authorization_endpoint: result.authorization_endpoint,
      token_endpoint: result.token_endpoint,
      registration_endpoint: result.registration_endpoint,
      client_id_issued_at: result.client_id_issued_at.map(|dt| dt.timestamp()),
      token_endpoint_auth_method: result.token_endpoint_auth_method,
      scopes: result.scopes,
      has_client_secret: result.encrypted_client_secret.is_some(),
      has_registration_access_token: result.encrypted_registration_access_token.is_some(),
      created_by: result.created_by,
      created_at: result.created_at,
      updated_at: result.updated_at,
    })
  }

  async fn get_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfig>, McpError> {
    Ok(self.db_service.get_mcp_oauth_config(id).await?)
  }

  async fn list_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpOAuthConfig>, McpError> {
    Ok(
      self
        .db_service
        .list_mcp_oauth_configs_by_server(mcp_server_id)
        .await?,
    )
  }

  async fn delete_oauth_config(&self, id: &str) -> Result<(), McpError> {
    self.db_service.delete_oauth_config_cascade(id).await?;
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
    // Delete existing tokens for this (config_id, user_id) to prevent orphaned rows
    self
      .db_service
      .delete_oauth_tokens_by_config_and_user(config_id, user_id)
      .await?;

    let (enc_at, salt_at, nonce_at) =
      encrypt_api_key(self.db_service.encryption_key(), access_token)?;

    let (enc_rt, salt_rt, nonce_rt) = if let Some(ref rt) = refresh_token {
      let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), rt)?;
      (Some(e), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let now = self.time_service.utc_now();
    let expires_at = expires_in.map(|ei| now + chrono::Duration::seconds(ei));

    let row = McpOAuthTokenRow {
      id: Ulid::new().to_string(),
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
    Ok(McpOAuthToken {
      id: result.id,
      mcp_oauth_config_id: result.mcp_oauth_config_id,
      scopes_granted: result.scopes_granted,
      expires_at: result.expires_at.map(|dt| dt.timestamp()),
      has_access_token: true,
      has_refresh_token: result.encrypted_refresh_token.is_some(),
      created_by: result.created_by,
      created_at: result.created_at,
      updated_at: result.updated_at,
    })
  }

  async fn get_oauth_token(
    &self,
    user_id: &str,
    token_id: &str,
  ) -> Result<Option<McpOAuthToken>, McpError> {
    Ok(
      self
        .db_service
        .get_mcp_oauth_token(user_id, token_id)
        .await?,
    )
  }

  async fn exchange_oauth_token(
    &self,
    user_id: &str,
    config_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
  ) -> Result<McpOAuthToken, McpError> {
    let client_creds = self
      .db_service
      .get_decrypted_client_secret(config_id)
      .await?;

    let config = self
      .db_service
      .get_mcp_oauth_config(config_id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(config_id.to_string()))?;

    let mcp_server_url = self.get_mcp_server_url(&config.mcp_server_id).await?;

    let mut form_params = vec![
      ("grant_type".to_string(), "authorization_code".to_string()),
      ("code".to_string(), code.to_string()),
      ("redirect_uri".to_string(), redirect_uri.to_string()),
      ("code_verifier".to_string(), code_verifier.to_string()),
      ("resource".to_string(), mcp_server_url),
    ];

    if let Some((client_id, client_secret)) = client_creds {
      form_params.push(("client_id".to_string(), client_id));
      form_params.push(("client_secret".to_string(), client_secret));
    } else {
      form_params.push(("client_id".to_string(), config.client_id.clone()));
    }

    debug!(
      config_id,
      token_endpoint = config.token_endpoint,
      "Sending OAuth token exchange request"
    );
    let resp = self
      .http_client
      .post(&config.token_endpoint)
      .header("Accept", "application/json")
      .form(&form_params)
      .send()
      .await
      .map_err(|e| McpError::OAuthTokenExchangeFailed(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
      let body = resp.text().await.unwrap_or_default();
      warn!(
        config_id,
        status = status.as_u16(),
        "OAuth token exchange failed"
      );
      return Err(McpError::OAuthTokenExchangeFailed(format!(
        "HTTP {}: {}",
        status, body
      )));
    }

    let body = resp.text().await.unwrap_or_default();
    let token_resp: serde_json::Value = serde_json::from_str(&body)
      .map_err(|e| McpError::OAuthTokenExchangeFailed(format!("invalid token response: {}", e)))?;

    let access_token = token_resp["access_token"]
      .as_str()
      .ok_or_else(|| {
        McpError::OAuthTokenExchangeFailed("missing access_token in token response".to_string())
      })?
      .to_string();

    let refresh_token = token_resp["refresh_token"].as_str().map(|s| s.to_string());
    let expires_in = token_resp["expires_in"].as_i64();
    let scopes_granted = token_resp["scope"].as_str().map(|s| s.to_string());

    info!(config_id, "OAuth token exchange successful");
    self
      .store_oauth_token(
        user_id,
        config_id,
        &access_token,
        refresh_token,
        scopes_granted,
        expires_in,
      )
      .await
  }

  // ---- OAuth discovery ----

  async fn discover_oauth_metadata(&self, url: &str) -> Result<serde_json::Value, McpError> {
    let discovery_url = format!(
      "{}/.well-known/oauth-authorization-server",
      url.trim_end_matches('/')
    );
    debug!(url, discovery_url, "Starting OAuth metadata discovery");
    let resp = self
      .http_client
      .get(&discovery_url)
      .send()
      .await
      .map_err(|e| {
        warn!(url, error = %e, "OAuth metadata discovery request failed");
        McpError::OAuthDiscoveryFailed(e.to_string())
      })?;

    let status = resp.status();
    if !status.is_success() {
      let body = resp.text().await.unwrap_or_default();
      warn!(
        url,
        status = status.as_u16(),
        "OAuth metadata discovery failed"
      );
      return Err(McpError::OAuthDiscoveryFailed(format!(
        "HTTP {}: {}",
        status.as_u16(),
        body
      )));
    }

    let result = resp
      .json()
      .await
      .map_err(|e| McpError::OAuthDiscoveryFailed(e.to_string()))?;
    info!(url, "OAuth metadata discovery successful");
    Ok(result)
  }

  async fn discover_mcp_oauth_metadata(
    &self,
    mcp_server_url: &str,
  ) -> Result<serde_json::Value, McpError> {
    debug!(mcp_server_url, "Starting MCP OAuth metadata discovery");
    let origin = url::Url::parse(mcp_server_url)
      .map_err(|e| McpError::OAuthDiscoveryFailed(format!("Invalid MCP server URL: {}", e)))
      .map(|u| {
        format!(
          "{}://{}{}",
          u.scheme(),
          u.host_str().unwrap_or(""),
          u.port().map(|p| format!(":{}", p)).unwrap_or_default()
        )
      })?;

    let prs_url = format!("{}/.well-known/oauth-protected-resource", origin);
    debug!(prs_url, "Fetching Protected Resource Metadata");
    let prs_resp = self.http_client.get(&prs_url).send().await.map_err(|e| {
      warn!(mcp_server_url, error = %e, "Protected Resource Metadata fetch failed");
      McpError::OAuthDiscoveryFailed(format!("Protected Resource Metadata fetch failed: {}", e))
    })?;

    if !prs_resp.status().is_success() {
      warn!(
        mcp_server_url,
        status = prs_resp.status().as_u16(),
        "Protected Resource Metadata not available"
      );
      return Err(McpError::OAuthDiscoveryFailed(format!(
        "Protected Resource Metadata not available (HTTP {})",
        prs_resp.status()
      )));
    }

    let prs_meta: serde_json::Value = prs_resp.json().await.map_err(|e| {
      McpError::OAuthDiscoveryFailed(format!("Invalid Protected Resource Metadata: {}", e))
    })?;

    let as_url = prs_meta["authorization_servers"]
      .as_array()
      .and_then(|arr| arr.first())
      .and_then(|v| v.as_str())
      .ok_or_else(|| {
        McpError::OAuthDiscoveryFailed(
          "Protected Resource Metadata missing authorization_servers".to_string(),
        )
      })?
      .trim_end_matches('/');

    let resource = prs_meta["resource"].as_str().map(|s| s.to_string());

    let as_meta_url = format!("{}/.well-known/oauth-authorization-server", as_url);
    debug!(as_meta_url, "Fetching Authorization Server Metadata");
    let as_resp = self
      .http_client
      .get(&as_meta_url)
      .send()
      .await
      .map_err(|e| {
        warn!(mcp_server_url, error = %e, "AS Metadata fetch failed");
        McpError::OAuthDiscoveryFailed(format!("AS Metadata fetch failed: {}", e))
      })?;

    if !as_resp.status().is_success() {
      warn!(
        mcp_server_url,
        status = as_resp.status().as_u16(),
        "AS Metadata not available"
      );
      return Err(McpError::OAuthDiscoveryFailed(format!(
        "AS Metadata not available (HTTP {})",
        as_resp.status()
      )));
    }

    let as_meta: serde_json::Value = as_resp
      .json()
      .await
      .map_err(|e| McpError::OAuthDiscoveryFailed(format!("Invalid AS Metadata: {}", e)))?;

    let mut result = as_meta;
    if let Some(r) = resource {
      result["resource"] = serde_json::Value::String(r);
    }
    result["authorization_server_url"] = serde_json::Value::String(as_url.to_string());

    info!(mcp_server_url, "MCP OAuth metadata discovery successful");
    Ok(result)
  }

  async fn dynamic_register_client(
    &self,
    registration_endpoint: &str,
    redirect_uri: &str,
    scopes: Option<String>,
  ) -> Result<serde_json::Value, McpError> {
    debug!(
      registration_endpoint,
      redirect_uri, "Starting dynamic client registration"
    );
    let mut body = serde_json::json!({
      "client_name": "BodhiApp",
      "redirect_uris": [redirect_uri],
      "grant_types": ["authorization_code", "refresh_token"],
      "response_types": ["code"],
      "token_endpoint_auth_method": "none"
    });

    if let Some(ref s) = scopes {
      body["scope"] = serde_json::Value::String(s.clone());
    }

    let resp = self
      .http_client
      .post(registration_endpoint)
      .json(&body)
      .send()
      .await
      .map_err(|e| {
        warn!(registration_endpoint, error = %e, "Dynamic client registration request failed");
        McpError::OAuthDiscoveryFailed(format!("Dynamic registration request failed: {}", e))
      })?;

    if !resp.status().is_success() {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      warn!(
        registration_endpoint,
        status = status.as_u16(),
        "Dynamic client registration failed"
      );
      return Err(McpError::OAuthDiscoveryFailed(format!(
        "Dynamic registration failed (HTTP {}): {}",
        status, body
      )));
    }

    let result = resp.json().await.map_err(|e| {
      McpError::OAuthDiscoveryFailed(format!("Invalid registration response: {}", e))
    })?;
    info!(
      registration_endpoint,
      "Dynamic client registration successful"
    );
    Ok(result)
  }

  // ---- Unified auth config operations ----

  async fn create_auth_config(
    &self,
    user_id: &str,
    mcp_server_id: &str,
    request: CreateMcpAuthConfigRequest,
  ) -> Result<McpAuthConfigResponse, McpError> {
    match request {
      CreateMcpAuthConfigRequest::Header {
        name,
        header_key,
        header_value,
      } => {
        let header = self
          .create_auth_header(user_id, &name, mcp_server_id, &header_key, &header_value)
          .await?;
        Ok(McpAuthConfigResponse::from(header))
      }
      CreateMcpAuthConfigRequest::Oauth {
        name,
        client_id,
        authorization_endpoint,
        token_endpoint,
        client_secret,
        scopes,
        registration_type,
        registration_access_token,
        registration_endpoint,
        token_endpoint_auth_method,
        client_id_issued_at,
      } => {
        let config = self
          .create_oauth_config(
            user_id,
            &name,
            mcp_server_id,
            &client_id,
            client_secret,
            &authorization_endpoint,
            &token_endpoint,
            scopes,
            registration_type,
            registration_endpoint,
            token_endpoint_auth_method,
            client_id_issued_at,
            registration_access_token,
          )
          .await?;
        Ok(McpAuthConfigResponse::from(config))
      }
    }
  }

  async fn list_auth_configs(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigResponse>, McpError> {
    let headers = self.list_auth_headers_by_server(mcp_server_id).await?;
    let oauth_configs = self.list_oauth_configs_by_server(mcp_server_id).await?;
    let mut configs: Vec<McpAuthConfigResponse> = Vec::new();
    configs.extend(headers.into_iter().map(McpAuthConfigResponse::from));
    configs.extend(oauth_configs.into_iter().map(McpAuthConfigResponse::from));
    Ok(configs)
  }

  async fn get_auth_config(&self, id: &str) -> Result<Option<McpAuthConfigResponse>, McpError> {
    if let Some(header) = self.get_auth_header(id).await? {
      return Ok(Some(McpAuthConfigResponse::from(header)));
    }
    if let Some(oauth_config) = self.get_oauth_config(id).await? {
      return Ok(Some(McpAuthConfigResponse::from(oauth_config)));
    }
    Ok(None)
  }

  async fn delete_auth_config(&self, id: &str) -> Result<(), McpError> {
    // Try auth header first
    if self.get_auth_header(id).await?.is_some() {
      return self.delete_auth_header(id).await;
    }
    // Try OAuth config
    if self.get_oauth_config(id).await?.is_some() {
      return self.delete_oauth_config(id).await;
    }
    Err(McpError::McpNotFound(id.to_string()))
  }
}

#[cfg(test)]
#[path = "test_mcp_service.rs"]
mod test_mcp_service;
