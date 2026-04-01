use super::{
  CreateMcpAuthConfigRequest, McpAuthConfigParam, McpAuthConfigResponse, McpAuthConfigType,
  McpAuthParamType, McpAuthType, McpOAuthConfig, McpOAuthToken, McpRequest, McpServerRequest,
  RegistrationType,
};
use super::{
  McpAuthConfigEntity, McpAuthConfigParamEntity, McpAuthParamEntity, McpEntity,
  McpOAuthConfigDetailEntity, McpOAuthTokenEntity, McpServerEntity, McpWithServerEntity,
};
use super::{McpError, McpServerError};
use crate::db::{encryption::encrypt_api_key, DbService, TimeService};
use crate::new_ulid;
use crate::SafeReqwest;
use mcp_client::{McpAuthParams, McpClient};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
#[allow(clippy::too_many_arguments)]
pub trait McpService: Debug + Send + Sync {
  // ---- MCP Server admin operations ----

  async fn create_mcp_server(
    &self,
    tenant_id: &str,
    created_by: &str,
    request: McpServerRequest,
  ) -> Result<McpServerEntity, McpServerError>;

  async fn update_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
    updated_by: &str,
    request: McpServerRequest,
  ) -> Result<McpServerEntity, McpServerError>;

  async fn get_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpServerEntity>, McpServerError>;

  async fn list_mcp_servers(
    &self,
    tenant_id: &str,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServerEntity>, McpServerError>;

  async fn count_mcps_for_server(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<(i64, i64), McpServerError>;

  // ---- MCP user instance operations ----

  async fn list(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpWithServerEntity>, McpError>;

  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpWithServerEntity>, McpError>;

  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    request: McpRequest,
  ) -> Result<McpWithServerEntity, McpError>;

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    request: McpRequest,
  ) -> Result<McpWithServerEntity, McpError>;

  async fn delete(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), McpError>;

  // ---- MCP OAuth config operations ----

  async fn create_oauth_config(
    &self,
    tenant_id: &str,
    created_by: &str,
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

  async fn get_oauth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthConfig>, McpError>;

  // ---- MCP OAuth token operations ----

  async fn store_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    mcp_id: Option<String>,
    config_id: &str,
    access_token: &str,
    refresh_token: Option<String>,
    scopes_granted: Option<String>,
    expires_in: Option<i64>,
  ) -> Result<McpOAuthToken, McpError>;

  async fn get_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token_id: &str,
  ) -> Result<Option<McpOAuthToken>, McpError>;

  async fn delete_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token_id: &str,
  ) -> Result<(), McpError>;

  /// Exchange an authorization code for tokens via the OAuth token endpoint.
  async fn exchange_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    mcp_id: Option<String>,
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
    tenant_id: &str,
    created_by: &str,
    mcp_server_id: &str,
    request: CreateMcpAuthConfigRequest,
  ) -> Result<McpAuthConfigResponse, McpError>;

  async fn list_auth_configs(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigResponse>, McpError>;

  async fn get_auth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthConfigResponse>, McpError>;

  async fn delete_auth_config(&self, tenant_id: &str, id: &str) -> Result<(), McpError>;

  // ---- Auth params resolution ----

  /// Resolve authentication parameters (headers, query params) for an MCP instance.
  /// Used by proxy and execution endpoints to inject auth into upstream requests.
  async fn resolve_auth_params(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthParams>, McpError>;
}

/// Maximum number of concurrent refresh lock entries.
const MAX_REFRESH_LOCKS: usize = 1000;

pub struct DefaultMcpService {
  db_service: Arc<dyn DbService>,
  mcp_client: Arc<dyn McpClient>,
  time_service: Arc<dyn TimeService>,
  http_client: SafeReqwest,
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
  ) -> Result<Self, McpError> {
    Ok(Self {
      db_service,
      mcp_client,
      time_service,
      http_client: SafeReqwest::builder().allow_private_ips().build()?,
      refresh_locks: RwLock::new(HashMap::new()),
    })
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

  fn mcp_row_to_with_server(
    &self,
    row: McpEntity,
    server: &McpServerEntity,
  ) -> McpWithServerEntity {
    McpWithServerEntity {
      id: row.id,
      user_id: row.user_id,
      mcp_server_id: row.mcp_server_id,
      name: row.name,
      slug: row.slug,
      description: row.description,
      enabled: row.enabled,
      auth_type: row.auth_type,
      auth_config_id: row.auth_config_id,
      created_at: row.created_at,
      updated_at: row.updated_at,
      server_url: server.url.clone(),
      server_name: server.name.clone(),
      server_enabled: server.enabled,
    }
  }

  async fn get_mcp_with_server(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<(McpEntity, McpServerEntity)>, McpError> {
    let row = self.db_service.get_mcp(tenant_id, user_id, id).await?;
    match row {
      Some(mcp_row) => {
        let server = self
          .db_service
          .get_mcp_server(tenant_id, &mcp_row.mcp_server_id)
          .await?;
        match server {
          Some(s) => Ok(Some((mcp_row, s))),
          None => Ok(None),
        }
      }
      None => Ok(None),
    }
  }

  async fn get_mcp_server_url(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<String, McpError> {
    let server = self
      .db_service
      .get_mcp_server(tenant_id, mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(mcp_server_id.to_string()))?;
    Ok(server.url)
  }

  async fn resolve_oauth_token(
    &self,
    tenant_id: &str,
    mcp_id: &str,
  ) -> Result<Option<McpAuthParams>, McpError> {
    let lock = self
      .get_refresh_lock(&format!("oauth_refresh:{}", mcp_id))
      .await;
    let _guard = lock.lock().await;

    let token = match self
      .db_service
      .get_latest_oauth_token_by_mcp(tenant_id, mcp_id)
      .await?
    {
      Some(t) => t,
      None => return Err(McpError::OAuthTokenNotFound(mcp_id.to_string())),
    };

    let now = self.time_service.utc_now();
    let is_expired = token
      .expires_at
      .map(|exp_ts| now.timestamp() >= exp_ts - 60)
      .unwrap_or(false);

    if is_expired {
      if !token.has_refresh_token {
        warn!(
          mcp_id,
          "OAuth token expired with no refresh token available"
        );
        return Err(McpError::OAuthTokenExpired(mcp_id.to_string()));
      }

      debug!(mcp_id, "OAuth token expired, attempting refresh");
      let refresh_token = self
        .db_service
        .get_decrypted_refresh_token(tenant_id, &token.id)
        .await?
        .ok_or_else(|| McpError::OAuthTokenExpired(mcp_id.to_string()))?;

      let config = self
        .db_service
        .get_mcp_oauth_config_detail(tenant_id, &token.auth_config_id)
        .await?
        .ok_or_else(|| McpError::McpNotFound(token.auth_config_id.clone()))?;

      let client_creds = self
        .db_service
        .get_decrypted_client_secret(tenant_id, &token.auth_config_id)
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

      let mcp_server_url = self
        .get_mcp_server_url(tenant_id, &config.mcp_server_id)
        .await?;
      form_params.push(("resource".to_string(), mcp_server_url));

      debug!(
        mcp_id,
        token_endpoint = config.token_endpoint,
        "Sending token refresh request"
      );
      let resp = self
        .http_client
        .post(&config.token_endpoint)?
        .form(&form_params)
        .send()
        .await
        .map_err(|e| McpError::OAuthRefreshFailed(e.to_string()))?;

      if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        warn!(
          mcp_id,
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

      // Use new refresh token if provided, otherwise re-encrypt the old one
      let effective_refresh = new_refresh.as_deref().unwrap_or(&refresh_token);
      let (enc_rt, salt_rt, nonce_rt) = {
        let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), effective_refresh)?;
        (Some(e), Some(s), Some(n))
      };

      let new_expires_at = new_expires_in.map(|ei| now + chrono::Duration::seconds(ei));

      let (enc_at, salt_at, nonce_at) =
        encrypt_api_key(self.db_service.encryption_key(), &new_access_token)?;

      let updated_row = McpOAuthTokenEntity {
        id: token.id,
        tenant_id: tenant_id.to_string(),
        mcp_id: Some(mcp_id.to_string()),
        auth_config_id: token.auth_config_id,
        user_id: token.user_id,
        encrypted_access_token: enc_at,
        access_token_salt: salt_at,
        access_token_nonce: nonce_at,
        encrypted_refresh_token: enc_rt,
        refresh_token_salt: salt_rt,
        refresh_token_nonce: nonce_rt,
        scopes_granted: token.scopes_granted,
        expires_at: new_expires_at,
        created_at: token.created_at,
        updated_at: now,
      };

      self.db_service.update_mcp_oauth_token(&updated_row).await?;

      info!(mcp_id, "OAuth token refreshed successfully");
      return Ok(Some(McpAuthParams {
        headers: vec![(
          "Authorization".to_string(),
          format!("Bearer {}", new_access_token),
        )],
        query_params: vec![],
      }));
    }

    debug!(mcp_id, "OAuth token not expired, using cached credentials");
    // Read access token from mcp_oauth_tokens
    let access_token = self
      .db_service
      .get_decrypted_oauth_access_token(tenant_id, &token.id)
      .await?
      .ok_or_else(|| McpError::OAuthTokenNotFound(mcp_id.to_string()))?;
    Ok(Some(McpAuthParams {
      headers: vec![(
        "Authorization".to_string(),
        format!("Bearer {}", access_token),
      )],
      query_params: vec![],
    }))
  }

  async fn resolve_auth_params_for_mcp(
    &self,
    tenant_id: &str,
    mcp_row: &McpEntity,
  ) -> Result<Option<McpAuthParams>, McpError> {
    match mcp_row.auth_type {
      McpAuthType::Header | McpAuthType::Public => {
        // Read credentials from mcp_auth_params table
        Ok(
          self
            .db_service
            .get_decrypted_auth_params(tenant_id, &mcp_row.id)
            .await?,
        )
      }
      McpAuthType::Oauth => self.resolve_oauth_token(tenant_id, &mcp_row.id).await,
    }
  }
}

#[async_trait::async_trait]
impl McpService for DefaultMcpService {
  // ---- MCP Server admin operations ----

  async fn create_mcp_server(
    &self,
    tenant_id: &str,
    created_by: &str,
    request: McpServerRequest,
  ) -> Result<McpServerEntity, McpServerError> {
    let trimmed_url = request.url.trim();

    if let Some(existing) = self
      .db_service
      .get_mcp_server_by_url(tenant_id, trimmed_url)
      .await?
    {
      return Err(McpServerError::UrlAlreadyExists(existing.url));
    }

    let now = self.time_service.utc_now();
    let row = McpServerEntity {
      id: new_ulid(),
      tenant_id: tenant_id.to_string(),
      url: trimmed_url.to_string(),
      name: request.name,
      description: request.description,
      enabled: request.enabled,
      created_by: created_by.to_string(),
      updated_by: created_by.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_server(tenant_id, &row).await?;
    Ok(result)
  }

  async fn update_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
    updated_by: &str,
    request: McpServerRequest,
  ) -> Result<McpServerEntity, McpServerError> {
    let trimmed_url = request.url.trim();

    let existing = self
      .db_service
      .get_mcp_server(tenant_id, id)
      .await?
      .ok_or_else(|| McpServerError::McpServerNotFound(id.to_string()))?;

    if let Some(dup) = self
      .db_service
      .get_mcp_server_by_url(tenant_id, trimmed_url)
      .await?
    {
      if dup.id != existing.id {
        return Err(McpServerError::UrlAlreadyExists(dup.url));
      }
    }

    let now = self.time_service.utc_now();
    let row = McpServerEntity {
      id: existing.id,
      tenant_id: tenant_id.to_string(),
      url: trimmed_url.to_string(),
      name: request.name,
      description: request.description,
      enabled: request.enabled,
      created_by: existing.created_by,
      updated_by: updated_by.to_string(),
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp_server(tenant_id, &row).await?;
    Ok(result)
  }

  async fn get_mcp_server(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpServerEntity>, McpServerError> {
    Ok(self.db_service.get_mcp_server(tenant_id, id).await?)
  }

  async fn list_mcp_servers(
    &self,
    tenant_id: &str,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServerEntity>, McpServerError> {
    Ok(self.db_service.list_mcp_servers(tenant_id, enabled).await?)
  }

  async fn count_mcps_for_server(
    &self,
    tenant_id: &str,
    server_id: &str,
  ) -> Result<(i64, i64), McpServerError> {
    Ok(
      self
        .db_service
        .count_mcps_by_server_id(tenant_id, server_id)
        .await?,
    )
  }

  // ---- MCP user instance operations ----

  async fn list(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<McpWithServerEntity>, McpError> {
    Ok(
      self
        .db_service
        .list_mcps_with_server(tenant_id, user_id)
        .await?,
    )
  }

  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpWithServerEntity>, McpError> {
    match self.get_mcp_with_server(tenant_id, user_id, id).await? {
      Some((row, server)) => Ok(Some(self.mcp_row_to_with_server(row, &server))),
      None => Ok(None),
    }
  }

  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    request: McpRequest,
  ) -> Result<McpWithServerEntity, McpError> {
    let mcp_server_id = request
      .mcp_server_id
      .as_deref()
      .ok_or_else(|| McpError::McpServerNotFound("mcp_server_id is required".to_string()))?;

    let mcp_server = self
      .db_service
      .get_mcp_server(tenant_id, mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(mcp_server_id.to_string()))?;

    if !mcp_server.enabled {
      return Err(McpError::McpDisabled);
    }

    if self
      .db_service
      .get_mcp_by_slug(tenant_id, user_id, &request.slug)
      .await?
      .is_some()
    {
      return Err(McpError::SlugExists(request.slug.clone()));
    }

    let now = self.time_service.utc_now();
    let mcp_id = new_ulid();
    let row = McpEntity {
      id: mcp_id.clone(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      mcp_server_id: mcp_server.id.clone(),
      name: request.name,
      slug: request.slug,
      description: request.description,
      enabled: request.enabled,
      auth_type: request.auth_type,
      auth_config_id: request.auth_config_id,
      created_at: now,
      updated_at: now,
    };

    // Pre-encrypt credentials if provided
    let auth_params = if let Some(ref credentials) = request.credentials {
      if credentials.is_empty() {
        None
      } else {
        let mut params = Vec::new();
        for cred in credentials {
          let (encrypted, salt, nonce) =
            encrypt_api_key(self.db_service.encryption_key(), &cred.value)?;
          params.push(McpAuthParamEntity {
            id: new_ulid(),
            tenant_id: tenant_id.to_string(),
            mcp_id: mcp_id.clone(),
            param_type: cred.param_type.as_str().to_string(),
            param_key: cred.param_key.clone(),
            encrypted_value: encrypted,
            value_salt: salt,
            value_nonce: nonce,
            created_at: now,
            updated_at: now,
          });
        }
        Some(params)
      }
    } else {
      None
    };

    let result = self
      .db_service
      .create_mcp_with_auth(
        tenant_id,
        &row,
        auth_params,
        request.oauth_token_id,
        user_id,
      )
      .await?;

    Ok(self.mcp_row_to_with_server(result, &mcp_server))
  }

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    request: McpRequest,
  ) -> Result<McpWithServerEntity, McpError> {
    let (existing, server) = self
      .get_mcp_with_server(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if request.slug.to_lowercase() != existing.slug.to_lowercase()
      && self
        .db_service
        .get_mcp_by_slug(tenant_id, user_id, &request.slug)
        .await?
        .is_some()
    {
      return Err(McpError::SlugExists(request.slug.clone()));
    }

    let new_auth_type = request.auth_type;
    let (resolved_auth_type, resolved_auth_config_id) = if existing.auth_type != new_auth_type {
      if existing.auth_type == McpAuthType::Oauth {
        // Clean up OAuth tokens when switching away from OAuth
        let _ = self
          .db_service
          .delete_oauth_tokens_by_mcp(tenant_id, id)
          .await;
      }
      (new_auth_type, request.auth_config_id)
    } else {
      let config_id = request.auth_config_id.or(existing.auth_config_id);
      (existing.auth_type, config_id)
    };

    let now = self.time_service.utc_now();
    let row = McpEntity {
      id: id.to_string(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      mcp_server_id: existing.mcp_server_id,
      name: request.name,
      slug: request.slug,
      description: request.description,
      enabled: request.enabled,
      auth_type: resolved_auth_type,
      auth_config_id: resolved_auth_config_id,
      created_at: existing.created_at,
      updated_at: now,
    };

    // Pre-encrypt credentials if provided
    let auth_params = if let Some(ref credentials) = request.credentials {
      let mut params = Vec::new();
      for cred in credentials {
        let (encrypted, salt, nonce) =
          encrypt_api_key(self.db_service.encryption_key(), &cred.value)?;
        params.push(McpAuthParamEntity {
          id: new_ulid(),
          tenant_id: tenant_id.to_string(),
          mcp_id: id.to_string(),
          param_type: cred.param_type.as_str().to_string(),
          param_key: cred.param_key.clone(),
          encrypted_value: encrypted,
          value_salt: salt,
          value_nonce: nonce,
          created_at: now,
          updated_at: now,
        });
      }
      Some(params)
    } else {
      None
    };

    let result = self
      .db_service
      .update_mcp_with_auth(
        tenant_id,
        &row,
        auth_params,
        request.oauth_token_id,
        user_id,
      )
      .await?;

    Ok(self.mcp_row_to_with_server(result, &server))
  }

  async fn delete(&self, tenant_id: &str, user_id: &str, id: &str) -> Result<(), McpError> {
    let (existing, _) = self
      .get_mcp_with_server(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if existing.auth_type == McpAuthType::Oauth {
      let _ = self
        .db_service
        .delete_oauth_tokens_by_mcp(tenant_id, id)
        .await;
    }

    // Credentials are cleaned up by CASCADE FK on mcp_auth_params.mcp_id
    self.db_service.delete_mcp(tenant_id, user_id, id).await?;
    Ok(())
  }

  // ---- MCP OAuth config operations ----

  async fn create_oauth_config(
    &self,
    tenant_id: &str,
    created_by: &str,
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
    let config_id = new_ulid();

    // Create base auth config row
    let base_row = McpAuthConfigEntity {
      id: config_id.clone(),
      tenant_id: tenant_id.to_string(),
      mcp_server_id: mcp_server_id.to_string(),
      config_type: McpAuthConfigType::Oauth.as_str().to_string(),
      name: name.to_string(),
      created_by: created_by.to_string(),
      created_at: now,
      updated_at: now,
    };
    // Build OAuth detail row
    let detail_row = McpOAuthConfigDetailEntity {
      auth_config_id: config_id.clone(),
      tenant_id: tenant_id.to_string(),
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
      scopes: scopes.clone(),
      created_at: now,
      updated_at: now,
    };

    // Atomically create config + OAuth detail
    let (_config_result, result) = self
      .db_service
      .create_auth_config_oauth(tenant_id, &base_row, &detail_row)
      .await?;

    Ok(McpOAuthConfig {
      id: config_id,
      name: name.to_string(),
      mcp_server_id: mcp_server_id.to_string(),
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
      created_at: result.created_at,
      updated_at: result.updated_at,
    })
  }

  async fn get_oauth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthConfig>, McpError> {
    Ok(
      self
        .db_service
        .get_mcp_oauth_config_detail(tenant_id, id)
        .await?,
    )
  }

  // ---- MCP OAuth token operations ----

  async fn store_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    mcp_id: Option<String>,
    config_id: &str,
    access_token: &str,
    refresh_token: Option<String>,
    scopes_granted: Option<String>,
    expires_in: Option<i64>,
  ) -> Result<McpOAuthToken, McpError> {
    let (enc_rt, salt_rt, nonce_rt) = if let Some(ref rt) = refresh_token {
      let (e, s, n) = encrypt_api_key(self.db_service.encryption_key(), rt)?;
      (Some(e), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let (enc_at, salt_at, nonce_at) =
      encrypt_api_key(self.db_service.encryption_key(), access_token)?;

    let now = self.time_service.utc_now();
    let expires_at = expires_in.map(|ei| now + chrono::Duration::seconds(ei));

    let row = McpOAuthTokenEntity {
      id: new_ulid(),
      tenant_id: tenant_id.to_string(),
      mcp_id: mcp_id.clone(),
      auth_config_id: config_id.to_string(),
      user_id: user_id.to_string(),
      encrypted_access_token: enc_at,
      access_token_salt: salt_at,
      access_token_nonce: nonce_at,
      encrypted_refresh_token: enc_rt,
      refresh_token_salt: salt_rt,
      refresh_token_nonce: nonce_rt,
      scopes_granted: scopes_granted.clone(),
      expires_at,
      created_at: now,
      updated_at: now,
    };

    // Atomically delete existing tokens for (mcp_id, user_id) and insert new one
    let result = self
      .db_service
      .store_oauth_token(tenant_id, mcp_id.clone(), user_id, &row)
      .await?;

    Ok(McpOAuthToken {
      id: result.id,
      mcp_id: result.mcp_id,
      auth_config_id: result.auth_config_id,
      scopes_granted: result.scopes_granted,
      expires_at: result.expires_at.map(|dt| dt.timestamp()),
      has_refresh_token: result.encrypted_refresh_token.is_some(),
      user_id: result.user_id,
      created_at: result.created_at,
      updated_at: result.updated_at,
    })
  }

  async fn get_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token_id: &str,
  ) -> Result<Option<McpOAuthToken>, McpError> {
    Ok(
      self
        .db_service
        .get_mcp_oauth_token(tenant_id, user_id, token_id)
        .await?,
    )
  }

  async fn delete_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token_id: &str,
  ) -> Result<(), McpError> {
    self
      .db_service
      .delete_mcp_oauth_token(tenant_id, user_id, token_id)
      .await?;
    Ok(())
  }

  async fn exchange_oauth_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    mcp_id: Option<String>,
    config_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
  ) -> Result<McpOAuthToken, McpError> {
    let client_creds = self
      .db_service
      .get_decrypted_client_secret(tenant_id, config_id)
      .await?;

    let config = self
      .db_service
      .get_mcp_oauth_config_detail(tenant_id, config_id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(config_id.to_string()))?;

    let mcp_server_url = self
      .get_mcp_server_url(tenant_id, &config.mcp_server_id)
      .await?;

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
      .post(&config.token_endpoint)?
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
        tenant_id,
        user_id,
        mcp_id,
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
      .get(&discovery_url)?
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
    let prs_resp = self.http_client.get(&prs_url)?.send().await.map_err(|e| {
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
      .get(&as_meta_url)?
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
      .post(registration_endpoint)?
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
    tenant_id: &str,
    created_by: &str,
    mcp_server_id: &str,
    request: CreateMcpAuthConfigRequest,
  ) -> Result<McpAuthConfigResponse, McpError> {
    match request {
      CreateMcpAuthConfigRequest::Header { name, entries } => {
        let now = self.time_service.utc_now();
        let config_id = new_ulid();

        // Build base auth config row
        let base_row = McpAuthConfigEntity {
          id: config_id.clone(),
          tenant_id: tenant_id.to_string(),
          mcp_server_id: mcp_server_id.to_string(),
          config_type: McpAuthConfigType::Header.as_str().to_string(),
          name: name.clone(),
          created_by: created_by.to_string(),
          created_at: now,
          updated_at: now,
        };

        // Build param entries
        let mut param_rows = Vec::new();
        let mut result_entries = Vec::new();
        for entry in entries {
          let param_id = new_ulid();
          param_rows.push(McpAuthConfigParamEntity {
            id: param_id.clone(),
            tenant_id: tenant_id.to_string(),
            auth_config_id: config_id.clone(),
            param_type: entry.param_type.as_str().to_string(),
            param_key: entry.param_key.clone(),
            created_at: now,
            updated_at: now,
          });
          result_entries.push(McpAuthConfigParam {
            id: param_id,
            param_type: entry.param_type,
            param_key: entry.param_key,
          });
        }

        // Atomically create config + params
        self
          .db_service
          .create_auth_config_header(tenant_id, &base_row, param_rows)
          .await?;

        Ok(McpAuthConfigResponse::Header {
          id: config_id,
          name,
          mcp_server_id: mcp_server_id.to_string(),
          created_by: created_by.to_string(),
          entries: result_entries,
          created_at: now,
          updated_at: now,
        })
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
            tenant_id,
            created_by,
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
        Ok(McpAuthConfigResponse::Oauth {
          id: config.id,
          name: config.name,
          mcp_server_id: config.mcp_server_id,
          created_by: created_by.to_string(),
          registration_type: config.registration_type,
          client_id: config.client_id,
          authorization_endpoint: config.authorization_endpoint,
          token_endpoint: config.token_endpoint,
          registration_endpoint: config.registration_endpoint,
          scopes: config.scopes,
          client_id_issued_at: config.client_id_issued_at,
          token_endpoint_auth_method: config.token_endpoint_auth_method,
          has_client_secret: config.has_client_secret,
          has_registration_access_token: config.has_registration_access_token,
          created_at: config.created_at,
          updated_at: config.updated_at,
        })
      }
    }
  }

  async fn list_auth_configs(
    &self,
    tenant_id: &str,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigResponse>, McpError> {
    let configs = self
      .db_service
      .list_mcp_auth_configs_by_server(tenant_id, mcp_server_id)
      .await?;

    let mut results = Vec::new();
    for config in configs {
      if let Some(resp) = self.get_auth_config(tenant_id, &config.id).await? {
        results.push(resp);
      }
    }
    Ok(results)
  }

  async fn get_auth_config(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthConfigResponse>, McpError> {
    let base = match self.db_service.get_mcp_auth_config(tenant_id, id).await? {
      Some(b) => b,
      None => return Ok(None),
    };

    match base.config_type.as_str() {
      "header" => {
        let params = self
          .db_service
          .list_mcp_auth_config_params(tenant_id, &base.id)
          .await?;
        let entries: Vec<McpAuthConfigParam> = params
          .into_iter()
          .map(|p| {
            let param_type = p
              .param_type
              .parse::<McpAuthParamType>()
              .unwrap_or(McpAuthParamType::Header);
            McpAuthConfigParam {
              id: p.id,
              param_type,
              param_key: p.param_key,
            }
          })
          .collect();
        Ok(Some(McpAuthConfigResponse::Header {
          id: base.id,
          name: base.name,
          mcp_server_id: base.mcp_server_id,
          created_by: base.created_by,
          entries,
          created_at: base.created_at,
          updated_at: base.updated_at,
        }))
      }
      "oauth" => {
        let oauth = self
          .db_service
          .get_mcp_oauth_config_detail(tenant_id, &base.id)
          .await?;
        match oauth {
          Some(config) => Ok(Some(McpAuthConfigResponse::Oauth {
            id: base.id,
            name: base.name,
            mcp_server_id: base.mcp_server_id,
            created_by: base.created_by,
            registration_type: config.registration_type,
            client_id: config.client_id,
            authorization_endpoint: config.authorization_endpoint,
            token_endpoint: config.token_endpoint,
            registration_endpoint: config.registration_endpoint,
            scopes: config.scopes,
            client_id_issued_at: config.client_id_issued_at,
            token_endpoint_auth_method: config.token_endpoint_auth_method,
            has_client_secret: config.has_client_secret,
            has_registration_access_token: config.has_registration_access_token,
            created_at: config.created_at,
            updated_at: config.updated_at,
          })),
          None => Ok(None),
        }
      }
      _ => Ok(None),
    }
  }

  async fn delete_auth_config(&self, tenant_id: &str, id: &str) -> Result<(), McpError> {
    // CASCADE FK handles children (params, details, tokens)
    self
      .db_service
      .delete_mcp_auth_config(tenant_id, id)
      .await?;
    Ok(())
  }

  async fn resolve_auth_params(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpAuthParams>, McpError> {
    let (mcp_row, _server) = self
      .get_mcp_with_server(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    self.resolve_auth_params_for_mcp(tenant_id, &mcp_row).await
  }
}

#[cfg(test)]
#[path = "test_mcp_service.rs"]
mod test_mcp_service;
