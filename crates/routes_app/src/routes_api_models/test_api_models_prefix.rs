use crate::{
  create_api_model_handler, delete_api_model_handler, fetch_models_handler, get_api_model_handler,
  list_api_models_handler, sync_models_handler, test_api_model_handler, update_api_model_handler,
  ApiKey, ApiKeyUpdateAction, ApiModelResponse, CreateApiModelRequest, UpdateApiModelRequest,
  ENDPOINT_API_MODELS,
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
#[case::prefix_removal(
  json!({
    "id": "test-prefix-removal",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-test-key-123",
    "models": ["gpt-4"],
    "prefix": "azure/"
  }),
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "models": ["gpt-4"],
    "prefix": null
  }),
  json!({
    "id": "test-prefix-removal",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "***",
    "models": ["gpt-4"],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::prefix_addition(
  json!({
    "id": "test-prefix-addition",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-test-key-123",
    "models": ["gpt-4"],
    "prefix": null
  }),
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "models": ["gpt-4"],
    "prefix": "azure/"
  }),
  json!({
    "id": "test-prefix-addition",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "***",
    "models": ["gpt-4"],
    "prefix": "azure/",
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::prefix_empty_string_removal(
  json!({
    "id": "test-empty-string-removal",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-test-key-123",
    "models": ["gpt-4"],
    "prefix": "azure/"
  }),
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "models": ["gpt-4"],
    "prefix": ""
  }),
  json!({
    "id": "test-empty-string-removal",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "***",
    "models": ["gpt-4"],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::prefix_change(
  json!({
    "id": "test-prefix-change",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-test-key-123",
    "models": ["gpt-4"],
    "prefix": "azure/"
  }),
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "models": ["gpt-4"],
    "prefix": "openai:"
  }),
  json!({
    "id": "test-prefix-change",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "***",
    "models": ["gpt-4"],
    "prefix": "openai:",
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::no_prefix_no_change(
  json!({
    "id": "test-no-prefix-no-change",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-test-key-123",
    "models": ["gpt-4"],
    "prefix": null
  }),
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "models": ["gpt-4"],
    "prefix": null
  }),
  json!({
    "id": "test-no-prefix-no-change",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key_masked": "***",
    "models": ["gpt-4"],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::models_and_url_update(
  json!({
    "id": "test-models-url-update",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-old-key-123",
    "models": ["gpt-3.5-turbo"],
    "prefix": null
  }),
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v2",
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "prefix": null
  }),
  json!({
    "id": "test-models-url-update",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v2",
    "api_key_masked": "***",
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_api_model_prefix_lifecycle(
  #[case] create_json: serde_json::Value,
  #[case] update_json: serde_json::Value,
  #[case] expected_get_json: serde_json::Value,

  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let _base_time = db_service.now();

  // Parse create request
  let create_request: CreateApiModelRequest = serde_json::from_value(create_json.clone())?;

  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()
      .await?,
  );

  // Step 1: Create the API model
  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  assert_eq!(create_response.status(), StatusCode::CREATED);

  // Get the generated model ID from the response
  let create_api_response: ApiModelResponse = create_response.json().await?;
  let model_id = create_api_response.id;

  // Step 2: Update the API model
  let update_request: UpdateApiModelRequest = serde_json::from_value(update_json)?;
  let update_response = test_router(app_service.clone())
    .oneshot(Request::put(format!("{}/{}", ENDPOINT_API_MODELS, model_id)).json(update_request)?)
    .await?;

  assert_eq!(update_response.status(), StatusCode::OK);

  // Step 3: Get the API model and verify final state
  let get_response = test_router(app_service)
    .oneshot(
      Request::get(format!("{}/{}", ENDPOINT_API_MODELS, model_id))
        .body(axum::body::Body::empty())?,
    )
    .await?;

  assert_eq!(get_response.status(), StatusCode::OK);

  let api_response: ApiModelResponse = get_response.json().await?;

  // Build expected response with actual timestamps and generated ID
  let mut expected_response: ApiModelResponse = serde_json::from_value(expected_get_json)?;
  expected_response.id = api_response.id.clone(); // Use the generated UUID
  expected_response.created_at = api_response.created_at;
  expected_response.updated_at = api_response.updated_at;

  // Use pretty_assertions for comprehensive comparison
  assert_eq!(expected_response, api_response);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_forward_all_requires_prefix(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()
      .await?,
  );

  // Try to create API model with forward_all=true but no prefix
  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::none(),
    models: vec!["gpt-4".to_string()],
    prefix: None,                  // No prefix provided
    forward_all_with_prefix: true, // But forward_all is enabled
  };

  let response = test_router(app_service)
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Should return 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error message
  let error_body: serde_json::Value = response.json().await?;
  assert_eq!(
    error_body["error"]["code"].as_str().unwrap(),
    "obj_validation_error-forward_all_requires_prefix"
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_api_model_duplicate_prefix_error(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()
      .await?,
  );

  // Create first API model with prefix
  let first_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::none(),
    models: vec!["gpt-4".to_string()],
    prefix: Some("azure/".to_string()),
    forward_all_with_prefix: false,
  };

  let response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(first_request)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);

  // Try to create second API model with same prefix
  let second_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKey::none(),
    models: vec!["claude-3".to_string()],
    prefix: Some("azure/".to_string()), // Same prefix
    forward_all_with_prefix: false,
  };

  let response = test_router(app_service)
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(second_request)?)
    .await?;

  // Should return 400 Bad Request with prefix_exists error
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  let error_body: serde_json::Value = response.json().await?;
  assert_eq!(error_body["error"]["code"], "db_error-prefix_exists");

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_api_model_duplicate_prefix_error(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()
      .await?,
  );

  // Create first API model with prefix "azure/"
  let first_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::none(),
    models: vec!["gpt-4".to_string()],
    prefix: Some("azure/".to_string()),
    forward_all_with_prefix: false,
  };

  let response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(first_request)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);

  // Create second API model with different prefix "anthropic/"
  let second_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKey::none(),
    models: vec!["claude-3".to_string()],
    prefix: Some("anthropic/".to_string()),
    forward_all_with_prefix: false,
  };

  let response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(second_request)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);
  let second_model: ApiModelResponse = response.json().await?;
  let second_model_id = second_model.id;

  // Try to update second model to use first model's prefix
  let update_request = UpdateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdateAction::Keep,
    models: vec!["claude-3".to_string()],
    prefix: Some("azure/".to_string()), // Trying to use existing prefix
    forward_all_with_prefix: false,
  };

  let response = test_router(app_service)
    .oneshot(
      Request::put(format!("{}/{}", ENDPOINT_API_MODELS, second_model_id)).json(update_request)?,
    )
    .await?;

  // Should return 400 Bad Request with prefix_exists error
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  let error_body: serde_json::Value = response.json().await?;
  assert_eq!(error_body["error"]["code"], "db_error-prefix_exists");

  Ok(())
}
