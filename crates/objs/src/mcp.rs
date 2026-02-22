use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};
use utoipa::ToSchema;

// ============================================================================
// McpServer - Admin-managed MCP server registry (public API model)
// ============================================================================

/// Admin-managed MCP server registry entry.
/// Admins/managers register MCP server URLs that users can then create instances of.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpServer {
  /// Unique identifier (UUID)
  pub id: String,
  /// MCP server endpoint URL (trimmed, case-insensitive unique)
  pub url: String,
  /// Human-readable display name
  pub name: String,
  /// Optional description
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this MCP server is enabled
  pub enabled: bool,
  /// User who created this entry
  pub created_by: String,
  /// User who last updated this entry
  pub updated_by: String,
  /// When this entry was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this entry was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpServerInfo - Nested server context in MCP instance responses
// ============================================================================

/// Minimal MCP server info embedded in MCP instance responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpServerInfo {
  pub id: String,
  pub url: String,
  pub name: String,
  pub enabled: bool,
}

// ============================================================================
// McpAuthType - Authentication type for MCP instances
// ============================================================================

#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  ToSchema,
  Default,
  Display,
  EnumString,
  IntoStaticStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum McpAuthType {
  #[default]
  Public,
  Header,
  Oauth,
}

impl McpAuthType {
  pub fn as_str(&self) -> &'static str {
    self.into()
  }
}

// ============================================================================
// Mcp - User-owned MCP instance (public API model)
// ============================================================================

/// User-owned MCP server instance with tool caching and filtering.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Mcp {
  /// Unique instance identifier (UUID)
  pub id: String,
  /// Server info resolved via JOIN
  pub mcp_server: McpServerInfo,
  /// User-defined slug for this instance
  pub slug: String,
  /// Human-readable name
  pub name: String,
  /// Optional description for this instance
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this instance is enabled
  pub enabled: bool,
  /// Cached tool schemas from the MCP server (JSON array)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_cache: Option<Vec<McpTool>>,
  /// Whitelisted tool names (empty = block all)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools_filter: Option<Vec<String>>,
  pub auth_type: McpAuthType,
  /// Reference to the auth config (mcp_auth_headers.id or mcp_oauth_configs.id)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_uuid: Option<String>,
  /// When this instance was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this instance was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpAuthHeader - Public API model for header-based auth config
// ============================================================================

/// Header-based authentication configuration (secrets masked).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpAuthHeader {
  /// Unique identifier (UUID)
  pub id: String,
  /// Human-readable display name
  pub name: String,
  /// Parent MCP server ID
  pub mcp_server_id: String,
  /// HTTP header name (e.g. "Authorization", "X-API-Key")
  pub header_key: String,
  /// Whether an encrypted header value is stored
  pub has_header_value: bool,
  /// User who created this config
  pub created_by: String,
  /// When this config was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this config was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpOAuthConfig - Public API model for OAuth 2.1 pre-registered client config
// ============================================================================

/// OAuth 2.1 config for pre-registered or dynamically registered client (secrets masked).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpOAuthConfig {
  pub id: String,
  /// Human-readable display name
  pub name: String,
  pub mcp_server_id: String,
  pub registration_type: RegistrationType,
  pub client_id: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_endpoint: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_id_issued_at: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_endpoint_auth_method: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes: Option<String>,
  pub has_client_secret: bool,
  pub has_registration_access_token: bool,
  pub created_by: String,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpOAuthToken - Public API model for OAuth 2.1 stored token
// ============================================================================

/// OAuth 2.1 token stored for a config (secrets masked).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpOAuthToken {
  pub id: String,
  pub mcp_oauth_config_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scopes_granted: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<i64>,
  pub has_access_token: bool,
  pub has_refresh_token: bool,
  pub created_by: String,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// McpTool - Cached tool schema from MCP server
// ============================================================================

/// Tool schema cached from an MCP server's tools/list response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpTool {
  /// Tool name as declared by the MCP server
  pub name: String,
  /// Human-readable description of the tool
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// JSON Schema for tool input parameters
  #[serde(skip_serializing_if = "Option::is_none")]
  pub input_schema: Option<serde_json::Value>,
}

// ============================================================================
// McpExecutionRequest / McpExecutionResponse
// ============================================================================

/// Request to execute a tool on an MCP server instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpExecutionRequest {
  /// Tool parameters as JSON
  pub params: serde_json::Value,
}

