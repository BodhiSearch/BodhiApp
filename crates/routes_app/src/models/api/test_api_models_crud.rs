use crate::{
  api_models_create, api_models_destroy, api_models_fetch_models, api_models_show, api_models_sync,
  api_models_test, api_models_update, ENDPOINT_MODELS_API,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{delete, get, post, put},
  Router,
};
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::test_utils::{
  seed_test_api_models, test_db_service, AppServiceStubBuilder, TestDbService,
};
use services::AuthContext;
use services::{ApiAliasResponse, ApiKey, ApiKeyUpdate, ApiModelRequest};
use services::{ApiFormat::OpenAI, ResourceRole};
use std::sync::Arc;
use tower::ServiceExt;
use ulid::Ulid;

/// Create expected ApiAliasResponse for testing
fn create_expected_response(
  id: &str,
  api_format: &str,
  base_url: &str,
  has_api_key: bool,
  models: Vec<String>,
  prefix: Option<String>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
) -> ApiAliasResponse {
  use std::str::FromStr;
  ApiAliasResponse {
    source: "api".to_string(),
    id: id.to_string(),
    api_format: services::ApiFormat::from_str(api_format).unwrap(),
    base_url: base_url.to_string(),
    has_api_key,
    models,
    prefix,
    forward_all_with_prefix: false,
    created_at,
    updated_at,
  }
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
    // Inject a default test auth context so handlers can call require_tenant_id()
    .layer(axum::Extension(AuthContext::test_session(
      "test-user",
      "testuser",
      ResourceRole::PowerUser,
    )))
    .with_state(app_service)
}

#[rstest]
#[case::no_trailing_slash("https://api.openai.com/v1", "https://api.openai.com/v1")]
#[case::single_trailing_slash("https://api.openai.com/v1/", "https://api.openai.com/v1")]
#[case::multiple_trailing_slashes("https://api.openai.com/v1///", "https://api.openai.com/v1")]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .build()
    .await?;

  let create_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: input_url.to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test123456789".to_string())?),
    models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  // Make POST request to create API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?;

  // Verify response status
  assert_eq!(response.status(), StatusCode::CREATED);

  // Verify response body
  let api_response = response.json::<ApiAliasResponse>().await?;

  // Verify the response structure (note: ID is now auto-generated ULID)
  assert_eq!(api_response.api_format, services::ApiFormat::OpenAI);
  assert_eq!(api_response.base_url, expected_url);
  assert!(api_response.has_api_key);
  assert_eq!(
    api_response.models,
    vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()]
  );
  assert_eq!(api_response.prefix, None);

  // Verify that ID is a valid ULID
  assert!(Ulid::from_string(&api_response.id).is_ok());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .build()
    .await?;

  let create_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test123456789".to_string())?),
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  // Make POST request to create API model (should succeed since ULIDs are unique)
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?;

  // Verify response status is 201 Created (no duplicate ID issue with ULIDs)
  assert_eq!(response.status(), StatusCode::CREATED);

  // Verify response structure
  let api_response = response.json::<ApiAliasResponse>().await?;
  assert!(Ulid::from_string(&api_response.id).is_ok());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .build()
    .await?;

  // Make GET request to retrieve specific API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::get(format!("{}/openai-gpt4", ENDPOINT_MODELS_API)).body(Body::empty())?)
    .await?;

  // Verify response status
  assert_eq!(response.status(), StatusCode::OK);

  // Verify response body
  let api_response = response.json::<ApiAliasResponse>().await?;

  // Create expected response
  let expected_response = create_expected_response(
    "openai-gpt4",
    "openai",
    "https://api.openai.com/v1",
    true, // has_api_key
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
#[anyhow_trace]
async fn test_get_api_model_handler_not_found(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database (no seeded data)
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  // Make GET request to retrieve non-existent API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::get(format!("{}/non-existent-alias", ENDPOINT_MODELS_API)).body(Body::empty())?,
    )
    .await?;

  // Verify response status is 404 Not Found
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  // Verify error code
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("api_model_service_error-not_found", error_code);

  Ok(())
}

#[rstest]
#[case::no_trailing_slash("https://api.openai.com/v2", "https://api.openai.com/v2")]
#[case::single_trailing_slash("https://api.openai.com/v2/", "https://api.openai.com/v2")]
#[case::multiple_trailing_slashes("https://api.openai.com/v2///", "https://api.openai.com/v2")]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .build()
    .await?;

  let update_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: input_url.to_string(), // Updated URL with potential trailing slashes
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-updated123456789".to_string())?), // New API key
    models: vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()], // Updated models
    prefix: Some("openai".to_string()),
    forward_all_with_prefix: false,
  };

  // Make PUT request to update existing API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::put(format!("{}/openai-gpt4", ENDPOINT_MODELS_API)).json(update_form)?)
    .await?;

  // Verify response status
  assert_eq!(response.status(), StatusCode::OK);

  // Verify response body
  let api_response = response.json::<ApiAliasResponse>().await?;

  // Create expected response with updated values
  let expected_response = create_expected_response(
    "openai-gpt4",
    "openai",
    expected_url, // Expected URL with trailing slashes removed
    true,         // has_api_key (updated key)
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
#[anyhow_trace]
async fn test_update_api_model_handler_not_found(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database (no seeded data)
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let update_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v2".to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-updated123456789".to_string())?),
    models: vec!["gpt-4-turbo".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
  };

  // Make PUT request to update non-existent API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::put(format!("{}/non-existent-alias", ENDPOINT_MODELS_API)).json(update_form)?)
    .await?;

  // Verify response status is 404 Not Found
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  // Verify error code
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("api_model_service_error-not_found", error_code);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .build()
    .await?;

  // Make DELETE request to delete existing API model
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::delete(format!("{}/openai-gpt4", ENDPOINT_MODELS_API)).body(Body::empty())?)
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
#[anyhow_trace]
async fn test_delete_api_model_handler_not_found(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // Create app service with clean database (no seeded data)
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  // Make DELETE request to delete non-existent API model
  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::delete(format!("{}/non-existent-alias", ENDPOINT_MODELS_API)).body(Body::empty())?,
    )
    .await?;

  // Verify response status is 404 Not Found
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  // Verify error code
  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("api_model_service_error-not_found", error_code);

  Ok(())
}
