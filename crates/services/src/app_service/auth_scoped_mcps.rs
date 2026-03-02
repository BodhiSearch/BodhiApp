use crate::{
  AppService, AuthContext, CreateMcpAuthConfigRequest, McpAuthConfigResponse, McpError,
  McpExecutionRequest, McpExecutionResponse, McpOAuthToken, McpServer, McpServerError, McpTool,
  Mcp, McpAuthType,
};
use std::sync::Arc;

/// Auth-scoped wrapper around McpService that injects user_id from AuthContext.
/// User-scoped methods automatically inject the authenticated user's ID.
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
  pub async fn list(&self) -> Result<Vec<Mcp>, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    self.app_service.mcp_service().list(user_id).await
  }

  /// Get a specific MCP instance by ID for the authenticated user.
  pub async fn get(&self, id: &str) -> Result<Option<Mcp>, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    let mcp = self.app_service.mcp_service().get(user_id, id).await?;
    Ok(mcp)
  }

  /// Create a new MCP instance for the authenticated user.
  #[allow(clippy::too_many_arguments)]
  pub async fn create(
    &self,
    name: &str,
    slug: &str,
    mcp_server_id: &str,
    description: Option<String>,
    enabled: bool,
    tools_cache: Option<Vec<McpTool>>,
    tools_filter: Option<Vec<String>>,
    auth_type: McpAuthType,
    auth_uuid: Option<String>,
  ) -> Result<Mcp, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    let mcp = self
      .app_service
      .mcp_service()
      .create(
        user_id,
        name,
        slug,
        mcp_server_id,
        description,
        enabled,
        tools_cache,
        tools_filter,
        auth_type,
        auth_uuid,
      )
      .await?;
    Ok(mcp)
  }

  /// Update an existing MCP instance for the authenticated user.
  #[allow(clippy::too_many_arguments)]
  pub async fn update(
    &self,
    id: &str,
    name: &str,
    slug: &str,
    description: Option<String>,
    enabled: bool,
    tools_filter: Option<Vec<String>>,
    tools_cache: Option<Vec<McpTool>>,
    auth_type: Option<McpAuthType>,
    auth_uuid: Option<String>,
  ) -> Result<Mcp, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    let mcp = self
      .app_service
      .mcp_service()
      .update(
        user_id,
        id,
        name,
        slug,
        description,
        enabled,
        tools_filter,
        tools_cache,
        auth_type,
        auth_uuid,
      )
      .await?;
    Ok(mcp)
  }

  /// Delete an MCP instance for the authenticated user.
  pub async fn delete(&self, id: &str) -> Result<(), McpError> {
    let user_id = self.auth_context.require_user_id()?;
    self.app_service.mcp_service().delete(user_id, id).await?;
    Ok(())
  }

  /// Fetch and refresh tools for an MCP instance owned by the authenticated user.
  pub async fn fetch_tools(&self, id: &str) -> Result<Vec<McpTool>, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    let tools = self
      .app_service
      .mcp_service()
      .fetch_tools(user_id, id)
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
    let user_id = self.auth_context.require_user_id()?;
    let response = self
      .app_service
      .mcp_service()
      .execute(user_id, id, tool_name, request)
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
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .create_auth_config(user_id, mcp_server_id, request)
      .await
  }

  /// Delete an auth config. No authorization check — middleware handles access control.
  pub async fn delete_auth_config(&self, config_id: &str) -> Result<(), McpError> {
    self
      .app_service
      .mcp_service()
      .delete_auth_config(config_id)
      .await
  }

  // ========== OAuth token methods (user-scoped) ==========

  /// Get an OAuth token by ID for the authenticated user.
  pub async fn get_oauth_token(
    &self,
    token_id: &str,
  ) -> Result<Option<McpOAuthToken>, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .get_oauth_token(user_id, token_id)
      .await
  }

  /// Delete an OAuth token for the authenticated user.
  pub async fn delete_oauth_token(&self, token_id: &str) -> Result<(), McpError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .db_service()
      .delete_mcp_oauth_token(user_id, token_id)
      .await?;
    Ok(())
  }

  /// Exchange an authorization code for tokens.
  pub async fn exchange_oauth_token(
    &self,
    config_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
  ) -> Result<McpOAuthToken, McpError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .exchange_oauth_token(user_id, config_id, code, redirect_uri, code_verifier)
      .await
  }

  // ========== Auth config read methods (direct delegation) ==========

  /// List all auth configs for a server.
  pub async fn list_auth_configs(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthConfigResponse>, McpError> {
    self
      .app_service
      .mcp_service()
      .list_auth_configs(mcp_server_id)
      .await
  }

  /// Get an auth config by ID.
  pub async fn get_auth_config(
    &self,
    config_id: &str,
  ) -> Result<Option<McpAuthConfigResponse>, McpError> {
    self
      .app_service
      .mcp_service()
      .get_auth_config(config_id)
      .await
  }

  /// Get an OAuth config by ID.
  pub async fn get_oauth_config(
    &self,
    config_id: &str,
  ) -> Result<Option<crate::McpOAuthConfig>, McpError> {
    self
      .app_service
      .mcp_service()
      .get_oauth_config(config_id)
      .await
  }

  // ========== Server-level methods (Part C) ==========

  /// Create a new MCP server. Injects user_id as created_by.
  pub async fn create_mcp_server(
    &self,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
  ) -> Result<McpServer, McpServerError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .create_mcp_server(name, url, description, enabled, user_id)
      .await
  }

  /// Update an MCP server. Injects user_id as updated_by.
  pub async fn update_mcp_server(
    &self,
    id: &str,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
  ) -> Result<McpServer, McpServerError> {
    let user_id = self.auth_context.require_user_id()?;
    self
      .app_service
      .mcp_service()
      .update_mcp_server(id, name, url, description, enabled, user_id)
      .await
  }

  /// Get an MCP server by ID.
  pub async fn get_mcp_server(
    &self,
    id: &str,
  ) -> Result<Option<McpServer>, McpServerError> {
    self.app_service.mcp_service().get_mcp_server(id).await
  }

  /// List MCP servers, optionally filtered by enabled status.
  pub async fn list_mcp_servers(
    &self,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServer>, McpServerError> {
    self
      .app_service
      .mcp_service()
      .list_mcp_servers(enabled)
      .await
  }

  /// Count enabled/disabled MCPs for a server.
  pub async fn count_mcps_for_server(
    &self,
    server_id: &str,
  ) -> Result<(i64, i64), McpServerError> {
    self
      .app_service
      .mcp_service()
      .count_mcps_for_server(server_id)
      .await
  }

  // ========== Tool discovery (direct delegation) ==========

  /// Fetch tools from an MCP server without creating an instance.
  pub async fn fetch_tools_for_server(
    &self,
    server_id: &str,
    auth_header_key: Option<String>,
    auth_header_value: Option<String>,
    auth_uuid: Option<String>,
  ) -> Result<Vec<McpTool>, McpError> {
    self
      .app_service
      .mcp_service()
      .fetch_tools_for_server(server_id, auth_header_key, auth_header_value, auth_uuid)
      .await
  }

  // ========== OAuth discovery/DCR (direct delegation) ==========

  /// Discover OAuth metadata from an authorization server URL.
  pub async fn discover_oauth_metadata(
    &self,
    url: &str,
  ) -> Result<serde_json::Value, McpError> {
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

