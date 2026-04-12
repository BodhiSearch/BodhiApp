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
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::test_utils::{
  openai_model, seed_test_api_models, test_db_service, AppServiceStubBuilder, TestDbService,
};
use services::AuthContext;
use services::{ApiAliasResponse, ApiKey, ApiKeyUpdate, ApiModelRequest, MockAiApiService};
use services::{ApiFormat, ApiFormat::OpenAI, ResourceRole};
use std::sync::Arc;
use tower::ServiceExt;
use ulid::Ulid;

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
  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .returning(|_, _, _, _, _| Ok(vec![openai_model("gpt-4"), openai_model("gpt-3.5-turbo")]));
  // Create app service with clean database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_service(Arc::new(mock_ai))
    .build()
    .await?;

  let create_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: input_url.to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test123456789".to_string())?),
    models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
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
  let model_ids: Vec<&str> = api_response.models.iter().map(|m| m.id()).collect();
  assert_eq!(model_ids, vec!["gpt-4", "gpt-3.5-turbo"]);
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

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .returning(|_, _, _, _, _| Ok(vec![openai_model("gpt-4")]));

  // Create app service with seeded database
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_service(Arc::new(mock_ai))
    .build()
    .await?;

  let create_form = ApiModelRequest {
    api_format: OpenAI,
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-test123456789".to_string())?),
    models: vec!["gpt-4".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
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
async fn test_create_api_model_handler_anthropic_oauth_stores_extra_fields(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  use serde_json::json;

  let extra_headers = json!({"anthropic-beta": "claude-code-20250219,oauth-2025-04-20"});
  let extra_body = json!({"system": [{"type": "text", "text": "You are Claude Code..."}]});

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .returning(|_, _, _, _, _| Ok(vec![openai_model("claude-3-5-sonnet")]));

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_service(Arc::new(mock_ai))
    .build()
    .await?;

  let create_form = ApiModelRequest {
    api_format: ApiFormat::AnthropicOAuth,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Set(ApiKey::some("sk-ant-oat01-token".to_string())?),
    models: vec!["claude-3-5-sonnet".to_string()],
    prefix: None,
    forward_all_with_prefix: false,
    extra_headers: Some(extra_headers.clone()),
    extra_body: Some(extra_body.clone()),
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(ENDPOINT_MODELS_API).json(create_form)?)
    .await?;

  assert_eq!(response.status(), StatusCode::CREATED);

  let api_response = response.json::<ApiAliasResponse>().await?;
  assert_eq!(api_response.api_format, ApiFormat::AnthropicOAuth);
  assert_eq!(api_response.extra_headers, Some(extra_headers));
  assert_eq!(api_response.extra_body, Some(extra_body));
  assert!(api_response.has_api_key);

  Ok(())
}
