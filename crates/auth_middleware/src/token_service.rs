use crate::{AuthError, DefaultTokenCache, TokenCache};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use jsonwebtoken::{DecodingKey, Validation};
use objs::BadRequestError;
use services::{
  AppRegInfo, AuthService, CacheService, Claims, JsonWebTokenError, SecretService, SecretServiceExt,
};
use std::sync::Arc;
use tower_sessions::Session;

#[async_trait::async_trait]
pub trait TokenService {
  fn extract_token(&self, headers: &HeaderMap) -> Result<String, AuthError>;
  fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
  async fn exchange_token(&self, token: &str) -> Result<(String, Option<String>), AuthError>;
  fn decode_access_token_no_validation(
    &self,
    token: &str,
  ) -> Result<jsonwebtoken::TokenData<Claims>, JsonWebTokenError>;
}

pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
}

impl DefaultTokenService {
  pub fn new(
    auth_service: Arc<dyn AuthService>,
    secret_service: Arc<dyn SecretService>,
    cache_service: Arc<dyn CacheService>,
  ) -> Self {
    Self {
      auth_service,
      secret_service,
      cache_service,
    }
  }

  fn get_app_reg_info(&self) -> Result<AppRegInfo, AuthError> {
    self
      .secret_service
      .app_reg_info()?
      .ok_or(AuthError::AppRegInfoMissing)
  }

  pub async fn get_valid_session_token(
    &self,
    session: Session,
    access_token: String,
  ) -> Result<String, AuthError> {
    // Validate session token
    let claims = self.decode_access_token_no_validation(&access_token)?;
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

  pub async fn get_valid_bearer_token(&self, headers: HeaderMap) -> Result<String, AuthError> {
    let request_token = self.extract_token(&headers)?;
    let claims = self.decode_access_token_no_validation(&request_token)?;
    let token_cache = DefaultTokenCache::new(self.cache_service.clone());
    if token_cache.is_token_in_cache(&claims.claims.jti, &request_token)? {
      return Ok(request_token);
    }
    let claims = self.validate_token(&request_token)?;
    let (access_token, refresh_token) = self.exchange_token(&request_token).await?;
    token_cache.store_token_pair(&claims.jti, &access_token, refresh_token);
    Ok(access_token)
  }
}

#[async_trait::async_trait]
impl TokenService for DefaultTokenService {
  fn extract_token(&self, headers: &HeaderMap) -> Result<String, AuthError> {
    let header = headers
      .get(AUTHORIZATION)
      .ok_or(AuthError::AuthHeaderNotFound)?;

    let token = header
      .to_str()
      .map_err(|e| {
        AuthError::BadRequest(BadRequestError::new(format!(
          "authorization header is not valid utf-8: {e}"
        )))
      })?
      .strip_prefix("Bearer ")
      .ok_or_else(|| {
        AuthError::BadRequest(BadRequestError::new(
          "authorization header is malformed".to_string(),
        ))
      })?
      .to_string();

    if token.is_empty() {
      return Err(AuthError::BadRequest(BadRequestError::new(
        "token not found in authorization header".to_string(),
      )));
    }

    Ok(token)
  }

  fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
    let app_reg_info = self.get_app_reg_info()?;
    let header = jsonwebtoken::decode_header(token)?;

    // Validate KID
    if header.kid != Some(app_reg_info.kid.clone()) {
      return Err(AuthError::KidMismatch(
        app_reg_info.kid,
        header.kid.unwrap_or_default(),
      ));
    }

    // Validate algorithm
    if header.alg != app_reg_info.alg {
      return Err(AuthError::AlgMismatch(
        format!("{:?}", app_reg_info.alg),
        format!("{:?}", header.alg),
      ));
    }

    // Setup validation
    let key_pem = format!(
      "-----BEGIN RSA PUBLIC KEY-----\n{}\n-----END RSA PUBLIC KEY-----",
      app_reg_info.public_key
    );
    let key = DecodingKey::from_rsa_pem(key_pem.as_bytes())?;
    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[app_reg_info.issuer]);
    validation.validate_aud = false;

    // Validate token
    let token_data = jsonwebtoken::decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
  }

  async fn exchange_token(&self, token: &str) -> Result<(String, Option<String>), AuthError> {
    let app_reg_info = self.get_app_reg_info()?;

    let (access_token, refresh_token) = self
      .auth_service
      .exchange_token(
        &app_reg_info.client_id,
        &app_reg_info.client_secret,
        token,
        "urn:ietf:params:oauth:token-type:refresh_token",
        vec!["openid".to_string(), "scope_user".to_string()],
      )
      .await?;

    Ok((access_token, refresh_token))
  }

  fn decode_access_token_no_validation(
    &self,
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
  use crate::{DefaultTokenService, TokenService};
  use axum::http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
  use services::{MockAuthService, MockSecretService, MokaCacheService};
  use std::sync::Arc;

  #[tokio::test]
  async fn test_extract_token_valid() {
    let service = create_test_service();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer test-token"));
    let result = service.extract_token(&headers);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-token");
  }

  #[tokio::test]
  async fn test_extract_token_invalid_format() {
    let service = create_test_service();
    let mut headers = HeaderMap::new();
    headers.insert(
      AUTHORIZATION,
      HeaderValue::from_static("InvalidFormat test-token"),
    );
    let result = service.extract_token(&headers);
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_extract_token_empty() {
    let service = create_test_service();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));
    let result = service.extract_token(&headers);
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_extract_token_missing_header() {
    let service = create_test_service();
    let headers = HeaderMap::new();
    let result = service.extract_token(&headers);
    assert!(result.is_err());
  }

  // Helper function to create test service
  fn create_test_service() -> DefaultTokenService {
    let app_service = Arc::new(MockAuthService::default());
    let secret_service = Arc::new(MockSecretService::default());
    DefaultTokenService::new(
      app_service,
      secret_service,
      Arc::new(MokaCacheService::default()),
    )
  }
}
