use objs::{ToolDefinition, ToolExecutionRequest, UserToolConfig};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to update a user's tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateToolConfigRequest {
  /// Whether the tool is enabled for this user
  pub enabled: bool,
  /// Optional API key for the tool (will be encrypted)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub api_key: Option<String>,
}

/// Request to execute a tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecuteToolRequest {
  /// Tool call ID from LLM
  pub tool_call_id: String,
  /// Function arguments as JSON
  pub arguments: serde_json::Value,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response with list of tool definitions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsResponse {
  pub tools: Vec<ToolDefinition>,
}

/// Response with single tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetToolConfigResponse {
  pub config: UserToolConfig,
}

/// Response with list of user tool configurations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolConfigsResponse {
  pub configs: Vec<UserToolConfig>,
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<ToolExecutionRequest> for ExecuteToolRequest {
  fn from(req: ToolExecutionRequest) -> Self {
    Self {
      tool_call_id: req.tool_call_id,
      arguments: req.arguments,
    }
  }
}

impl From<ExecuteToolRequest> for ToolExecutionRequest {
  fn from(req: ExecuteToolRequest) -> Self {
    Self {
      tool_call_id: req.tool_call_id,
      arguments: req.arguments,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;
  use serde_json::json;

  #[rstest]
  fn test_update_tool_config_request_serialization() {
    let req = UpdateToolConfigRequest {
      enabled: true,
      api_key: Some("sk-test123".to_string()),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(true, json["enabled"]);
    assert_eq!("sk-test123", json["api_key"]);
  }

  #[rstest]
  fn test_update_tool_config_request_without_api_key() {
    let req = UpdateToolConfigRequest {
      enabled: false,
      api_key: None,
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(false, json["enabled"]);
    assert!(json.get("api_key").is_none());
  }

  #[rstest]
  fn test_execute_tool_request_serialization() {
    let req = ExecuteToolRequest {
      tool_call_id: "call_123".to_string(),
      arguments: json!({"query": "test query", "num_results": 5}),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("call_123", json["tool_call_id"]);
    assert_eq!("test query", json["arguments"]["query"]);
    assert_eq!(5, json["arguments"]["num_results"]);
  }

  #[rstest]
  fn test_execute_tool_request_conversion() {
    let dto = ExecuteToolRequest {
      tool_call_id: "call_123".to_string(),
      arguments: json!({"query": "test"}),
    };

    let domain: ToolExecutionRequest = dto.clone().into();
    assert_eq!("call_123", domain.tool_call_id);
    assert_eq!(json!({"query": "test"}), domain.arguments);

    let back: ExecuteToolRequest = domain.into();
    assert_eq!(dto.tool_call_id, back.tool_call_id);
  }
}
