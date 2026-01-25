use objs::{AppToolsetConfig, ToolDefinition, ToolsetExecutionRequest};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

// ============================================================================
// Validation Patterns
// ============================================================================

static TOOLSET_NAME_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9 _-]{0,22}[a-zA-Z0-9]$").unwrap());

fn validate_toolset_name(name: &str) -> Result<(), ValidationError> {
  if !TOOLSET_NAME_REGEX.is_match(name) {
    return Err(ValidationError::new("invalid_toolset_name"));
  }
  Ok(())
}

fn default_true() -> bool {
  true
}

// ============================================================================
// Toolset CRUD DTOs
// ============================================================================

/// Request to create a toolset
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateToolsetRequest {
  /// Toolset scope UUID identifier (e.g., "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
  #[validate(length(min = 1))]
  pub scope_uuid: String,

  /// User-defined name for this toolset (2-24 chars, alphanumeric + spaces/dash/underscore)
  #[validate(length(min = 1, max = 24), custom(function = "validate_toolset_name"))]
  pub name: String,

  /// Optional description for this toolset
  #[serde(skip_serializing_if = "Option::is_none")]
  #[validate(length(max = 255))]
  pub description: Option<String>,

  /// Whether this toolset is enabled
  #[serde(default = "default_true")]
  pub enabled: bool,

  /// API key for the toolset
  pub api_key: String,
}

/// Request to update a toolset (full PUT - all fields required except api_key)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateToolsetRequest {
  /// User-defined name for this toolset
  #[validate(length(min = 1, max = 24), custom(function = "validate_toolset_name"))]
  pub name: String,

  /// Optional description for this toolset
  #[serde(skip_serializing_if = "Option::is_none")]
  #[validate(length(max = 255))]
  pub description: Option<String>,

  /// Whether this toolset is enabled
  pub enabled: bool,

  /// API key update action (Keep or Set)
  #[serde(default)]
  pub api_key: ApiKeyUpdateDto,
}

/// API key update enum (mirrors services::db::ApiKeyUpdate)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[serde(tag = "action", content = "value")]
pub enum ApiKeyUpdateDto {
  /// Keep the existing API key unchanged
  #[default]
  Keep,
  /// Set a new API key (or clear if None)
  Set(Option<String>),
}

/// Toolset response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolsetResponse {
  /// Unique instance identifier (UUID)
  pub id: String,
  /// User-defined name for this toolset
  pub name: String,
  /// Toolset scope UUID identifier
  pub scope_uuid: String,
  /// Toolset scope identifier (e.g., "scope_toolset-builtin-exa-web-search")
  pub scope: String,
  /// Optional description for this toolset
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this toolset is enabled
  pub enabled: bool,
  /// Whether this toolset has an API key configured
  pub has_api_key: bool,
  /// Whether the toolset type is enabled at app level
  pub app_enabled: bool,
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
}

// ============================================================================
// Toolset type DTOs (admin endpoints)
// ============================================================================

/// Toolset type response (for admin listing)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolsetTypeResponse {
  /// Toolset scope UUID identifier
  pub scope_uuid: String,
  /// Toolset scope identifier (e.g., "scope_toolset-builtin-exa-web-search")
  pub scope: String,
  /// Human-readable name (e.g., "Exa Web Search")
  pub name: String,
  /// Description of the toolset
  pub description: String,
  /// Whether the toolset is enabled at app level (admin-controlled)
  pub app_enabled: bool,
  /// Tools provided by this toolset
  pub tools: Vec<ToolDefinition>,
}

/// List of toolset types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetTypesResponse {
  pub types: Vec<ToolsetTypeResponse>,
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
  use super::*;
  use rstest::rstest;
  use serde_json::json;

  #[rstest]
  fn test_create_toolset_request_serialization() {
    let req = CreateToolsetRequest {
      scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
      name: "My Exa".to_string(),
      description: Some("Test instance".to_string()),
      enabled: true,
      api_key: "sk-test123".to_string(),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("4ff0e163-36fb-47d6-a5ef-26e396f067d6", json["scope_uuid"]);
    assert_eq!("My Exa", json["name"]);
    assert_eq!(true, json["enabled"]);
    assert_eq!("sk-test123", json["api_key"]);
  }

  #[rstest]
  fn test_update_toolset_request_with_api_key_keep() {
    let req = UpdateToolsetRequest {
      name: "Updated Name".to_string(),
      description: None,
      enabled: false,
      api_key: ApiKeyUpdateDto::Keep,
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("Updated Name", json["name"]);
    assert_eq!(false, json["enabled"]);
    assert_eq!("Keep", json["api_key"]["action"]);
  }

  #[rstest]
  fn test_update_toolset_request_with_api_key_set() {
    let req = UpdateToolsetRequest {
      name: "Updated Name".to_string(),
      description: Some("Updated desc".to_string()),
      enabled: true,
      api_key: ApiKeyUpdateDto::Set(Some("new-key".to_string())),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("Updated Name", json["name"]);
    assert_eq!("Updated desc", json["description"]);
    assert_eq!(true, json["enabled"]);
    assert_eq!("Set", json["api_key"]["action"]);
    assert_eq!("new-key", json["api_key"]["value"]);
  }

  #[rstest]
  fn test_api_key_update_dto_default() {
    let dto: ApiKeyUpdateDto = Default::default();
    match dto {
      ApiKeyUpdateDto::Keep => (),
      _ => panic!("Default should be Keep"),
    }
  }

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
