use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
// McpAuthParams - Authentication parameters for MCP server connections
// ============================================================================

/// Authentication parameters for MCP server connections.
/// Supports both HTTP headers and URL query parameters.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpAuthParams {
  /// HTTP headers to inject as default headers on the request client.
  /// Each entry is (header_name, header_value).
  pub headers: Vec<(String, String)>,
  /// URL query parameters to append to the MCP server URL.
  /// Each entry is (param_key, param_value).
  pub query_params: Vec<(String, String)>,
}