/// Response from MCP tool execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpExecutionResponse {
  /// Successful result (JSON), if any
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<serde_json::Value>,
  /// Error message, if execution failed
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

// ============================================================================
// RegistrationType - OAuth registration type enum
// ============================================================================

/// OAuth 2.1 registration type: pre-registered client or dynamic client registration (DCR).
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  ToSchema,
  Default,
  Display,
  EnumString,
  IntoStaticStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum RegistrationType {
  #[default]
  PreRegistered,
  DynamicRegistration,
}

impl RegistrationType {
  pub fn as_str(&self) -> &'static str {
    self.into()
  }
}

// ============================================================================
// Unified auth config discriminated union types
// ============================================================================

/// Discriminated union for creating any type of MCP auth config.
/// The JSON `"type"` field determines the variant: `"header"` or `"oauth"`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CreateMcpAuthConfigRequest {
  Header {
    name: String,
    header_key: String,
    header_value: String,
  },
  Oauth {
    name: String,
    client_id: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    scopes: Option<String>,
    /// `"pre-registered"` (default) or `"dynamic-registration"`
    #[serde(default)]
    registration_type: RegistrationType,
    #[serde(default)]
    registration_access_token: Option<String>,
    #[serde(default)]
    registration_endpoint: Option<String>,
    #[serde(default)]
    token_endpoint_auth_method: Option<String>,
    #[serde(default)]
    client_id_issued_at: Option<i64>,
  },
}

/// Discriminated union response for any type of MCP auth config.
/// The JSON `"type"` field determines the variant: `"header"` or `"oauth"`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum McpAuthConfigResponse {
  Header {
    id: String,
    name: String,
    mcp_server_id: String,
    header_key: String,
    has_header_value: bool,
    created_by: String,
    #[schema(value_type = String, format = "date-time")]
    created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    updated_at: DateTime<Utc>,
  },
  Oauth {
    id: String,
    name: String,
    mcp_server_id: String,
    registration_type: RegistrationType,
    client_id: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    registration_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id_issued_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_endpoint_auth_method: Option<String>,
    has_client_secret: bool,
    has_registration_access_token: bool,
    created_by: String,
    #[schema(value_type = String, format = "date-time")]
    created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    updated_at: DateTime<Utc>,
  },
}

impl McpAuthConfigResponse {
  pub fn id(&self) -> &str {
    match self {
      McpAuthConfigResponse::Header { id, .. } => id,
      McpAuthConfigResponse::Oauth { id, .. } => id,
    }
  }

  pub fn mcp_server_id(&self) -> &str {
    match self {
      McpAuthConfigResponse::Header { mcp_server_id, .. } => mcp_server_id,
      McpAuthConfigResponse::Oauth { mcp_server_id, .. } => mcp_server_id,
    }
  }

  pub fn created_by(&self) -> &str {
    match self {
      McpAuthConfigResponse::Header { created_by, .. } => created_by,
      McpAuthConfigResponse::Oauth { created_by, .. } => created_by,
    }
  }
}

impl From<McpAuthHeader> for McpAuthConfigResponse {
  fn from(h: McpAuthHeader) -> Self {
    McpAuthConfigResponse::Header {
      id: h.id,
      name: h.name,
      mcp_server_id: h.mcp_server_id,
      header_key: h.header_key,
      has_header_value: h.has_header_value,
      created_by: h.created_by,
      created_at: h.created_at,
      updated_at: h.updated_at,
    }
  }
}

impl From<McpOAuthConfig> for McpAuthConfigResponse {
  fn from(o: McpOAuthConfig) -> Self {
    McpAuthConfigResponse::Oauth {
      id: o.id,
      name: o.name,
      mcp_server_id: o.mcp_server_id,
      registration_type: o.registration_type,
      client_id: o.client_id,
      authorization_endpoint: o.authorization_endpoint,
      token_endpoint: o.token_endpoint,
      registration_endpoint: o.registration_endpoint,
      scopes: o.scopes,
      client_id_issued_at: o.client_id_issued_at,
      token_endpoint_auth_method: o.token_endpoint_auth_method,
      has_client_secret: o.has_client_secret,
      has_registration_access_token: o.has_registration_access_token,
      created_by: o.created_by,
      created_at: o.created_at,
      updated_at: o.updated_at,
    }
  }
}

