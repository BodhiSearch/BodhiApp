use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::new_ulid;

use super::TokenEntity;
use crate::db::{DbService, TimeService};
use crate::tokens::{
  default_grants_json, token_checksum, CreateTokenRequest, PaginatedTokenResponse, TokenCreated,
  TokenDetail, TokenServiceError, UpdateTokenRequest, BODHIAPP_TOKEN_PREFIX,
};
use crate::TokenStatus;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait TokenService: Send + Sync + std::fmt::Debug {
  async fn create_api_token(
    &self,
    tenant_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), TokenServiceError>;

  /// Generate a new API token with cryptographic random bytes, hash it, and persist it.
  /// Returns a `TokenCreated` containing the raw token string (shown once to the user).
  async fn create_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    request: CreateTokenRequest,
  ) -> Result<TokenCreated, TokenServiceError>;

  async fn list_api_tokens(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<PaginatedTokenResponse, TokenServiceError>;

  async fn get_api_token_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<TokenDetail>, TokenServiceError>;

  async fn get_api_token_by_prefix(
    &self,
    prefix: &str,
  ) -> Result<Option<TokenEntity>, TokenServiceError>;

  async fn update_api_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), TokenServiceError>;

  /// Fetch a token by id, update its name and status, and persist.
  async fn update_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    request: UpdateTokenRequest,
  ) -> Result<TokenDetail, TokenServiceError>;

  /// Permanently delete a token owned by the user. Errors if it does not exist.
  async fn delete_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), TokenServiceError>;
}

#[derive(Debug, derive_new::new)]
pub struct DefaultTokenService {
  db_service: Arc<dyn DbService>,
  time_service: Arc<dyn TimeService>,
}

#[async_trait]
impl TokenService for DefaultTokenService {
  async fn create_api_token(
    &self,
    tenant_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), TokenServiceError> {
    self.db_service.create_api_token(tenant_id, token).await?;
    Ok(())
  }

  async fn create_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    request: CreateTokenRequest,
  ) -> Result<TokenCreated, TokenServiceError> {
    // Look up tenant to get client_id for token suffix
    let tenant_row = self
      .db_service
      .get_tenant_by_id(tenant_id)
      .await
      .map_err(TokenServiceError::Db)?;
    let client_id = tenant_row
      .map(|r| r.client_id)
      .unwrap_or_else(|| tenant_id.to_string());

    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let checksum = token_checksum(&random_string);
    // Format: sk-bodhiapp_<random><checksum>.<client_id> for tenant-scoped tokens
    let token_str = format!(
      "{}{}{}.{}",
      BODHIAPP_TOKEN_PREFIX, random_string, checksum, client_id
    );

    // Prefix stored for DB lookup = constant prefix + first 8 chars of the random
    // part (before the checksum), independent of checksum length.
    let token_prefix = format!("{}{}", BODHIAPP_TOKEN_PREFIX, &random_string[..8]);

    // Hash full token with SHA-256 (including .<client_id> suffix)
    let mut hasher = Sha256::new();
    hasher.update(token_str.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let now = self.time_service.utc_now();

    let mut api_token = TokenEntity {
      id: new_ulid(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      name: request.name.unwrap_or_default(),
      token_prefix,
      token_hash,
      scopes: request.scope.to_string(),
      status: TokenStatus::Active,
      grants: serde_json::to_string(&request.grants).unwrap_or_else(|_| default_grants_json()),
      last_used_at: None,
      created_at: now,
      updated_at: now,
    };

    self
      .db_service
      .create_api_token(tenant_id, &mut api_token)
      .await?;

    Ok(TokenCreated { token: token_str })
  }

  async fn list_api_tokens(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<PaginatedTokenResponse, TokenServiceError> {
    let (tokens, total) = self
      .db_service
      .list_api_tokens(tenant_id, user_id, page, per_page)
      .await?;
    Ok(PaginatedTokenResponse {
      data: tokens.into_iter().map(TokenDetail::from).collect(),
      total,
      page,
      page_size: per_page,
    })
  }

  async fn get_api_token_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<TokenDetail>, TokenServiceError> {
    Ok(
      self
        .db_service
        .get_api_token_by_id(tenant_id, user_id, id)
        .await?
        .map(TokenDetail::from),
    )
  }

  async fn get_api_token_by_prefix(
    &self,
    prefix: &str,
  ) -> Result<Option<TokenEntity>, TokenServiceError> {
    Ok(self.db_service.get_api_token_by_prefix(prefix).await?)
  }

  async fn update_api_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), TokenServiceError> {
    self
      .db_service
      .update_api_token(tenant_id, user_id, token)
      .await?;
    Ok(())
  }

  async fn update_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    request: UpdateTokenRequest,
  ) -> Result<TokenDetail, TokenServiceError> {
    let mut token = self
      .db_service
      .get_api_token_by_id(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| crate::EntityError::NotFound("Token".to_string()))?;
    token.name = request.name;
    token.status = request.status;
    self
      .update_api_token(tenant_id, user_id, &mut token)
      .await?;
    Ok(TokenDetail::from(token))
  }

  async fn delete_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), TokenServiceError> {
    // Existence check yields a 404 consistent with update_token (DbError::ItemNotFound
    // would otherwise surface as a 500). The repository delete still guards the race.
    self
      .db_service
      .get_api_token_by_id(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| crate::EntityError::NotFound("Token".to_string()))?;
    self
      .db_service
      .delete_api_token(tenant_id, user_id, id)
      .await?;
    Ok(())
  }
}

#[cfg(test)]
#[path = "test_token_service.rs"]
mod test_token_service;
