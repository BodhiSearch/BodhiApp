use crate::ai_apis::error::AiApiClientFactoryError;
use crate::db::{DbError, DbService};
use crate::models::llm_liberty_credentials_repository::LlmLibertyCredentialsRepository;
use crate::models::llm_liberty_envelope::ResolvedLlmLibertyCredentials;
use crate::SafeReqwest;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use errmeta::{AppError, ErrorType};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait LlmLibertyRefresh: Send + Sync + std::fmt::Debug {
  async fn force_refresh(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias_id: &str,
  ) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError>;
}

pub struct DefaultLlmLibertyRefresh {
  db: Arc<dyn DbService>,
  http: SafeReqwest,
}

impl std::fmt::Debug for DefaultLlmLibertyRefresh {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DefaultLlmLibertyRefresh").finish()
  }
}

impl DefaultLlmLibertyRefresh {
  pub fn new(db: Arc<dyn DbService>, http: SafeReqwest) -> Self {
    Self { db, http }
  }
}

#[async_trait]
impl LlmLibertyRefresh for DefaultLlmLibertyRefresh {
  async fn force_refresh(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias_id: &str,
  ) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError> {
    force_refresh_credentials(self.db.as_ref(), &self.http, tenant_id, user_id, alias_id).await
  }
}

/// How many seconds before actual expiry we consider the token "about to expire".
const REFRESH_SKEW_SECS: i64 = 60;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LlmLibertyRefreshError {
  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  AiApi(#[from] AiApiClientFactoryError),

  #[error("LLM Liberty credentials not found for alias '{0}'.")]
  #[error_meta(error_type = ErrorType::NotFound, code = "llm_liberty_refresh-not_found")]
  NotFound(String),

  #[error("LLM Liberty refresh response is missing access_token.")]
  #[error_meta(error_type = ErrorType::Authentication, code = "llm_liberty_refresh-missing_access_token")]
  MissingAccessToken,

  #[error("LLM Liberty refresh response is missing expires_in.")]
  #[error_meta(error_type = ErrorType::Authentication, code = "llm_liberty_refresh-missing_expires_in")]
  MissingExpiresIn,
}

// Per-alias async mutexes. A global std::sync::Mutex guards insertion into the
// map; each entry is an Arc<tokio::sync::Mutex<()>> that serializes refreshes
// for a single alias without holding the global lock during the HTTP call.
//
// NOTE: In a multi-node deployment two nodes can race on the same alias.
// The duplicate refresh is benign (both succeed; last write wins on the DB row).
// This is an accepted limitation documented in the project plan.
static REFRESH_LOCKS: Lazy<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> =
  Lazy::new(|| Mutex::new(HashMap::new()));

fn alias_lock(alias_id: &str) -> Arc<tokio::sync::Mutex<()>> {
  let mut map = REFRESH_LOCKS.lock().expect("refresh lock poisoned");
  map
    .entry(alias_id.to_string())
    .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
    .clone()
}

/// Ensure the access token for the given alias is fresh, refreshing if needed.
///
/// Acquires a per-alias mutex to serialize concurrent refresh attempts on a
/// single node. Reads the stored credentials, checks expiry against a 60-second
/// skew window, and on expiry calls the provider's `oauth.token_url` to rotate
/// tokens before persisting. Returns the (possibly fresh) credentials.
pub async fn ensure_fresh_credentials<R: LlmLibertyCredentialsRepository + ?Sized>(
  db: &R,
  http: &SafeReqwest,
  tenant_id: &str,
  user_id: &str,
  alias_id: &str,
) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError> {
  let lock = alias_lock(alias_id);
  let _guard = lock.lock().await;

  let creds = db
    .get_llm_liberty_credentials(tenant_id, user_id, alias_id)
    .await?
    .ok_or_else(|| LlmLibertyRefreshError::NotFound(alias_id.to_string()))?;

  let threshold = Utc::now() + Duration::seconds(REFRESH_SKEW_SECS);
  if creds.expires_at > threshold {
    return Ok(creds);
  }

  let refreshed = do_refresh(http, &creds).await?;

  db.update_llm_liberty_tokens(
    tenant_id,
    alias_id,
    &refreshed.access_token,
    &refreshed.refresh_token,
    refreshed.expires_at,
  )
  .await?;

  Ok(refreshed)
}

/// Force a refresh regardless of skew window. Used by the upstream-401 retry path:
/// the provider may invalidate access tokens before `expires_at` (e.g. third-party
/// usage flagging), in which case we must rotate before retrying.
pub async fn force_refresh_credentials<R: LlmLibertyCredentialsRepository + ?Sized>(
  db: &R,
  http: &SafeReqwest,
  tenant_id: &str,
  user_id: &str,
  alias_id: &str,
) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError> {
  let lock = alias_lock(alias_id);
  let _guard = lock.lock().await;

  let creds = db
    .get_llm_liberty_credentials(tenant_id, user_id, alias_id)
    .await?
    .ok_or_else(|| LlmLibertyRefreshError::NotFound(alias_id.to_string()))?;

  let refreshed = do_refresh(http, &creds).await?;

  db.update_llm_liberty_tokens(
    tenant_id,
    alias_id,
    &refreshed.access_token,
    &refreshed.refresh_token,
    refreshed.expires_at,
  )
  .await?;

  Ok(refreshed)
}

async fn do_refresh(
  http: &SafeReqwest,
  creds: &ResolvedLlmLibertyCredentials,
) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError> {
  let body = serde_json::json!({
    "grant_type": "refresh_token",
    "client_id": creds.oauth_client_id,
    "refresh_token": creds.refresh_token,
  });

  let resp = http
    .post(&creds.oauth_token_url)
    .map_err(AiApiClientFactoryError::from)?
    .header("Content-Type", "application/json")
    .json(&body)
    .send()
    .await
    .map_err(AiApiClientFactoryError::from)?;

  let status = resp.status();
  if !status.is_success() {
    let text = resp.text().await.unwrap_or_default();
    return Err(LlmLibertyRefreshError::AiApi(
      AiApiClientFactoryError::status_to_error(status, text),
    ));
  }

  let json: serde_json::Value = resp.json().await.map_err(AiApiClientFactoryError::from)?;

  let new_access = json["access_token"]
    .as_str()
    .ok_or(LlmLibertyRefreshError::MissingAccessToken)?
    .to_string();

  let new_refresh = json["refresh_token"]
    .as_str()
    .unwrap_or(&creds.refresh_token)
    .to_string();

  let expires_in = json["expires_in"]
    .as_i64()
    .ok_or(LlmLibertyRefreshError::MissingExpiresIn)?;
  let new_expires_at = Utc::now() + Duration::seconds(expires_in);

  Ok(ResolvedLlmLibertyCredentials {
    access_token: new_access,
    refresh_token: new_refresh,
    expires_at: new_expires_at,
    ..creds.clone()
  })
}
