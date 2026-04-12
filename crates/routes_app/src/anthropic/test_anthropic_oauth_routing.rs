use crate::anthropic_messages_create_handler;
use crate::test_utils::RequestAuthContextExt;
use anyhow_trace::anyhow_trace;
use axum::{extract::Request, routing::post, Router};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  inference::{InferenceError, LlmEndpoint, MockInferenceService},
  test_utils::{anthropic_model, AppServiceStubBuilder, TEST_TENANT_ID, TEST_USER_ID},
  ApiAliasBuilder, ApiFormat, AuthContext, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn ok_response() -> Result<axum::response::Response, InferenceError> {
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "application/json")
      .body(axum::body::Body::from(
        r#"{"id":"msg-123","content":[{"type":"text","text":"Hi"}]}"#,
      ))
      .unwrap(),
  )
}

async fn seed_anthropic_oauth_alias(
  builder: &mut AppServiceStubBuilder,
) -> anyhow::Result<services::ApiAlias> {
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-oauth-alias")
    .api_format(ApiFormat::AnthropicOAuth)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![anthropic_model("claude-3-5-sonnet-20241022")])
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
async fn test_messages_create_forwards_to_anthropic_oauth_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  seed_anthropic_oauth_alias(&mut builder).await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote_with_params()
    .withf(|endpoint, _req, alias, _key, _params, _headers| {
      *endpoint == LlmEndpoint::AnthropicMessages && alias.id == "anthropic-oauth-alias"
    })
    .times(1)
    .return_once(|_, _, _, _, _, _| ok_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .json(json!({
          "model": "claude-3-5-sonnet-20241022",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
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

// OpenAI alias rejection path. AnthropicOAuth acceptance is covered by the test above.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_messages_create_rejects_openai_alias() -> anyhow::Result<()> {
  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;
  let openai_alias = ApiAliasBuilder::test_default()
    .id("openai-alias")
    .api_format(ApiFormat::OpenAI)
    .base_url("https://api.openai.com/v1")
    .models(vec![services::test_utils::openai_model("gpt-4o")])
    .build_with_time(db_service.now())
    .unwrap();
  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &openai_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route(
      "/anthropic/v1/messages",
      post(anthropic_messages_create_handler),
    )
    .with_state(router_state);

  let err_response = app
    .oneshot(
      Request::post("/anthropic/v1/messages")
        .json(json!({
          "model": "gpt-4o",
          "max_tokens": 100,
          "messages": [{"role": "user", "content": "hi"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          TEST_USER_ID,
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, err_response.status());
  let body = err_response.json::<serde_json::Value>().await?;
  assert_eq!("error", body["type"].as_str().unwrap());
  assert_eq!(
    "invalid_request_error",
    body["error"]["type"].as_str().unwrap()
  );
  Ok(())
}
