use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use ulid::Ulid;

use super::ApiToken;
use crate::db::{DbService, TimeService};
use crate::tokens::TokenServiceError;
use crate::{TokenScope, TokenStatus};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait TokenService: Send + Sync + std::fmt::Debug {
  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), TokenServiceError>;

  /// Generate a new API token with cryptographic random bytes, hash it, and persist it.
  /// Returns the raw token string (for display to the user) and the persisted ApiToken.
  async fn create_token(
    &self,
    user_id: &str,
    name: String,
    scope: TokenScope,
  ) -> Result<(String, ApiToken), TokenServiceError>;

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), TokenServiceError>;

  async fn get_api_token_by_id(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, TokenServiceError>;

  async fn get_api_token_by_prefix(
    &self,
    prefix: &str,
  ) -> Result<Option<ApiToken>, TokenServiceError>;

  async fn update_api_token(
    &self,
    user_id: &str,
    token: &mut ApiToken,
  ) -> Result<(), TokenServiceError>;

  /// Fetch a token by id, update its name and status, and persist.
  async fn update_token(
    &self,
    user_id: &str,
    id: &str,
    name: String,
    status: TokenStatus,
  ) -> Result<ApiToken, TokenServiceError>;
}

#[derive(Debug, derive_new::new)]
pub struct DefaultTokenService {
  db_service: Arc<dyn DbService>,
  time_service: Arc<dyn TimeService>,
}

#[async_trait]
impl TokenService for DefaultTokenService {
  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), TokenServiceError> {
    self.db_service.create_api_token(token).await?;
    Ok(())
  }

  async fn create_token(
    &self,
    user_id: &str,
    name: String,
    scope: TokenScope,
  ) -> Result<(String, ApiToken), TokenServiceError> {
    // Generate cryptographically secure random token
    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let token_str = format!("bodhiapp_{}", random_string);

    // Extract prefix (first 8 chars after "bodhiapp_")
    let token_prefix = token_str[.."bodhiapp_".len() + 8].to_string();

    // Hash token with SHA-256
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let now = self.time_service.utc_now();

    let mut api_token = ApiToken {
      id: Ulid::new().to_string(),
      user_id: user_id.to_string(),
      name,
      token_prefix,
      token_hash,
      scopes: scope.to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };

    self.db_service.create_api_token(&mut api_token).await?;

    Ok((token_str, api_token))
  }

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), TokenServiceError> {
    Ok(
      self
        .db_service
        .list_api_tokens(user_id, page, per_page)
        .await?,
    )
  }

  async fn get_api_token_by_id(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, TokenServiceError> {
    Ok(self.db_service.get_api_token_by_id(user_id, id).await?)
  }

  async fn get_api_token_by_prefix(
    &self,
    prefix: &str,
  ) -> Result<Option<ApiToken>, TokenServiceError> {
    Ok(self.db_service.get_api_token_by_prefix(prefix).await?)
  }

  async fn update_api_token(
    &self,
    user_id: &str,
    token: &mut ApiToken,
  ) -> Result<(), TokenServiceError> {
    self.db_service.update_api_token(user_id, token).await?;
    Ok(())
  }

  async fn update_token(
    &self,
    user_id: &str,
    id: &str,
    name: String,
    status: TokenStatus,
  ) -> Result<ApiToken, TokenServiceError> {
    let mut token = self
      .get_api_token_by_id(user_id, id)
      .await?
      .ok_or_else(|| crate::EntityError::NotFound("Token".to_string()))?;
    token.name = name;
    token.status = status;
    self.update_api_token(user_id, &mut token).await?;
    Ok(token)
  }
}

#[cfg(test)]
#[path = "test_token_service.rs"]
mod test_token_service;
