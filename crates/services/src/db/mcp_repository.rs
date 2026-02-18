use crate::db::{DbError, McpRow, McpServerRow};

#[async_trait::async_trait]
pub trait McpRepository: Send + Sync {
  // MCP server URL allowlist (admin-managed)
  async fn set_mcp_server_enabled(
    &self,
    url: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServerRow, DbError>;

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError>;

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError>;

  async fn list_mcp_servers(&self) -> Result<Vec<McpServerRow>, DbError>;

  // MCP user instances
  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;

  async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError>;

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError>;

  async fn list_mcps(&self, user_id: &str) -> Result<Vec<McpRow>, DbError>;

  async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError>;

  async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError>;
}
