use crate::test_utils::{mock_ai_factory_returning, RequestAuthContextExt};
use crate::{
  responses_cancel_handler, responses_create_handler, responses_delete_handler,
  responses_get_handler, responses_input_items_handler,
};
use anyhow_trace::anyhow_trace;
use axum::{
  extract::Request,
  routing::{delete, get, post},
  Router,
};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use server_core::test_utils::RequestTestExt;
use services::{
  models::llm_liberty_envelope::LlmLibertyEnvelope,
  test_utils::{
    openai_model, test_llm_liberty_envelope_codex, AppServiceStubBuilder, TEST_TENANT_ID,
    TEST_USER_ID,
  },
  AiApiClientFactoryError, ApiAliasBuilder, ApiFormat, AuthContext, LibertySource,
  MockAiApiClientFactory, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn codex_liberty_envelope(access_token: &str) -> LlmLibertyEnvelope {
  let mut env = test_llm_liberty_envelope_codex();
  env.access_token = access_token.to_string();
  env.refresh_token = "refresh-test".into();
  env.expires_at = (chrono::Utc::now() + chrono::Duration::days(365)).timestamp();
  env
}

fn ok_response() -> Result<axum::response::Response, AiApiClientFactoryError> {
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "application/json")
      .body(axum::body::Body::from(r#"{"id":"resp-123"}"#))
      .unwrap(),
  )
}

async fn seed_responses_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("resp-alias")
    .api_format(ApiFormat::OpenAIResponses)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .prefix("resp/".to_string())
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  Ok(api_alias)
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_success() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(serde_json::json!({"model": "resp/gpt-4o", "input": "hello"}))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_get_success() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses/{response_id}", get(responses_get_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1/responses/resp-abc-123?model=resp%2Fgpt-4o")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_delete_success() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/v1/responses/{response_id}",
      delete(responses_delete_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::delete("/v1/responses/resp-abc-123?model=resp%2Fgpt-4o")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_input_items_success() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/v1/responses/{response_id}/input_items",
      get(responses_input_items_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1/responses/resp-abc-123/input_items?model=resp%2Fgpt-4o")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_cancel_success() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/v1/responses/{response_id}/cancel",
      post(responses_cancel_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses/resp-abc-123/cancel?model=resp%2Fgpt-4o")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_get_forwards_extra_query_params() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses/{response_id}", get(responses_get_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1/responses/resp-abc-123?model=resp%2Fgpt-4o&limit=10")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_input_items_forwards_extra_query_params() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/v1/responses/{response_id}/input_items",
      get(responses_input_items_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1/responses/resp-abc-123/input_items?model=resp%2Fgpt-4o&after=item_456")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_forwards_query_params() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let app_service = builder
    .ai_api_client_factory(mock_ai_factory_returning(ok_response))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses?foo=bar")
        .json(serde_json::json!({"model": "resp/gpt-4o", "input": "hello"}))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_forwards_to_llm_liberty_oauth_codex_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let alias_id = "codex-liberty-alias";
  let api_alias = ApiAliasBuilder::test_default()
    .id(alias_id)
    .api_format(ApiFormat::LlmLibertyOauth)
    .base_url("https://chatgpt.com/backend-api/codex")
    .models(vec![openai_model("gpt-5.2")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  db_service
    .create_llm_liberty_credentials(
      TEST_TENANT_ID,
      TEST_USER_ID,
      alias_id,
      &codex_liberty_envelope("codex-access-token"),
    )
    .await?;

  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai.expect_safe_http_client().returning(|| {
    services::SafeReqwest::builder()
      .allow_private_ips()
      .build()
      .unwrap()
  });
  mock_ai
    .expect_for_liberty()
    .withf(|source| match source {
      LibertySource::Resolved {
        creds, alias_id, ..
      } => creds.access_token == "codex-access-token" && *alias_id == "codex-liberty-alias",
      LibertySource::Envelope(_) => false,
    })
    .times(1)
    .returning(|_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_forward_request_with_method()
        .times(1)
        .returning(|_, _, _, _, _| {
          Ok(
            axum::response::Response::builder()
              .status(200)
              .header("content-type", "application/json")
              .body(axum::body::Body::from(r#"{"id":"resp-123"}"#))
              .unwrap(),
          )
        });
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let app_service = builder
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(serde_json::json!({"model": "gpt-5.2", "input": "hello"}))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// Liberty CRUD forwards through the factory; upstream returns its own status.
#[rstest]
#[case::get(
  "GET",
  "/v1/responses/resp-abc-123?model=gpt-5.2",
  "/responses/resp-abc-123"
)]
#[case::delete(
  "DELETE",
  "/v1/responses/resp-abc-123?model=gpt-5.2",
  "/responses/resp-abc-123"
)]
#[case::input_items(
  "GET",
  "/v1/responses/resp-abc-123/input_items?model=gpt-5.2",
  "/responses/resp-abc-123/input_items"
)]
#[case::cancel(
  "POST",
  "/v1/responses/resp-abc-123/cancel?model=gpt-5.2",
  "/responses/resp-abc-123/cancel"
)]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_crud_forwards_for_liberty_codex_alias(
  #[case] method: &str,
  #[case] path: &str,
  #[case] expected_upstream_path: &'static str,
) -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let alias_id = "codex-crud-alias";
  let api_alias = ApiAliasBuilder::test_default()
    .id(alias_id)
    .api_format(ApiFormat::LlmLibertyOauth)
    .base_url("https://chatgpt.com/backend-api/codex")
    .models(vec![openai_model("gpt-5.2")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  db_service
    .create_llm_liberty_credentials(
      TEST_TENANT_ID,
      TEST_USER_ID,
      alias_id,
      &codex_liberty_envelope("codex-access-token"),
    )
    .await?;

  let mut mock_ai = MockAiApiClientFactory::new();
  mock_ai.expect_safe_http_client().returning(|| {
    services::SafeReqwest::builder()
      .allow_private_ips()
      .build()
      .unwrap()
  });
  mock_ai
    .expect_for_liberty()
    .withf(|source| matches!(source, LibertySource::Resolved { .. }))
    .times(1)
    .returning(move |_| {
      let mut client = services::ai_apis::ai_api_client::MockAiApiClient::new();
      client
        .expect_forward_request_with_method()
        .withf(move |_method, p, body, _query, _headers| {
          p == expected_upstream_path && body.is_none()
        })
        .times(1)
        .returning(|_, _, _, _, _| {
          Ok(
            axum::response::Response::builder()
              .status(200)
              .header("content-type", "application/json")
              .body(axum::body::Body::from(r#"{"id":"resp-abc-123"}"#))
              .unwrap(),
          )
        });
      Ok(Box::new(client) as Box<dyn services::AiApiClient>)
    });

  let app_service = builder
    .ai_api_client_factory(Arc::new(mock_ai))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/v1/responses/{response_id}",
      get(responses_get_handler).delete(responses_delete_handler),
    )
    .route(
      "/v1/responses/{response_id}/input_items",
      get(responses_input_items_handler),
    )
    .route(
      "/v1/responses/{response_id}/cancel",
      post(responses_cancel_handler),
    )
    .with_state(router_state);

  let request = match method {
    "GET" => Request::get(path).body(axum::body::Body::empty())?,
    "DELETE" => Request::delete(path).body(axum::body::Body::empty())?,
    "POST" => Request::post(path).body(axum::body::Body::empty())?,
    _ => unreachable!("unexpected method: {method}"),
  }
  .with_auth_context(AuthContext::test_session(
    TEST_USER_ID,
    "testuser",
    ResourceRole::User,
  ));

  let response = app.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
