use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};
use utoipa::ToSchema;

// ============================================================================
// ToolScope - OAuth scope for tool authorization
// ============================================================================

/// Tool-specific OAuth scopes for third-party app authorization
///
/// These are discrete permissions (not hierarchical like TokenScope).
/// First-party clients (session, bodhiapp_ tokens) bypass scope checks.
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  EnumString,
  strum::Display,
  EnumIter,
  Serialize,
  Deserialize,
  ToSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ToolScope {
  #[strum(serialize = "scope_tool-builtin-exa-web-search")]
  #[serde(rename = "scope_tool-builtin-exa-web-search")]
  BuiltinExaWebSearch,
}

impl ToolScope {
  /// Extract tool scopes from space-separated scope string
  pub fn from_scope_string(scope: &str) -> Vec<Self> {
    scope
      .split_whitespace()
      .filter_map(|s| s.parse::<ToolScope>().ok())
      .collect()
  }

  /// Get corresponding tool_id for this scope
  pub fn tool_id(&self) -> &'static str {
    match self {
      Self::BuiltinExaWebSearch => "builtin-exa-web-search",
    }
  }

  /// Get scope for a given tool_id
  pub fn scope_for_tool_id(tool_id: &str) -> Option<Self> {
    match tool_id {
      "builtin-exa-web-search" => Some(Self::BuiltinExaWebSearch),
      _ => None,
    }
  }

  /// Get the scope string for OAuth authorization
  pub fn scope_string(&self) -> String {
    self.to_string()
  }
}

// ============================================================================
// ToolDefinition - OpenAI-compatible tool definition format
// ============================================================================

/// Tool definition in OpenAI format for LLM function calling
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
  /// Unique function name (e.g., "builtin-exa-web-search")
  pub name: String,
  /// Human-readable description for LLM
  pub description: String,
  /// JSON Schema for function parameters
  pub parameters: serde_json::Value,
}

// ============================================================================
// UserToolConfig - Per-user tool configuration (public API model)
// ============================================================================

/// User's configuration for a specific tool (API model - no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserToolConfig {
  /// Tool identifier (e.g., "builtin-exa-web-search")
  pub tool_id: String,
  /// Whether the tool is enabled for this user
  pub enabled: bool,
  /// When this configuration was created
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  /// When this configuration was last updated
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

// ============================================================================
// ToolExecution - Request/Response for tool execution
// ============================================================================

/// Request to execute a tool (from LLM tool_calls)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolExecutionRequest {
  /// Unique identifier for this tool call (from LLM response)
  pub tool_call_id: String,
  /// Function arguments as JSON
  pub arguments: serde_json::Value,
}

