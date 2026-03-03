// NOTE: Types in this file are utility/action DTOs that don't use services/DB
// for persistence. They don't follow the <Domain>Request/<Domain>Response naming
// convention used by CRUD entities.

use serde::{Deserialize, Serialize};
use services::{AppToolsetConfig, ToolDefinition, ToolsetDefinition, ToolsetExecutionRequest};
use utoipa::ToSchema;

// ============================================================================
// Toolset Response DTOs
// ============================================================================

/// Toolset response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolsetResponse {
  /// Unique instance identifier (UUID)
  pub id: String,
  /// User-defined slug for this toolset
  pub slug: String,
  /// Toolset type identifier (e.g., "builtin-exa-search")
  pub toolset_type: String,
  /// Optional description for this toolset
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this toolset is enabled
  pub enabled: bool,
  /// Whether this toolset has an API key configured
  pub has_api_key: bool,
  /// Tools provided by this toolset type
  pub tools: Vec<ToolDefinition>,
  /// When this toolset was created
  #[schema(value_type = String, format = "date-time")]
  pub created_at: chrono::DateTime<chrono::Utc>,
  /// When this toolset was last updated
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// List of toolsets
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetsResponse {
  pub toolsets: Vec<ToolsetResponse>,
  pub toolset_types: Vec<AppToolsetConfig>,
}

// ============================================================================
// Toolset type DTOs (admin endpoints)
// ============================================================================

/// List of toolset types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetTypesResponse {
  pub types: Vec<ToolsetDefinition>,
}

// ============================================================================
// Execute DTOs
// ============================================================================

/// Request to execute a toolset
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecuteToolsetRequest {
  /// Function parameters as JSON
  pub params: serde_json::Value,
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<ToolsetExecutionRequest> for ExecuteToolsetRequest {
  fn from(req: ToolsetExecutionRequest) -> Self {
    Self { params: req.params }
  }
}

impl From<ExecuteToolsetRequest> for ToolsetExecutionRequest {
  fn from(req: ExecuteToolsetRequest) -> Self {
    Self { params: req.params }
  }
}

#[cfg(test)]
mod tests {
  use super::ExecuteToolsetRequest;
  use rstest::rstest;
  use serde_json::json;
  use services::ToolsetExecutionRequest;

  #[rstest]
  fn test_execute_toolset_request_serialization() {
    let req = ExecuteToolsetRequest {
      params: json!({"query": "test query", "num_results": 5}),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("test query", json["params"]["query"]);
    assert_eq!(5, json["params"]["num_results"]);
  }

  #[rstest]
  fn test_execute_toolset_request_conversion() {
    let dto = ExecuteToolsetRequest {
      params: json!({"query": "test"}),
    };

    let domain: ToolsetExecutionRequest = dto.clone().into();
    assert_eq!(json!({"query": "test"}), domain.params);

    let back: ExecuteToolsetRequest = domain.into();
    assert_eq!(dto.params, back.params);
  }
}
