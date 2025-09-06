use chrono::{DateTime, Utc};
use objs::{ApiAlias, ApiFormat};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Request to create a new API model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-...",
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "prefix": "openai"
}))]
pub struct CreateApiModelRequest {
  /// API format/protocol (e.g., "openai")
  pub api_format: ApiFormat,

  /// API base URL
  #[validate(url(message = "Base URL must be a valid URL"))]
  pub base_url: String,

  /// API key for authentication
  #[validate(length(min = 1, message = "API key must not be empty"))]
  pub api_key: String,

  /// List of available models
  #[validate(length(min = 1, message = "Models list must not be empty"))]
  pub models: Vec<String>,

  /// Optional prefix for model namespacing (e.g., "azure/" for "azure/gpt-4", "openai:" for "openai:gpt-4")
  pub prefix: Option<String>,
}

/// Request to update an existing API model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-new-key",
    "models": ["gpt-4-turbo", "gpt-3.5-turbo"],
    "prefix": "openai"
}))]
pub struct UpdateApiModelRequest {
  /// API format/protocol (required)
  pub api_format: ApiFormat,

  /// API base URL (required)
  #[validate(url(message = "Base URL must be a valid URL"))]
  pub base_url: String,

  /// API key for authentication (optional, only update if provided for security)
  #[validate(length(min = 1, message = "API key must not be empty"))]
  pub api_key: Option<String>,

  /// List of available models (required)
  #[validate(length(min = 1, message = "Models list must not be empty"))]
  pub models: Vec<String>,

  /// Optional prefix for model namespacing
  pub prefix: Option<String>,
}

/// Request to test API connectivity with a prompt
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_test_prompt_credentials"))]
#[schema(example = json!({
    "api_key": "sk-...",
    "base_url": "https://api.openai.com/v1",
    "model": "gpt-4",
    "prompt": "Hello, how are you?"
}))]
pub struct TestPromptRequest {
  /// API key for authentication (provide either api_key OR id, api_key takes preference if both provided)
  #[validate(length(min = 1))]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[schema(required = false, nullable = false)]
  pub api_key: Option<String>,

  /// API model ID to look up stored credentials (provide either api_key OR id, api_key takes preference if both provided)
  #[validate(length(min = 1))]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[schema(required = false, nullable = false)]
  pub id: Option<String>,

  /// API base URL (optional when using id)
  #[validate(url)]
  pub base_url: String,

  /// Model to use for testing
  #[validate(length(min = 1))]
  pub model: String,

  /// Test prompt (max 30 characters for cost control)
  #[validate(length(min = 1, max = 30))]
  pub prompt: String,
}

/// Request to fetch available models from provider
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_fetch_models_credentials"))]
#[schema(example = json!({
    "api_key": "sk-...",
    "base_url": "https://api.openai.com/v1"
}))]
pub struct FetchModelsRequest {
  /// API key for authentication (provide either api_key OR id, api_key takes preference if both provided)
  #[validate(length(min = 1))]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[schema(required = false, nullable = false)]
  pub api_key: Option<String>,

  /// API model ID to look up stored credentials (provide either api_key OR id, api_key takes preference if both provided)
  #[validate(length(min = 1))]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[schema(required = false, nullable = false)]
  pub id: Option<String>,

  /// API base URL (optional when using id)
  #[validate(url)]
  pub base_url: String,
}

/// Response containing API model configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "openai-gpt4",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "sk-...abc123",
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "prefix": "openai",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
}))]
pub struct ApiModelResponse {
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  pub api_key_masked: String,
  pub models: Vec<String>,
  pub prefix: Option<String>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl ApiModelResponse {
  /// Create a response from an ApiModelAlias with masked API key
  pub fn from_alias(alias: ApiAlias, api_key: Option<String>) -> Self {
    Self {
      id: alias.id,
      api_format: alias.api_format,
      base_url: alias.base_url,
      api_key_masked: api_key
        .map(|k| mask_api_key(&k))
        .unwrap_or_else(|| "***".to_string()),
      models: alias.models,
      prefix: alias.prefix,
      created_at: alias.created_at,
      updated_at: alias.updated_at,
    }
  }
}

/// Response from testing API connectivity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "success": true,
    "response": "Hello! I'm doing well, thank you.",
    "error": null
}))]
pub struct TestPromptResponse {
  pub success: bool,
  pub response: Option<String>,
  pub error: Option<String>,
}

impl TestPromptResponse {
  pub fn success(response: String) -> Self {
    Self {
      success: true,
      response: Some(response),
      error: None,
    }
  }

  pub fn failure(error: String) -> Self {
    Self {
      success: false,
      response: None,
      error: Some(error),
    }
  }
}

/// Response containing available models from provider
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "models": ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"]
}))]
pub struct FetchModelsResponse {
  pub models: Vec<String>,
}

/// Response containing available API formats
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "data": ["openai"]
}))]
pub struct ApiFormatsResponse {
  pub data: Vec<ApiFormat>,
}

/// Paginated response for API model listings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PaginatedApiModelResponse {
  pub data: Vec<ApiModelResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// Mask an API key to show only first 3 and last 6 characters
