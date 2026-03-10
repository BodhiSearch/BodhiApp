use crate::auth::AuthContextError;
use crate::auth::AuthServiceError;
use crate::tenants::TenantError;
use crate::{AppService, AuthContext, UserListResponse};
use errmeta::AppError;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthScopedUserError {
  #[error(transparent)]
  Auth(#[from] AuthServiceError),
  #[error(transparent)]
  Tenant(#[from] TenantError),
  #[error(transparent)]
  AuthContext(#[from] AuthContextError),
}

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
  ) -> Result<UserListResponse, AuthScopedUserError> {
    let token = self.require_token()?;
    Ok(
      self
        .app_service
        .auth_service()
        .list_users(token, page, page_size)
        .await?,
    )
  }

  /// Assign a role to a user. Injects the reviewer's token.
  /// Also upserts tenant-user membership in the local DB.
  pub async fn assign_user_role(
    &self,
    target_user_id: &str,
    role: &str,
  ) -> Result<(), AuthScopedUserError> {
    let token = self.require_token()?;
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .auth_service()
      .assign_user_role(token, target_user_id, role)
      .await?;
    self
      .app_service
      .tenant_service()
      .upsert_tenant_user(tenant_id, target_user_id)
      .await?;
    Ok(())
  }

  /// Remove a user. Injects the reviewer's token.
  /// Also removes tenant-user membership from the local DB.
  pub async fn remove_user(&self, target_user_id: &str) -> Result<(), AuthScopedUserError> {
    let token = self.require_token()?;
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .auth_service()
      .remove_user(token, target_user_id)
      .await?;
    self
      .app_service
      .tenant_service()
      .delete_tenant_user(tenant_id, target_user_id)
      .await?;
    Ok(())
  }
}
