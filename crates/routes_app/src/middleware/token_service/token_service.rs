use crate::middleware::auth::AuthError;
use serde::{Deserialize, Serialize};
use services::AuthContext;
use services::{access_token_key, refresh_token_key};
use services::{
  db::{DbService, TimeService},
  extract_claims, AuthService, CacheService, Claims, ConcurrencyService, ExpClaims, ScopeClaims,
  SettingService, Tenant, TenantError, TenantService, TokenError, TokenStatus,
};
use services::{ResourceRole, TokenScope, UserScope};
use sha2::{Digest, Sha256};
use std::{str::FromStr, sync::Arc};
use tower_sessions::Session;

const BEARER_PREFIX: &str = "Bearer ";
const BODHIAPP_TOKEN_PREFIX: &str = "bodhiapp_";

/// TTL for cached exchange results in seconds (5 minutes)
const EXCHANGE_CACHE_TTL_SECS: i64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedExchangeResult {
  pub token: String,
  pub client_id: String,
  pub tenant_id: String,
  pub app_client_id: String,
  pub role: Option<String>,
  pub access_request_id: Option<String>,
  /// Unix timestamp when this entry was cached
  #[serde(default)]
  pub cached_at: i64,
}

pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  tenant_service: Arc<dyn TenantService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  setting_service: Arc<dyn SettingService>,
  concurrency_service: Arc<dyn ConcurrencyService>,
  time_service: Arc<dyn TimeService>,
}

impl DefaultTokenService {
  pub fn new(
    auth_service: Arc<dyn AuthService>,
    tenant_service: Arc<dyn TenantService>,
    cache_service: Arc<dyn CacheService>,
    db_service: Arc<dyn DbService>,
    setting_service: Arc<dyn SettingService>,
    concurrency_service: Arc<dyn ConcurrencyService>,
    time_service: Arc<dyn TimeService>,
  ) -> Self {
    Self {
      auth_service,
      tenant_service,
      cache_service,
      db_service,
      setting_service,
      concurrency_service,
      time_service,
    }
  }

