// NOTE: Types in this file are utility/action DTOs that don't use services/DB
// for persistence. They don't follow the <Domain>Request/<Domain>Response naming
// convention used by CRUD entities.

use serde::{Deserialize, Serialize};
use services::{CreateMcpAuthConfigRequest, McpOAuthToken, McpTool};
use utoipa::{IntoParams, ToSchema};

// ============================================================================
// MCP Server Query DTOs
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct McpServerQuery {
  pub enabled: Option<bool>,
}

// ============================================================================
// MCP Instance DTOs (tool discovery, execution)
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

// ============================================================================
// Unified Auth Config DTOs
// ============================================================================

/// Wrapper for creating auth configs with server_id in body instead of path
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAuthConfig {
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
  pub user_id: String,
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
      user_id: t.user_id,
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
