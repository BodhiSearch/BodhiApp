use crate::AuthError;
use jsonwebtoken::{DecodingKey, Validation};
use services::{
  AppRegInfo, AuthService, Claims, JsonWebTokenError, SecretService, SecretServiceExt,
};
use std::sync::Arc;
use tower_sessions::Session;

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
