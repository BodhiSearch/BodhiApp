use objs::{McpServer, McpTool};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// MCP Server (admin allowlist) DTOs
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct EnableMcpServerRequest {
  pub url: String,
  #[serde(default = "default_true")]
  pub enabled: bool,
}

fn default_true() -> bool {
  true
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct McpServerUrlQuery {
  pub url: Option<String>,
}

// ============================================================================
// MCP Instance CRUD DTOs
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateMcpRequest {
  pub name: String,
  pub slug: String,
  pub url: String,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default = "default_true")]
  pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateMcpRequest {
  pub name: String,
  pub slug: String,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default = "default_true")]
  pub enabled: bool,
}

// ============================================================================
// Response types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct McpResponse {
  pub id: String,
  pub mcp_server_id: String,
  pub url: String,
  pub slug: String,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub enabled: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_cache: Option<Vec<McpTool>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_filter: Option<Vec<String>>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListMcpsResponse {
  pub mcps: Vec<McpResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListMcpServersResponse {
  pub mcp_servers: Vec<McpServer>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct McpToolsResponse {
  pub tools: Vec<McpTool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct McpExecuteRequest {
  pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct McpExecuteResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

impl From<objs::Mcp> for McpResponse {
  fn from(mcp: objs::Mcp) -> Self {
    McpResponse {
      id: mcp.id,
      mcp_server_id: mcp.mcp_server_id,
      url: mcp.url,
      slug: mcp.slug,
      name: mcp.name,
      description: mcp.description,
      enabled: mcp.enabled,
      tools_cache: mcp.tools_cache,
      tools_filter: mcp.tools_filter,
      created_at: mcp.created_at.to_rfc3339(),
      updated_at: mcp.updated_at.to_rfc3339(),
    }
  }
}
