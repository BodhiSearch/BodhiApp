use crate::{
  create_api_model_handler, delete_api_model_handler, fetch_models_handler, get_api_model_handler,
  list_api_models_handler, sync_models_handler, test_api_model_handler, update_api_model_handler,
  ApiKey, ApiKeyUpdateAction, ApiModelResponse, CreateApiModelRequest, FetchModelsRequest,
  PaginatedApiModelResponse, TestCreds, TestPromptRequest, UpdateApiModelRequest,
  ENDPOINT_API_MODELS,
};
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use chrono::{DateTime, Utc};
use mockall::predicate;
use objs::ApiAliasBuilder;
use objs::ApiFormat::OpenAI;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  db::{ModelRepository},
  test_utils::{seed_test_api_models, test_db_service, AppServiceStubBuilder, TestDbService},
};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
use validator::Validate;

/// Create expected ApiModelResponse for testing
fn create_expected_response(
  id: &str,
  api_format: &str,
  base_url: &str,
  api_key_masked: Option<&str>,
  models: Vec<String>,
  prefix: Option<String>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
) -> ApiModelResponse {
  use std::str::FromStr;
  ApiModelResponse {
    id: id.to_string(),
    api_format: objs::ApiFormat::from_str(api_format).unwrap(),
    base_url: base_url.to_string(),
    api_key_masked: api_key_masked.map(|s| s.to_string()),
    models,
    prefix,
    forward_all_with_prefix: false,
    created_at,
    updated_at,
  }
}

/// Create expected ApiModelResponse for list view (masked API key)
fn create_expected_list_response(
  id: &str,
  models: Vec<String>,
  created_at: DateTime<Utc>,
) -> ApiModelResponse {
  create_expected_response(
    id,
    "openai",
    "https://api.openai.com/v1",
    Some("***"), // Masked in list view
    models,
    None, // No prefix in original seed data
    created_at,
    created_at,
  )
}

/// Create expected ApiModelResponse for list view with prefix support
fn create_expected_list_response_with_prefix(
  id: &str,
  models: Vec<String>,
  prefix: Option<String>,
  created_at: DateTime<Utc>,
) -> ApiModelResponse {
  create_expected_response(
    id,
    "openai",
    "https://api.openai.com/v1",
    Some("***"), // Masked in list view
    models,
    prefix,
    created_at,
    created_at,
  )
}

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
async fn test_list_api_models_handler(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let base_time = db_service.now();

  // Seed the database with test API model aliases
  let _expected_aliases = seed_test_api_models(&db_service, base_time).await?;

  // Create app service with the seeded database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  // Make request to list API models
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::get(ENDPOINT_API_MODELS)
        .body(Body::empty())
        .unwrap(),
    )
    .await?
    .json::<PaginatedApiModelResponse>()
    .await?;

  // Verify response structure
  assert_eq!(response.total, 7);
  assert_eq!(response.page, 1);
  assert_eq!(response.page_size, 30);
  assert_eq!(response.data.len(), 7);

  // Create expected response array sorted by created_at DESC (newest first)
  let expected_data = vec![
    create_expected_list_response("openai-gpt4", vec!["gpt-4".to_string()], base_time),
    create_expected_list_response(
      "openai-gpt35-turbo",
      vec!["gpt-3.5-turbo".to_string()],
      base_time - chrono::Duration::seconds(10),
    ),
    create_expected_list_response(
      "openai-gpt4-turbo",
      vec!["gpt-4-turbo".to_string()],
      base_time - chrono::Duration::seconds(20),
    ),
    create_expected_list_response(
      "openai-gpt4-vision",
      vec!["gpt-4-vision-preview".to_string()],
      base_time - chrono::Duration::seconds(30),
    ),
    create_expected_list_response(
      "openai-multi-model",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      base_time - chrono::Duration::seconds(40),
    ),
    create_expected_list_response_with_prefix(
      "azure-openai",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      Some("azure/".to_string()),
      base_time - chrono::Duration::seconds(50),
    ),
    create_expected_list_response_with_prefix(
      "custom-alias",
      vec!["custom-model-1".to_string()],
      Some("my.custom_".to_string()),
      base_time - chrono::Duration::seconds(60),
    ),
  ];

  let expected_response = PaginatedApiModelResponse {
    data: expected_data,
    total: 7,
    page: 1,
    page_size: 30,
  };

  // Use pretty_assertions for comprehensive comparison
  assert_eq!(expected_response, response);

  Ok(())
}

