use objs::{AppToolsetConfig, ToolDefinition, ToolsetExecutionRequest, UserToolsetConfig};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to update a user's toolset configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateToolsetConfigRequest {
  /// Whether the toolset is enabled for this user
  pub enabled: bool,
  /// Optional API key for the toolset (will be encrypted)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub api_key: Option<String>,
}

/// Request to execute a toolset
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecuteToolsetRequest {
  /// Tool call ID from LLM
  pub tool_call_id: String,
  /// Function arguments as JSON
  pub arguments: serde_json::Value,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response with list of toolset definitions (enhanced with status)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetsResponse {
  pub toolsets: Vec<ToolsetListItem>,
}

/// Response with single toolset configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetToolsetConfigResponse {
  pub config: UserToolsetConfig,
}

/// Response with list of user toolset configurations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetConfigsResponse {
  pub configs: Vec<UserToolsetConfig>,
}

// ============================================================================
// App-level Toolset Configuration DTOs
// ============================================================================

/// Response with app-level toolset configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppToolsetConfigResponse {
  /// The app-level toolset configuration
  #[serde(flatten)]
  pub config: AppToolsetConfig,
}

/// Response with list of app-level toolset configurations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListAppToolsetConfigsResponse {
  pub configs: Vec<AppToolsetConfig>,
}

/// Enhanced toolset list item with app-level and user-level status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolsetListItem {
  /// Toolset definition
  #[serde(flatten)]
  pub definition: ToolDefinition,
  /// Whether the toolset is enabled at app level (admin-controlled)
  pub app_enabled: bool,
  /// User's configuration for this toolset (if any)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user_config: Option<UserToolsetConfigSummary>,
}

/// Summary of user's toolset configuration (for list responses)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserToolsetConfigSummary {
  /// Whether the user has enabled this toolset
  pub enabled: bool,
  /// Whether the user has configured an API key
  pub has_api_key: bool,
}

/// Enhanced toolset config response with app-level status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnhancedToolsetConfigResponse {
  /// Toolset identifier
  pub toolset_id: String,
  /// Whether the toolset is enabled at app level
  pub app_enabled: bool,
  /// User's configuration
  pub config: UserToolsetConfig,
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<ToolsetExecutionRequest> for ExecuteToolsetRequest {
  fn from(req: ToolsetExecutionRequest) -> Self {
    Self {
      tool_call_id: req.tool_call_id,
      arguments: req.arguments,
    }
  }
}

impl From<ExecuteToolsetRequest> for ToolsetExecutionRequest {
  fn from(req: ExecuteToolsetRequest) -> Self {
    Self {
      tool_call_id: req.tool_call_id,
      tool_name: String::new(), // Will be set by the handler based on toolset_id
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
  fn test_update_toolset_config_request_serialization() {
    let req = UpdateToolsetConfigRequest {
      enabled: true,
      api_key: Some("sk-test123".to_string()),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(true, json["enabled"]);
    assert_eq!("sk-test123", json["api_key"]);
  }

  #[rstest]
  fn test_update_toolset_config_request_without_api_key() {
    let req = UpdateToolsetConfigRequest {
      enabled: false,
      api_key: None,
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(false, json["enabled"]);
    assert!(json.get("api_key").is_none());
  }

  #[rstest]
  fn test_execute_toolset_request_serialization() {
    let req = ExecuteToolsetRequest {
      tool_call_id: "call_123".to_string(),
      arguments: json!({"query": "test query", "num_results": 5}),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("call_123", json["tool_call_id"]);
    assert_eq!("test query", json["arguments"]["query"]);
    assert_eq!(5, json["arguments"]["num_results"]);
  }

  #[rstest]
  fn test_execute_toolset_request_conversion() {
    let dto = ExecuteToolsetRequest {
      tool_call_id: "call_123".to_string(),
      arguments: json!({"query": "test"}),
    };

    let domain: ToolsetExecutionRequest = dto.clone().into();
    assert_eq!("call_123", domain.tool_call_id);
    assert_eq!(json!({"query": "test"}), domain.arguments);

    let back: ExecuteToolsetRequest = domain.into();
    assert_eq!(dto.tool_call_id, back.tool_call_id);
  }
}