  pub async fn validate_bearer_token(&self, header: &str) -> Result<AuthContext, AuthError> {
    let bearer_token = header
      .strip_prefix(BEARER_PREFIX)
      .ok_or_else(|| TokenError::InvalidToken("authorization header is malformed".to_string()))?
      .trim();
    if bearer_token.is_empty() {
      return Err(TokenError::InvalidToken(
        "token not found in authorization header".to_string(),
      ))?;
    }

    if let Some(after_prefix) = bearer_token.strip_prefix(BODHIAPP_TOKEN_PREFIX) {
      // Format: bodhiapp_<random>.<client_id>
      // Split on last '.' to extract random_part and client_id
      let dot_pos = after_prefix
        .rfind('.')
        .ok_or_else(|| TokenError::InvalidToken("Token missing client_id suffix".to_string()))?;
      let random_part = &after_prefix[..dot_pos];
      let token_client_id = &after_prefix[dot_pos + 1..];

      if random_part.len() < 8 {
        return Err(TokenError::InvalidToken("Token too short".to_string()))?;
      }

      // Extract prefix from the random part (not including .<client_id>)
      let token_prefix = format!("{}{}", BODHIAPP_TOKEN_PREFIX, &random_part[..8]);

      let api_token = self
        .db_service
        .get_api_token_by_prefix(&token_prefix)
        .await
        .map_err(AuthError::DbError)?;

      let Some(api_token) = api_token else {
        return Err(TokenError::InvalidToken("Token not found".to_string()))?;
      };

      if api_token.status != TokenStatus::Active {
        return Err(AuthError::TokenInactive);
      }

      let mut hasher = Sha256::new();
      hasher.update(bearer_token.as_bytes());
      let provided_hash = format!("{:x}", hasher.finalize());

      if !constant_time_eq::constant_time_eq(
        provided_hash.as_bytes(),
        api_token.token_hash.as_bytes(),
      ) {
        return Err(TokenError::InvalidToken("Invalid token".to_string()))?;
      }

      let token_scope = TokenScope::from_str(&api_token.scopes)
        .map_err(|e| TokenError::InvalidToken(format!("Invalid scope: {}", e)))?;

      let user_id = api_token.user_id.clone();

      let tenant = self
        .tenant_service
        .get_tenant_by_client_id(token_client_id)
        .await?
        .ok_or(TenantError::NotFound)?;

      let client_id = tenant.client_id.clone();
      let tenant_id = tenant.id.clone();
      return Ok(AuthContext::ApiToken {
        client_id,
        tenant_id,
        user_id,
        role: token_scope,
        token: bearer_token.to_string(),
      });
    }

    let bearer_claims = extract_claims::<ExpClaims>(bearer_token)?;
    if bearer_claims.exp < self.time_service.utc_now().timestamp() as u64 {
      return Err(TokenError::Expired)?;
    }

    let mut hasher = Sha256::new();
    hasher.update(bearer_token.as_bytes());
    let token_digest = format!("{:x}", hasher.finalize())[0..32].to_string();

    let now = self.time_service.utc_now().timestamp();
    let cached_auth_context = if let Some(cached_json) = self
      .cache_service
      .get(&format!("exchanged_token:{}", &token_digest))
    {
      if let Ok(cached_result) = serde_json::from_str::<CachedExchangeResult>(&cached_json) {
        // Check TTL: reject stale cache entries
        if cached_result.cached_at > 0 && (now - cached_result.cached_at) > EXCHANGE_CACHE_TTL_SECS
        {
          None
        } else {
          let scope_claims = extract_claims::<ScopeClaims>(&cached_result.token)?;
          if scope_claims.exp < now as u64 {
            None
          } else {
            let role = cached_result
              .role
              .as_ref()
              .and_then(|r| r.parse::<UserScope>().ok());
            Some(AuthContext::ExternalApp {
              client_id: cached_result.client_id.clone(),
              tenant_id: cached_result.tenant_id.clone(),
              user_id: scope_claims.sub,
              role,
              token: cached_result.token,
              external_app_token: bearer_token.to_string(),
              app_client_id: cached_result.app_client_id,
              access_request_id: cached_result.access_request_id,
            })
          }
        }
      } else {
        None
      }
    } else {
      None
    };

    if let Some(auth_context) = cached_auth_context {
      return Ok(auth_context);
    }

    let (auth_context, cached_result) = self.handle_external_client_token(bearer_token).await?;
    if let Ok(cached_json) = serde_json::to_string(&cached_result) {
      self
        .cache_service
        .set(&format!("exchanged_token:{}", &token_digest), &cached_json);
    }
    Ok(auth_context)
  }

