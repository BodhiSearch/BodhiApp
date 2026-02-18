use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// McpServer - Admin URL allowlist entry (public API model)
// ============================================================================

/// Admin-managed MCP server URL allowlist entry.
/// Admins/managers register MCP server URLs that users can then create instances of.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpServer {
  /// Unique identifier (UUID)
  pub id: String,
  /// MCP server URL (exact match, no normalization)
  pub url: String,
  /// Whether this MCP server URL is enabled
  pub enabled: bool,
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
// Mcp - User-owned MCP instance (public API model)
// ============================================================================

/// User-owned MCP server instance with tool caching and filtering.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Mcp {
  /// Unique instance identifier (UUID)
  pub id: String,
  /// Reference to the admin-allowed MCP server
  pub mcp_server_id: String,
  /// MCP server URL (resolved from mcp_servers via join)
  pub url: String,
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
  /// When this instance was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this instance was last updated
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

#[cfg(test)]
mod tests {
  use crate::mcp::{
    validate_mcp_description, validate_mcp_slug, MAX_MCP_DESCRIPTION_LEN, MAX_MCP_SLUG_LEN,
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
}
