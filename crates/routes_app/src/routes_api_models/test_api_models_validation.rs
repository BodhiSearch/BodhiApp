use crate::{
  create_api_model_handler, delete_api_model_handler, fetch_models_handler, get_api_model_handler,
  list_api_models_handler, sync_models_handler, test_api_model_handler, update_api_model_handler,
  ApiKey, ApiModelResponse, CreateApiModelRequest, FetchModelsRequest, TestCreds,
  TestPromptRequest, ENDPOINT_API_MODELS,
};
use anyhow_trace::anyhow_trace;
use axum::{
  http::{Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::test_utils::{test_db_service, AppServiceStubBuilder, TestDbService};
use services::ApiFormat::OpenAI;
use std::sync::Arc;
use tower::ServiceExt;
use validator::Validate;

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
  Router::new()
    .route(ENDPOINT_API_MODELS, get(list_api_models_handler))
    .route(ENDPOINT_API_MODELS, post(create_api_model_handler))
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      get(get_api_model_handler),
    )
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      put(update_api_model_handler),
    )
    .route(
      &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
      delete(delete_api_model_handler),
    )
    .route(
      &format!("{}/test", ENDPOINT_API_MODELS),
      post(test_api_model_handler),
    )
    .route(
      &format!("{}/fetch-models", ENDPOINT_API_MODELS),
      post(fetch_models_handler),
    )
    .route(
      &format!("{}/{{id}}/sync-models", ENDPOINT_API_MODELS),
      post(sync_models_handler),
    )
    .with_state(Arc::new(router_state))
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_handler_validation_error_empty_api_key(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  // Test with raw JSON to trigger deserialization error for empty API key
  let json_request = json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "",  // Invalid: empty api_key
    "models": ["gpt-4"]
  });

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(json_request)?)
    .await?;

  // Verify response status is 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error response contains validation error code for API key
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("json_rejection_error", error_code);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_handler_validation_error_invalid_url(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "not-a-valid-url".to_string(), // Invalid: not a valid URL
    api_key: ApiKey::some("sk-test123456789".to_string())?,
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error response contains validation error code for URL
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("obj_validation_error-validation_errors", error_code);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_handler_validation_error_empty_models(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string())?,
    models: vec![], // Invalid: empty models array
    prefix: None,
    forward_all_with_prefix: false,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error response contains validation error code
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("obj_validation_error-validation_errors", error_code);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_handler_forward_all_with_prefix_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string())?,
    models: vec![], // Empty models is valid for forward_all mode
    prefix: Some("fwd/".to_string()),
    forward_all_with_prefix: true,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 201 Created
  assert_eq!(response.status(), StatusCode::CREATED);

  // Verify the API model was created with forward_all_with_prefix=true
  let response_body = response.json::<ApiModelResponse>().await?;
  assert_eq!(response_body.forward_all_with_prefix, true);
  assert_eq!(response_body.prefix, Some("fwd/".to_string()));
  assert_eq!(response_body.models, Vec::<String>::new());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_handler_forward_all_without_prefix_fails(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string())?,
    models: vec![],
    prefix: None, // Invalid: forward_all_with_prefix requires a prefix
    forward_all_with_prefix: true,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error response contains validation error code for prefix
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!(
    "obj_validation_error-forward_all_requires_prefix",
    error_code
  );

  Ok(())
}

#[rstest]
fn test_api_key_masking() {
  use crate::mask_api_key;

  assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-...abcdef");
  assert_eq!(mask_api_key("short"), "***");
}

#[rstest]
fn test_creds_enum_validation() {
  // Test with ApiKey credentials
  let test_request_with_key = TestPromptRequest {
    creds: TestCreds::ApiKey(ApiKey::some("sk-direct-key".to_string()).unwrap()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "Hello".to_string(),
  };
  assert!(test_request_with_key.validate().is_ok());

  // Test with Id credentials
  let test_request_with_id = TestPromptRequest {
    creds: TestCreds::Id("stored-model-id".to_string()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "Hello".to_string(),
  };
  assert!(test_request_with_id.validate().is_ok());

  // Test with no authentication (ApiKey(None))
  let test_request_no_auth = TestPromptRequest {
    creds: TestCreds::ApiKey(ApiKey::none()),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    prompt: "Hello".to_string(),
  };
  assert!(test_request_no_auth.validate().is_ok());

  // Test FetchModelsRequest variants
  let fetch_request_with_key = FetchModelsRequest {
    creds: TestCreds::ApiKey(ApiKey::some("sk-direct-key".to_string()).unwrap()),
    base_url: "https://api.openai.com/v1".to_string(),
  };
  assert!(fetch_request_with_key.validate().is_ok());

  let fetch_request_with_id = FetchModelsRequest {
    creds: TestCreds::Id("stored-model-id".to_string()),
    base_url: "https://api.openai.com/v1".to_string(),
  };
  assert!(fetch_request_with_id.validate().is_ok());
}
