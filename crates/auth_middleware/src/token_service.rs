use crate::{AuthError, SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};
use chrono::Utc;
use objs::{AppRegInfoMissingError, ResourceRole, ResourceScope, TokenScope, UserScope};
use services::{
  db::{DbService, TokenStatus},
  extract_claims, AppRegInfo, AuthService, CacheService, Claims, ConcurrencyService, ExpClaims,
  ScopeClaims, SecretService, SecretServiceExt, SettingService, TokenError,
};
use sha2::{Digest, Sha256};
use std::{str::FromStr, sync::Arc};
use tower_sessions::Session;

const BEARER_PREFIX: &str = "Bearer ";
const BODHIAPP_TOKEN_PREFIX: &str = "bodhiapp_";

pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  setting_service: Arc<dyn SettingService>,
  concurrency_service: Arc<dyn ConcurrencyService>,
}

impl DefaultTokenService {
  pub fn new(
    auth_service: Arc<dyn AuthService>,
    secret_service: Arc<dyn SecretService>,
    cache_service: Arc<dyn CacheService>,
    db_service: Arc<dyn DbService>,
    setting_service: Arc<dyn SettingService>,
    concurrency_service: Arc<dyn ConcurrencyService>,
  ) -> Self {
    Self {
      auth_service,
      secret_service,
      cache_service,
      db_service,
      setting_service,
      concurrency_service,
    }
  }

  pub async fn validate_bearer_token(
    &self,
    header: &str,
  ) -> Result<(String, ResourceScope), AuthError> {
    // Extract token from header
    let bearer_token = header
      .strip_prefix(BEARER_PREFIX)
      .ok_or_else(|| TokenError::InvalidToken("authorization header is malformed".to_string()))?
      .trim();
    if bearer_token.is_empty() {
      return Err(TokenError::InvalidToken(
        "token not found in authorization header".to_string(),
      ))?;
    }

    // Check if it's a database-backed token (starts with "bodhiapp_")
    if bearer_token.starts_with(BODHIAPP_TOKEN_PREFIX) {
      // DATABASE TOKEN VALIDATION

      // 1. Extract prefix (first 8 chars after "bodhiapp_")
      let prefix_end = BODHIAPP_TOKEN_PREFIX.len() + 8;
      if bearer_token.len() < prefix_end {
        return Err(TokenError::InvalidToken("Token too short".to_string()))?;
      }
      let token_prefix = &bearer_token[..prefix_end];

      // 2. Lookup token in database by prefix
      let api_token = self
        .db_service
        .get_api_token_by_prefix(token_prefix)
        .await
        .map_err(AuthError::DbError)?;

      let Some(api_token) = api_token else {
        return Err(TokenError::InvalidToken("Token not found".to_string()))?;
      };

      // 3. Check token status
      if api_token.status != TokenStatus::Active {
        return Err(AuthError::TokenInactive);
      }

      // 4. Hash the provided token
      let mut hasher = Sha256::new();
      hasher.update(bearer_token.as_bytes());
      let provided_hash = format!("{:x}", hasher.finalize());

      // 5. Constant-time comparison with stored hash
      if !constant_time_eq::constant_time_eq(
        provided_hash.as_bytes(),
        api_token.token_hash.as_bytes(),
      ) {
        return Err(TokenError::InvalidToken("Invalid token".to_string()))?;
      }

      // 6. Parse scopes string to TokenScope enum
      let token_scope = TokenScope::from_str(&api_token.scopes)
        .map_err(|e| TokenError::InvalidToken(format!("Invalid scope: {}", e)))?;

      // 7. Return ResourceScope::Token with the bearer token itself as the access token
      return Ok((bearer_token.to_string(), ResourceScope::Token(token_scope)));
    }

    // EXTERNAL CLIENT TOKEN VALIDATION (keep existing logic)
    // Check if token has valid expiration first
    let bearer_claims = extract_claims::<ExpClaims>(bearer_token)?;
    if bearer_claims.exp < Utc::now().timestamp() as u64 {
      return Err(TokenError::Expired)?;
    }

    // Create token digest for cache lookup
    let mut hasher = Sha256::new();
    hasher.update(bearer_token.as_bytes());
    let token_digest = format!("{:x}", hasher.finalize())[0..12].to_string();

    // Check cache for exchanged token
    let cached_token = if let Some(access_token) = self
      .cache_service
      .get(&format!("exchanged_token:{}", &token_digest))
    {
      let scope_claims = extract_claims::<ScopeClaims>(&access_token)?;
      if scope_claims.exp < Utc::now().timestamp() as u64 {
        None
      } else {
        let user_scope = UserScope::from_scope(&scope_claims.scope)?;
        Some((access_token, ResourceScope::User(user_scope)))
      }
    } else {
      None
    };

    if let Some((access_token, resource_scope)) = cached_token {
      return Ok((access_token, resource_scope));
    }

    // Exchange external client token
    let (access_token, resource_scope) = self.handle_external_client_token(bearer_token).await?;
    self
      .cache_service
      .set(&format!("exchanged_token:{}", &token_digest), &access_token);
    Ok((access_token, resource_scope))
  }

