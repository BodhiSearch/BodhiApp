use crate::db::{DbError, McpRow, McpServerRow, McpWithServerRow};

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

  /// Get the decrypted auth header for an MCP instance.
  /// Returns Some((header_key, header_value)) if auth_type is "header", None otherwise.
  async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError>;
}
