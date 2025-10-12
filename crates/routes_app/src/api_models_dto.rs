use chrono::{DateTime, Utc};
use objs::{ApiAlias, ApiFormat};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// Validated API key wrapper - validates length when Some, allows None for public APIs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(transparent)]
pub struct ApiKey(Option<String>);

impl ApiKey {
  /// Create ApiKey with no authentication
  pub fn none() -> Self {
    ApiKey(None)
  }

  /// Create ApiKey with validation
  pub fn some(key: String) -> Result<Self, ValidationError> {
    if key.is_empty() {
      let mut err = ValidationError::new("api_key_empty");
      err.message = Some("API key must not be empty".into());
      return Err(err);
    }
    if key.len() > 4096 {
      let mut err = ValidationError::new("api_key_too_long");
      err.message =
        Some(format!("API key must not exceed 4096 characters, got {}", key.len()).into());
      return Err(err);
    }
    Ok(ApiKey(Some(key)))
  }

  /// Get as Option<&str>
  pub fn as_option(&self) -> Option<&str> {
    self.0.as_deref()
  }

  /// Check if None
  pub fn is_none(&self) -> bool {
    self.0.is_none()
  }

  /// Check if Some
  pub fn is_some(&self) -> bool {
    self.0.is_some()
  }
}

/// Custom deserializer to validate on deserialization
impl<'de> Deserialize<'de> for ApiKey {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
      None => Ok(ApiKey::none()),
      Some(key) => ApiKey::some(key).map_err(serde::de::Error::custom),
    }
  }
}

/// Credentials for test/fetch operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum TestCreds {
  /// Look up credentials from stored API model
  #[schema(example = json!({"type": "id", "value": "openai-gpt4"}))]
  Id(String),

  /// Use direct API key (null for no authentication)
  #[schema(example = json!({"type": "api_key", "value": "sk-1234567890abcdef"}))]
  ApiKey(ApiKey),
}

impl Default for TestCreds {
  fn default() -> Self {
    TestCreds::ApiKey(ApiKey::none())
  }
}

/// Represents an API key update action for API model updates
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(tag = "action", content = "value", rename_all = "lowercase")]
pub enum ApiKeyUpdateAction {
  /// Keep the existing API key unchanged
  Keep,
  /// Set a new API key (or add one if none exists) - can be None for public APIs
  Set(ApiKey),
}

impl From<ApiKeyUpdateAction> for services::db::ApiKeyUpdate {
  fn from(action: ApiKeyUpdateAction) -> Self {
    match action {
      ApiKeyUpdateAction::Keep => services::db::ApiKeyUpdate::Keep,
      ApiKeyUpdateAction::Set(key) => {
        services::db::ApiKeyUpdate::Set(key.as_option().map(|s| s.to_string()))
      }
    }
  }
}

/// Request to create a new API model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-...",  // Optional - null or omit for public APIs
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "prefix": "openai"
}))]
pub struct CreateApiModelRequest {
  /// API format/protocol (e.g., "openai")
  pub api_format: ApiFormat,

  /// API base URL
  #[validate(url(message = "Base URL must be a valid URL"))]
  pub base_url: String,

  /// API key for authentication (null for public APIs)
  #[serde(default = "ApiKey::none", skip_serializing_if = "ApiKey::is_none")]
  pub api_key: ApiKey,

  /// List of available models
  #[validate(length(min = 1, message = "Models list must not be empty"))]
  pub models: Vec<String>,

  /// Optional prefix for model namespacing (e.g., "azure/" for "azure/gpt-4", "openai:" for "openai:gpt-4")
  pub prefix: Option<String>,
}

fn default_api_key_keep() -> ApiKeyUpdateAction {
  ApiKeyUpdateAction::Keep
}

/// Request to update an existing API model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "keep"},
    "models": ["gpt-4-turbo", "gpt-3.5-turbo"],
    "prefix": "openai"
}))]
pub struct UpdateApiModelRequest {
  /// API format/protocol (required)
  pub api_format: ApiFormat,

  /// API base URL (required)
  #[validate(url(message = "Base URL must be a valid URL"))]
  pub base_url: String,

  /// API key update action (Keep/Set with Some or None)
  #[serde(default = "default_api_key_keep")]
  pub api_key: ApiKeyUpdateAction,

  /// List of available models (required)
  #[validate(length(min = 1, message = "Models list must not be empty"))]
  pub models: Vec<String>,

  /// Optional prefix for model namespacing
  pub prefix: Option<String>,
}

/// Request to test API connectivity with a prompt
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "creds": {"type": "api_key", "value": "sk-..."},
    "base_url": "https://api.openai.com/v1",
    "model": "gpt-4",
    "prompt": "Hello, how are you?"
}))]
pub struct TestPromptRequest {
  /// Credentials to use for testing
  #[serde(default)]
  pub creds: TestCreds,

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
    "creds": {"type": "api_key", "value": null},
    "base_url": "http://localhost:8080/v1"
}))]
pub struct FetchModelsRequest {
  /// Credentials to use for fetching models
  #[serde(default)]
  pub creds: TestCreds,

