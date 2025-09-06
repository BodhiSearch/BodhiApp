use crate::AuthError;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, Validation};
use objs::{AppRegInfoMissingError, ResourceScope, Role, TokenScope, UserScope};
use services::{
  db::{DbService, TokenStatus},
  extract_claims, AppRegInfo, AuthService, CacheService, Claims, ExpClaims, OfflineClaims,
  ScopeClaims, SecretService, SecretServiceExt, SettingService, TokenError, TOKEN_TYPE_OFFLINE,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tower_sessions::Session;

const BEARER_PREFIX: &str = "Bearer ";
const SCOPE_OFFLINE_ACCESS: &str = "offline_access";
const LEEWAY_SECONDS: i64 = 60; // 1 minute leeway for clock skew

pub fn create_token_digest(bearer_token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bearer_token.as_bytes());
  format!("{:x}", hasher.finalize())[0..12].to_string()
}

pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  setting_service: Arc<dyn SettingService>,
}

impl DefaultTokenService {
  pub fn new(
    auth_service: Arc<dyn AuthService>,
    secret_service: Arc<dyn SecretService>,
    cache_service: Arc<dyn CacheService>,
    db_service: Arc<dyn DbService>,
    setting_service: Arc<dyn SettingService>,
  ) -> Self {
    Self {
      auth_service,
      secret_service,
      cache_service,
      db_service,
      setting_service,
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

    // Check token is found and active
    let api_token = if let Ok(Some(api_token)) = self
      .db_service
      .get_api_token_by_token_id(bearer_token)
      .await
    {
      if api_token.status == TokenStatus::Inactive {
        return Err(AuthError::TokenInactive);
      } else {
        api_token
      }
    } else {
      let bearer_claims = extract_claims::<ExpClaims>(bearer_token)?;
      if bearer_claims.exp < Utc::now().timestamp() as u64 {
        return Err(TokenError::Expired)?;
      }
      let token_digest = create_token_digest(bearer_token);
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
      let (access_token, resource_scope) = self.handle_external_client_token(bearer_token).await?;
      self
        .cache_service
        .set(&format!("exchanged_token:{}", &token_digest), &access_token);
      return Ok((access_token, resource_scope));
    };

    // Check if token is in cache and not expired
    if let Some(access_token) = self
      .cache_service
      .get(&format!("token:{}", api_token.token_id))
    {
      let mut validation = Validation::default();
      validation.insecure_disable_signature_validation();
      validation.validate_exp = true;
      validation.validate_aud = false;
      let token_data = jsonwebtoken::decode::<ExpClaims>(
        &access_token,
        &DecodingKey::from_secret(&[]), // dummy key for parsing
        &validation,
      );
      if let Ok(token_data) = token_data {
        let offline_scope = token_data.claims.scope;
        let scope = TokenScope::from_scope(&offline_scope)?;
        return Ok((access_token, ResourceScope::Token(scope)));
      } else {
        self
          .cache_service
          .remove(&format!("token:{}", api_token.token_id));
      }
    }

    // If token is active and not found in cache, proceed with full validation
    let app_reg_info: AppRegInfo = self
      .secret_service
      .app_reg_info()?
      .ok_or(AppRegInfoMissingError)?;

    // Validate claims - iat, expiry, tpe, azp, scope: offline_access
    let claims = extract_claims::<OfflineClaims>(bearer_token)?;
    self.validate_token_claims(&claims, &app_reg_info.client_id)?;

    // Exchange token
    let (access_token, _) = self
      .auth_service
      .refresh_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        bearer_token,
      )
      .await?;

    // store the retrieved access token in cache
    self
      .cache_service
      .set(&format!("token:{}", api_token.token_id), &access_token);
    let scope = extract_claims::<ScopeClaims>(&access_token)?;
    let token_scope = TokenScope::from_scope(&scope.scope)?;
    Ok((access_token, ResourceScope::Token(token_scope)))
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

  fn validate_token_claims(
    &self,
    claims: &OfflineClaims,
    client_id: &str,
  ) -> Result<(), AuthError> {
    // Validate token expiration
    let now = Utc::now().timestamp();
    let leeway = Duration::seconds(LEEWAY_SECONDS);

    // Check if token is not yet valid (with leeway)
    if claims.iat > (now + leeway.num_seconds()) as u64 {
      return Err(AuthError::InvalidToken(format!(
        "token is not yet valid, issued at {}",
        claims.iat
      )));
    }

    // Check token type
    if claims.typ != TOKEN_TYPE_OFFLINE {
      return Err(AuthError::InvalidToken(
        "token type must be Offline".to_string(),
      ));
    }

    // Check authorized party
    if claims.azp != client_id {
      return Err(AuthError::InvalidToken(
        "invalid token authorized party".to_string(),
      ));
    }

    // Check scope
    if !claims
      .scope
      .split(' ')
      .map(|s| s.to_string())
      .collect::<Vec<_>>()
      .contains(&SCOPE_OFFLINE_ACCESS.to_string())
    {
      return Err(AuthError::InvalidToken(
        "token missing required scope: offline_access".to_string(),
      ));
    }

    Ok(())
  }

  pub async fn get_valid_session_token(
    &self,
    session: Session,
    access_token: String,
  ) -> Result<(String, Role), AuthError> {
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
      let roles = claims
        .resource_access
        .get(&client_id)
        .ok_or(AuthError::MissingRoles)?;
      let role = Role::from_resource_role(&roles.roles)?;
      return Ok((access_token, role));
    }

    // Token is expired, try to refresh
    let refresh_token = session.get::<String>("refresh_token").await?;

    // Add better error handling and logging
    let Some(refresh_token) = refresh_token else {
      tracing::warn!("Refresh token not found in session for expired access token");
      return Err(AuthError::RefreshTokenNotFound);
    };

    tracing::info!("Attempting to refresh expired access token");

    // Get app registration info
    let app_reg_info: AppRegInfo = self
      .secret_service
      .app_reg_info()?
      .ok_or(AppRegInfoMissingError)?;

    // Attempt token refresh
    let (new_access_token, new_refresh_token) = match self
      .auth_service
      .refresh_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        &refresh_token,
      )
      .await
    {
      Ok(tokens) => tokens,
      Err(e) => {
        tracing::error!("Failed to refresh token: {:?}", e);
        return Err(e.into());
      }
    };

