use objs::{McpServerInfo, McpTool};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// MCP Server (mcp_servers table) DTOs
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateMcpServerRequest {
  pub url: String,
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
  pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateMcpServerRequest {
  pub url: String,
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct McpServerResponse {
  pub id: String,
  pub url: String,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub enabled: bool,
  pub created_by: String,
  pub updated_by: String,
  pub enabled_mcp_count: i64,
  pub disabled_mcp_count: i64,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct McpServerQuery {
  pub enabled: Option<bool>,
}

// ============================================================================
// MCP Instance (mcps table) DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum McpAuth {
  #[serde(rename = "public")]
  Public,
  #[serde(rename = "header")]
  Header {
    header_key: String,
    header_value: String,
  },
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct FetchMcpToolsRequest {
  pub mcp_server_id: String,
  pub auth: McpAuth,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateMcpRequest {
  pub name: String,
  pub slug: String,
  pub mcp_server_id: String,
  #[serde(default)]
  pub description: Option<String>,
  pub enabled: bool,
  #[serde(default)]
  pub tools_cache: Option<Vec<McpTool>>,
  #[serde(default)]
  pub tools_filter: Option<Vec<String>>,
  pub auth: McpAuth,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateMcpRequest {
  pub name: String,
  pub slug: String,
  #[serde(default)]
  pub description: Option<String>,
  pub enabled: bool,
  #[serde(default)]
  pub tools_filter: Option<Vec<String>>,
  #[serde(default)]
  pub tools_cache: Option<Vec<McpTool>>,
  #[serde(default)]
  pub auth: Option<McpAuth>,
}

// ============================================================================
// Response types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct McpResponse {
  pub id: String,
  pub mcp_server: McpServerInfo,
  pub slug: String,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub enabled: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_cache: Option<Vec<McpTool>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_filter: Option<Vec<String>>,
  pub auth_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_header_key: Option<String>,
  pub has_auth_header_value: bool,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListMcpsResponse {
  pub mcps: Vec<McpResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListMcpServersResponse {
  pub mcp_servers: Vec<McpServerResponse>,
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
      mcp_server: mcp.mcp_server,
      slug: mcp.slug,
      name: mcp.name,
      description: mcp.description,
      enabled: mcp.enabled,
      tools_cache: mcp.tools_cache,
      tools_filter: mcp.tools_filter,
      auth_type: mcp.auth_type,
      auth_header_key: mcp.auth_header_key,
      has_auth_header_value: mcp.has_auth_header_value,
      created_at: mcp.created_at.to_rfc3339(),
      updated_at: mcp.updated_at.to_rfc3339(),
    }
  }
}
