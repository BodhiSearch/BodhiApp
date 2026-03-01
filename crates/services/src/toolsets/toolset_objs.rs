use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// ToolsetDefinition - Toolset containing multiple tools
// ============================================================================

/// A toolset is a connector that provides one or more tools.
/// Example: Exa Web Search toolset provides search, find_similar, get_contents, answer tools.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetDefinition {
  /// Toolset type identifier (e.g., "builtin-exa-search")
  pub toolset_type: String,
  /// Human-readable name (e.g., "Exa Web Search")
  pub name: String,
  /// Description of the toolset
  pub description: String,
  /// Tools provided by this toolset (in OpenAI format)
  pub tools: Vec<ToolDefinition>,
}

// ============================================================================
// ToolDefinition - OpenAI-compatible tool definition format
// ============================================================================

/// Tool definition in OpenAI format for LLM function calling.
/// Tool name follows Claude MCP convention: toolset__{toolset_name}__{tool_name}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolDefinition {
  /// Type of tool (always "function" for now)
  #[serde(rename = "type")]
  pub tool_type: String,
  /// Function definition details
  pub function: FunctionDefinition,
}

/// Function definition within a tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct FunctionDefinition {
  /// Simple tool name (e.g., "search", "findSimilar"). Frontend composes fully qualified name.
  pub name: String,
  /// Human-readable description for LLM
  pub description: String,
  /// JSON Schema for function parameters
  pub parameters: serde_json::Value,
}

// ============================================================================
// Toolset - Multi-instance toolset configuration (public API model)
// ============================================================================

/// User-owned toolset instance with UUID identification
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Toolset {
  /// Unique instance identifier (UUID)
  pub id: String,
  /// User-defined slug for this instance
  pub slug: String,
  /// Toolset type identifier (e.g., "builtin-exa-search")
  pub toolset_type: String,
  /// Optional description for this instance
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this instance is enabled
  pub enabled: bool,
  /// Whether this instance has an API key configured
  pub has_api_key: bool,
  /// When this instance was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this instance was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

/// Application-level toolset configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppToolsetConfig {
  /// Toolset type identifier (e.g., "builtin-exa-search")
  pub toolset_type: String,
  /// Human-readable name (e.g., "Exa Web Search")
  pub name: String,
  /// Description of the toolset
  pub description: String,
  /// Whether this toolset type is enabled at app level
  pub enabled: bool,
  /// User who last updated this config
  pub updated_by: String,
  /// When this config was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this config was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Toolset validation functions
// ============================================================================

static TOOLSET_SLUG_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9-]+$").expect("Invalid toolset slug regex"));

pub const MAX_TOOLSET_SLUG_LEN: usize = 24;
pub const MAX_TOOLSET_DESCRIPTION_LEN: usize = 255;

/// Validate toolset instance slug format and length
pub fn validate_toolset_slug(slug: &str) -> Result<(), String> {
  if slug.is_empty() {
    return Err("Toolset slug cannot be empty".to_string());
  }
  if slug.len() > MAX_TOOLSET_SLUG_LEN {
    return Err(format!(
      "Toolset slug cannot exceed {} characters",
      MAX_TOOLSET_SLUG_LEN
    ));
  }
  if !TOOLSET_SLUG_REGEX.is_match(slug) {
    return Err("Toolset slug can only contain alphanumeric characters and hyphens".to_string());
  }
  Ok(())
}

/// Validate toolset instance description length
pub fn validate_toolset_description(description: &str) -> Result<(), String> {
  if description.len() > MAX_TOOLSET_DESCRIPTION_LEN {
    return Err(format!(
      "Toolset description cannot exceed {} characters",
      MAX_TOOLSET_DESCRIPTION_LEN
    ));
  }
  Ok(())
}

// ============================================================================
// ToolsetExecution - Request/Response for toolset tool execution
// ============================================================================

/// Request to execute a tool within a toolset (from LLM tool_calls)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetExecutionRequest {
  /// Function parameters as JSON
  pub params: serde_json::Value,
}

/// Response from toolset tool execution (to send back to LLM)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetExecutionResponse {
  /// Successful result (JSON), if any
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<serde_json::Value>,
  /// Error message, if execution failed
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

#[cfg(test)]
#[path = "test_toolset_objs.rs"]
mod test_toolset_objs;