  /// Handle external client token validation and exchange
  async fn handle_external_client_token(
    &self,
    external_token: &str,
  ) -> Result<(String, ResourceScope), AuthError> {
    // Get app registration info
    let app_reg_info: AppRegInfo = self
      .secret_service
      .app_reg_info()?
      .ok_or(AppRegInfoMissingError)?;

    // Parse token claims to validate issuer
    let claims = extract_claims::<ScopeClaims>(external_token)?;

    // Validate that it's from the same issuer
    if claims.iss != self.setting_service.auth_issuer() {
      return Err(TokenError::InvalidIssuer(claims.iss))?;
    }

    // Validate that current client is in the audience
    if let Some(aud) = &claims.aud {
      if aud != &app_reg_info.client_id {
        return Err(TokenError::InvalidAudience(aud.clone()))?;
      }
    } else {
      return Err(TokenError::InvalidToken(
        "missing audience field".to_string(),
      ))?;
    }

    // Extract user scopes from the external token for exchange
    let mut scopes: Vec<&str> = claims
      .scope
      .split_whitespace()
      .filter(|s| s.starts_with("scope_user_"))
      .collect();
    if scopes.is_empty() {
      return Err(TokenError::ScopeEmpty)?;
    }
    scopes.extend(["openid", "email", "profile", "roles"]);
    // Exchange the external token for our client token
    let (access_token, _) = self
      .auth_service
      .exchange_app_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        external_token,
        scopes.iter().map(|s| s.to_string()).collect(),
      )
      .await?;

    // Extract scope from exchanged token
    let scope_claims = extract_claims::<ScopeClaims>(&access_token)?;
    let user_scope = UserScope::from_scope(&scope_claims.scope)?;

