use crate::AuthError;
use axum::http::{header::AUTHORIZATION, HeaderMap};
use jsonwebtoken::{DecodingKey, Validation};
use objs::BadRequestError;
use services::{AppRegInfo, AuthService, Claims, SecretService, SecretServiceExt};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait TokenService {
  fn extract_token(&self, headers: &HeaderMap) -> Result<String, AuthError>;
  fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
  async fn exchange_token(&self, token: &str) -> Result<(String, Option<String>), AuthError>;
}

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

  fn get_app_reg_info(&self) -> Result<AppRegInfo, AuthError> {
    self
      .secret_service
      .app_reg_info()?
      .ok_or(AuthError::AppRegInfoMissing)
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
}

#[cfg(test)]
mod tests {
  use crate::{DefaultTokenService, TokenService};
  use axum::http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
  use services::{MockAuthService, MockSecretService};
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
    DefaultTokenService::new(app_service, secret_service)
  }
}
