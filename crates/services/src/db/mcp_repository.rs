use crate::db::{
  DbError, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow, McpRow, McpServerRow,
  McpWithServerRow,
};

#[async_trait::async_trait]
pub trait McpRepository: Send + Sync {
  // MCP server registry (admin-managed)
  async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError>;

  async fn update_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError>;

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError>;

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError>;

  async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError>;

  /// Returns (enabled_count, disabled_count) for MCPs referencing this server
  async fn count_mcps_by_server_id(&self, server_id: &str) -> Result<(i64, i64), DbError>;

  /// Clear tools_cache and tools_filter on all MCPs linked to a server
  async fn clear_mcp_tools_by_server_id(&self, server_id: &str) -> Result<u64, DbError>;

  // MCP user instances
  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;

  async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError>;

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError>;

  async fn list_mcps_with_server(&self, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError>;

  async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;

  async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError>;

  // MCP auth header configs
  async fn create_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError>;

  async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<McpAuthHeaderRow>, DbError>;

  async fn update_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError>;

  async fn delete_mcp_auth_header(&self, id: &str) -> Result<(), DbError>;

  async fn list_mcp_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeaderRow>, DbError>;

  /// Get the decrypted auth header (key, value) for an MCP auth header config.
  async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError>;

  // MCP OAuth config operations
  async fn create_mcp_oauth_config(
    &self,
    row: &McpOAuthConfigRow,
  ) -> Result<McpOAuthConfigRow, DbError>;

  async fn get_mcp_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfigRow>, DbError>;

  async fn list_mcp_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpOAuthConfigRow>, DbError>;

  async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError>;

  /// Delete an OAuth config and all its associated tokens in a single transaction.
  async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError>;

  /// Get (client_id, decrypted_client_secret) for an OAuth config.
  async fn get_decrypted_client_secret(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError>;

  // MCP OAuth token operations
  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError>;

  async fn get_mcp_oauth_token(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthTokenRow>, DbError>;

  async fn get_latest_oauth_token_by_config(
    &self,
    config_id: &str,
  ) -> Result<Option<McpOAuthTokenRow>, DbError>;

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError>;

  async fn delete_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<(), DbError>;

  async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError>;

  /// Delete existing tokens for a specific (config_id, user_id) pair.
  /// Used before inserting a new token to prevent orphaned rows.
  async fn delete_oauth_tokens_by_config_and_user(
    &self,
    config_id: &str,
    user_id: &str,
  ) -> Result<(), DbError>;

  /// Get decrypted OAuth bearer header (Authorization, Bearer <token>) by token ID.
  /// Not user-scoped; used for admin preview flows.
  async fn get_decrypted_oauth_bearer(&self, id: &str)
    -> Result<Option<(String, String)>, DbError>;
}
