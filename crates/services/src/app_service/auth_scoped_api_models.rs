use crate::{
  ApiModelOutput, ApiModelRequest, ApiModelServiceError, AppService, AuthContext,
  PaginatedApiModelOutput,
};
use std::sync::Arc;

/// Auth-scoped wrapper around ApiModelService that injects tenant_id and user_id from AuthContext.
pub struct AuthScopedApiModelService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedApiModelService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  /// Create a new API model configuration
  pub async fn create(
    &self,
    form: ApiModelRequest,
  ) -> Result<ApiModelOutput, ApiModelServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .api_model_service()
      .create(tenant_id, user_id, form)
      .await
  }

  /// Update an existing API model configuration
  pub async fn update(
    &self,
    id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiModelOutput, ApiModelServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .api_model_service()
      .update(tenant_id, user_id, id, form)
      .await
  }

  /// Delete an API model configuration
  pub async fn delete(&self, id: &str) -> Result<(), ApiModelServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .api_model_service()
      .delete(tenant_id, user_id, id)
      .await
  }

  /// Get a specific API model configuration
  pub async fn get(&self, id: &str) -> Result<ApiModelOutput, ApiModelServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .api_model_service()
      .get(tenant_id, user_id, id)
      .await
  }

  /// List API model configurations with pagination
  pub async fn list(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<PaginatedApiModelOutput, ApiModelServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .api_model_service()
      .list(tenant_id, user_id, page, page_size)
      .await
  }

  /// Synchronously fetch and cache models for an API model alias
  pub async fn sync_cache(&self, id: &str) -> Result<ApiModelOutput, ApiModelServiceError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .api_model_service()
      .sync_cache(tenant_id, user_id, id)
      .await
  }
}
