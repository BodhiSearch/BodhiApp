use objs::{ToolDefinition, ToolsetDefinition, ToolsetExecutionRequest};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

// ============================================================================
// Validation Patterns
// ============================================================================

static TOOLSET_SLUG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9-]+$").unwrap());

fn validate_toolset_slug(slug: &str) -> Result<(), ValidationError> {
  if !TOOLSET_SLUG_REGEX.is_match(slug) {
    return Err(ValidationError::new("invalid_toolset_slug"));
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
  /// Toolset type identifier (e.g., "builtin-exa-search")
  #[validate(length(min = 1))]
  pub toolset_type: String,

  /// User-defined slug for this toolset (1-24 chars, alphanumeric + hyphens)
  #[validate(length(min = 1, max = 24), custom(function = "validate_toolset_slug"))]
  pub slug: String,

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
  /// User-defined slug for this toolset
  #[validate(length(min = 1, max = 24), custom(function = "validate_toolset_slug"))]
  pub slug: String,

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
  pub toolset_types: Vec<objs::AppToolsetConfig>,
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
  use super::*;
  use rstest::rstest;
  use serde_json::json;

  #[rstest]
  fn test_create_toolset_request_serialization() {
    let req = CreateToolsetRequest {
      toolset_type: "builtin-exa-search".to_string(),
      slug: "my-exa".to_string(),
      description: Some("Test instance".to_string()),
      enabled: true,
      api_key: "sk-test123".to_string(),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("builtin-exa-search", json["toolset_type"]);
    assert_eq!("my-exa", json["slug"]);
    assert_eq!(true, json["enabled"]);
    assert_eq!("sk-test123", json["api_key"]);
  }

  #[rstest]
  fn test_update_toolset_request_with_api_key_keep() {
    let req = UpdateToolsetRequest {
      slug: "updated-name".to_string(),
      description: None,
      enabled: false,
      api_key: ApiKeyUpdateDto::Keep,
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("updated-name", json["slug"]);
    assert_eq!(false, json["enabled"]);
    assert_eq!("Keep", json["api_key"]["action"]);
  }

  #[rstest]
  fn test_update_toolset_request_with_api_key_set() {
    let req = UpdateToolsetRequest {
      slug: "updated-name".to_string(),
      description: Some("Updated desc".to_string()),
      enabled: true,
      api_key: ApiKeyUpdateDto::Set(Some("new-key".to_string())),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("updated-name", json["slug"]);
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
