use crate::{ApiToken, AppService, AuthContext, TokenScope, TokenServiceError, TokenStatus};
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
  ) -> Result<(Vec<ApiToken>, usize), TokenServiceError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .list_api_tokens(user_id, page, per_page)
      .await
  }

  /// Get a specific API token by id for the authenticated user
  pub async fn get_token(&self, id: &str) -> Result<Option<ApiToken>, TokenServiceError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .get_api_token_by_id(user_id, id)
      .await
  }

  /// Create an API token for the authenticated user.
  /// Delegates token generation (random bytes, hashing, ULID) to TokenService.
  pub async fn create_token(
    &self,
    name: String,
    scope: TokenScope,
  ) -> Result<(String, ApiToken), TokenServiceError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .create_token(user_id, name, scope)
      .await
  }

  /// Update an existing API token (name and status) for the authenticated user.
  pub async fn update_token(
    &self,
    id: &str,
    name: String,
    status: TokenStatus,
  ) -> Result<ApiToken, TokenServiceError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .token_service()
      .update_token(user_id, id, name, status)
      .await
  }
}