  async fn handle_external_client_token(
    &self,
    external_token: &str,
  ) -> Result<(AuthContext, CachedExchangeResult), AuthError> {
    let claims = extract_claims::<ScopeClaims>(external_token)?;
    let original_azp = claims.azp.clone();

    if claims.iss != self.setting_service.auth_issuer().await {
      return Err(TokenError::InvalidIssuer(claims.iss))?;
    }

    let aud = claims
      .aud
      .as_ref()
      .ok_or_else(|| TokenError::InvalidAudience("missing audience".to_string()))?;
    let instance = self
      .tenant_service
      .get_tenant_by_client_id(aud)
      .await?
      .ok_or_else(|| TokenError::InvalidAudience(aud.clone()))?;

    let access_request_scopes: Vec<&str> = claims
      .scope
      .split_whitespace()
      .filter(|s| s.starts_with("scope_access_request:"))
      .collect();

    let access_request_scope = access_request_scopes.first().copied();

    let validated_record = if let Some(access_request_scope) = access_request_scope {
      let record = self
        .db_service
        .get_by_access_request_scope(&instance.id, access_request_scope)
        .await
        .map_err(AuthError::DbError)?
        .ok_or_else(|| {
          TokenError::AccessRequestValidation(
            services::AccessRequestValidationError::ScopeNotFound {
              scope: access_request_scope.to_string(),
            },
          )
        })?;

      if record.status != services::AppAccessRequestStatus::Approved {
        return Err(
          TokenError::AccessRequestValidation(
            services::AccessRequestValidationError::NotApproved {
              id: record.id.clone(),
              status: record.status.to_string(),
            },
          )
          .into(),
        );
      }

      if record.app_client_id != original_azp {
        return Err(
          TokenError::AccessRequestValidation(
            services::AccessRequestValidationError::AppClientMismatch {
              expected: record.app_client_id,
              found: original_azp.clone(),
            },
          )
          .into(),
        );
      }

      let user_id = record.user_id.as_ref().ok_or_else(|| {
        TokenError::AccessRequestValidation(services::AccessRequestValidationError::NotApproved {
          id: record.id.clone(),
          status: "missing user_id in approved request".to_string(),
        })
      })?;

      if user_id != &claims.sub {
        return Err(
          TokenError::AccessRequestValidation(
            services::AccessRequestValidationError::UserMismatch {
              expected: user_id.clone(),
              found: claims.sub.clone(),
            },
          )
          .into(),
        );
      }

      Some(record)
    } else {
      None
    };

    let mut exchange_scopes: Vec<&str> = access_request_scopes.clone();
    for standard_scope in ["openid", "email", "profile", "roles"] {
      if claims.scope.split_whitespace().any(|s| s == standard_scope) {
        exchange_scopes.push(standard_scope);
      }
    }
    let (access_token, _) = self
      .auth_service
      .exchange_app_token(
        &instance.client_id,
        &instance.client_secret,
        external_token,
        exchange_scopes.iter().map(|s| s.to_string()).collect(),
      )
      .await?;

    let scope_claims = extract_claims::<ScopeClaims>(&access_token)?;

    if let Some(ref validated_record) = validated_record {
      if let Some(access_request_id) = &scope_claims.access_request_id {
        if access_request_id != &validated_record.id {
          tracing::warn!(
            expected_id = %validated_record.id,
            claim_id = %access_request_id,
            "Access request ID mismatch between KC claim and DB record"
          );
          return Err(
            TokenError::AccessRequestValidation(
              services::AccessRequestValidationError::AccessRequestIdMismatch {
                claim: access_request_id.clone(),
                expected: validated_record.id.clone(),
              },
            )
            .into(),
          );
        }
      } else {
        tracing::warn!(
          scope = access_request_scope.unwrap_or("unknown"),
          record_id = %validated_record.id,
          "KC did not return access_request_id claim despite valid scope"
        );
        return Err(
          TokenError::AccessRequestValidation(
            services::AccessRequestValidationError::AccessRequestIdMismatch {
              claim: "missing".to_string(),
              expected: validated_record.id.clone(),
            },
          )
          .into(),
        );
      }
    }

    let role = if let Some(ref validated_record) = validated_record {
      validated_record
        .approved_role
        .as_ref()
        .and_then(|r| r.parse::<UserScope>().ok())
    } else {
      None
    };

    // Verify approved_role doesn't exceed user's resource role (prevent privilege escalation via DB tampering)
    if let Some(approved_role) = role {
      let user_resource_role = scope_claims
        .resource_access
        .get(&instance.client_id)
        .and_then(|rc| ResourceRole::from_resource_role(&rc.roles).ok());
      if let Some(resource_role) = user_resource_role {
        let max_scope = resource_role.max_user_scope();
        if !max_scope.has_access_to(&approved_role) {
          tracing::warn!(
            approved_role = %approved_role,
            resource_role = %resource_role,
            max_scope = %max_scope,
            "DB approved_role exceeds user's resource role"
          );
          return Err(
            TokenError::AccessRequestValidation(
              services::AccessRequestValidationError::PrivilegeEscalation {
                approved_role: approved_role.to_string(),
                max_scope: max_scope.to_string(),
              },
            )
            .into(),
          );
        }
      }
    }

    let role_str = role.map(|r| r.to_string());

    let cached_result = CachedExchangeResult {
      token: access_token.clone(),
      client_id: instance.client_id.clone(),
      tenant_id: instance.id.clone(),
      app_client_id: original_azp.clone(),
      role: role_str,
      access_request_id: scope_claims.access_request_id.clone(),
      cached_at: self.time_service.utc_now().timestamp(),
    };

    let auth_context = AuthContext::ExternalApp {
      client_id: instance.client_id,
      tenant_id: instance.id,
      user_id: scope_claims.sub,
      role,
      token: access_token,
      external_app_token: external_token.to_string(),
      app_client_id: original_azp,
      access_request_id: scope_claims.access_request_id,
    };

    Ok((auth_context, cached_result))
  }