#[rstest]
#[case::no_trailing_slash("https://api.openai.com/v1", "https://api.openai.com/v1")]
#[case::single_trailing_slash("https://api.openai.com/v1/", "https://api.openai.com/v1")]
#[case::multiple_trailing_slashes("https://api.openai.com/v1///", "https://api.openai.com/v1")]
#[awt]
#[tokio::test]
async fn test_create_api_model_handler_success(
  #[case] input_url: &str,
  #[case] expected_url: &str,

  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: input_url.to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string()).unwrap(),
    models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  // Make POST request to create API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status
  assert_eq!(response.status(), StatusCode::CREATED);

  // Verify response body
  let api_response = response.json::<ApiModelResponse>().await?;

  // Verify the response structure (note: ID is now auto-generated UUID)
  assert_eq!(api_response.api_format, objs::ApiFormat::OpenAI);
  assert_eq!(api_response.base_url, expected_url);
  assert_eq!(api_response.api_key_masked, Some("***".to_string()));
  assert_eq!(
    api_response.models,
    vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()]
  );
  assert_eq!(api_response.prefix, None);

  // Verify that ID is a valid UUID
  assert!(Uuid::parse_str(&api_response.id).is_ok());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_create_api_model_handler_generates_uuid(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let base_time = db_service.now();

  // Seed database with existing API model
  seed_test_api_models(&db_service, base_time).await?;

  // Create app service with seeded database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string()).unwrap(),
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  // Make POST request to create API model (should succeed since UUIDs are unique)
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 201 Created (no duplicate ID issue with UUIDs)
  assert_eq!(response.status(), StatusCode::CREATED);

  // Verify response structure
  let api_response = response.json::<ApiModelResponse>().await?;
  assert!(Uuid::parse_str(&api_response.id).is_ok());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_create_api_model_handler_validation_error_empty_api_key(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

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

  // Verify error response contains validation error for API key
  let error_response = response.json::<serde_json::Value>().await?;
  let error_message = error_response["error"]["message"].as_str().unwrap();
  assert!(error_message.contains("API key must not be empty"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_create_api_model_handler_validation_error_invalid_url(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "not-a-valid-url".to_string(), // Invalid: not a valid URL
    api_key: ApiKey::some("sk-test123456789".to_string()).unwrap(),
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error response contains validation error for URL
  let error_response = response.json::<serde_json::Value>().await?;
  let error_message = error_response["error"]["message"].as_str().unwrap();
  assert!(error_message.contains("Base URL must be a valid URL"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_create_api_model_handler_validation_error_empty_models(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string()).unwrap(),
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
async fn test_create_api_model_handler_forward_all_with_prefix_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string()).unwrap(),
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
async fn test_create_api_model_handler_forward_all_without_prefix_fails(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123456789".to_string()).unwrap(),
    models: vec![],
    prefix: None, // Invalid: forward_all_with_prefix requires a prefix
    forward_all_with_prefix: true,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?;

  // Verify response status is 400 Bad Request
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  // Verify error response contains validation error for prefix
  let error_response = response.json::<serde_json::Value>().await?;
  let error_message = error_response["error"]["message"].as_str().unwrap();
  assert!(error_message.contains("Prefix is required"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_sync_models_handler_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Set up mock AI API service with expectations
  let mut mock_ai = services::MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .with(
      predicate::eq(Some("sk-test123".to_string())),
      predicate::eq("https://api.openai.com/v1"),
    )
    .returning(|_, _| {
      Ok(vec![
        "gpt-4".to_string(),
        "gpt-3.5-turbo".to_string(),
        "gpt-4-turbo".to_string(),
      ])
    });

  // Create app service with clean database and mock AI service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .ai_api_service(Arc::new(mock_ai))
      .with_secret_service()
      .build()?,
  );

  // First create an API model
  let create_request = CreateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKey::some("sk-test123".to_string()).unwrap(),
    models: vec![],
    prefix: Some("fwd/".to_string()),
    forward_all_with_prefix: true,
  };

  let create_response = test_router(app_service.clone())
    .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
    .await?
    .json::<ApiModelResponse>()
    .await?;

  // Sync models
  let sync_response = test_router(app_service)
    .oneshot(
      Request::post(&format!(
        "/bodhi/v1/api-models/{}/sync-models",
        create_response.id
      ))
      .body(Body::empty())
      .unwrap(),
    )
    .await?;

  // Verify response status is 200 OK
  assert_eq!(StatusCode::OK, sync_response.status());

  // Verify response contains the API model with cached models (unprefixed)
  let sync_body = sync_response.json::<ApiModelResponse>().await?;
  assert_eq!(create_response.id, sync_body.id);
  assert_eq!(OpenAI, sync_body.api_format);
  // Models should be returned without prefix - UI applies prefix
  assert_eq!(
    vec!["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"],
    sync_body.models
  );
  assert_eq!(Some("fwd/".to_string()), sync_body.prefix);
  assert_eq!(true, sync_body.forward_all_with_prefix);

  Ok(())
}

#[rstest]
#[case::no_trailing_slash("https://api.openai.com/v2", "https://api.openai.com/v2")]
#[case::single_trailing_slash("https://api.openai.com/v2/", "https://api.openai.com/v2")]
#[case::multiple_trailing_slashes("https://api.openai.com/v2///", "https://api.openai.com/v2")]
#[awt]
#[tokio::test]
async fn test_update_api_model_handler_success(
  #[case] input_url: &str,
  #[case] expected_url: &str,

  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let base_time = db_service.now();

  // Seed database with existing API model
  seed_test_api_models(&db_service, base_time).await?;

  // Create app service with seeded database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let update_request = UpdateApiModelRequest {
    api_format: OpenAI,
    base_url: input_url.to_string(), // Updated URL with potential trailing slashes
    api_key: ApiKeyUpdateAction::Set(ApiKey::some("sk-updated123456789".to_string()).unwrap()), // New API key
    models: vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()], // Updated models
    prefix: Some("openai".to_string()),
    forward_all_with_prefix: false,
  };

  // Make PUT request to update existing API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::put(&format!("{}/openai-gpt4", ENDPOINT_API_MODELS)).json(update_request)?)
    .await?;

  // Verify response status
  assert_eq!(response.status(), StatusCode::OK);

  // Verify response body
  let api_response = response.json::<ApiModelResponse>().await?;

  // Create expected response with updated values
  let expected_response = create_expected_response(
    "openai-gpt4",
    "openai",
    expected_url, // Expected URL with trailing slashes removed
    Some("***"),  // Updated API key masked
    vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()], // Updated models
    Some("openai".to_string()), // Updated prefix
    base_time,    // Original created_at
    api_response.updated_at, // Use actual updated_at (FrozenTimeService returns same time)
  );

  assert_eq!(expected_response, api_response);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_update_api_model_handler_not_found(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database (no seeded data)
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  let update_request = UpdateApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v2".to_string(),
    api_key: ApiKeyUpdateAction::Set(ApiKey::some("sk-updated123456789".to_string()).unwrap()),
    models: vec!["gpt-4-turbo".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  // Make PUT request to update non-existent API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::put(&format!("{}/non-existent-alias", ENDPOINT_API_MODELS)).json(update_request)?,
    )
    .await?;

  // Verify response status is 404 Not Found
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  // Verify error message
  let error_response = response.json::<serde_json::Value>().await?;
  let error_message = error_response["error"]["message"].as_str().unwrap();
  assert!(error_message.contains("API model 'non-existent-alias' not found"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_delete_api_model_handler_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let base_time = db_service.now();

  // Seed database with existing API model
  seed_test_api_models(&db_service, base_time).await?;

  // Create app service with seeded database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  // Make DELETE request to delete existing API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::delete(&format!("{}/openai-gpt4", ENDPOINT_API_MODELS))
        .body(Body::empty())
        .unwrap(),
    )
    .await?;

  // Verify response status is 204 No Content
  assert_eq!(response.status(), StatusCode::NO_CONTENT);

  // Verify response has no body
  let body = response.into_body();
  let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
  assert!(body_bytes.is_empty());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_delete_api_model_handler_not_found(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database (no seeded data)
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  // Make DELETE request to delete non-existent API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::delete(&format!("{}/non-existent-alias", ENDPOINT_API_MODELS))
        .body(Body::empty())
        .unwrap(),
    )
    .await?;

  // Verify response status is 404 Not Found
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  // Verify error message
  let error_response = response.json::<serde_json::Value>().await?;
  let error_message = error_response["error"]["message"].as_str().unwrap();
  assert!(error_message.contains("API model 'non-existent-alias' not found"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_get_api_model_handler_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let base_time = db_service.now();

  // Seed database with existing API model
  seed_test_api_models(&db_service, base_time).await?;

  // Create app service with seeded database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  // Make GET request to retrieve specific API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::get(&format!("{}/openai-gpt4", ENDPOINT_API_MODELS))
        .body(Body::empty())
        .unwrap(),
    )
    .await?;

  // Verify response status
  assert_eq!(response.status(), StatusCode::OK);

  // Verify response body
  let api_response = response.json::<ApiModelResponse>().await?;

  // Create expected response
  let expected_response = create_expected_response(
    "openai-gpt4",
    "openai",
    "https://api.openai.com/v1",
    Some("***"), // Masked in get view (no API key provided)
    vec!["gpt-4".to_string()],
    None, // No prefix in original seed data
    base_time,
    base_time,
  );

  assert_eq!(expected_response, api_response);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_get_api_model_handler_not_found(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database (no seeded data)
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .with_secret_service()
    .build()?;

  // Make GET request to retrieve non-existent API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::get(&format!("{}/non-existent-alias", ENDPOINT_API_MODELS))
        .body(Body::empty())
        .unwrap(),
    )
    .await?;

  // Verify response status is 404 Not Found
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  // Verify error message
  let error_response = response.json::<serde_json::Value>().await?;
  let error_message = error_response["error"]["message"].as_str().unwrap();
  assert!(error_message.contains("API model 'non-existent-alias' not found"));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_create_api_model_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();

  // Generate a unique ID for the test
  let test_id = Uuid::new_v4().to_string();

  // Create API model via database
  let api_alias = ApiAliasBuilder::test_default()
    .id(test_id.clone())
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec!["gpt-4".to_string()])
    .build_with_time(now)
    .unwrap();

  db_service
    .create_api_model_alias(&api_alias, Some("sk-test123".to_string()))
    .await?;

  // Verify it was created
  let retrieved = db_service.get_api_model_alias(&test_id).await?;
  assert!(retrieved.is_some());
  assert_eq!(retrieved.unwrap().id, test_id);

  // Verify API key is encrypted but retrievable
  let api_key = db_service.get_api_key_for_alias(&test_id).await?;
  assert_eq!(api_key, Some("sk-test123".to_string()));

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_delete_api_model(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();

  // Create API model
  let api_alias = ApiAliasBuilder::test_default()
    .id("to-delete")
    .api_format(OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec!["gpt-4".to_string()])
    .build_with_time(now)
    .unwrap();

  db_service
    .create_api_model_alias(&api_alias, Some("sk-test".to_string()))
    .await?;

  // Verify it exists
  assert!(db_service.get_api_model_alias("to-delete").await?.is_some());

  // Delete it
  db_service.delete_api_model_alias("to-delete").await?;

  // Verify it's gone
  assert!(db_service.get_api_model_alias("to-delete").await?.is_none());

  Ok(())
}

#[test]
fn test_api_key_masking() {
  use crate::mask_api_key;

  assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-...abcdef");
  assert_eq!(mask_api_key("short"), "***");
}

#[test]
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
      .with_secret_service()
      .build()?,
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
    .oneshot(Request::put(&format!("{}/{}", ENDPOINT_API_MODELS, model_id)).json(update_request)?)
    .await?;

  assert_eq!(update_response.status(), StatusCode::OK);

  // Step 3: Get the API model and verify final state
  let get_response = test_router(app_service)
    .oneshot(
      Request::get(&format!("{}/{}", ENDPOINT_API_MODELS, model_id))
        .body(axum::body::Body::empty())
        .unwrap(),
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
async fn test_create_api_model_forward_all_requires_prefix(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?,
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
async fn test_create_api_model_duplicate_prefix_error(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?,
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
async fn test_update_api_model_duplicate_prefix_error(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?,
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
      Request::put(&format!("{}/{}", ENDPOINT_API_MODELS, second_model_id)).json(update_request)?,
    )
    .await?;

  // Should return 400 Bad Request with prefix_exists error
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  let error_body: serde_json::Value = response.json().await?;
  assert_eq!(error_body["error"]["code"], "db_error-prefix_exists");

  Ok(())
}
