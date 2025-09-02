use chrono::{DateTime, Utc};
use objs::ApiModelAlias;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Request to create a new API model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "id": "openai-gpt4",
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-...",
    "models": ["gpt-4", "gpt-3.5-turbo"]
}))]
pub struct CreateApiModelRequest {
  /// Unique identifier for this API configuration
  #[validate(length(
    min = 1,
    max = 100,
    message = "ID must not be empty and should be between 1 and 100 characters"
  ))]
  pub id: String,

  /// Provider name (e.g., "openai", "anthropic")
  #[validate(length(
    min = 1,
    max = 50,
    message = "Provider must not be empty and should be between 1 and 50 characters"
  ))]
  pub provider: String,

  /// API base URL
  #[validate(url(message = "Base URL must be a valid URL"))]
  pub base_url: String,

  /// API key for authentication
  #[validate(length(min = 1, message = "API key must not be empty"))]
  pub api_key: String,

  /// List of available models
  #[validate(length(min = 1, message = "Models list must not be empty"))]
  pub models: Vec<String>,
}

/// Request to update an existing API model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-new-key",
    "models": ["gpt-4-turbo", "gpt-3.5-turbo"]
}))]
pub struct UpdateApiModelRequest {
  /// Provider name (optional)
  #[validate(length(
    min = 1,
    max = 50,
    message = "Provider must not be empty and should be between 1 and 50 characters"
  ))]
  pub provider: Option<String>,

  /// API base URL (optional)
  #[validate(url(message = "Base URL must be a valid URL"))]
  pub base_url: Option<String>,

  /// API key for authentication (optional, only update if provided)
  #[validate(length(min = 1, message = "API key must not be empty"))]
  pub api_key: Option<String>,

  /// List of available models (optional)
  #[validate(length(min = 1, message = "Models list must not be empty"))]
  pub models: Option<Vec<String>>,
}

/// Request to test API connectivity with a prompt
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "api_key": "sk-...",
    "base_url": "https://api.openai.com/v1",
    "model": "gpt-4",
    "prompt": "Hello, how are you?"
}))]
pub struct TestPromptRequest {
  /// API key for authentication
  #[validate(length(min = 1))]
  pub api_key: String,

  /// API base URL
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
#[schema(example = json!({
    "api_key": "sk-...",
    "base_url": "https://api.openai.com/v1"
}))]
pub struct FetchModelsRequest {
  /// API key for authentication
  #[validate(length(min = 1))]
  pub api_key: String,

  /// API base URL
  #[validate(url)]
  pub base_url: String,
}

/// Response containing API model configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "openai-gpt4",
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "sk-...abc123",
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
}))]
pub struct ApiModelResponse {
  pub id: String,
  pub provider: String,
  pub base_url: String,
  pub api_key_masked: String,
  pub models: Vec<String>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl ApiModelResponse {
  /// Create a response from an ApiModelAlias with masked API key
  pub fn from_alias(alias: ApiModelAlias, api_key: Option<String>) -> Self {
    Self {
      id: alias.id,
      provider: alias.provider,
      base_url: alias.base_url,
      api_key_masked: api_key
        .map(|k| mask_api_key(&k))
        .unwrap_or_else(|| "***".to_string()),
      models: alias.models,
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

#[cfg(test)]
mod tests {
  use super::{
    mask_api_key, CreateApiModelRequest, FetchModelsRequest, TestPromptRequest, TestPromptResponse,
  };
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
      id: "test".to_string(),
      provider: "openai".to_string(),
      base_url: "not-a-url".to_string(),
      api_key: "key".to_string(),
      models: vec!["gpt-4".to_string()],
    };

    assert!(request.validate().is_err());

    let valid_request = CreateApiModelRequest {
      id: "test".to_string(),
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test".to_string(),
      models: vec!["gpt-4".to_string()],
    };

    assert!(valid_request.validate().is_ok());
  }

  #[test]
  fn test_prompt_request_validation() {
    let too_long = TestPromptRequest {
      api_key: "sk-test".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "This prompt is way too long and exceeds the 30 character limit".to_string(),
    };
    assert!(too_long.validate().is_err());

    let valid = TestPromptRequest {
      api_key: "sk-test".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello, how are you?".to_string(),
    };
    assert!(valid.validate().is_ok());
  }

  #[test]
  fn test_fetch_models_request_validation() {
    let invalid = FetchModelsRequest {
      api_key: "".to_string(),
      base_url: "not-a-url".to_string(),
    };
    assert!(invalid.validate().is_err());

    let valid = FetchModelsRequest {
      api_key: "sk-test".to_string(),
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
}
