// ============================================================================
// VIOLATION DOCUMENTATION:
// Handler tests in this file use MockInferenceService.expect_forward_local() for SSE streaming.
// These tests CANNOT be migrated to build_test_router() without complex mock setup.
// The 401 test for chat/embeddings endpoints is already in models_test.rs.
// No allow tests are added here because they would require MockInferenceService expectations.
// ============================================================================

use crate::chat_completions_handler;
use crate::test_utils::RequestAuthContextExt;
use anyhow_trace::anyhow_trace;
use async_openai::types::chat::{
  ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
  CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
  CreateChatCompletionStreamResponse,
};
use axum::{extract::Request, response::Response, routing::post, Router};
use futures_util::StreamExt;
use mockall::predicate::{eq, function};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::test_utils::{RequestTestExt, ResponseTestExt};
use services::{
  inference::{InferenceError, LlmEndpoint, MockInferenceService},
  test_utils::{openai_model, AppServiceStubBuilder},
  Alias, AuthContext, ResourceRole,
};
use std::sync::Arc;
use tower::ServiceExt;

fn non_streamed_axum_response() -> Result<Response, InferenceError> {
  let response = json! {{
    "id": "testid",
    "model": "testalias-exists:instruct",
    "choices": [
      {
        "index": 0,
        "message": {
          "role": "assistant",
          "content": "The day that comes after Monday is Tuesday."
        },
      }],
    "created": 1704067200,
    "object": "chat.completion",
  }};
  let body = serde_json::to_string(&response).unwrap();
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "application/json")
      .body(axum::body::Body::from(body))
      .unwrap(),
  )
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_handler_non_stream() -> anyhow::Result<()> {
  let request = CreateChatCompletionRequestArgs::default()
    .model("testalias-exists:instruct")
    .messages(vec![ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessageArgs::default()
        .content("What day comes after Monday?")
        .build()?,
    )])
    .build()?;
  let request_value = serde_json::to_value(&request)?;
  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_local()
    .with(
      eq(LlmEndpoint::ChatCompletions),
      eq(request_value),
      function(
        |alias: &Alias| matches!(alias, Alias::User(u) if u.alias == "testalias-exists:instruct"),
      ),
    )
    .times(1)
    .return_once(move |_, _, _| non_streamed_axum_response());
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);
  let response = app
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(request)?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let result: CreateChatCompletionResponse = response.json().await?;
  assert_eq!(
    "The day that comes after Monday is Tuesday.",
    result
      .choices
      .first()
      .expect("expected at least one choice")
      .message
      .content
      .as_ref()
      .expect("expected content in message")
  );
  Ok(())
}

