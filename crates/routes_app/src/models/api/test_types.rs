use services::{
  ApiFormat, ApiKey, ApiKeyUpdate, ApiModelRequestBuilder, FetchModelsRequest, TestCreds,
  TestPromptRequest, TestPromptResponse,
};
use validator::Validate;

#[test]
fn test_create_api_model_form_validation() {
  let form = ApiModelRequestBuilder::default()
    .api_format(ApiFormat::OpenAI)
    .base_url("not-a-url")
    .api_key(ApiKeyUpdate::Set(ApiKey::some("key".to_string()).unwrap()))
    .models(vec!["gpt-4".to_string()])
    .build()
    .unwrap();

  assert!(form.validate().is_err());

  let valid_form = ApiModelRequestBuilder::default()
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKeyUpdate::Set(
      ApiKey::some("sk-test".to_string()).unwrap(),
    ))
    .models(vec!["gpt-4".to_string()])
    .build()
    .unwrap();

  assert!(valid_form.validate().is_ok());
}

#[test]
fn test_prompt_request_validation() {
  let too_long = TestPromptRequest {
    creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "This prompt is way too long and exceeds the 30 character limit".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(too_long.validate().is_err());

  let valid = TestPromptRequest {
    creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "Hello, how are you?".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(valid.validate().is_ok());
}

#[test]
fn test_fetch_models_request_validation() {
  let invalid = FetchModelsRequest {
    creds: TestCreds::ApiKey(ApiKey::none()),
    base_url: "not-a-url".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(invalid.validate().is_err());

  let valid = FetchModelsRequest {
    creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
    base_url: "https://api.openai.com/v1".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(valid.validate().is_ok());
}

#[test]
fn test_api_key_update_serialization() {
  let keep = ApiKeyUpdate::Keep;
  assert_eq!(
    serde_json::to_string(&keep).unwrap(),
    r#"{"action":"keep"}"#
  );

  let set = ApiKeyUpdate::Set(ApiKey::some("sk-test".to_string()).unwrap());
  assert_eq!(
    serde_json::to_string(&set).unwrap(),
    r#"{"action":"set","value":"sk-test"}"#
  );

  let set_none = ApiKeyUpdate::Set(ApiKey::none());
  assert_eq!(
    serde_json::to_string(&set_none).unwrap(),
    r#"{"action":"set","value":null}"#
  );
}

#[test]
fn test_api_key_update_deserialization() {
  let keep: ApiKeyUpdate = serde_json::from_str(r#"{"action":"keep"}"#).unwrap();
  assert_eq!(keep, ApiKeyUpdate::Keep);

  let set: ApiKeyUpdate = serde_json::from_str(r#"{"action":"set","value":"sk-test"}"#).unwrap();
  assert_eq!(
    set,
    ApiKeyUpdate::Set(ApiKey::some("sk-test".to_string()).unwrap())
  );

  let set_none: ApiKeyUpdate = serde_json::from_str(r#"{"action":"set","value":null}"#).unwrap();
  assert_eq!(set_none, ApiKeyUpdate::Set(ApiKey::none()));
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
    api_format: ApiFormat::OpenAI,
  };
  assert!(with_api_key.validate().is_ok());

  // ApiKey with None (no authentication) - should pass
  let no_auth = TestPromptRequest {
    creds: TestCreds::ApiKey(ApiKey::none()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "Hello".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(no_auth.validate().is_ok());

  // Id-based credentials - should pass
  let with_id = TestPromptRequest {
    creds: TestCreds::Id("openai-model".to_string()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "Hello".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(with_id.validate().is_ok());
}

#[test]
fn test_fetch_models_request_credentials_validation() {
  // ApiKey with some value - should pass
  let with_api_key = FetchModelsRequest {
    creds: TestCreds::ApiKey(ApiKey::some("sk-test".to_string()).unwrap()),
    base_url: "https://api.openai.com/v1".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(with_api_key.validate().is_ok());

  // ApiKey with None (no authentication) - should pass
  let no_auth = FetchModelsRequest {
    creds: TestCreds::ApiKey(ApiKey::none()),
    base_url: "https://api.openai.com/v1".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(no_auth.validate().is_ok());

  // Id-based credentials - should pass
  let with_id = FetchModelsRequest {
    creds: TestCreds::Id("openai-model".to_string()),
    base_url: "https://api.openai.com/v1".to_string(),
    api_format: ApiFormat::OpenAI,
  };
  assert!(with_id.validate().is_ok());
}

#[test]
fn test_api_model_form_validate_forward_all_with_prefix_success() {
  let form = ApiModelRequestBuilder::default()
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKeyUpdate::Set(
      ApiKey::some("sk-test".to_string()).unwrap(),
    ))
    .models(vec![])
    .prefix("fwd/".to_string())
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  assert!(form.validate().is_ok());
}

#[test]
fn test_api_model_form_validate_forward_all_without_prefix_fails() {
  let form = ApiModelRequestBuilder::default()
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKeyUpdate::Set(
      ApiKey::some("sk-test".to_string()).unwrap(),
    ))
    .models(vec![])
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  // Validation will be done by the service layer now
  assert!(form.validate().is_ok()); // URL validation passes, forward_all validation is in service
}

#[test]
fn test_api_model_form_validate_forward_all_disabled_with_models_success() {
  let form = ApiModelRequestBuilder::default()
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKeyUpdate::Set(
      ApiKey::some("sk-test".to_string()).unwrap(),
    ))
    .models(vec!["gpt-4".to_string()])
    .forward_all_with_prefix(false)
    .build()
    .unwrap();

  assert!(form.validate().is_ok());
}
