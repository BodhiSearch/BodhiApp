use crate::{
  AppService, AuthContext, CreateTokenRequest, PaginatedTokenResponse, TokenCreated, TokenDetail,
  TokenServiceError, UpdateTokenRequest,
};
use std::sync::Arc;

pub struct AuthScopedTokenService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedTokenService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  /// List API tokens for the authenticated user
  pub async fn list_tokens(
    &self,
    page: usize,
    per_page: usize,
  ) -> Result<PaginatedTokenResponse, TokenServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .list_api_tokens(tenant_id, user_id, page, per_page)
      .await
  }

  /// Get a specific API token by id for the authenticated user
  pub async fn get_token(&self, id: &str) -> Result<Option<TokenDetail>, TokenServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .get_api_token_by_id(tenant_id, user_id, id)
      .await
  }

  /// Create an API token for the authenticated user.
  /// Delegates token generation (random bytes, hashing, ULID) to TokenService.
  pub async fn create_token(
    &self,
    request: CreateTokenRequest,
  ) -> Result<TokenCreated, TokenServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .create_token(tenant_id, user_id, request)
      .await
  }

  /// Update an existing API token (name and status) for the authenticated user.
  pub async fn update_token(
    &self,
    id: &str,
    request: UpdateTokenRequest,
  ) -> Result<TokenDetail, TokenServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .update_token(tenant_id, user_id, id, request)
      .await
  }
}