  /// API base URL (required - always needed to know where to fetch models from)
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
  pub api_key_masked: Option<String>,
  pub models: Vec<String>,
  pub prefix: Option<String>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl ApiModelResponse {
  /// Create a response from an ApiModelAlias with masked API key
  ///
  /// # Parameters
  /// * `alias` - The API alias model
  /// * `has_api_key` - Whether an API key exists for this model
  ///
  /// # Returns
  /// * `api_key_masked`: `Some("***")` if key exists, `None` if no key stored
  pub fn from_alias(alias: ApiAlias, has_api_key: bool) -> Self {
    Self {
      id: alias.id,
      api_format: alias.api_format,
      base_url: alias.base_url,
      api_key_masked: if has_api_key {
        Some("***".to_string())
      } else {
        None
      },
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

#[cfg(test)]
mod tests {
  use crate::{
    mask_api_key, ApiKey, ApiKeyUpdateAction, CreateApiModelRequest, FetchModelsRequest, TestCreds,
    TestPromptRequest, TestPromptResponse,
  };
  use objs::ApiFormat::OpenAI;
  use services::db::ApiKeyUpdate;
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
      api_key: ApiKey::some("key".to_string()).unwrap(),
      models: vec!["gpt-4".to_string()],
      prefix: None,
    };

    assert!(request.validate().is_err());

    let valid_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: ApiKey::some("sk-test".to_string()).unwrap(),
      models: vec!["gpt-4".to_string()],
      prefix: None,
    };

    assert!(valid_request.validate().is_ok());
  }

  #[test]
  fn test_prompt_request_validation() {
    let too_long = TestPromptRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "This prompt is way too long and exceeds the 30 character limit".to_string(),
    };
    assert!(too_long.validate().is_err());

    let valid = TestPromptRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello, how are you?".to_string(),
    };
    assert!(valid.validate().is_ok());
  }

  #[test]
  fn test_fetch_models_request_validation() {
    let invalid = FetchModelsRequest {
      creds: TestCreds::ApiKey(ApiKey::none()),
      base_url: "not-a-url".to_string(),
    };
    assert!(invalid.validate().is_err());

    let valid = FetchModelsRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
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
    // ApiKey with some value - should pass
    let with_api_key = TestPromptRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(with_api_key.validate().is_ok());

    // ApiKey with None (no authentication) - should pass
    let no_auth = TestPromptRequest {
      creds: TestCreds::ApiKey(ApiKey::none()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(no_auth.validate().is_ok());

    // Id-based credentials - should pass
    let with_id = TestPromptRequest {
      creds: TestCreds::Id("openai-model".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(with_id.validate().is_ok());
  }

  #[test]
  fn test_fetch_models_request_credentials_validation() {
    // ApiKey with some value - should pass
    let with_api_key = FetchModelsRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(with_api_key.validate().is_ok());

    // ApiKey with None (no authentication) - should pass
    let no_auth = FetchModelsRequest {
      creds: TestCreds::ApiKey(ApiKey::none()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(no_auth.validate().is_ok());

    // Id-based credentials - should pass
    let with_id = FetchModelsRequest {
      creds: TestCreds::Id("openai-model".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(with_id.validate().is_ok());
  }

  #[test]
  fn test_api_key_update_action_serialization() {
    use super::ApiKey;

    let keep = ApiKeyUpdateAction::Keep;
    assert_eq!(
      serde_json::to_string(&keep).unwrap(),
      r#"{"action":"keep"}"#
    );

    let set = ApiKeyUpdateAction::Set(ApiKey::some("sk-test".to_string()).unwrap());
    assert_eq!(
      serde_json::to_string(&set).unwrap(),
      r#"{"action":"set","value":"sk-test"}"#
    );

    let set_none = ApiKeyUpdateAction::Set(ApiKey::none());
    assert_eq!(
      serde_json::to_string(&set_none).unwrap(),
      r#"{"action":"set","value":null}"#
    );
  }

  #[test]
  fn test_api_key_update_action_deserialization() {
    use super::ApiKey;

    let keep: ApiKeyUpdateAction = serde_json::from_str(r#"{"action":"keep"}"#).unwrap();
    assert_eq!(keep, ApiKeyUpdateAction::Keep);

    let set: ApiKeyUpdateAction =
      serde_json::from_str(r#"{"action":"set","value":"sk-test"}"#).unwrap();
    assert_eq!(
      set,
      ApiKeyUpdateAction::Set(ApiKey::some("sk-test".to_string()).unwrap())
    );

    let set_none: ApiKeyUpdateAction =
      serde_json::from_str(r#"{"action":"set","value":null}"#).unwrap();
    assert_eq!(set_none, ApiKeyUpdateAction::Set(ApiKey::none()));
  }

  #[test]
  fn test_api_key_update_action_conversion() {
    use super::ApiKey;

    let keep = ApiKeyUpdate::from(ApiKeyUpdateAction::Keep);
    assert_eq!(keep, ApiKeyUpdate::Keep);

    let set = ApiKeyUpdate::from(ApiKeyUpdateAction::Set(
      ApiKey::some("key".to_string()).unwrap(),
    ));
    assert_eq!(set, ApiKeyUpdate::Set(Some("key".to_string())));

    let set_none = ApiKeyUpdate::from(ApiKeyUpdateAction::Set(ApiKey::none()));
    assert_eq!(set_none, ApiKeyUpdate::Set(None));
  }
}
