use crate::AuthError;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, Validation};
use services::{
  db::{DbService, TokenStatus},
  AppRegInfo, AuthService, CacheService, Claims, ExpClaims, JsonWebTokenError, OfflineClaims,
  SecretService, SecretServiceExt, GRANT_REFRESH_TOKEN, TOKEN_TYPE_OFFLINE,
};
use std::sync::Arc;
use tower_sessions::Session;

const BEARER_PREFIX: &str = "Bearer ";
const SCOPE_TOKEN_USER: &str = "scope_token_user";
const LEEWAY_SECONDS: i64 = 60; // 1 minute leeway for clock skew

pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
}

impl DefaultTokenService {
  pub fn new(
    auth_service: Arc<dyn AuthService>,
    secret_service: Arc<dyn SecretService>,
    cache_service: Arc<dyn CacheService>,
    db_service: Arc<dyn DbService>,
  ) -> Self {
    Self {
      auth_service,
      secret_service,
      cache_service,
      db_service,
    }
  }

  pub async fn validate_bearer_token(&self, header: &str) -> Result<String, AuthError> {
    // Extract token from header
    let offline_token = header
      .strip_prefix(BEARER_PREFIX)
      .ok_or_else(|| AuthError::TokenValidation("authorization header is malformed".to_string()))?
      .trim();
    if offline_token.is_empty() {
      return Err(AuthError::TokenValidation(
        "token not found in authorization header".to_string(),
      ));
    }

    // Check token is found and active
    let api_token =
      if let Ok(Some(api_token)) = self.db_service.get_valid_api_token(offline_token).await {
        if api_token.status == TokenStatus::Inactive {
          return Err(AuthError::TokenInactive);
        } else {
          api_token
        }
      } else {
        return Err(AuthError::TokenNotFound);
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
      if jsonwebtoken::decode::<ExpClaims>(
        &access_token,
        &DecodingKey::from_secret(&[]), // dummy key for parsing
        &validation,
      )
      .is_ok()
      {
        return Ok(access_token);
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
      .ok_or(AuthError::AppRegInfoMissing)?;

    // Validate token signature and claims
    let claims = self.validate_token_signature(offline_token, &app_reg_info)?;
    self.validate_token_claims(&claims, &app_reg_info.client_id)?;

    // Exchange token
    let (access_token, _) = self
      .auth_service
      .exchange_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        offline_token,
        GRANT_REFRESH_TOKEN,
        vec![],
      )
      .await?;

    // store the retrieved access token in cache to avoid going back to auth server next time on
    self
      .cache_service
      .set(&format!("token:{}", api_token.token_id), &access_token);
    Ok(access_token)
  }

  fn validate_token_signature(
    &self,
    token: &str,
    app_reg_info: &AppRegInfo,
  ) -> Result<OfflineClaims, AuthError> {
    // Decode header first to validate kid and alg
    let header = jsonwebtoken::decode_header(token)
      .map_err(|_| AuthError::TokenValidation("invalid token header format".to_string()))?;

    // Check header values
    if header.kid.as_deref() != Some(&app_reg_info.kid) {
      return Err(AuthError::TokenValidation(
        "invalid token key identifier".to_string(),
      ));
    }

    if header.alg != app_reg_info.alg {
      return Err(AuthError::TokenValidation(
        "invalid token signing algorithm".to_string(),
      ));
    }

    // Setup validation
    let key_pem = format!(
      "-----BEGIN RSA PUBLIC KEY-----\n{}\n-----END RSA PUBLIC KEY-----",
      app_reg_info.public_key
    );
    let key = DecodingKey::from_rsa_pem(key_pem.as_bytes())
      .map_err(|err| AuthError::TokenValidation(format!("invalid token public key: {err}")))?;

    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[&app_reg_info.issuer]);
    validation.validate_exp = false;
    validation.validate_aud = false;
    let items: &[String] = &[];
    validation.set_required_spec_claims(items);

    // Validate and decode token
    let token_data =
      jsonwebtoken::decode::<OfflineClaims>(token, &key, &validation).map_err(|err| {
        AuthError::TokenValidation(format!("token signature validation failed, err: {err}"))
      })?;

    Ok(token_data.claims)
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
      return Err(AuthError::TokenValidation(format!(
        "token is not yet valid, issued at {}",
        claims.iat
      )));
    }

    // Check token type
    if claims.typ != TOKEN_TYPE_OFFLINE {
      return Err(AuthError::TokenValidation(
        "token type must be Offline".to_string(),
      ));
    }

    // Check authorized party
    if claims.azp != client_id {
      return Err(AuthError::TokenValidation(
        "invalid token authorized party".to_string(),
      ));
    }

    // Check scope
    if !claims
      .scope
      .split(' ')
      .map(|s| s.to_string())
      .collect::<Vec<_>>()
      .contains(&SCOPE_TOKEN_USER.to_string())
    {
      return Err(AuthError::TokenValidation(
        "token missing required scope: scope_token_user".to_string(),
      ));
    }

