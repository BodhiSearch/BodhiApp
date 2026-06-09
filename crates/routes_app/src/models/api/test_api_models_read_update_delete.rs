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
  openai_model, seed_test_api_models, test_db_service, AppServiceStubBuilder, TestDbService,
};
use services::AuthContext;
use services::{
  ApiAliasResponse, ApiKey, ApiKeyUpdate, ApiModel, ApiModelRequest, DefaultApiModelRequest,
  MockAiApiClientFactory,
};
use services::{ApiFormat::OpenAI, ResourceRole};
use std::sync::Arc;
use tower::ServiceExt;

#[allow(clippy::too_many_arguments)]
fn create_expected_response(
  id: &str,
  name: &str,
  api_format: &str,
  base_url: &str,
  has_api_key: bool,
  models: Vec<ApiModel>,
  prefix: Option<String>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
) -> ApiAliasResponse {
  use std::str::FromStr;
  ApiAliasResponse {
    source: "api".to_string(),
    id: id.to_string(),
    name: name.to_string(),
    api_format: services::ApiFormat::from_str(api_format).unwrap(),
    base_url: base_url.to_string(),
    has_api_key,
    models,
    prefix,
    forward_all_with_prefix: false,
    extra_headers: None,
    extra_body: None,
    llm_liberty: None,
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
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_api_model_handler_success(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let base_time = db_service.now();

  seed_test_api_models(&db_service, base_time).await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::get(format!("{}/openai-gpt4", ENDPOINT_MODELS_API)).body(Body::empty())?)
    .await?;

  assert_eq!(response.status(), StatusCode::OK);

  let api_response = response.json::<ApiAliasResponse>().await?;

  let expected_response = create_expected_response(
    "openai-gpt4",
    "test-name", // seed fixture (test_default builder) sets name = "test-name"
    "openai",
    "https://api.openai.com/v1",
    true,
    vec![openai_model("gpt-4")],
    None,
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
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::get(format!("{}/non-existent-alias", ENDPOINT_MODELS_API)).body(Body::empty())?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::NOT_FOUND);

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

  seed_test_api_models(&db_service, base_time).await?;

  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai.expect_for_alias().returning(|_, _| {
    let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
    client
      .expect_fetch_models()
      .returning(|| Ok(vec![openai_model("gpt-4-turbo"), openai_model("gpt-4")]));
    Ok(Box::new(client) as Box<dyn services::AiApiClient>)
  });

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;

  let update_form = ApiModelRequest::default_for(
    OpenAI,
    DefaultApiModelRequest {
      name: "Updated OpenAI Models".to_string(),
      base_url: input_url.to_string(),
      api_key: ApiKeyUpdate::Set(ApiKey::some("sk-updated123456789".to_string())?),
      models: vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()],
      prefix: Some("openai".to_string()),
      forward_all_with_prefix: false,
      extra_headers: None,
      extra_body: None,
    },
  );

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::put(format!("{}/openai-gpt4", ENDPOINT_MODELS_API)).json(update_form)?)
    .await?;

  assert_eq!(response.status(), StatusCode::OK);

  let api_response = response.json::<ApiAliasResponse>().await?;

  let expected_response = create_expected_response(
    "openai-gpt4",
    "Updated OpenAI Models",
    "openai",
    expected_url,
    true,
    vec![openai_model("gpt-4-turbo"), openai_model("gpt-4")],
    Some("openai".to_string()),
    base_time,
    api_response.updated_at, // FrozenTimeService returns the same time, so copy it through
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
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let update_form = ApiModelRequest::default_for(
    OpenAI,
    DefaultApiModelRequest {
      name: "Nonexistent Model".to_string(),
      base_url: "https://api.openai.com/v2".to_string(),
      api_key: ApiKeyUpdate::Set(ApiKey::some("sk-updated123456789".to_string())?),
      models: vec!["gpt-4-turbo".to_string()],
      prefix: None,
      forward_all_with_prefix: false,
      extra_headers: None,
      extra_body: None,
    },
  );

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::put(format!("{}/non-existent-alias", ENDPOINT_MODELS_API)).json(update_form)?)
    .await?;

  assert_eq!(response.status(), StatusCode::NOT_FOUND);

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

  seed_test_api_models(&db_service, base_time).await?;

  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::delete(format!("{}/openai-gpt4", ENDPOINT_MODELS_API)).body(Body::empty())?)
    .await?;

  assert_eq!(response.status(), StatusCode::NO_CONTENT);

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
  let app_service = AppServiceStubBuilder::default()
    .db_service(Arc::new(db_service))
    .build()
    .await?;

  let response = test_router(Arc::new(app_service))
    .oneshot(
      Request::delete(format!("{}/non-existent-alias", ENDPOINT_MODELS_API)).body(Body::empty())?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  let error_response = response.json::<serde_json::Value>().await?;
  let error_code = error_response["error"]["code"].as_str().unwrap();
  assert_eq!("api_model_service_error-not_found", error_code);

  Ok(())
}
