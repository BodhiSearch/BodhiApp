use crate::auth::AuthContextError;
use crate::db::{DbError, DbService};
use crate::{AppService, AuthContext, UserAccessRequestEntity, UserAccessRequestStatus};
use std::sync::Arc;

/// Auth-scoped wrapper around user access request DB operations.
/// Injects tenant_id from AuthContext automatically.
pub struct AuthScopedUserAccessRequestService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

/// Error type for auth-scoped user access request operations.
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = errmeta::AppError)]
pub enum AuthScopedUserAccessRequestError {
  #[error(transparent)]
  #[error_meta(error_type = errmeta::ErrorType::InternalServer, args_delegate = false)]
  AuthContext(#[from] AuthContextError),

  #[error(transparent)]
  #[error_meta(error_type = errmeta::ErrorType::InternalServer, args_delegate = false)]
  Db(#[from] DbError),
}

impl AuthScopedUserAccessRequestService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  fn db(&self) -> Arc<dyn DbService> {
    self.app_service.db_service()
  }

  fn tenant_id_or_empty(&self) -> &str {
    self.auth_context.tenant_id().unwrap_or("")
  }

  /// Insert a pending access request for the current tenant.
  pub async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequestEntity, AuthScopedUserAccessRequestError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let result = self
      .db()
      .insert_pending_request(tenant_id, username, user_id)
      .await?;
    Ok(result)
  }

  /// Get a pending access request for a user in the current tenant.
  pub async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequestEntity>, DbError> {
    let tenant_id = self.tenant_id_or_empty();
    self.db().get_pending_request(tenant_id, user_id).await
  }

  /// List pending access requests for the current tenant with pagination.
  pub async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError> {
    let tenant_id = self.tenant_id_or_empty();
    self
      .db()
      .list_pending_requests(tenant_id, page, per_page)
      .await
  }

  /// List all access requests for the current tenant with pagination.
  pub async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError> {
    let tenant_id = self.tenant_id_or_empty();
    self.db().list_all_requests(tenant_id, page, per_page).await
  }

  /// Get an access request by ID, scoped to current tenant.
  pub async fn get_request_by_id(
    &self,
    id: &str,
  ) -> Result<Option<UserAccessRequestEntity>, DbError> {
    let tenant_id = self.tenant_id_or_empty();
    self.db().get_request_by_id(tenant_id, id).await
  }

  /// Update the status of an access request.
  pub async fn update_request_status(
    &self,
    id: &str,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), AuthScopedUserAccessRequestError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .db()
      .update_request_status(tenant_id, id, status, reviewer)
      .await?;
    Ok(())
  }
}