  /// Validates and optionally refreshes the dashboard access token stored in the session.
  /// Uses distributed locking to prevent concurrent dashboard token refreshes.
  /// Returns the valid dashboard access token string on success.
  pub async fn get_valid_dashboard_token(
    &self,
    session: Session,
    dashboard_token: String,
  ) -> Result<String, AuthError> {
    let claims = extract_claims::<Claims>(&dashboard_token)?;
    let now = self.time_service.utc_now().timestamp();
    if now < (claims.exp as i64 - 30) {
      return Ok(dashboard_token);
    }

    // Token expired, refresh with lock
    let session_id = session
      .id()
      .ok_or_else(|| AuthError::RefreshTokenNotFound)?;
    let lock_key = format!("dashboard:{}:refresh", session_id);

    let auth_service = Arc::clone(&self.auth_service);
    let setting_service = Arc::clone(&self.setting_service);
    let time_service = Arc::clone(&self.time_service);
    let session_clone = session.clone();

    let result = self
      .concurrency_service
      .with_lock_str(
        &lock_key,
        Box::new(move || {
          Box::pin(async move {
            let inner_result: Result<String, AuthError> = async move {
              // Double-check: re-read from session (another request may have refreshed)
              let current = session_clone
                .get::<String>(services::DASHBOARD_ACCESS_TOKEN_KEY)
                .await?;

              let Some(current) = current else {
                return Err(AuthError::RefreshTokenNotFound);
              };

              let current_claims = extract_claims::<Claims>(&current)?;
              let now = time_service.utc_now().timestamp();
              if now < current_claims.exp as i64 {
                return Ok(current);
              }

              // Still expired, refresh
              let refresh_token = session_clone
                .get::<String>(services::DASHBOARD_REFRESH_TOKEN_KEY)
                .await?;
              let Some(refresh_token) = refresh_token else {
                return Err(AuthError::RefreshTokenNotFound);
              };

              let client_id = setting_service.multitenant_client_id().await.map_err(|_| {
                AuthError::InvalidToken("Missing multi-tenant client config".to_string())
              })?;
              let client_secret =
                setting_service
                  .multitenant_client_secret()
                  .await
                  .map_err(|_| {
                    AuthError::InvalidToken("Missing multi-tenant client secret".to_string())
                  })?;

              let (new_access_token, new_refresh_token) = auth_service
                .refresh_token(&client_id, &client_secret, &refresh_token)
                .await?;

              session_clone
                .insert(services::DASHBOARD_ACCESS_TOKEN_KEY, &new_access_token)
                .await?;
              if let Some(ref new_refresh) = new_refresh_token {
                session_clone
                  .insert(services::DASHBOARD_REFRESH_TOKEN_KEY, new_refresh)
                  .await?;
              }
              session_clone
                .save()
                .await
                .map_err(AuthError::TowerSession)?;

              Ok(new_access_token)
            }
            .await;

            inner_result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
          })
        }),
      )
      .await;

    result.map_err(|e| {
      e.downcast::<AuthError>()
        .map(|boxed| *boxed)
        .unwrap_or_else(|e| {
          AuthError::InvalidToken(format!("Dashboard token refresh error: {}", e))
        })
    })
  }

