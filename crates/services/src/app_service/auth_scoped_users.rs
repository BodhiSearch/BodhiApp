use crate::auth::AuthServiceError;
use crate::{AppService, AuthContext, UserListResponse};
use crate::auth::AuthContextError;
use std::sync::Arc;

/// Auth-scoped wrapper around AuthService that injects the reviewer's token from AuthContext.
/// All methods automatically inject the authenticated user's token as the `reviewer_token` param.
pub struct AuthScopedUserService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedUserService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  fn require_token(&self) -> Result<&str, AuthContextError> {
    self
      .auth_context
      .token()
      .ok_or(AuthContextError::MissingToken)
  }

  /// List all users. Injects the reviewer's token.
  pub async fn list_users(
    &self,
    page: Option<u32>,
    page_size: Option<u32>,
  ) -> Result<UserListResponse, AuthServiceError> {
    let token = self.require_token()?;
    self
      .app_service
      .auth_service()
      .list_users(token, page, page_size)
      .await
  }

  /// Assign a role to a user. Injects the reviewer's token.
  pub async fn assign_user_role(
    &self,
    target_user_id: &str,
    role: &str,
  ) -> Result<(), AuthServiceError> {
    let token = self.require_token()?;
    self
      .app_service
      .auth_service()
      .assign_user_role(token, target_user_id, role)
      .await
  }

  /// Remove a user. Injects the reviewer's token.
  pub async fn remove_user(&self, target_user_id: &str) -> Result<(), AuthServiceError> {
    let token = self.require_token()?;
    self
      .app_service
      .auth_service()
      .remove_user(token, target_user_id)
      .await
  }
}