/// List wrapper for unified auth config responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpAuthConfigsListResponse {
  pub auth_configs: Vec<McpAuthConfigResponse>,
}

// ============================================================================
// Validation functions (reuse toolset regex/limits pattern)
// ============================================================================

use once_cell::sync::Lazy;
use regex::Regex;

static MCP_SLUG_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9-]+$").expect("Invalid MCP slug regex"));

pub const MAX_MCP_SLUG_LEN: usize = 24;
pub const MAX_MCP_DESCRIPTION_LEN: usize = 255;
pub const MAX_MCP_SERVER_NAME_LEN: usize = 100;
pub const MAX_MCP_SERVER_URL_LEN: usize = 2048;
pub const MAX_MCP_AUTH_CONFIG_NAME_LEN: usize = 100;

/// Validate MCP instance slug format and length
pub fn validate_mcp_slug(slug: &str) -> Result<(), String> {
  if slug.is_empty() {
    return Err("MCP slug cannot be empty".to_string());
  }
  if slug.len() > MAX_MCP_SLUG_LEN {
    return Err(format!(
      "MCP slug cannot exceed {} characters",
      MAX_MCP_SLUG_LEN
    ));
  }
  if !MCP_SLUG_REGEX.is_match(slug) {
    return Err("MCP slug can only contain alphanumeric characters and hyphens".to_string());
  }
  Ok(())
}

/// Validate MCP instance description length
pub fn validate_mcp_description(description: &str) -> Result<(), String> {
  if description.len() > MAX_MCP_DESCRIPTION_LEN {
    return Err(format!(
      "MCP description cannot exceed {} characters",
      MAX_MCP_DESCRIPTION_LEN
    ));
  }
  Ok(())
}

/// Validate MCP server name (required, max 100 chars)
pub fn validate_mcp_server_name(name: &str) -> Result<(), String> {
  if name.is_empty() {
    return Err("MCP server name cannot be empty".to_string());
  }
  if name.len() > MAX_MCP_SERVER_NAME_LEN {
    return Err(format!(
      "MCP server name cannot exceed {} characters",
      MAX_MCP_SERVER_NAME_LEN
    ));
  }
  Ok(())
}

/// Validate MCP server URL (required, valid URL format, max 2048 chars)
pub fn validate_mcp_server_url(url: &str) -> Result<(), String> {
  if url.is_empty() {
    return Err("MCP server URL cannot be empty".to_string());
  }
  if url.len() > MAX_MCP_SERVER_URL_LEN {
    return Err(format!(
      "MCP server URL cannot exceed {} characters",
      MAX_MCP_SERVER_URL_LEN
    ));
  }
  url::Url::parse(url).map_err(|_| "MCP server URL is not a valid URL".to_string())?;
  Ok(())
}

/// Validate MCP server description length (reuses same limit as MCP instance)
pub fn validate_mcp_server_description(description: &str) -> Result<(), String> {
  if description.len() > MAX_MCP_DESCRIPTION_LEN {
    return Err(format!(
      "MCP server description cannot exceed {} characters",
      MAX_MCP_DESCRIPTION_LEN
    ));
  }
  Ok(())
}

/// Validate auth config name (required, max 100 chars)
pub fn validate_mcp_auth_config_name(name: &str) -> Result<(), String> {
  if name.is_empty() {
    return Err("Auth config name cannot be empty".to_string());
  }
  if name.len() > MAX_MCP_AUTH_CONFIG_NAME_LEN {
    return Err(format!(
      "Auth config name cannot exceed {} characters",
      MAX_MCP_AUTH_CONFIG_NAME_LEN
    ));
  }
  Ok(())
}

/// Validate OAuth endpoint URL (authorization_endpoint, token_endpoint, etc.)
pub fn validate_oauth_endpoint_url(url: &str, field_name: &str) -> Result<(), String> {
  if url.is_empty() {
    return Err(format!("{} cannot be empty", field_name));
  }
  if url.len() > MAX_MCP_SERVER_URL_LEN {
    return Err(format!(
      "{} cannot exceed {} characters",
      field_name, MAX_MCP_SERVER_URL_LEN
    ));
  }
  url::Url::parse(url).map_err(|_| format!("{} is not a valid URL", field_name))?;
  Ok(())
}

#[cfg(test)]
#[path = "test_mcp_validation.rs"]
mod test_mcp_validation;

#[cfg(test)]
#[path = "test_mcp_types.rs"]
mod test_mcp_types;