/// Response from tool execution (to send back to LLM)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolExecutionResponse {
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
  use chrono::Timelike;
  use rstest::rstest;
  use serde_json::json;

  // ============================================================================
  // ToolScope Tests
  // ============================================================================

  #[rstest]
  #[case(
    "scope_tool-builtin-exa-web-search",
    Ok(ToolScope::BuiltinExaWebSearch)
  )]
  fn test_tool_scope_from_str_valid(
    #[case] input: &str,
    #[case] expected: Result<ToolScope, strum::ParseError>,
  ) {
    let result = input.parse::<ToolScope>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected.unwrap());
  }

  #[test]
  fn test_tool_scope_from_str_invalid() {
    let result = "invalid-scope".parse::<ToolScope>();
    assert!(result.is_err());
  }

  #[test]
  fn test_tool_scope_from_scope_string() {
    let scope = "offline_access scope_tool-builtin-exa-web-search openid";
    let scopes = ToolScope::from_scope_string(scope);
    assert_eq!(scopes, vec![ToolScope::BuiltinExaWebSearch]);
  }

  #[test]
  fn test_tool_scope_from_scope_string_multiple() {
    let scope = "scope_tool-builtin-exa-web-search";
    let scopes = ToolScope::from_scope_string(scope);
    assert_eq!(scopes, vec![ToolScope::BuiltinExaWebSearch]);
  }

  #[test]
  fn test_tool_scope_from_scope_string_empty() {
    let scope = "offline_access openid";
    let scopes = ToolScope::from_scope_string(scope);
    assert_eq!(scopes, Vec::<ToolScope>::new());
  }

  #[test]
  fn test_tool_scope_tool_id() {
    assert_eq!(
      ToolScope::BuiltinExaWebSearch.tool_id(),
      "builtin-exa-web-search"
    );
  }

  #[test]
  fn test_tool_scope_for_tool_id() {
    assert_eq!(
      ToolScope::scope_for_tool_id("builtin-exa-web-search"),
      Some(ToolScope::BuiltinExaWebSearch)
    );
    assert_eq!(ToolScope::scope_for_tool_id("unknown-tool"), None);
  }

  #[test]
  fn test_tool_scope_serialization() {
    let scope = ToolScope::BuiltinExaWebSearch;
    let json = serde_json::to_string(&scope).unwrap();
    assert_eq!(json, "\"scope_tool-builtin-exa-web-search\"");
  }

  #[test]
  fn test_tool_scope_deserialization() {
    let json = "\"scope_tool-builtin-exa-web-search\"";
    let scope: ToolScope = serde_json::from_str(json).unwrap();
    assert_eq!(scope, ToolScope::BuiltinExaWebSearch);
  }

  #[test]
  fn test_tool_scope_display() {
    assert_eq!(
      ToolScope::BuiltinExaWebSearch.to_string(),
      "scope_tool-builtin-exa-web-search"
    );
  }

  // ============================================================================
  // ToolDefinition Tests
  // ============================================================================

  #[test]
  fn test_tool_definition_serialization() {
    let def = ToolDefinition {
      tool_type: "function".to_string(),
      function: FunctionDefinition {
        name: "builtin-exa-web-search".to_string(),
        description: "Search the web".to_string(),
        parameters: json!({
          "type": "object",
          "properties": {
            "query": {
              "type": "string",
              "description": "Search query"
            }
          },
          "required": ["query"]
        }),
      },
    };

    let json = serde_json::to_value(&def).unwrap();
    assert_eq!(json["type"], "function");
    assert_eq!(json["function"]["name"], "builtin-exa-web-search");
    assert_eq!(json["function"]["parameters"]["required"][0], "query");
  }

  #[test]
  fn test_tool_definition_deserialization() {
    let json = json!({
      "type": "function",
      "function": {
        "name": "test-tool",
        "description": "Test tool",
        "parameters": {
          "type": "object",
          "properties": {}
        }
      }
    });

    let def: ToolDefinition = serde_json::from_value(json).unwrap();
    assert_eq!(def.tool_type, "function");
    assert_eq!(def.function.name, "test-tool");
  }

  // ============================================================================
  // UserToolConfig Tests
  // ============================================================================

  #[test]
  fn test_user_tool_config_serialization() {
    let now = chrono::Utc::now().with_nanosecond(0).unwrap();
    let config = UserToolConfig {
      tool_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      created_at: now,
      updated_at: now,
    };

    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["tool_id"], "builtin-exa-web-search");
    assert_eq!(json["enabled"], true);
    assert!(
      json.get("api_key").is_none(),
      "API key should never be serialized"
    );
  }

  // ============================================================================
  // ToolExecution Tests
  // ============================================================================

  #[test]
  fn test_tool_execution_request_serialization() {
    let req = ToolExecutionRequest {
      tool_call_id: "call_123".to_string(),
      arguments: json!({"query": "test query", "num_results": 5}),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["tool_call_id"], "call_123");
    assert_eq!(json["arguments"]["query"], "test query");
  }

  #[test]
  fn test_tool_execution_response_success() {
    let resp = ToolExecutionResponse {
      tool_call_id: "call_123".to_string(),
      result: Some(json!({"results": ["result1", "result2"]})),
      error: None,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["tool_call_id"], "call_123");
    assert!(json.get("result").is_some());
    assert!(json.get("error").is_none());
  }

  #[test]
  fn test_tool_execution_response_error() {
    let resp = ToolExecutionResponse {
      tool_call_id: "call_123".to_string(),
      result: None,
      error: Some("Tool execution failed".to_string()),
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["tool_call_id"], "call_123");
    assert!(json.get("result").is_none());
    assert_eq!(json["error"], "Tool execution failed");
  }
}
