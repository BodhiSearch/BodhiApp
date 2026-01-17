use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::ToSchema;

// ============================================================================
// ToolsetScope - OAuth scope for toolset authorization
// ============================================================================

/// Toolset-specific OAuth scopes for third-party app authorization
///
/// These are discrete permissions (not hierarchical like TokenScope).
/// One scope grants access to all tools within the toolset.
/// First-party clients (session, bodhiapp_ tokens) bypass scope checks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ToolsetScope {
  /// The scope string (e.g., "scope_toolset-builtin-exa-web-search")
  scope: String,
  /// The corresponding toolset ID (e.g., "builtin-exa-web-search")
  toolset_id: String,
}

/// Known toolset scope identifiers
pub mod toolset_scopes {
  pub const BUILTIN_EXA_WEB_SEARCH_SCOPE: &str = "scope_toolset-builtin-exa-web-search";
  pub const BUILTIN_EXA_WEB_SEARCH_ID: &str = "builtin-exa-web-search";
}

impl ToolsetScope {
  /// Create the Exa Web Search toolset scope
  pub fn builtin_exa_web_search() -> Self {
    Self {
      scope: toolset_scopes::BUILTIN_EXA_WEB_SEARCH_SCOPE.to_string(),
      toolset_id: toolset_scopes::BUILTIN_EXA_WEB_SEARCH_ID.to_string(),
    }
  }

  /// Get all known toolset scopes
  pub fn all() -> Vec<Self> {
    vec![Self::builtin_exa_web_search()]
  }

  /// Extract toolset scopes from space-separated scope string
  pub fn from_scope_string(scope: &str) -> Vec<Self> {
    scope
      .split_whitespace()
      .filter_map(|s| s.parse::<ToolsetScope>().ok())
      .collect()
  }

  /// Get corresponding toolset_id for this scope
  pub fn toolset_id(&self) -> &str {
    &self.toolset_id
  }

  /// Get scope for a given toolset_id
  pub fn scope_for_toolset_id(toolset_id: &str) -> Option<Self> {
    Self::all().into_iter().find(|s| s.toolset_id == toolset_id)
  }

  /// Get the scope string for OAuth authorization
  pub fn scope_string(&self) -> &str {
    &self.scope
  }
}

impl fmt::Display for ToolsetScope {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.scope)
  }
}

/// Error when parsing an invalid toolset scope string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseToolsetScopeError(String);

impl fmt::Display for ParseToolsetScopeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "invalid toolset scope: {}", self.0)
  }
}

impl std::error::Error for ParseToolsetScopeError {}

impl FromStr for ToolsetScope {
  type Err = ParseToolsetScopeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::all()
      .into_iter()
      .find(|scope| scope.scope == s)
      .ok_or_else(|| ParseToolsetScopeError(s.to_string()))
  }
}

// ============================================================================
// ToolsetDefinition - Toolset containing multiple tools
// ============================================================================

/// A toolset is a connector that provides one or more tools.
/// Example: Exa Web Search toolset provides search, find_similar, get_contents, answer tools.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetDefinition {
  /// Unique toolset identifier (e.g., "builtin-exa-web-search")
  pub toolset_id: String,
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
/// Tool name follows Claude MCP convention: toolset__{toolset_id}__{tool_name}
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
  /// Fully qualified tool name: toolset__{toolset_id}__{tool_name}
  pub name: String,
  /// Human-readable description for LLM
  pub description: String,
  /// JSON Schema for function parameters
  pub parameters: serde_json::Value,
}

// ============================================================================
// UserToolsetConfig - Per-user toolset configuration (public API model)
// ============================================================================

/// User's configuration for a specific toolset (API model - no sensitive data).
/// API key is stored at toolset level (one key for all tools in toolset).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserToolsetConfig {
  /// Toolset identifier (e.g., "builtin-exa-web-search")
  pub toolset_id: String,
  /// Whether the toolset is enabled for this user
  pub enabled: bool,
  /// When this configuration was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this configuration was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// AppToolsetConfig - App-level toolset configuration (public API model)
// ============================================================================

/// App-level configuration for a toolset (admin-controlled)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppToolsetConfig {
  /// Toolset identifier (e.g., "builtin-exa-web-search")
  pub toolset_id: String,
  /// Whether the toolset is enabled for this app instance
  pub enabled: bool,
  /// User ID of the admin who last updated this configuration
  pub updated_by: String,
  /// When this configuration was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this configuration was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// ToolsetExecution - Request/Response for toolset tool execution
// ============================================================================

/// Request to execute a tool within a toolset (from LLM tool_calls)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetExecutionRequest {
  /// Unique identifier for this tool call (from LLM response)
  pub tool_call_id: String,
  /// Fully qualified tool name: toolset__{toolset_id}__{tool_name}
  pub tool_name: String,
  /// Function arguments as JSON
  pub arguments: serde_json::Value,
}

/// Response from toolset tool execution (to send back to LLM)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetExecutionResponse {
  /// Tool call ID this response is for
  pub tool_call_id: String,
  /// Successful result (JSON), if any
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<serde_json::Value>,
  /// Error message, if execution failed
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_toolset_scope_from_scope_string_extracts_known_scopes() {
    let scope = "offline_access scope_toolset-builtin-exa-web-search openid";
    let scopes = ToolsetScope::from_scope_string(scope);
    assert_eq!(scopes, vec![ToolsetScope::builtin_exa_web_search()]);
  }

  #[test]
  fn test_toolset_scope_from_scope_string_returns_empty_for_no_matches() {
    let scope = "offline_access openid";
    let scopes = ToolsetScope::from_scope_string(scope);
    assert!(scopes.is_empty());
  }

  #[test]
  fn test_toolset_scope_for_toolset_id_lookup() {
    assert_eq!(
      ToolsetScope::scope_for_toolset_id("builtin-exa-web-search"),
      Some(ToolsetScope::builtin_exa_web_search())
    );
    assert_eq!(ToolsetScope::scope_for_toolset_id("unknown-toolset"), None);
  }

  #[test]
  fn test_toolset_scope_all_registry() {
    assert!(ToolsetScope::all().contains(&ToolsetScope::builtin_exa_web_search()));
  }
}
