use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
  /// Authentication type: "public", "header", "oauth-pre-registered"
  pub auth_type: String,
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

#[cfg(test)]
mod tests {
  use crate::mcp::{
    validate_mcp_description, validate_mcp_server_description, validate_mcp_server_name,
    validate_mcp_server_url, validate_mcp_slug, MAX_MCP_DESCRIPTION_LEN, MAX_MCP_SERVER_NAME_LEN,
    MAX_MCP_SERVER_URL_LEN, MAX_MCP_SLUG_LEN,
  };

  #[test]
  fn test_validate_mcp_slug_accepts_valid_slugs() {
    assert!(validate_mcp_slug("my-mcp").is_ok());
    assert!(validate_mcp_slug("MyMcp123").is_ok());
    assert!(validate_mcp_slug("a").is_ok());
    assert!(validate_mcp_slug("deepwiki-1").is_ok());
  }

  #[test]
  fn test_validate_mcp_slug_rejects_empty() {
    let result = validate_mcp_slug("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
  }

  #[test]
  fn test_validate_mcp_slug_rejects_too_long() {
    let long_slug = "a".repeat(MAX_MCP_SLUG_LEN + 1);
    let result = validate_mcp_slug(&long_slug);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot exceed"));
  }

  #[test]
  fn test_validate_mcp_slug_rejects_invalid_characters() {
    assert!(validate_mcp_slug("my_mcp").is_err());
    assert!(validate_mcp_slug("my mcp").is_err());
    assert!(validate_mcp_slug("my.mcp").is_err());
    assert!(validate_mcp_slug("my@mcp").is_err());
  }

  #[test]
  fn test_validate_mcp_slug_accepts_max_length() {
    let max_slug = "a".repeat(MAX_MCP_SLUG_LEN);
    assert!(validate_mcp_slug(&max_slug).is_ok());
  }

  #[test]
  fn test_validate_mcp_description_accepts_valid_descriptions() {
    assert!(validate_mcp_description("").is_ok());
    assert!(validate_mcp_description("A short description").is_ok());
    assert!(validate_mcp_description("A description with special chars: @#$%").is_ok());
  }

  #[test]
  fn test_validate_mcp_description_rejects_too_long() {
    let long_desc = "a".repeat(MAX_MCP_DESCRIPTION_LEN + 1);
    let result = validate_mcp_description(&long_desc);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot exceed"));
  }

  #[test]
  fn test_validate_mcp_description_accepts_max_length() {
    let max_desc = "a".repeat(MAX_MCP_DESCRIPTION_LEN);
    assert!(validate_mcp_description(&max_desc).is_ok());
  }

  #[test]
  fn test_validate_mcp_server_name_accepts_valid() {
    assert!(validate_mcp_server_name("DeepWiki MCP").is_ok());
    assert!(validate_mcp_server_name("a").is_ok());
    assert!(validate_mcp_server_name(&"a".repeat(MAX_MCP_SERVER_NAME_LEN)).is_ok());
  }

  #[test]
  fn test_validate_mcp_server_name_rejects_empty() {
    assert!(validate_mcp_server_name("").is_err());
  }

  #[test]
  fn test_validate_mcp_server_name_rejects_too_long() {
    assert!(validate_mcp_server_name(&"a".repeat(MAX_MCP_SERVER_NAME_LEN + 1)).is_err());
  }

  #[test]
  fn test_validate_mcp_server_url_accepts_valid() {
    assert!(validate_mcp_server_url("https://mcp.deepwiki.com/mcp").is_ok());
    assert!(validate_mcp_server_url("http://localhost:8080/mcp").is_ok());
  }

  #[test]
  fn test_validate_mcp_server_url_rejects_empty() {
    assert!(validate_mcp_server_url("").is_err());
  }

  #[test]
  fn test_validate_mcp_server_url_rejects_invalid() {
    assert!(validate_mcp_server_url("not-a-url").is_err());
    assert!(validate_mcp_server_url("ftp missing colon").is_err());
  }

  #[test]
  fn test_validate_mcp_server_url_rejects_too_long() {
    let long_url = format!("https://example.com/{}", "a".repeat(MAX_MCP_SERVER_URL_LEN));
    assert!(validate_mcp_server_url(&long_url).is_err());
  }

  #[test]
  fn test_validate_mcp_server_description_accepts_valid() {
    assert!(validate_mcp_server_description("").is_ok());
    assert!(validate_mcp_server_description("A test server").is_ok());
    assert!(validate_mcp_server_description(&"a".repeat(MAX_MCP_DESCRIPTION_LEN)).is_ok());
  }

  #[test]
  fn test_validate_mcp_server_description_rejects_too_long() {
    assert!(validate_mcp_server_description(&"a".repeat(MAX_MCP_DESCRIPTION_LEN + 1)).is_err());
  }
}