  pub async fn get_valid_session_token(
    &self,
    session: Session,
    access_token: String,
    tenant: &Tenant,
  ) -> Result<(String, Option<ResourceRole>), AuthError> {
    // Validate session token
    let claims = extract_claims::<Claims>(&access_token)?;

    let instance_client_id = tenant.client_id.clone();
    let instance_client_secret = tenant.client_secret.clone();

    // Check if token is expired
    let now = self.time_service.utc_now().timestamp();
    if now < claims.exp as i64 {
      // Token still valid, return immediately
      let role = claims
        .resource_access
        .get(&instance_client_id)
        .map(|roles| ResourceRole::from_resource_role(&roles.roles))
        .transpose()?;
      return Ok((access_token, role));
    }

    // Token is expired, use concurrency control to ensure only one refresh happens
    // Extract session ID for lock key
    let session_id = session
      .id()
      .ok_or_else(|| AuthError::RefreshTokenNotFound)?;
    let lock_key = format!("{}:{}:refresh_token", instance_client_id, session_id);

    // Extract user_id from expired token for logging
    let user_id = claims.sub.clone();

    // Clone values for use in the closure
    let auth_service = Arc::clone(&self.auth_service);
    let time_service = Arc::clone(&self.time_service);
    let session_clone = session.clone();
    // client_id and client_secret already fetched — clone into closure
    let closure_client_id = instance_client_id.clone();
    let closure_client_secret = instance_client_secret.clone();

    // Execute refresh logic with distributed lock
    let result = self
      .concurrency_service
      .with_lock_auth(
        &lock_key,
        Box::new(move || {
          let client_id = closure_client_id.clone();
          let client_secret = closure_client_secret.clone();
          Box::pin(async move {
            // Wrap the entire logic in a closure that maps AuthError to boxed error
            let inner_result: Result<(String, Option<ResourceRole>), AuthError> = async move {
              // Double-checked locking: re-fetch token from session
              // (another request might have already refreshed it)
              let current_access_token = session_clone
                .get::<String>(&access_token_key(&client_id))
                .await?;

              let Some(current_access_token) = current_access_token else {
                tracing::warn!(
                  "Access token not found in session after acquiring lock for user: {}",
                  user_id
                );
                return Err(AuthError::RefreshTokenNotFound);
              };

              // Re-validate the current token - it might have been refreshed
              let current_claims = extract_claims::<Claims>(&current_access_token)?;
              let now = time_service.utc_now().timestamp();

              if now < current_claims.exp as i64 {
                // Token was refreshed by another request, use it
                tracing::info!(
                  "Token already refreshed by concurrent request for user: {}",
                  user_id
                );
                let role = current_claims
                  .resource_access
                  .get(&client_id)
                  .map(|roles| ResourceRole::from_resource_role(&roles.roles))
                  .transpose()?;
                return Ok((current_access_token, role));
              }

              // Token still expired, we need to refresh it
              let refresh_token = session_clone
                .get::<String>(&refresh_token_key(&client_id))
                .await?;

              let Some(refresh_token) = refresh_token else {
                tracing::warn!("Refresh token not found in session for expired access token");
                return Err(AuthError::RefreshTokenNotFound);
              };

              tracing::info!(
                "Attempting to refresh expired access token for user: {}",
                user_id
              );

              // Attempt token refresh with retry logic in auth_service
              let (new_access_token, new_refresh_token) = match auth_service
                .refresh_token(&client_id, &client_secret, &refresh_token)
                .await
              {
                Ok(tokens) => {
                  tracing::info!("Token refresh successful for user: {}", user_id);
                  tokens
                }
                Err(e) => {
                  tracing::error!("Failed to refresh token for user {}: {}", user_id, e);
                  return Err(e.into());
                }
              };

              // Extract claims from new token first to validate and get role
              let new_claims = extract_claims::<Claims>(&new_access_token)?;

              // Store new tokens in session (namespaced by client_id)
              session_clone
                .insert(&access_token_key(&client_id), &new_access_token)
                .await?;

              if let Some(new_refresh_token) = new_refresh_token.as_ref() {
                session_clone
                  .insert(&refresh_token_key(&client_id), new_refresh_token)
                  .await?;
                tracing::debug!(
                  "Updated access and refresh tokens in session for user: {}",
                  user_id
                );
              } else {
                tracing::debug!(
                  "Updated access token in session (no new refresh token) for user: {}",
                  user_id
                );
              }

              // Explicitly save session to ensure persistence
              session_clone.save().await.map_err(|e| {
                tracing::error!(
                  "Failed to save session after token refresh for user {}: {:?}",
                  user_id,
                  e
                );
                AuthError::TowerSession(e)
              })?;

              tracing::info!(
                "Session saved successfully after token refresh for user: {}",
                user_id
              );

              let role = new_claims
                .resource_access
                .get(&client_id)
                .map(|resource_claims| ResourceRole::from_resource_role(&resource_claims.roles))
                .transpose()?;

              tracing::info!(
                "Successfully refreshed token for user {} with role: {:?}",
                user_id,
                role
              );
              Ok((new_access_token, role))
            }
            .await;

            // Map AuthError to boxed error for trait compatibility
            inner_result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
          })
        }),
      )
      .await;

    // Map back from boxed error to AuthError
    result.map_err(|e| {
      e.downcast::<AuthError>()
        .map(|boxed| *boxed)
        .unwrap_or_else(|e| {
          AuthError::InvalidToken(format!("Unexpected token refresh error: {}", e))
        })
    })
  }
}

#[cfg(test)]
#[path = "test_token_service.rs"]
mod test_token_service;
