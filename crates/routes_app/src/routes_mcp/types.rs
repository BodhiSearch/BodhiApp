use objs::{
  CreateMcpAuthConfigRequest, McpAuthConfigResponse, McpAuthType, McpOAuthToken, McpServerInfo,
  McpTool,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub auth_config: Option<CreateMcpAuthConfigRequest>,
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_config: Option<McpAuthConfigResponse>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct McpServerQuery {
  pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListMcpServersResponse {
  pub mcp_servers: Vec<McpServerResponse>,
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
  #[serde(default)]
  pub auth: Option<McpAuth>,
  #[serde(default)]
  pub auth_uuid: Option<String>,
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
  #[serde(default)]
  pub auth_type: McpAuthType,
  #[serde(default)]
  pub auth_uuid: Option<String>,
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
  pub auth_type: Option<McpAuthType>,
  #[serde(default)]
  pub auth_uuid: Option<String>,
}

// ============================================================================
// MCP Instance Response types
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
  pub auth_type: McpAuthType,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_uuid: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListMcpsResponse {
  pub mcps: Vec<McpResponse>,
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
      auth_uuid: mcp.auth_uuid,
      created_at: mcp.created_at.to_rfc3339(),
      updated_at: mcp.updated_at.to_rfc3339(),
    }
  }
}

// ============================================================================
// Unified Auth Config DTOs
// ============================================================================

/// Wrapper for creating auth configs with server_id in body instead of path
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAuthConfigBody {
  pub mcp_server_id: String,
  #[serde(flatten)]
  pub config: CreateMcpAuthConfigRequest,
}

/// Query params for listing auth configs
#[derive(Debug, Deserialize, IntoParams)]
pub struct AuthConfigsQuery {
  pub mcp_server_id: String,
}

// ============================================================================
// OAuth Token DTOs
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthTokenResponse {
  pub id: String,
  pub mcp_oauth_config_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes_granted: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<i64>,
  pub has_access_token: bool,
  pub has_refresh_token: bool,
  pub created_by: String,
  pub created_at: String,
  pub updated_at: String,
}

impl From<McpOAuthToken> for OAuthTokenResponse {
  fn from(t: McpOAuthToken) -> Self {
    OAuthTokenResponse {
      id: t.id,
      mcp_oauth_config_id: t.mcp_oauth_config_id,
      scopes_granted: t.scopes_granted,
      expires_at: t.expires_at,
      has_access_token: t.has_access_token,
      has_refresh_token: t.has_refresh_token,
      created_by: t.created_by,
      created_at: t.created_at.to_rfc3339(),
      updated_at: t.updated_at.to_rfc3339(),
    }
  }
}

// ============================================================================
// OAuth Flow DTOs
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthLoginRequest {
  pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthLoginResponse {
  pub authorization_url: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct OAuthTokenExchangeRequest {
  pub code: String,
  pub redirect_uri: String,
  pub state: String,
}

// ============================================================================
// OAuth Discovery DTOs
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct OAuthDiscoverAsRequest {
  pub url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthDiscoverAsResponse {
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes_supported: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct OAuthDiscoverMcpRequest {
  pub mcp_server_url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OAuthDiscoverMcpResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub authorization_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes_supported: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resource: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub authorization_server_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DynamicRegisterRequest {
  pub registration_endpoint: String,
  pub redirect_uri: String,
  #[serde(default)]
  pub scopes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DynamicRegisterResponse {
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_secret: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_id_issued_at: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_endpoint_auth_method: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_access_token: Option<String>,
}
