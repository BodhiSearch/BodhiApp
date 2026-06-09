use crate::test_utils::RequestAuthContextExt;
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
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  models::llm_liberty_envelope::LlmLibertyEnvelope,
  test_utils::{
    openai_model, test_llm_liberty_envelope_codex, AppServiceStubBuilder, TEST_TENANT_ID,
    TEST_USER_ID,
  },
  ApiAliasBuilder, ApiFormat, AuthContext, DefaultAiApiClientFactory, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn codex_liberty_envelope_errors(access_token: &str) -> LlmLibertyEnvelope {
  let mut env = test_llm_liberty_envelope_codex();
  env.access_token = access_token.to_string();
  env.refresh_token = "refresh-test".into();
  env.expires_at = (chrono::Utc::now() + chrono::Duration::days(365)).timestamp();
  env
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_missing_model() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"input": "hello"}))?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_missing_input() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"model": "gpt-4o"}))?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_model_not_found() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"model": "nonexistent-model", "input": "hello"}))?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "data_service_error-alias_not_found",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_wrong_format() -> anyhow::Result<()> {
  // Seed an openai-format alias (not openai_responses) and verify it's rejected
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .prefix("openai/".to_string())
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"model": "openai/gpt-4o", "input": "hello"}))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  let message = body["error"]["message"].as_str().unwrap();
  assert!(
    message.contains("openai_responses"),
    "Error should mention openai_responses format: {}",
    message
  );
  Ok(())
}

#[rstest]
#[case::path_traversal("..%2F..%2Fadmin")]
#[case::slash_in_id("resp%2Fevil")]
#[case::dot_dot("resp..test")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_invalid_response_id(#[case] response_id: &str) -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/v1/responses/{response_id}",
      get(responses_get_handler).delete(responses_delete_handler),
    )
    .route(
      "/v1/responses/{response_id}/cancel",
      post(responses_cancel_handler),
    )
    .route(
      "/v1/responses/{response_id}/input_items",
      get(responses_input_items_handler),
    )
    .with_state(router_state);

  let response = app
    .clone()
    .oneshot(
      Request::get(format!("/v1/responses/{}?model=resp%2Fgpt-4o", response_id))
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let response = app
    .clone()
    .oneshot(
      Request::delete(format!("/v1/responses/{}?model=resp%2Fgpt-4o", response_id))
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let response = app
    .clone()
    .oneshot(
      Request::post(format!(
        "/v1/responses/{}/cancel?model=resp%2Fgpt-4o",
        response_id
      ))
      .body(axum::body::Body::empty())?
      .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  let response = app
    .oneshot(
      Request::get(format!(
        "/v1/responses/{}/input_items?model=resp%2Fgpt-4o",
        response_id
      ))
      .body(axum::body::Body::empty())?
      .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_invalid_stream_field() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"model": "gpt-4o", "input": "hello", "stream": "yes"}))?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_get_missing_model_param() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses/{response_id}", get(responses_get_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::get("/v1/responses/resp-abc-123")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_delete_missing_model_param() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
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
      Request::delete("/v1/responses/resp-abc-123")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_input_items_missing_model_param() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
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
      Request::get("/v1/responses/resp-abc-123/input_items")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_cancel_missing_model_param() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
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
      Request::post("/v1/responses/resp-abc-123/cancel")
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  assert_eq!(
    "oai_route_error-invalid_request",
    body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_create_rejects_liberty_with_unsupported_provider() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let alias_id = "gemini-liberty-alias";
  let api_alias = ApiAliasBuilder::test_default()
    .id(alias_id)
    .api_format(ApiFormat::LlmLibertyOauth)
    .base_url("https://api.example.com/gemini")
    .models(vec![openai_model("gemini-model")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;
  let mut unsupported_env = codex_liberty_envelope_errors("token");
  unsupported_env.provider = "google-gemini".to_string();
  db_service
    .create_llm_liberty_credentials(TEST_TENANT_ID, TEST_USER_ID, alias_id, &unsupported_env)
    .await?;

  let real_factory = DefaultAiApiClientFactory::new()?;
  let app_service = builder
    .ai_api_client_factory(Arc::new(real_factory))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"model": "gemini-model", "input": "hi"}))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  let message = body["error"]["message"].as_str().unwrap_or("");
  assert!(
    message.contains("google-gemini") || message.to_lowercase().contains("provider"),
    "expected provider-mismatch message, got: {}",
    message
  );
  Ok(())
}
