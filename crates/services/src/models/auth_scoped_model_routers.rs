use crate::{AppService, AuthContext, ModelRouterError, ModelRouterRequest, ModelRouterResponse};
use std::sync::Arc;

/// Auth-scoped wrapper around ModelRouterService that injects tenant_id and user_id from AuthContext.
pub struct AuthScopedModelRouterService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedModelRouterService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  pub async fn create(
    &self,
    form: ModelRouterRequest,
  ) -> Result<ModelRouterResponse, ModelRouterError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .model_router_service()
      .create(tenant_id, user_id, form)
      .await
  }

  pub async fn update(
    &self,
    id: &str,
    form: ModelRouterRequest,
  ) -> Result<ModelRouterResponse, ModelRouterError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .model_router_service()
      .update(tenant_id, user_id, id, form)
      .await
  }

  pub async fn delete(&self, id: &str) -> Result<(), ModelRouterError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .model_router_service()
      .delete(tenant_id, user_id, id)
      .await
  }

  pub async fn get(&self, id: &str) -> Result<ModelRouterResponse, ModelRouterError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .model_router_service()
      .get(tenant_id, user_id, id)
      .await
  }
}
