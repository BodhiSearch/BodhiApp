use crate::AuthError;
use jsonwebtoken::{DecodingKey, Validation};
use services::{
  AppRegInfo, AuthService, Claims, JsonWebTokenError, OfflineClaims, SecretService,
  SecretServiceExt, GRANT_REFRESH_TOKEN, TOKEN_TYPE_OFFLINE,
};
use std::sync::Arc;
use tower_sessions::Session;

const BEARER_PREFIX: &str = "Bearer ";
const SCOPE_TOKEN_USER: &str = "scope_token_user";

pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
}

impl DefaultTokenService {
  pub fn new(auth_service: Arc<dyn AuthService>, secret_service: Arc<dyn SecretService>) -> Self {
    Self {
      auth_service,
      secret_service,
    }
  }

  pub async fn validate_bearer_token(&self, header: &str) -> Result<String, AuthError> {
    // Extract token from header
    let token = header
      .strip_prefix(BEARER_PREFIX)
      .ok_or_else(|| AuthError::TokenValidation("authorization header is malformed".to_string()))?
      .trim();
    if token.is_empty() {
      return Err(AuthError::TokenValidation(
        "token not found in authorization header".to_string(),
      ));
    }

    // Get app registration info
    let app_reg_info: AppRegInfo = self
      .secret_service
      .app_reg_info()?
      .ok_or(AuthError::AppRegInfoMissing)?;

    // Validate token signature and decode claims
    let claims = self.validate_token_signature(token, &app_reg_info)?;

    // Validate claims
    self.validate_token_claims(&claims, &app_reg_info.client_id)?;

    // Exchange token for access token
    let (access_token, _) = self
      .auth_service
      .exchange_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        token,
        GRANT_REFRESH_TOKEN,
        vec![],
      )
      .await?;

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
    validation.validate_aud = false;

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
    let now = time::OffsetDateTime::now_utc();
    if now.unix_timestamp() < claims.claims.exp as i64 {
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
  use crate::{AuthError, DefaultTokenService};
  use mockall::predicate::*;
  use objs::test_utils::setup_l10n;
  use objs::FluentLocalizationService;
  use rstest::{fixture, rstest};
  use serde_json::json;
  use services::{
    test_utils::{
      build_token, offline_token_cliams, SecretServiceStub, TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    },
    AppRegInfoBuilder, MockAuthService, MockSecretService, GRANT_REFRESH_TOKEN,
  };
  use std::sync::Arc;

  #[fixture]
  fn token_service() -> Arc<DefaultTokenService> {
    Arc::new(DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(MockSecretService::default()),
    ))
  }

  #[rstest]
  #[case::empty("")]
  #[case::malformed("bearer foobar")]
  #[case::empty_bearer("Bearer ")]
  #[case::empty_bearer_2("Bearer  ")]
  #[tokio::test]
  async fn test_validate_bearer_token_header_errors(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] header: &str,
    token_service: Arc<DefaultTokenService>,
  ) -> anyhow::Result<()> {
    let result = token_service.validate_bearer_token(header).await;
    assert!(result.is_err());
    assert_eq!(matches!(result, Err(AuthError::TokenValidation(_))), true);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_validate_bearer_token_success(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    // Given
    let claims = offline_token_cliams();
    let (token, _) = build_token(claims)?;
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
  #[tokio::test]
  async fn test_validate_bearer_token_validation_errors(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] claims_override: serde_json::Value,
  ) -> anyhow::Result<()> {
    // Given
    let secret_service =
      SecretServiceStub::default().with_app_reg_info(&AppRegInfoBuilder::test_default().build()?);
    let token_service = Arc::new(DefaultTokenService::new(
      Arc::new(MockAuthService::default()),
      Arc::new(secret_service),
    ));
    let mut claims = offline_token_cliams();
    claims
      .as_object_mut()
      .unwrap()
      .extend(claims_override.as_object().unwrap().clone());
    let (token, _) = build_token(claims)?;

    // When
    let result = token_service
      .validate_bearer_token(&format!("Bearer {}", token))
      .await;

    // Then
    assert!(result.is_err());
    assert!(matches!(result, Err(AuthError::TokenValidation(_))));
    Ok(())
  }
}
