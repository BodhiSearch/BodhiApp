use crate::{
  api_models_create, api_models_destroy, api_models_fetch_models, api_models_show, api_models_sync,
  api_models_test, api_models_update, ENDPOINT_MODELS_API,
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
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::test_utils::{openai_model, test_db_service, AppServiceStubBuilder, TestDbService};
use services::AuthContext;
use services::{ApiAliasResponse, ApiKeyUpdate, ApiModelRequest, MockAiApiService};
use services::{ApiFormat::OpenAI, ResourceRole};
use std::sync::Arc;
use tower::ServiceExt;

fn make_mock_ai() -> MockAiApiService {
  let mut mock_ai = MockAiApiService::new();
  mock_ai.expect_fetch_models().returning(|_, _, _, _, _| {
    Ok(vec![
      openai_model("gpt-4"),
      openai_model("gpt-3.5-turbo"),
      openai_model("claude-3"),
    ])
  });
  mock_ai
}

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route(ENDPOINT_MODELS_API, post(api_models_create))
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_API, "{id}"),
      get(api_models_show),
    )
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_API, "{id}"),
      put(api_models_update),
    )
    .route(
      &format!("{}/{}", ENDPOINT_MODELS_API, "{id}"),
      delete(api_models_destroy),
    )
    .route(
      &format!("{}/test", ENDPOINT_MODELS_API),
      post(api_models_test),
    )
    .route(
      &format!("{}/fetch-models", ENDPOINT_MODELS_API),
      post(api_models_fetch_models),
    )
    .route(
      &format!("{}/{{id}}/sync-models", ENDPOINT_MODELS_API),
      post(api_models_sync),
    )
    .layer(axum::Extension(AuthContext::test_session(
      "test-user",
      "testuser",
      ResourceRole::PowerUser,
    )))
    .with_state(app_service)
}

#[rstest]
#[case::prefix_removal(
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-test-key-123"},
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
    "source": "api",
    "id": "placeholder",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "has_api_key": true,
    "models": [{"provider": "openai", "id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::prefix_addition(
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-test-key-123"},
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
    "source": "api",
    "id": "placeholder",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "has_api_key": true,
    "models": [{"provider": "openai", "id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}],
    "prefix": "azure/",
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::prefix_empty_string_removal(
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-test-key-123"},
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
    "source": "api",
    "id": "placeholder",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "has_api_key": true,
    "models": [{"provider": "openai", "id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::prefix_change(
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-test-key-123"},
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
    "source": "api",
    "id": "placeholder",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "has_api_key": true,
    "models": [{"provider": "openai", "id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}],
    "prefix": "openai:",
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::no_prefix_no_change(
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-test-key-123"},
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
    "source": "api",
    "id": "placeholder",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "has_api_key": true,
    "models": [{"provider": "openai", "id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}],
    "prefix": null,
    "forward_all_with_prefix": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  })
)]
#[case::models_and_url_update(
  json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-old-key-123"},
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
    "source": "api",
    "id": "placeholder",
    "api_format": "openai",
    "base_url": "https://api.openai.com/v2",
    "has_api_key": true,
    "models": [
      {"provider": "openai", "id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"},
      {"provider": "openai", "id": "gpt-3.5-turbo", "object": "model", "created": 0, "owned_by": "openai"}
    ],
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

  // Parse create form
  let create_form: ApiModelRequest = serde_json::from_value(create_json.clone())?;

  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .ai_api_service(Arc::new(make_mock_ai()))
      .build()
      .await?,
  );

  // Step 1: Create the API model
  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?;

  assert_eq!(create_response.status(), StatusCode::CREATED);

  // Get the generated model ID from the response
  let create_api_response: ApiAliasResponse = create_response.json().await?;
  let model_id = create_api_response.id;

  // Step 2: Update the API model
  let update_form: ApiModelRequest = serde_json::from_value(update_json)?;
  let update_response = test_router(app_service.clone())
    .oneshot(Request::put(format!("{}/{}", ENDPOINT_MODELS_API, model_id)).json(update_form)?)
    .await?;

  assert_eq!(update_response.status(), StatusCode::OK);

  // Step 3: Get the API model and verify final state
  let get_response = test_router(app_service)
    .oneshot(
      Request::get(format!("{}/{}", ENDPOINT_MODELS_API, model_id))
        .body(axum::body::Body::empty())?,
    )
    .await?;

  assert_eq!(get_response.status(), StatusCode::OK);

  let api_response: ApiAliasResponse = get_response.json().await?;

  // Build expected response with actual timestamps and generated ID
  let mut expected_response: ApiAliasResponse = serde_json::from_value(expected_get_json)?;
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
  let create_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: None,                  // No prefix provided
    forward_all_with_prefix: true, // But forward_all is enabled
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(app_service)
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?;

  // Should return 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error message
  let error_body: serde_json::Value = response.json().await?;
  assert_eq!(
    error_body["error"]["code"].as_str().unwrap(),
    "api_model_service_error-validation"
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
      .ai_api_service(Arc::new(make_mock_ai()))
      .build()
      .await?,
  );

  // Create first API model with prefix
  let first_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: Some("azure/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(first_form)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);

  // Try to create second API model with same prefix
  let second_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["claude-3".to_string()],
    prefix: Some("azure/".to_string()), // Same prefix
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(app_service)
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(second_form)?)
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
      .ai_api_service(Arc::new(make_mock_ai()))
      .build()
      .await?,
  );

  // Create first API model with prefix "azure/"
  let first_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["gpt-4".to_string()],
    prefix: Some("azure/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(first_form)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);

  // Create second API model with different prefix "anthropic/"
  let second_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["claude-3".to_string()],
    prefix: Some("anthropic/".to_string()),
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(second_form)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);
  let second_model: ApiAliasResponse = response.json().await?;
  let second_model_id = second_model.id;

  // Try to update second model to use first model's prefix
  let update_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec!["claude-3".to_string()],
    prefix: Some("azure/".to_string()), // Trying to use existing prefix
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(app_service)
    .oneshot(
      Request::put(format!("{}/{}", ENDPOINT_MODELS_API, second_model_id)).json(update_form)?,
    )
    .await?;

  // Should return 400 Bad Request with prefix_exists error
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  let error_body: serde_json::Value = response.json().await?;
  assert_eq!(error_body["error"]["code"], "db_error-prefix_exists");

  Ok(())
}