    // Store new tokens in session
    session.insert("access_token", &new_access_token).await?;
    if let Some(refresh_token) = new_refresh_token.as_ref() {
      session.insert("refresh_token", refresh_token).await?;
      tracing::info!("Updated both access and refresh tokens in session");
    } else {
      tracing::info!("Updated access token in session (no new refresh token provided)");
    }

    // Extract claims from new token
    let claims = extract_claims::<Claims>(&new_access_token)?;
    let client_id = self
      .secret_service
      .app_reg_info()?
      .ok_or(AppRegInfoMissingError)?
      .client_id;
    let resource_claims = claims
      .resource_access
      .get(&client_id)
      .ok_or(AuthError::MissingRoles)?;
    let role = Role::from_resource_role(&resource_claims.roles)?;

    tracing::info!("Successfully refreshed token for role: {:?}", role);
    Ok((new_access_token, role))
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    create_token_digest, token_service::SCOPE_OFFLINE_ACCESS, AuthError, DefaultTokenService,
  };
  use anyhow_trace::anyhow_trace;
  use chrono::{Duration, Utc};
  use mockall::predicate::*;
  use objs::{
    test_utils::setup_l10n, FluentLocalizationService, ResourceScope, TokenScope, UserScope,
  };
  use rstest::rstest;
  use serde_json::{json, Value};
  use services::{
    db::DbService,
    test_utils::{
      build_token, offline_token_claims, test_db_service, SecretServiceStub, SettingServiceStub,
      TestDbService, ISSUER, TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, AuthServiceError, CacheService, MockAuthService, MockSecretService,
    MockSettingService, MokaCacheService, TOKEN_TYPE_OFFLINE,
  };
  use std::{collections::HashMap, sync::Arc};
  use uuid::Uuid;

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
    ));
    let result = token_service.validate_bearer_token(header).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::Token(_))));
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[case::scope_token_user("offline_access scope_token_user", TokenScope::User)]
  #[case::scope_token_user_power_user(
    "offline_access scope_token_power_user",
    TokenScope::PowerUser
  )]
  #[case::scope_token_user_manager("offline_access scope_token_manager", TokenScope::Manager)]
  #[case::scope_token_user_admin("offline_access scope_token_admin", TokenScope::Admin)]
  #[awt]
  #[tokio::test]
  async fn test_validate_bearer_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
    #[case] scope: &str,
    #[case] expected_role: TokenScope,
  ) -> anyhow::Result<()> {
    // Given
    let claims = offline_token_claims();
    let (offline_token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &offline_token)
      .await?;
    let (refreshed_token, _) = build_token(
      json! {{"iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": scope}},
    )?;
    let refreshed_token_cl = refreshed_token.clone();
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut mock_auth = MockAuthService::new();
    mock_auth
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(offline_token.clone()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((refreshed_token_cl, Some("new_refresh_token".to_string()))));

    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    ));

    // When
    let (result, role) = token_service
      .validate_bearer_token(&format!("Bearer {}", offline_token))
      .await?;

    // Then
    assert_eq!(refreshed_token, result);
    assert_eq!(ResourceScope::Token(expected_role), role);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[case::scope_token_user("", "missing_offline_access")]
  #[case::scope_token_user("scope_token_user", "missing_offline_access")]
  #[case::scope_token_user("offline_access", "missing_token_scope")]
  #[awt]
  #[tokio::test]
  async fn test_token_service_bearer_token_exchanged_token_scope_invalid(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
    #[case] scope: &str,
    #[case] err_msg: &str,
  ) -> anyhow::Result<()> {
    // Given
    let claims = offline_token_claims();
    let (offline_token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &offline_token)
      .await?;
    let (refreshed_token, _) = build_token(
      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": scope}},
    )?;
    let refreshed_token_cl = refreshed_token.clone();
    let mut mock_auth = MockAuthService::new();
    mock_auth
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(offline_token.clone()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((refreshed_token_cl, Some("new_refresh_token".to_string()))));

    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(SecretServiceStub::default().with_app_reg_info_default()),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    ));

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", offline_token))
      .await;
    assert!(result.is_err());
    assert_eq!(err_msg, result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case::invalid_type(
    json!({"typ": "Invalid"}),"token type must be Offline"
  )]
  #[case::wrong_azp(
    json!({"azp": "wrong-client"}),"invalid token authorized party"
  )]
  #[case::no_offline_access_scope(
    json!({"scope": "openid profile"}),"token missing required scope: offline_access"
  )]
  #[awt]
  #[tokio::test]
  async fn test_validate_bearer_token_validation_errors(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] claims_override: serde_json::Value,
    #[case] expected: &str,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let mut claims = offline_token_claims();
    claims
      .as_object_mut()
      .unwrap()
      .extend(claims_override.as_object().unwrap().clone());
    let (offline_token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &offline_token)
      .await?;
    let secret_service =
      SecretServiceStub::default().with_app_reg_info(&AppRegInfoBuilder::test_default().build()?);
    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    ));

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", offline_token))
      .await;

    // Then
    assert!(result.is_err());
    let api_error = objs::ApiError::from(result.unwrap_err());
    assert_eq!(expected, api_error.args["var_0"]);
    assert_eq!("auth_error-invalid_token", api_error.code);
    Ok(())
  }

  #[rstest]
  #[case( json!({
    "iat": Utc::now().timestamp() + 3600,  // issued 1 hour in future
    "jti": "test-jti",
    "iss": ISSUER,
    "sub": "test-sub",
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": TEST_CLIENT_ID,
    "scope": SCOPE_OFFLINE_ACCESS
  }))]
  #[awt]
  #[tokio::test]
  async fn test_token_time_validation_failures(
    #[case] claims: Value,
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let (token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &token)
      .await?;
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let auth_service = MockAuthService::new();
    let token_service = DefaultTokenService::new(
      Arc::new(auth_service),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    );

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await;

    // Then
    assert!(result.is_err());
    assert!(
      matches!(result, Err(AuthError::InvalidToken(msg)) if msg.starts_with("token is not yet valid"))
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_token_validation_success_with_leeway(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let now = Utc::now().timestamp();
    let claims = json!({
      "exp": now + 30, // expires in 30 seconds
      "iat": now - 30, // issued 30 seconds ago
      "jti": "test-jti",
      "iss": ISSUER,
      "sub": "test-sub",
      "typ": TOKEN_TYPE_OFFLINE,
      "azp": TEST_CLIENT_ID,
      "scope": SCOPE_OFFLINE_ACCESS
    });
    let (offline_token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &offline_token)
      .await?;
    let (refreshed_token, _) = build_token(
      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": "offline_access scope_token_user"}},
    )?;
    let refreshed_token_cl = refreshed_token.clone();
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut auth_service = MockAuthService::new();
    auth_service
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(offline_token.clone()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((refreshed_token_cl, None)));
    let token_service = DefaultTokenService::new(
      Arc::new(auth_service),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    );

    // When
    let (result, token_scope) = token_service
      .validate_bearer_token(&format!("Bearer {}", offline_token))
      .await?;

    // Then
    assert_eq!(refreshed_token, result);
    assert_eq!(ResourceScope::Token(TokenScope::User), token_scope);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_token_validation_auth_service_error(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let claims = offline_token_claims();
    let (token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &token)
      .await?;
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut auth_service = MockAuthService::new();
    auth_service
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(token.clone()),
      )
      .times(1)
      .return_once(|_, _, _| {
        Err(AuthServiceError::AuthServiceApiError(
          "server unreachable".to_string(),
        ))
      });
    let token_service = DefaultTokenService::new(
      Arc::new(auth_service),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    );

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await;

    // Then
    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::AuthService(_))));
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_token_validation_with_cache_hit(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let claims = offline_token_claims();
    let (offline_token, _) = build_token(claims)?;
    let (access_token, _) = build_token(
      json! {{"jti": "test-jti", "sub": "test-sub", "exp": Utc::now().timestamp() + 3600, "scope": "offline_access scope_token_user"}},
    )?;
    let api_token = test_db_service
      .create_api_token_from("test-token", &offline_token)
      .await?;
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let auth_service = MockAuthService::new(); // Should not be called
    let cache_service = MokaCacheService::default();
    let cache_key = format!("token:{}", api_token.token_id);
    cache_service.set(&cache_key, &access_token);
    let token_service = DefaultTokenService::new(
      Arc::new(auth_service),
      Arc::new(secret_service),
      Arc::new(cache_service),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    );

    // When
    let (result, scope) = token_service
      .validate_bearer_token(&format!("Bearer {}", offline_token))
      .await?;

    // Then
    assert_eq!(access_token, result);
    assert_eq!(ResourceScope::Token(TokenScope::User), scope);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_token_validation_with_expired_cache(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let claims = offline_token_claims();
    let (offline_token, _) = build_token(claims)?;
    let api_token = test_db_service
      .create_api_token_from("test_token", &offline_token)
      .await?;
    let (refreshed_token, _) = build_token(
      json! {{"iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": "offline_access scope_token_user"}},
    )?;
    let refreshed_token_cl = refreshed_token.clone();
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut mock_auth = MockAuthService::new();
    let cache_service = MokaCacheService::default();

    // Create an expired access token and store it in cache
    let expired_claims = json!({
      "exp": Utc::now().timestamp() - 3600, // expired 1 hour ago
      "iat": Utc::now().timestamp() - 7200,  // issued 2 hours ago
    });
    let (expired_access_token, _) = build_token(expired_claims)?;

    // Store expired token in cache
    cache_service.set(
      &format!("token:{}", api_token.token_id),
      &expired_access_token,
    );

    // Expect token exchange to be called since cached token is expired
    mock_auth
      .expect_refresh_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(offline_token.clone()),
      )
      .times(1)
      .return_once(|_, _, _| Ok((refreshed_token_cl, None)));

    let token_service = DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(secret_service),
      Arc::new(cache_service),
      Arc::new(test_db_service),
      Arc::new(MockSettingService::default()),
    );

    // When
    let (result, token_scope) = token_service
      .validate_bearer_token(&format!("Bearer {}", offline_token))
      .await?;

    // Then
    assert_eq!(refreshed_token, result);
    assert_eq!(ResourceScope::Token(TokenScope::User), token_scope);
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
