use crate::{
  AppService, AppToolsetConfig, AuthContext, ToolsetDefinition, ToolsetEntity, ToolsetError,
  ToolsetExecutionRequest, ToolsetExecutionResponse, ToolsetRequest,
};
use std::sync::Arc;

/// Auth-scoped wrapper around ToolService that injects user_id and tenant_id from AuthContext.
/// User-scoped methods (list, get, create, update, delete, execute) automatically
/// inject the authenticated user's ID and tenant ID.
/// Admin methods (set_app_toolset_enabled) inject user_id as the `updated_by` field.
pub struct AuthScopedToolService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedToolService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  /// List all toolsets for the authenticated user.
  pub async fn list(&self) -> Result<Vec<ToolsetEntity>, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let toolsets = self
      .app_service
      .tool_service()
      .list(tenant_id, user_id)
      .await?;
    Ok(toolsets)
  }

  /// Get a specific toolset by ID for the authenticated user.
  pub async fn get(&self, id: &str) -> Result<Option<ToolsetEntity>, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let toolset = self
      .app_service
      .tool_service()
      .get(tenant_id, user_id, id)
      .await?;
    Ok(toolset)
  }

  /// Create a new toolset for the authenticated user.
  pub async fn create(&self, request: ToolsetRequest) -> Result<ToolsetEntity, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let toolset = self
      .app_service
      .tool_service()
      .create(tenant_id, user_id, request)
      .await?;
    Ok(toolset)
  }

  /// Update an existing toolset for the authenticated user.
  pub async fn update(
    &self,
    id: &str,
    request: ToolsetRequest,
  ) -> Result<ToolsetEntity, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let toolset = self
      .app_service
      .tool_service()
      .update(tenant_id, user_id, id, request)
      .await?;
    Ok(toolset)
  }

  /// Delete a toolset for the authenticated user.
  pub async fn delete(&self, id: &str) -> Result<(), ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .tool_service()
      .delete(tenant_id, user_id, id)
      .await?;
    Ok(())
  }

  /// Execute a tool on a toolset owned by the authenticated user.
  pub async fn execute(
    &self,
    id: &str,
    tool_name: &str,
    request: ToolsetExecutionRequest,
  ) -> Result<ToolsetExecutionResponse, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let response = self
      .app_service
      .tool_service()
      .execute(tenant_id, user_id, id, tool_name, request)
      .await?;
    Ok(response)
  }

  /// Enable or disable a toolset type at app level.
  /// Injects the authenticated user's ID as the `updated_by` field.
  pub async fn set_app_toolset_enabled(
    &self,
    toolset_type: &str,
    enabled: bool,
  ) -> Result<AppToolsetConfig, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let updated_by = self.auth_context.require_user_id()?;
    let config = self
      .app_service
      .tool_service()
      .set_app_toolset_enabled(tenant_id, toolset_type, enabled, updated_by)
      .await?;
    Ok(config)
  }

  // ========== Pass-through methods (no user_id needed) ==========

  /// List all available toolset types.
  pub fn list_types(&self) -> Vec<ToolsetDefinition> {
    self.app_service.tool_service().list_types()
  }

  /// Get a toolset type by identifier.
  pub fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition> {
    self.app_service.tool_service().get_type(toolset_type)
  }

  /// Validate a toolset type identifier.
  pub fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError> {
    self.app_service.tool_service().validate_type(toolset_type)
  }

  /// List app-level toolset configurations.
  pub async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfig>, ToolsetError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let configs = self
      .app_service
      .tool_service()
      .list_app_toolset_configs(tenant_id)
      .await?;
    Ok(configs)
  }
}
