use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Tool schema cached from an MCP server's tools/list response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct McpTool {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpAuthParams {
  /// Injected as default headers on the request client.
  pub headers: Vec<(String, String)>,
  /// Appended to the MCP server URL.
  pub query_params: Vec<(String, String)>,
}
