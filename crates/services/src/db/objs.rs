#[allow(unused_imports)]
use objs::{is_default, BuilderError};
pub use objs::{
  AppAccessRequestStatus, AppStatus, DownloadStatus, FlowType, McpAuthType, RegistrationType,
  TokenStatus, UserAccessRequestStatus,
};

/// Represents an API key update operation for API model aliases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyUpdate {
  /// Keep the existing API key unchanged
  Keep,
  /// Set a new API key (or add one if none exists) - Option<String> supports both setting and clearing
  Set(Option<String>),
}

// ============================================================================
// ToolsetRow - Database row for user toolset instances
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ToolsetRow {
  pub id: String,
  pub user_id: String,
  pub toolset_type: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub encrypted_api_key: Option<String>,
  pub salt: Option<String>,
  pub nonce: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// AppToolsetConfigRow - Database row for app-level toolset type configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct AppToolsetConfigRow {
  pub id: String,
  pub toolset_type: String,
  pub enabled: bool,
  pub updated_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// AppAccessRequestRow - Database row for app access request consent tracking
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct AppAccessRequestRow {
  pub id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub flow_type: FlowType,
  pub redirect_uri: Option<String>,
  pub status: AppAccessRequestStatus,
  pub requested: String,
  pub approved: Option<String>,
  pub user_id: Option<String>,
  pub requested_role: String,
  pub approved_role: Option<String>,
  pub access_request_scope: Option<String>,
  pub error_message: Option<String>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// McpServerRow - Database row for admin MCP server URL allowlist
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpServerRow {
  pub id: String,
  pub url: String,
  pub name: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub created_by: String,
  pub updated_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Joined MCP instance + server info from SQL JOIN query
#[derive(Debug, Clone, PartialEq)]
pub struct McpWithServerRow {
  pub id: String,
  pub created_by: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub tools_cache: Option<String>,
  pub tools_filter: Option<String>,
  pub auth_type: McpAuthType,
  pub auth_uuid: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  // Server info from JOIN
  pub server_url: String,
  pub server_name: String,
  pub server_enabled: bool,
}

// ============================================================================
// McpRow - Database row for user-owned MCP instances
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpRow {
  pub id: String,
  pub created_by: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub tools_cache: Option<String>,
  pub tools_filter: Option<String>,
  pub auth_type: McpAuthType,
  pub auth_uuid: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// McpAuthHeaderRow - Database row for header-based MCP authentication configs
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpAuthHeaderRow {
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
  pub header_key: String,
  pub encrypted_header_value: String,
  pub header_value_salt: String,
  pub header_value_nonce: String,
  pub created_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// McpOAuthConfigRow - Database row for OAuth 2.1 client configs (pre-registered or dynamic)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpOAuthConfigRow {
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
  pub registration_type: RegistrationType,
  pub client_id: String,
  pub encrypted_client_secret: Option<String>,
  pub client_secret_salt: Option<String>,
  pub client_secret_nonce: Option<String>,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub registration_endpoint: Option<String>,
  pub encrypted_registration_access_token: Option<String>,
  pub registration_access_token_salt: Option<String>,
  pub registration_access_token_nonce: Option<String>,
  pub client_id_issued_at: Option<chrono::DateTime<chrono::Utc>>,
  pub token_endpoint_auth_method: Option<String>,
  pub scopes: Option<String>,
  pub created_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// McpOAuthTokenRow - Database row for OAuth 2.1 stored tokens
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpOAuthTokenRow {
  pub id: String,
  pub mcp_oauth_config_id: String,
  pub encrypted_access_token: String,
  pub access_token_salt: String,
  pub access_token_nonce: String,
  pub encrypted_refresh_token: Option<String>,
  pub refresh_token_salt: Option<String>,
  pub refresh_token_nonce: Option<String>,
  pub scopes_granted: Option<String>,
  pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
  pub created_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}