pub fn mask_api_key(api_key: &str) -> String {
  let len = api_key.len();
  if len <= 12 {
    // If key is too short, just show asterisks
    return "***".to_string();
  }

  let first_3 = &api_key[..3];
  let last_6 = &api_key[len - 6..];
  format!("{}...{}", first_3, last_6)
}

/// Validate that at least one of api_key or id is provided for TestPromptRequest
/// If both are provided, api_key takes preference
fn validate_test_prompt_credentials(
  request: &TestPromptRequest,
) -> Result<(), validator::ValidationError> {
  match (&request.api_key, &request.id) {
    (None, None) => {
      let mut error = validator::ValidationError::new("credentials_missing");
      error.message = Some("Either api_key or id must be provided".into());
      Err(error)
    }
    _ => Ok(()), // Both provided (api_key preferred) or one provided - all valid
  }
}

/// Validate that at least one of api_key or id is provided for FetchModelsRequest
/// If both are provided, api_key takes preference
fn validate_fetch_models_credentials(
  request: &FetchModelsRequest,
) -> Result<(), validator::ValidationError> {
  match (&request.api_key, &request.id) {
    (None, None) => {
      let mut error = validator::ValidationError::new("credentials_missing");
      error.message = Some("Either api_key or id must be provided".into());
      Err(error)
    }
    _ => Ok(()), // Both provided (api_key preferred) or one provided - all valid
  }
}

#[cfg(test)]
mod tests {
  use super::{
    mask_api_key, CreateApiModelRequest, FetchModelsRequest, TestPromptRequest, TestPromptResponse,
  };
  use objs::ApiFormat::OpenAI;
  use validator::Validate;

  #[test]
  fn test_mask_api_key() {
    assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-...abcdef");
    assert_eq!(mask_api_key("short"), "***");
    assert_eq!(mask_api_key("exactlytwelv"), "***"); // exactly 12 chars
    assert_eq!(mask_api_key("thirteenchars"), "thi...nchars"); // 13 chars
  }

  #[test]
  fn test_create_api_model_request_validation() {
    let request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "not-a-url".to_string(),
      api_key: "key".to_string(),
      models: vec!["gpt-4".to_string()],
      prefix: None,
    };

    assert!(request.validate().is_err());

    let valid_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test".to_string(),
      models: vec!["gpt-4".to_string()],
      prefix: None,
    };

    assert!(valid_request.validate().is_ok());
  }

  #[test]
  fn test_prompt_request_validation() {
    let too_long = TestPromptRequest {
      api_key: Some("sk-test".to_string()),
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "This prompt is way too long and exceeds the 30 character limit".to_string(),
    };
    assert!(too_long.validate().is_err());

    let valid = TestPromptRequest {
      api_key: Some("sk-test".to_string()),
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello, how are you?".to_string(),
    };
    assert!(valid.validate().is_ok());
  }

  #[test]
  fn test_fetch_models_request_validation() {
    let invalid = FetchModelsRequest {
      api_key: Some("".to_string()),
      id: None,
      base_url: "not-a-url".to_string(),
    };
    assert!(invalid.validate().is_err());

    let valid = FetchModelsRequest {
      api_key: Some("sk-test".to_string()),
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(valid.validate().is_ok());
  }

  #[test]
  fn test_response_builders() {
    let success = TestPromptResponse::success("Hello!".to_string());
    assert!(success.success);
    assert_eq!(success.response, Some("Hello!".to_string()));
    assert!(success.error.is_none());

    let failure = TestPromptResponse::failure("API error".to_string());
    assert!(!failure.success);
    assert!(failure.response.is_none());
    assert_eq!(failure.error, Some("API error".to_string()));
  }

  #[test]
  fn test_test_prompt_request_credentials_validation() {
    // Both api_key and id provided - should pass (api_key takes preference)
    let both_provided = TestPromptRequest {
      api_key: Some("sk-test".to_string()),
      id: Some("openai-model".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(both_provided.validate().is_ok());

    // Neither provided - should fail
    let neither_provided = TestPromptRequest {
      api_key: None,
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(neither_provided.validate().is_err());

    // Only api_key provided - should pass
    let api_key_only = TestPromptRequest {
      api_key: Some("sk-test".to_string()),
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(api_key_only.validate().is_ok());

    // Only id provided - should pass
    let id_only = TestPromptRequest {
      api_key: None,
      id: Some("openai-model".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(id_only.validate().is_ok());
  }

  #[test]
  fn test_fetch_models_request_credentials_validation() {
    // Both api_key and id provided - should pass (api_key takes preference)
    let both_provided = FetchModelsRequest {
      api_key: Some("sk-test".to_string()),
      id: Some("openai-model".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(both_provided.validate().is_ok());

    // Neither provided - should fail
    let neither_provided = FetchModelsRequest {
      api_key: None,
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(neither_provided.validate().is_err());

    // Only api_key provided - should pass
    let api_key_only = FetchModelsRequest {
      api_key: Some("sk-test".to_string()),
      id: None,
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(api_key_only.validate().is_ok());

    // Only id provided - should pass
    let id_only = FetchModelsRequest {
      api_key: None,
      id: Some("openai-model".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(id_only.validate().is_ok());
  }
}