    Ok(())
  }

  pub async fn get_valid_session_token(
    &self,
    session: Session,
    access_token: String,
  ) -> Result<String, AuthError> {
    // Validate session token
    let claims = Self::decode_access_token_no_validation(&access_token)?;
    // Check if token is expired
    let now = Utc::now().timestamp();
    if now < claims.claims.exp as i64 {
      return Ok(access_token);
    }

    let Some(refresh_token) = session.get::<String>("refresh_token").await? else {
      return Err(AuthError::RefreshTokenNotFound);
    };

    // Token is expired, try to refresh
    let app_reg_info: AppRegInfo = self
      .secret_service
      .app_reg_info()?
      .ok_or(AuthError::AppRegInfoMissing)?;

    let (new_access_token, new_refresh_token) = self
      .auth_service
      .refresh_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        &refresh_token,
      )
      .await?;

    // Store new tokens in session
    session.insert("access_token", &new_access_token).await?;
    if let Some(refresh_token) = new_refresh_token.as_ref() {
      session.insert("refresh_token", refresh_token).await?;
    }

    Ok(new_access_token)
  }

  fn decode_access_token_no_validation(
    access_token: &str,
  ) -> Result<jsonwebtoken::TokenData<Claims>, JsonWebTokenError> {
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    let token_data = jsonwebtoken::decode::<Claims>(
      access_token,
      &DecodingKey::from_secret(&[]), // dummy key for parsing
      &validation,
    )?;
    Ok(token_data)
  }
}

#[cfg(test)]
mod tests {
  use crate::{token_service::SCOPE_TOKEN_USER, AuthError, DefaultTokenService};
  use chrono::Utc;
  use mockall::predicate::*;
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use rstest::rstest;
  use serde_json::{json, Value};
  use services::{
    db::DbService,
    test_utils::{
      build_token, offline_token_claims, test_db_service, SecretServiceStub, TestDbService, ISSUER,
      TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, AuthServiceError, CacheService, MockAuthService, MockSecretService,
    MokaCacheService, GRANT_REFRESH_TOKEN, TOKEN_TYPE_OFFLINE,
  };
  use std::sync::Arc;

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
    ));
    let result = token_service.validate_bearer_token(header).await;
    assert!(result.is_err());
    assert_eq!(matches!(result, Err(AuthError::TokenValidation(_))), true);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_validate_bearer_token_success(
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
    let mut mock_auth = MockAuthService::new();
    mock_auth
      .expect_exchange_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(token.clone()),
        eq(GRANT_REFRESH_TOKEN),
        eq(vec![]),
      )
      .returning(|_, _, _, _, _| {
        Ok((
          "new_access_token".to_string(),
          Some("new_refresh_token".to_string()),
        ))
      });

    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
    ));

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await?;

    // Then
    assert_eq!(result, "new_access_token");
    Ok(())
  }

  #[rstest]
  #[case::invalid_type(
    json!({"typ": "Invalid"}),
  )]
  #[case::wrong_azp(
    json!({"azp": "wrong-client"}),
  )]
  #[case::missing_scope(
    json!({"scope": "openid profile"}),
  )]
  #[awt]
  #[tokio::test]
  async fn test_validate_bearer_token_validation_errors(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] claims_override: serde_json::Value,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Given
    let mut claims = offline_token_claims();
    claims
      .as_object_mut()
      .unwrap()
      .extend(claims_override.as_object().unwrap().clone());
    let (token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &token)
      .await?;
    let secret_service =
      SecretServiceStub::default().with_app_reg_info(&AppRegInfoBuilder::test_default().build()?);
    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
    ));

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await;

    // Then
    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::TokenValidation(_))));
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
    "scope": SCOPE_TOKEN_USER
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
    );

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await;

    // Then
    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::TokenValidation(_))));
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
      "scope": SCOPE_TOKEN_USER
    });
    let (token, _) = build_token(claims)?;
    test_db_service
      .create_api_token_from("test_token", &token)
      .await?;
    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
    let mut auth_service = MockAuthService::new();
    auth_service
      .expect_exchange_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(token.clone()),
        eq(GRANT_REFRESH_TOKEN),
        eq(Vec::<String>::new()),
      )
      .times(1)
      .returning(|_, _, _, _, _| Ok(("new_access_token".to_string(), None)));
    let token_service = DefaultTokenService::new(
      Arc::new(auth_service),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
    );

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await?;

    // Then
    assert_eq!(result, "new_access_token");
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
      .expect_exchange_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(token.clone()),
        eq(GRANT_REFRESH_TOKEN),
        eq(Vec::<String>::new()),
      )
      .times(1)
      .returning(|_, _, _, _, _| {
        Err(AuthServiceError::AuthServiceApiError(
          "server unreachable".to_string(),
        ))
      });
    let token_service = DefaultTokenService::new(
      Arc::new(auth_service),
      Arc::new(secret_service),
      Arc::new(MokaCacheService::default()),
      Arc::new(test_db_service),
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
    let (token, _) = build_token(claims)?;
    let (access_token, _) = build_token(json! {{"exp": Utc::now().timestamp() + 3600}})?;
    let api_token = test_db_service
      .create_api_token_from("test-token", &token)
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
    );

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await?;

    // Then
    assert_eq!(result, access_token);
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
    let (token, _) = build_token(claims)?;
    let api_token = test_db_service
      .create_api_token_from("test_token", &token)
      .await?;

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
      .expect_exchange_token()
      .with(
        eq(TEST_CLIENT_ID),
        eq(TEST_CLIENT_SECRET),
        eq(token.clone()),
        eq(GRANT_REFRESH_TOKEN),
        eq(Vec::<String>::new()),
      )
      .times(1)
      .returning(|_, _, _, _, _| Ok(("new_access_token".to_string(), None)));

    let token_service = DefaultTokenService::new(
      Arc::new(mock_auth),
      Arc::new(secret_service),
      Arc::new(cache_service),
      Arc::new(test_db_service),
    );

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await?;

    // Then
    assert_eq!(result, "new_access_token");
    Ok(())
  }
}
