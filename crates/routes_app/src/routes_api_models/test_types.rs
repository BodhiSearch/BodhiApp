use crate::{
  mask_api_key, ApiKey, ApiKeyUpdateAction, CreateApiModelRequestBuilder, FetchModelsRequest,
  TestCreds, TestPromptRequest, TestPromptResponse, UpdateApiModelRequestBuilder,
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
  let request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("not-a-url")
    .api_key(ApiKey::some("key".to_string()).unwrap())
    .models(vec!["gpt-4".to_string()])
    .build()
    .unwrap();

  assert!(request.validate().is_err());

  let valid_request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKey::some("sk-test".to_string()).unwrap())
    .models(vec!["gpt-4".to_string()])
    .build()
    .unwrap();

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
  let keep = ApiKeyUpdate::from(ApiKeyUpdateAction::Keep);
  assert_eq!(keep, ApiKeyUpdate::Keep);

  let set = ApiKeyUpdate::from(ApiKeyUpdateAction::Set(
    ApiKey::some("key".to_string()).unwrap(),
  ));
  assert_eq!(set, ApiKeyUpdate::Set(Some("key".to_string())));

  let set_none = ApiKeyUpdate::from(ApiKeyUpdateAction::Set(ApiKey::none()));
  assert_eq!(set_none, ApiKeyUpdate::Set(None));
}

#[test]
fn test_create_api_model_request_validate_forward_all_with_prefix_success() {
  let request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKey::some("sk-test".to_string()).unwrap())
    .models(vec![])
    .prefix("fwd/".to_string())
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  assert!(request.validate_forward_all().is_ok());
}

#[test]
fn test_create_api_model_request_validate_forward_all_without_prefix_fails() {
  let request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKey::some("sk-test".to_string()).unwrap())
    .models(vec![])
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  let result = request.validate_forward_all();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code.as_ref(), "prefix_required");
}

#[test]
fn test_create_api_model_request_validate_forward_all_with_empty_prefix_fails() {
  let request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKey::some("sk-test".to_string()).unwrap())
    .models(vec![])
    .prefix("   ".to_string())
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  let result = request.validate_forward_all();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code.as_ref(), "prefix_required");
}

#[test]
fn test_create_api_model_request_validate_forward_all_disabled_with_models_success() {
  let request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKey::some("sk-test".to_string()).unwrap())
    .models(vec!["gpt-4".to_string()])
    .forward_all_with_prefix(false)
    .build()
    .unwrap();

  assert!(request.validate_forward_all().is_ok());
}

#[test]
fn test_create_api_model_request_validate_forward_all_disabled_without_models_fails() {
  let request = CreateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .api_key(ApiKey::some("sk-test".to_string()).unwrap())
    .models(vec![])
    .forward_all_with_prefix(false)
    .build()
    .unwrap();

  let result = request.validate_forward_all();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code.as_ref(), "models_required");
}

#[test]
fn test_update_api_model_request_validate_forward_all_with_prefix_success() {
  let request = UpdateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![])
    .prefix("fwd/".to_string())
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  assert!(request.validate_forward_all().is_ok());
}

#[test]
fn test_update_api_model_request_validate_forward_all_without_prefix_fails() {
  let request = UpdateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![])
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  let result = request.validate_forward_all();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code.as_ref(), "prefix_required");
}

#[test]
fn test_update_api_model_request_validate_forward_all_with_empty_prefix_fails() {
  let request = UpdateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![])
    .prefix("  ".to_string())
    .forward_all_with_prefix(true)
    .build()
    .unwrap();

  let result = request.validate_forward_all();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code.as_ref(), "prefix_required");
}

#[test]
fn test_update_api_model_request_validate_forward_all_disabled_with_models_success() {
  let request = UpdateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec!["gpt-4".to_string()])
    .forward_all_with_prefix(false)
    .build()
    .unwrap();

  assert!(request.validate_forward_all().is_ok());
}

#[test]
fn test_update_api_model_request_validate_forward_all_disabled_without_models_fails() {
  let request = UpdateApiModelRequestBuilder::default()
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![])
    .forward_all_with_prefix(false)
    .build()
    .unwrap();

  let result = request.validate_forward_all();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err.code.as_ref(), "models_required");
}
