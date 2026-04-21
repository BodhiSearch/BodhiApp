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
use services::test_utils::{test_db_service, AppServiceStubBuilder, TestDbService};
use services::AuthContext;
use services::{ApiAliasRepository, ApiFormat, ResourceRole};
use services::{ApiKeyUpdate, ApiModelRequest, FetchModelsRequest, TestCreds, TestPromptRequest};
use std::sync::Arc;
use tower::ServiceExt;

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
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_api_model_rejects_keep_when_api_format_changes(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::test_utils::{openai_model, TEST_TENANT_ID};
  use services::ApiAliasBuilder;

  // Seed an openai alias with a stored API key.
  let db_arc = Arc::new(db_service);
  let alias = ApiAliasBuilder::default()
    .id("stored-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(services::ApiModelVec::from(vec![openai_model("gpt-4")]))
    .build_with_time(chrono::Utc::now())?;
  db_arc
    .create_api_model_alias(
      TEST_TENANT_ID,
      "test-user",
      &alias,
      Some("sk-stored".to_string()),
    )
    .await?;

  let mut mock_ai = services::MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .returning(|_, _, _, _, _| Ok(vec![]));

  let app_service = AppServiceStubBuilder::default()
    .db_service(db_arc)
    .ai_api_service(Arc::new(mock_ai))
    .build()
    .await?;

  // Attempt to switch format to anthropic_oauth while keeping the stored key.
  let update_form = ApiModelRequest {
    api_format: ApiFormat::AnthropicOAuth,
    base_url: "https://api.anthropic.com/v1".to_string(),
    api_key: ApiKeyUpdate::Keep,
    models: vec![],
    prefix: Some("anth/".to_string()),
    forward_all_with_prefix: true,
    extra_headers: None,
    extra_body: None,
  };

  let response = test_router(Arc::new(app_service))
    .oneshot(Request::put(format!("{}/{}", ENDPOINT_MODELS_API, "stored-alias")).json(update_form)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let error = response.json::<serde_json::Value>().await?;
  let code = error["error"]["code"].as_str().unwrap_or("");
  assert_eq!("api_model_service_error-validation", code);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_api_models_test_id_uses_stored_api_format(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  use mockall::predicate::*;
  use services::test_utils::{anthropic_model, TEST_TENANT_ID};
  use services::ApiAliasBuilder;

  // Seed an anthropic_oauth alias.
  let db_arc = Arc::new(db_service);
  let alias = ApiAliasBuilder::default()
    .id("oauth-alias")
    .api_format(ApiFormat::AnthropicOAuth)
    .base_url("https://api.anthropic.com/v1")
    .models(services::ApiModelVec::from(vec![anthropic_model(
      "claude-sonnet-4-5-20250929",
    )]))
    .build_with_time(chrono::Utc::now())?;
  db_arc
    .create_api_model_alias(
      TEST_TENANT_ID,
      "test-user",
      &alias,
      Some("sk-ant-oat01-token".to_string()),
    )
    .await?;

  // Mock: test_prompt must be called with AnthropicOAuth (not OpenAI from payload).
  let mut mock_ai = services::MockAiApiService::new();
  mock_ai
    .expect_test_prompt()
    .withf(|_, _, _, _, fmt, _, _| *fmt == ApiFormat::AnthropicOAuth)
    .times(1)
    .returning(|_, _, _, _, _, _, _| Ok("ok".to_string()));

  let app_service = AppServiceStubBuilder::default()
    .db_service(db_arc)
    .ai_api_service(Arc::new(mock_ai))
    .build()
    .await?;

  // Payload claims openai; handler must use stored anthropic_oauth instead.
  let test_request = TestPromptRequest {
    creds: TestCreds::Id("oauth-alias".to_string()),
    base_url: "https://placeholder.example.com/v1".to_string(),
    model: "claude-sonnet-4-5-20250929".to_string(),
    prompt: "hi".to_string(),
    api_format: ApiFormat::OpenAI,
    extra_headers: None,
    extra_body: None,
  };
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(&format!("{}/test", ENDPOINT_MODELS_API)).json(test_request)?)
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_api_models_fetch_models_id_uses_stored_api_format(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::test_utils::{anthropic_model, TEST_TENANT_ID};
  use services::ApiAliasBuilder;

  let db_arc = Arc::new(db_service);
  let alias = ApiAliasBuilder::default()
    .id("oauth-alias")
    .api_format(ApiFormat::AnthropicOAuth)
    .base_url("https://api.anthropic.com/v1")
    .models(services::ApiModelVec::from(vec![anthropic_model(
      "claude-sonnet-4-5-20250929",
    )]))
    .build_with_time(chrono::Utc::now())?;
  db_arc
    .create_api_model_alias(
      TEST_TENANT_ID,
      "test-user",
      &alias,
      Some("sk-ant-oat01-token".to_string()),
    )
    .await?;

  let mut mock_ai = services::MockAiApiService::new();
  mock_ai
    .expect_fetch_models()
    .withf(|_, _, fmt, _, _| *fmt == ApiFormat::AnthropicOAuth)
    .times(1)
    .returning(|_, _, _, _, _| Ok(vec![]));

  let app_service = AppServiceStubBuilder::default()
    .db_service(db_arc)
    .ai_api_service(Arc::new(mock_ai))
    .build()
    .await?;

  let fetch_request = FetchModelsRequest {
    creds: TestCreds::Id("oauth-alias".to_string()),
    base_url: "https://placeholder.example.com/v1".to_string(),
    api_format: ApiFormat::OpenAI,
    extra_headers: None,
    extra_body: None,
  };
  let response = test_router(Arc::new(app_service))
    .oneshot(Request::post(&format!("{}/fetch-models", ENDPOINT_MODELS_API)).json(fetch_request)?)
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