    Ok((access_token, ResourceScope::User(user_scope)))
  }

  pub async fn get_valid_session_token(
    &self,
    session: Session,
    access_token: String,
  ) -> Result<(String, Option<ResourceRole>), AuthError> {
    // Validate session token
    let claims = extract_claims::<Claims>(&access_token)?;

    // Check if token is expired
    let now = Utc::now().timestamp();
    if now < claims.exp as i64 {
      // Token still valid, return immediately
      let client_id = self
        .secret_service
        .app_reg_info()?
        .ok_or(AppRegInfoMissingError)?
        .client_id;
      let role = claims
        .resource_access
        .get(&client_id)
        .map(|roles| ResourceRole::from_resource_role(&roles.roles))
        .transpose()?;
      return Ok((access_token, role));
    }

    // Token is expired, use concurrency control to ensure only one refresh happens
    // Extract session ID for lock key
    let session_id = session
      .id()
      .ok_or_else(|| AuthError::RefreshTokenNotFound)?;
    let lock_key = format!("refresh_token:{}", session_id);

    // Extract user_id from expired token for logging
    let user_id = claims.sub.clone();

    // Clone Arc references for use in the closure
    let auth_service = Arc::clone(&self.auth_service);
    let secret_service = Arc::clone(&self.secret_service);
    let session_clone = session.clone();

    // Execute refresh logic with distributed lock
    let result = self
      .concurrency_service
      .with_lock_auth(
        &lock_key,
        Box::new(move || {
          Box::pin(async move {
            // Wrap the entire logic in a closure that maps AuthError to boxed error
            let inner_result: Result<(String, Option<ResourceRole>), AuthError> = async move {
              // Double-checked locking: re-fetch token from session
              // (another request might have already refreshed it)
              let current_access_token = session_clone
                .get::<String>(SESSION_KEY_ACCESS_TOKEN)
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
              let now = Utc::now().timestamp();

              if now < current_claims.exp as i64 {
                // Token was refreshed by another request, use it
                tracing::info!(
                  "Token already refreshed by concurrent request for user: {}",
                  user_id
                );
                let client_id = secret_service
                  .app_reg_info()?
                  .ok_or(AppRegInfoMissingError)?
                  .client_id;
                let role = current_claims
                  .resource_access
                  .get(&client_id)
                  .map(|roles| ResourceRole::from_resource_role(&roles.roles))
                  .transpose()?;
                return Ok((current_access_token, role));
              }

              // Token still expired, we need to refresh it
              let refresh_token = session_clone
                .get::<String>(SESSION_KEY_REFRESH_TOKEN)
                .await?;

              let Some(refresh_token) = refresh_token else {
                tracing::warn!("Refresh token not found in session for expired access token");
                return Err(AuthError::RefreshTokenNotFound);
              };

              tracing::info!(
                "Attempting to refresh expired access token for user: {}",
                user_id
              );

              // Get app registration info
              let app_reg_info: AppRegInfo = secret_service
                .app_reg_info()?
                .ok_or(AppRegInfoMissingError)?;

              // Attempt token refresh with retry logic in auth_service
              let (new_access_token, new_refresh_token) = match auth_service
                .refresh_token(
                  &app_reg_info.client_id,
                  &app_reg_info.client_secret,
                  &refresh_token,
                )
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

              // Store new tokens in session
              session_clone
                .insert(SESSION_KEY_ACCESS_TOKEN, &new_access_token)
                .await?;

              if let Some(new_refresh_token) = new_refresh_token.as_ref() {
                session_clone
                  .insert(SESSION_KEY_REFRESH_TOKEN, new_refresh_token)
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

              let client_id = secret_service
                .app_reg_info()?
                .ok_or(AppRegInfoMissingError)?
                .client_id;
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
mod tests {
  use crate::{AuthError, DefaultTokenService};
  use anyhow_trace::anyhow_trace;
  use chrono::{Duration, Utc};
  use mockall::predicate::*;
  use objs::{
    test_utils::setup_l10n, FluentLocalizationService, ResourceScope, TokenScope, UserScope,
  };
  use rstest::rstest;
  use serde_json::json;
  use services::{
    db::{ApiToken, DbService, TokenStatus},
    test_utils::{
      build_token, test_db_service, SecretServiceStub, SettingServiceStub, TestDbService, ISSUER,
      TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, AuthServiceError, CacheService, LocalConcurrencyService, MockAuthService,
    MockSecretService, MockSettingService, MokaCacheService, TOKEN_TYPE_OFFLINE,
  };
  use sha2::{Digest, Sha256};
  use std::{collections::HashMap, sync::Arc};
  use uuid::Uuid;

  // Helper function for tests that need token digest (external client token tests)
  fn create_token_digest(bearer_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bearer_token.as_bytes());
    format!("{:x}", hasher.finalize())[0..12].to_string()
  }

  #[rstest]
  #[case::user("scope_token_user", TokenScope::User)]
  #[case::power_user("scope_token_power_user", TokenScope::PowerUser)]
  #[case::manager("scope_token_manager", TokenScope::Manager)]
  #[case::admin("scope_token_admin", TokenScope::Admin)]
  #[awt]
  #[tokio::test]
  async fn test_validate_bodhiapp_token_scope_variations(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] scope_str: &str,
    #[case] expected_scope: TokenScope,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Setup test database with token
    let token_str = "bodhiapp_test12345678901234567890123456789012";
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash the token
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database with specified scope
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: scope_str.to_string(),
      status: TokenStatus::Active,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    test_db_service.create_api_token(&mut api_token).await?;

    // Create token service
    let token_service = DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(MockSecretService::default()),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
      Arc::new(LocalConcurrencyService::new()),
    );

    // Validate token
    let (access_token, scope) = token_service
      .validate_bearer_token(&format!("Bearer {}", token_str))
      .await?;

    assert_eq!(token_str, access_token);
    assert_eq!(ResourceScope::Token(expected_scope), scope);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_validate_bodhiapp_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Setup test database with token
    let token_str = "bodhiapp_test12345678901234567890123456789012";
    // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash the token
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    test_db_service.create_api_token(&mut api_token).await?;

    // Create token service
    let token_service = DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(MockSecretService::default()),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
      Arc::new(LocalConcurrencyService::new()),
    );

    // Validate token
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token_str))
      .await;

    assert_eq!(true, result.is_ok());
    let (access_token, scope) = result.unwrap();
    assert_eq!(token_str, access_token);
    assert_eq!(ResourceScope::Token(TokenScope::User), scope);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_validate_bodhiapp_token_inactive(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Setup test database with inactive token
    let token_str = "bodhiapp_test12345678901234567890123456789012";
    // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
    let token_prefix = &token_str[.."bodhiapp_".len() + 8];

    // Hash the token
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database with Inactive status
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Inactive,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    test_db_service.create_api_token(&mut api_token).await?;

    // Create token service
    let token_service = DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(MockSecretService::default()),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
      Arc::new(LocalConcurrencyService::new()),
    );

    // Validate token - should fail due to inactive status
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token_str))
      .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::TokenInactive)));
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_validate_bodhiapp_token_invalid_hash(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Setup test database with token
    let stored_token_str = "bodhiapp_test12345678901234567890123456789012";
    let different_token_str = "bodhiapp_test12399999999999999999999999999999";
    // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
    let token_prefix = &stored_token_str[.."bodhiapp_".len() + 8];

    // Hash the stored token
    let mut hasher = Sha256::new();
    hasher.update(stored_token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Create ApiToken in database
    let mut api_token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test-user".to_string(),
      name: "Test Token".to_string(),
      token_prefix: token_prefix.to_string(),
      token_hash,
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    test_db_service.create_api_token(&mut api_token).await?;

    // Create token service
    let token_service = DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(MockSecretService::default()),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
      Arc::new(LocalConcurrencyService::new()),
    );

    // Try to validate with different token string (wrong hash)
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", different_token_str))
      .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::Token(_))));
    Ok(())
  }

  #[rstest]
  #[case::empty("")]
  #[case::malformed("bearer foobar")]
  #[case::empty_bearer("Bearer ")]
  #[case::empty_bearer_2("Bearer  ")]
  #[awt]
  #[tokio::test]
  async fn test_validate_bearer_token_header_errors(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] header: &str,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(MockSecretService::default()),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
      Arc::new(LocalConcurrencyService::new()),
    ));
    let result = token_service.validate_bearer_token(header).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::Token(_))));
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_validate_external_client_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given - Create a token from a different client but same issuer
    let external_client_id = "external-client";
    let sub = Uuid::new_v4().to_string();
    let external_token_claims = json!({
      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": ISSUER, // Same issuer as our app
      "sub": sub,
      "typ": TOKEN_TYPE_OFFLINE,
      "azp": external_client_id, // Different client
      "aud": TEST_CLIENT_ID, // Audience is our client
      "session_state": Uuid::new_v4().to_string(),
      "scope": "openid scope_user_user",
      "sid": Uuid::new_v4().to_string(),
    });
    let (external_token, _) = build_token(external_token_claims)?;

    // Setup mock auth service to return exchanged token
    let (exchanged_token, _) = build_token(
      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "test-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "scope_user_user"}},
    )?;
    let exchanged_token_cl = exchanged_token.clone();

    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut mock_auth = MockAuthService::new();

    // Expect token exchange to be called
    mock_auth
      .expect_exchange_app_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(external_token.clone()),
        eq(
          vec!["scope_user_user", "openid", "email", "profile", "roles"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        ),
      )
      .times(1)
      .return_once(|_, _, _, _| Ok((exchanged_token_cl, None)));
    let mut setting_service = MockSettingService::default();
    setting_service
      .expect_auth_issuer()
      .return_once(|| ISSUER.to_string());

    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(setting_service),
      Arc::new(LocalConcurrencyService::new()),
    ));

    // When - Try to validate the external token
    let (access_token, scope) = token_service
      .validate_bearer_token(&format!("Bearer {}", external_token))
      .await?;

    // Then - Should succeed with exchanged token
    assert_eq!(exchanged_token, access_token);
    assert_eq!(ResourceScope::User(UserScope::User), scope);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_external_client_token_cache_security_prevents_jti_forgery(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given - Create a legitimate external token from a different client
    let external_client_id = "external-client";
    let sub = Uuid::new_v4().to_string();
    let jti = Uuid::new_v4().to_string();
    let legitimate_token_claims = json!({
      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": jti.clone(),
      "iss": ISSUER,
      "sub": sub.clone(),
      "typ": TOKEN_TYPE_OFFLINE,
      "azp": external_client_id,
      "aud": TEST_CLIENT_ID,
      "session_state": Uuid::new_v4().to_string(),
      "scope": "openid scope_user_user",
      "sid": Uuid::new_v4().to_string(),
    });
    let (legitimate_token, _) = build_token(legitimate_token_claims)?;

    // Create a forged token with the same JTI but different content
    let forged_token_claims = json!({
      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": jti.clone(), // Same JTI as legitimate token
      "iss": ISSUER,
      "sub": "malicious-user", // Different subject
      "typ": TOKEN_TYPE_OFFLINE,
      "azp": external_client_id,
      "aud": TEST_CLIENT_ID,
      "session_state": Uuid::new_v4().to_string(),
      "scope": "openid scope_user_admin", // Different scope - trying to escalate
      "sid": Uuid::new_v4().to_string(),
    });
    let (forged_token, _) = build_token(forged_token_claims)?;

    // Setup mock auth service - legitimate token succeeds, forged token fails
    let (legitimate_exchanged_token, _) = build_token(
      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "legitimate-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "scope_user_user"}},
    )?;

    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut mock_auth = MockAuthService::new();
    let cache_service = Arc::new(MokaCacheService::default());

    // Expect token exchange for legitimate token to succeed
    mock_auth
      .expect_exchange_app_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(legitimate_token.clone()),
        eq(
          vec!["scope_user_user", "openid", "email", "profile", "roles"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        ),
      )
      .times(1)
      .return_once({
        let token = legitimate_exchanged_token.clone();
        move |_, _, _, _| Ok((token, None))
      });

    // Expect token exchange for forged token to fail with auth service error
    mock_auth
      .expect_exchange_app_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(forged_token.clone()),
        eq(
          vec!["scope_user_admin", "openid", "email", "profile", "roles"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        ),
      )
      .times(1)
      .return_once(|_, _, _, _| {
        Err(AuthServiceError::TokenExchangeError(
          "forged token rejected".to_string(),
        ))
      });

    let setting_service = SettingServiceStub::with_settings(HashMap::from([
      (
        "BODHI_AUTH_URL".to_string(),
        "https://id.mydomain.com".to_string(),
      ),
      ("BODHI_AUTH_REALM".to_string(), "myapp".to_string()),
    ]));

    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(secret_service),
      cache_service.clone(),
      Arc::new(test_db_service),
      Arc::new(setting_service),
      Arc::new(LocalConcurrencyService::new()),
    ));

    // When - First validate the legitimate token (this will cache it)
    let (legitimate_access_token, legitimate_scope) = token_service
      .validate_bearer_token(&format!("Bearer {}", legitimate_token))
      .await?;

    // Then - Verify legitimate token works as expected
    assert_eq!(legitimate_exchanged_token, legitimate_access_token);
    assert_eq!(ResourceScope::User(UserScope::User), legitimate_scope);

    // When - Try to validate the forged token with same JTI
    let forged_result = token_service
      .validate_bearer_token(&format!("Bearer {}", forged_token))
      .await;

    assert!(matches!(
      forged_result,
      Err(AuthError::AuthService(
        AuthServiceError::TokenExchangeError(_)
      ))
    ));
    let legitimate_digest = create_token_digest(&legitimate_token);
    let forged_digest = create_token_digest(&forged_token);
    assert_ne!(
      legitimate_digest, forged_digest,
      "Token digests should be different even with same JTI"
    );

    let cached_legitimate = cache_service.get(&format!("exchanged_token:{}", legitimate_digest));
    let cached_forged = cache_service.get(&format!("exchanged_token:{}", forged_digest));

    assert!(
      cached_legitimate.is_some(),
      "Legitimate token should be cached"
    );
    assert!(
      cached_forged.is_none(),
      "Forged token should not be cached due to validation failure"
    );

    Ok(())
  }
}
