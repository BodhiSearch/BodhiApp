use crate::models::{Alias, DataServiceError, UserAlias, UserAliasRequest};
use crate::{AppService, AuthContext};
use std::sync::Arc;

/// Auth-scoped wrapper around DataService that injects tenant_id and user_id from AuthContext.
/// Read operations (list, find) use optional user_id — anonymous callers get empty string.
/// Write operations (save, copy, delete) require an authenticated user.
pub struct AuthScopedDataService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedDataService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  fn tenant_id_or_empty(&self) -> &str {
    self.auth_context.tenant_id().unwrap_or("")
  }

  fn user_id_or_empty(&self) -> &str {
    self.auth_context.user_id().unwrap_or("")
  }

  /// List all aliases visible to the current user (user + model + API aliases).
  pub async fn list_aliases(&self) -> Result<Vec<Alias>, DataServiceError> {
    let tenant_id = self.tenant_id_or_empty();
    let user_id = self.user_id_or_empty();
    self
      .app_service
      .data_service()
      .list_aliases(tenant_id, user_id)
      .await
  }

  /// Find an alias by name for the current user.
  pub async fn find_alias(&self, alias: &str) -> Option<Alias> {
    let tenant_id = self.tenant_id_or_empty();
    let user_id = self.user_id_or_empty();
    self
      .app_service
      .data_service()
      .find_alias(tenant_id, user_id, alias)
      .await
  }

  /// Find a user alias by name for the current user.
  pub async fn find_user_alias(&self, alias: &str) -> Option<UserAlias> {
    let tenant_id = self.tenant_id_or_empty();
    let user_id = self.user_id_or_empty();
    self
      .app_service
      .data_service()
      .find_user_alias(tenant_id, user_id, alias)
      .await
  }

  /// Get a user alias by ID for the current user.
  pub async fn get_user_alias_by_id(&self, id: &str) -> Option<UserAlias> {
    let tenant_id = self.tenant_id_or_empty();
    let user_id = self.user_id_or_empty();
    self
      .app_service
      .data_service()
      .get_user_alias_by_id(tenant_id, user_id, id)
      .await
  }

  /// Copy a user alias to a new name. Requires an authenticated user.
  pub async fn copy_alias(&self, id: &str, new_alias: &str) -> Result<UserAlias, DataServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .data_service()
      .copy_alias(tenant_id, user_id, id, new_alias)
      .await
  }

  /// Delete a user alias by ID. Requires an authenticated user.
  pub async fn delete_alias(&self, id: &str) -> Result<(), DataServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .data_service()
      .delete_alias(tenant_id, user_id, id)
      .await
  }

  /// Create a user alias from a form. Validates file existence and duplicates.
  /// Requires an authenticated user.
  pub async fn create_alias_from_form(
    &self,
    form: UserAliasRequest,
  ) -> Result<UserAlias, DataServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .data_service()
      .create_alias_from_form(tenant_id, user_id, form)
      .await
  }

  /// Update a user alias from a form. Validates file existence.
  /// Requires an authenticated user.
  pub async fn update_alias_from_form(
    &self,
    id: &str,
    form: UserAliasRequest,
  ) -> Result<UserAlias, DataServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .data_service()
      .update_alias_from_form(tenant_id, user_id, id, form)
      .await
  }
}
