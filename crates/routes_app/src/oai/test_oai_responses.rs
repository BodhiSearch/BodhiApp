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
  inference::{InferenceError, LlmEndpoint, MockInferenceService},
  test_utils::{AppServiceStubBuilder, TEST_TENANT_ID, TEST_USER_ID},
  ApiAliasBuilder, ApiFormat, AuthContext, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn ok_response() -> Result<axum::response::Response, InferenceError> {
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "application/json")
      .body(axum::body::Body::from(r#"{"id":"resp-123"}"#))
      .unwrap(),
  )
}

// ============================================================================
// validate_responses_request — error paths
// ============================================================================

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

// ============================================================================
// Alias resolution — error paths
// ============================================================================

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
    .models(vec!["gpt-4o".to_string()])
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

// ============================================================================
// Success paths — verify correct LlmEndpoint dispatched to InferenceService
// ============================================================================

async fn seed_responses_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("resp-alias")
    .api_format(ApiFormat::OpenAIResponses)
    .base_url("https://api.openai.com/v1")
    .models(vec!["gpt-4o".to_string()])
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

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote()
    .withf(|endpoint, _req, alias, _key| {
      *endpoint == LlmEndpoint::Responses && alias.id == "resp-alias"
    })
    .times(1)
    .return_once(|_, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/responses", post(responses_create_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/responses")
        .json(json!({"model": "resp/gpt-4o", "input": "hello"}))?
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
async fn test_responses_get_success() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params| {
      *endpoint == LlmEndpoint::ResponsesGet("resp-abc-123".to_string()) && alias.id == "resp-alias"
    })
    .times(1)
    .return_once(|_, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
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

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote()
    .withf(|endpoint, _req, alias, _key| {
      *endpoint == LlmEndpoint::ResponsesDelete("resp-abc-123".to_string())
        && alias.id == "resp-alias"
    })
    .times(1)
    .return_once(|_, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
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

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params| {
      *endpoint == LlmEndpoint::ResponsesInputItems("resp-abc-123".to_string())
        && alias.id == "resp-alias"
    })
    .times(1)
    .return_once(|_, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
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

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote()
    .withf(|endpoint, _req, alias, _key| {
      *endpoint == LlmEndpoint::ResponsesCancel("resp-abc-123".to_string())
        && alias.id == "resp-alias"
    })
    .times(1)
    .return_once(|_, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
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

// ============================================================================
// response_id validation
// ============================================================================

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
    .route("/v1/responses/{response_id}", get(responses_get_handler))
    .route(
      "/v1/responses/{response_id}/cancel",
      post(responses_cancel_handler),
    )
    .with_state(router_state);

  // GET
  let response = app
    .clone()
    .oneshot(
      Request::get(format!("/v1/responses/{}?model=resp%2Fgpt-4o", response_id))
        .body(axum::body::Body::empty())?
        .with_auth_context(AuthContext::test_session("u", "u", ResourceRole::User)),
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());

  // POST cancel
  let response = app
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

  Ok(())
}

// ============================================================================
// Query parameter forwarding
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_responses_get_forwards_extra_query_params() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_responses_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, params| {
      *endpoint == LlmEndpoint::ResponsesGet("resp-abc-123".to_string())
        && alias.id == "resp-alias"
        && params
          .as_ref()
          .map(|p| p.contains(&("limit".to_string(), "10".to_string())) && p.len() == 1)
          .unwrap_or(false)
    })
    .times(1)
    .return_once(|_, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
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

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, params| {
      *endpoint == LlmEndpoint::ResponsesInputItems("resp-abc-123".to_string())
        && alias.id == "resp-alias"
        && params
          .as_ref()
          .map(|p| p.contains(&("after".to_string(), "item_456".to_string())) && p.len() == 1)
          .unwrap_or(false)
    })
    .times(1)
    .return_once(|_, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
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
