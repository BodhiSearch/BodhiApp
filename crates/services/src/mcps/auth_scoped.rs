use crate::{
  AppService, AuthContext, CreateMcpAuthConfigRequest, McpAuthConfigResponse, McpAuthParamInput,
  McpError, McpExecutionRequest, McpExecutionResponse, McpOAuthToken, McpRequest, McpServerEntity,
  McpServerError, McpServerRequest, McpTool, McpWithServerEntity,
};
use std::sync::Arc;

/// Auth-scoped wrapper around McpService that injects user_id and tenant_id from AuthContext.
/// User-scoped methods automatically inject the authenticated user's ID and tenant ID.
/// Server-level and discovery methods delegate directly (no user_id needed).
pub struct AuthScopedMcpService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

impl AuthScopedMcpService {
  pub fn new(app_service: Arc<dyn AppService>, auth_context: AuthContext) -> Self {
    Self {
      app_service,
      auth_context,
    }
  }

  // ========== User-scoped MCP instance methods ==========

  /// List MCP instances for the authenticated user.
  pub async fn list(&self) -> Result<Vec<McpWithServerEntity>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .list(tenant_id, user_id)
      .await
  }

  /// Get a specific MCP instance by ID for the authenticated user.
  pub async fn get(&self, id: &str) -> Result<Option<McpWithServerEntity>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .get(tenant_id, user_id, id)
      .await
  }

  /// Create a new MCP instance for the authenticated user.
  pub async fn create(&self, request: McpRequest) -> Result<McpWithServerEntity, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .create(tenant_id, user_id, request)
      .await
  }

  /// Update an existing MCP instance for the authenticated user.
  pub async fn update(
    &self,
    id: &str,
    request: McpRequest,
  ) -> Result<McpWithServerEntity, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .update(tenant_id, user_id, id, request)
      .await
  }

  /// Delete an MCP instance for the authenticated user.
  pub async fn delete(&self, id: &str) -> Result<(), McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .delete(tenant_id, user_id, id)
      .await?;
    Ok(())
  }

  /// Fetch and refresh tools for an MCP instance owned by the authenticated user.
  pub async fn fetch_tools(&self, id: &str) -> Result<Vec<McpTool>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let tools = self
      .app_service
      .mcp_service()
      .fetch_tools(tenant_id, user_id, id)
      .await?;
    Ok(tools)
  }

  /// Execute a tool on an MCP instance owned by the authenticated user.
  pub async fn execute(
    &self,
    id: &str,
    tool_name: &str,
    request: McpExecutionRequest,
  ) -> Result<McpExecutionResponse, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    let response = self
      .app_service
      .mcp_service()
      .execute(tenant_id, user_id, id, tool_name, request)
      .await?;
    Ok(response)
  }

  // ========== Auth config methods (user-scoped) ==========

  /// Create an auth config for the authenticated user.
  pub async fn create_auth_config(
    &self,
    mcp_server_id: &str,
    request: CreateMcpAuthConfigRequest,
  ) -> Result<McpAuthConfigResponse, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .create_auth_config(tenant_id, user_id, mcp_server_id, request)
      .await
  }

  /// Delete an auth config. No authorization check — middleware handles access control.
  pub async fn delete_auth_config(&self, config_id: &str) -> Result<(), McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .delete_auth_config(tenant_id, config_id)
      .await
  }

  // ========== OAuth token methods (user-scoped) ==========

  /// Get an OAuth token by ID for the authenticated user.
  pub async fn get_oauth_token(&self, token_id: &str) -> Result<Option<McpOAuthToken>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .get_oauth_token(tenant_id, user_id, token_id)
      .await
  }

  /// Delete an OAuth token for the authenticated user.
  pub async fn delete_oauth_token(&self, token_id: &str) -> Result<(), McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .delete_oauth_token(tenant_id, user_id, token_id)
      .await
  }

  /// Exchange an authorization code for tokens.
  pub async fn exchange_oauth_token(
    &self,
    mcp_id: Option<String>,
    config_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
  ) -> Result<McpOAuthToken, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .exchange_oauth_token(
        tenant_id,
        user_id,
        mcp_id,
        config_id,
        code,
        redirect_uri,
        code_verifier,
      )
      .await
  }

  // ========== Auth config read methods (direct delegation) ==========

  /// List all auth configs for a server.
  pub async fn list_auth_configs(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigResponse>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .list_auth_configs(tenant_id, mcp_server_id)
      .await
  }

  /// Get an auth config by ID.
  pub async fn get_auth_config(
    &self,
    config_id: &str,
  ) -> Result<Option<McpAuthConfigResponse>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .get_auth_config(tenant_id, config_id)
      .await
  }

  /// Get an OAuth config by ID.
  pub async fn get_oauth_config(
    &self,
    config_id: &str,
  ) -> Result<Option<crate::McpOAuthConfig>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .get_oauth_config(tenant_id, config_id)
      .await
  }

  // ========== Server-level methods (Part C) ==========

  /// Create a new MCP server. Injects user_id as created_by.
  pub async fn create_mcp_server(
    &self,
    request: McpServerRequest,
  ) -> Result<McpServerEntity, McpServerError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .create_mcp_server(tenant_id, user_id, request)
      .await
  }

  /// Update an MCP server. Injects user_id as updated_by.
  pub async fn update_mcp_server(
    &self,
    id: &str,
    request: McpServerRequest,
  ) -> Result<McpServerEntity, McpServerError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .update_mcp_server(tenant_id, id, user_id, request)
      .await
  }

  /// Get an MCP server by ID.
  pub async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerEntity>, McpServerError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .get_mcp_server(tenant_id, id)
      .await
  }

  /// List MCP servers, optionally filtered by enabled status.
  pub async fn list_mcp_servers(
    &self,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServerEntity>, McpServerError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .list_mcp_servers(tenant_id, enabled)
      .await
  }

  /// Count enabled/disabled MCPs for a server.
  pub async fn count_mcps_for_server(&self, server_id: &str) -> Result<(i64, i64), McpServerError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .count_mcps_for_server(tenant_id, server_id)
      .await
  }

  // ========== Tool discovery (direct delegation) ==========

  /// Fetch tools from an MCP server without creating an instance.
  pub async fn fetch_tools_for_server(
    &self,
    server_id: &str,
    credentials: Option<Vec<McpAuthParamInput>>,
    auth_config_id: Option<String>,
    oauth_token_id: Option<String>,
  ) -> Result<Vec<McpTool>, McpError> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    self
      .app_service
      .mcp_service()
      .fetch_tools_for_server(
        tenant_id,
        server_id,
        credentials,
        auth_config_id,
        oauth_token_id,
      )
      .await
  }

  // ========== OAuth discovery/DCR (direct delegation) ==========

  /// Discover OAuth metadata from an authorization server URL.
  pub async fn discover_oauth_metadata(&self, url: &str) -> Result<serde_json::Value, McpError> {
    self
      .app_service
      .mcp_service()
      .discover_oauth_metadata(url)
      .await
  }

  /// Discover OAuth metadata from an MCP server URL.
  pub async fn discover_mcp_oauth_metadata(
    &self,
    mcp_server_url: &str,
  ) -> Result<serde_json::Value, McpError> {
    self
      .app_service
      .mcp_service()
      .discover_mcp_oauth_metadata(mcp_server_url)
      .await
  }

  /// Dynamic client registration.
  pub async fn dynamic_register_client(
    &self,
    registration_endpoint: &str,
    redirect_uri: &str,
    scopes: Option<String>,
  ) -> Result<serde_json::Value, McpError> {
    self
      .app_service
      .mcp_service()
      .dynamic_register_client(registration_endpoint, redirect_uri, scopes)
      .await
  }
}