fn streamed_axum_response() -> Result<Response, InferenceError> {
  let stream = futures_util::stream::iter([
    " ", " After", " Monday", ",", " the", " next", " day", " is", " T", "ues", "day", ".",
  ])
  .enumerate()
  .map(|(i, value)| {
    let response = json! {{
      "id": format!("testid-{i}"),
      "model": "testalias-exists:instruct",
      "choices": [
        {
          "index": 0,
          "delta": {
            "role": "assistant",
            "content": value,
          },
        }],
      "created": 1704067200,
      "object": "chat.completion.chunk",
    }};
    let response: CreateChatCompletionStreamResponse =
      serde_json::from_value(response).expect("failed to deserialize stream response");
    let response =
      serde_json::to_string(&response).expect("failed to serialize stream response");
    format!("data: {response}\n\n")
  })
  .chain(futures_util::stream::iter([format!("data: {}\n\n", r#"{"choices":[{"finish_reason":"stop","index":0,"delta":{}}],"created":1717317061,"id":"chatcmpl-Twf1ixroh9WzY9Pvm4IGwNF4kB4EjTp4","model":"llama2:chat","object":"chat.completion.chunk","usage":{"completion_tokens":13,"prompt_tokens":15,"total_tokens":28}}"#)]))
  .then(|chunk| async move {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Ok::<_, std::io::Error>(chunk)
  });

  let body = axum::body::Body::from_stream(stream);
  Ok(
    axum::response::Response::builder()
      .status(200)
      .header("content-type", "text/event-stream")
      .body(body)
      .unwrap(),
  )
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_handler_stream() -> anyhow::Result<()> {
  let request = CreateChatCompletionRequestArgs::default()
    .model("testalias-exists:instruct")
    .stream(true)
    .messages(vec![ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessageArgs::default()
        .content("What day comes after Monday?")
        .build()?,
    )])
    .build()?;
  let request_value = serde_json::to_value(&request)?;
  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_local()
    .with(
      eq(LlmEndpoint::ChatCompletions),
      eq(request_value),
      function(
        |alias: &Alias| matches!(alias, Alias::User(u) if u.alias == "testalias-exists:instruct"),
      ),
    )
    .times(1)
    .return_once(move |_, _, _| streamed_axum_response());

  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);
  let response = app
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(request)?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let response: Vec<CreateChatCompletionStreamResponse> = response.sse().await?;
  let content = response.into_iter().fold(String::new(), |mut f, r| {
    let content = r
      .choices
      .first()
      .expect("expected at least one choice")
      .delta
      .content
      .as_deref()
      .unwrap_or_default();
    f.push_str(content);
    f
  });
  assert_eq!("  After Monday, the next day is Tuesday.", content);
  Ok(())
}

// ============================================================================
// validate_chat_completion_request error path tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_missing_model_field() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);

  let response = app
    .oneshot(Request::post("/v1/chat/completions").json(json!({
      "messages": [{"role": "user", "content": "Hello"}]
    }))?)
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
async fn test_chat_completions_missing_messages_field() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);

  let response = app
    .oneshot(Request::post("/v1/chat/completions").json(json!({
      "model": "test-model"
    }))?)
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
async fn test_chat_completions_invalid_stream_field() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);

  let response = app
    .oneshot(Request::post("/v1/chat/completions").json(json!({
      "model": "test-model",
      "messages": [{"role": "user", "content": "Hello"}],
      "stream": "yes"
    }))?)
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
// Format rejection tests — openai_responses aliases must not be routed via
// chat completions endpoint
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_rejects_responses_format_alias() -> anyhow::Result<()> {
  use services::test_utils::{TEST_TENANT_ID, TEST_USER_ID};
  use services::{ApiAliasBuilder, ApiFormat};

  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Seed an API alias with openai_responses format
  let api_alias = ApiAliasBuilder::test_default()
    .id("responses-alias")
    .api_format(ApiFormat::OpenAIResponses)
    .base_url("https://api.openai.com/v1")
    .models(vec![openai_model("gpt-4o")])
    .prefix("responses/".to_string())
    .build_with_time(db_service.now())
    .unwrap();

  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;

  let app_service = builder.build().await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(json!({
          "model": "responses/gpt-4o",
          "messages": [{"role": "user", "content": "Hello"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          "test-user",
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
    "Error message should mention the format: {}",
    message
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_forwards_anthropic_format_alias() -> anyhow::Result<()> {
  use services::test_utils::{TEST_TENANT_ID, TEST_USER_ID};
  use services::{ApiAliasBuilder, ApiFormat};

  let mut builder = AppServiceStubBuilder::default();
  builder.with_data_service().await;
  let db_service = builder.get_db_service().await;

  // Anthropic's less-advertised /v1/chat/completions endpoint accepts
  // OpenAI-compatible format. The opaque proxy pipeline already injects
  // x-api-key + anthropic-version auth for ApiFormat::Anthropic, so no
  // handler-level transformation is needed — forward as-is.
  let api_alias = ApiAliasBuilder::test_default()
    .id("anthropic-alias")
    .api_format(ApiFormat::Anthropic)
    .base_url("https://api.anthropic.com/v1")
    .models(vec![services::test_utils::anthropic_model(
      "claude-3-5-sonnet-20241022",
    )])
    .build_with_time(db_service.now())
    .unwrap();

  db_service
    .create_api_model_alias(TEST_TENANT_ID, TEST_USER_ID, &api_alias, None)
    .await?;

  let mut mock_inference = MockInferenceService::new();
  mock_inference
    .expect_forward_remote()
    .withf(|endpoint, _req, alias, _key| {
      *endpoint == LlmEndpoint::ChatCompletions && alias.id == "anthropic-alias"
    })
    .times(1)
    .return_once(|_, _, _, _| non_streamed_axum_response());

  let app_service = builder
    .inference_service(Arc::new(mock_inference))
    .build()
    .await?;
  let router_state: Arc<dyn services::AppService> = Arc::new(app_service);
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(router_state);

  let response = app
    .oneshot(
      Request::post("/v1/chat/completions")
        .json(json!({
          "model": "claude-3-5-sonnet-20241022",
          "messages": [{"role": "user", "content": "Hello"}]
        }))?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
