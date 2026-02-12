// ============================================================================
// VIOLATION DOCUMENTATION:
// Handler tests in this file use MockSharedContext.expect_forward_request() for SSE streaming.
// These tests CANNOT be migrated to build_test_router() without complex mock setup.
// The 401 test for chat/embeddings endpoints is already in models_test.rs.
// No allow tests are added here because they would require MockSharedContext expectations.
// ============================================================================

use crate::routes_oai::{chat_completions_handler, embeddings_handler};
use anyhow_trace::anyhow_trace;
use async_openai::types::{
  chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    CreateChatCompletionStreamResponse,
  },
  embeddings::{CreateEmbeddingRequest, CreateEmbeddingResponse, EmbeddingInput},
};
use axum::{extract::Request, routing::post, Router};
use futures_util::StreamExt;
use llama_server_proc::test_utils::mock_response;
use mockall::predicate::{eq, function};
use objs::Alias;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use rstest::rstest;
use serde_json::json;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  ContextError, DefaultRouterState, LlmEndpoint, MockSharedContext,
};
use services::test_utils::AppServiceStubBuilder;
use std::sync::Arc;
use tower::ServiceExt;

fn non_streamed_response() -> reqwest::Response {
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
  mock_response(response.to_string())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_handler_non_stream() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()?;
  let request = CreateChatCompletionRequestArgs::default()
    .model("testalias-exists:instruct")
    .messages(vec![ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessageArgs::default()
        .content("What day comes after Monday?")
        .build()?,
    )])
    .build()?;
  let request_value = serde_json::to_value(&request)?;
  let mut ctx = MockSharedContext::default();
  ctx
    .expect_forward_request()
    .with(
      eq(LlmEndpoint::ChatCompletions),
      eq(request_value),
      function(
        |alias: &Alias| matches!(alias, Alias::User(u) if u.alias == "testalias-exists:instruct"),
      ),
    )
    .times(1)
    .return_once(move |_, _, _| Ok(non_streamed_response()));
  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(Arc::new(router_state));
  let response = app
    .oneshot(Request::post("/v1/chat/completions").json(request)?)
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

fn streamed_response() -> Result<reqwest::Response, ContextError> {
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

  let body = reqwest::Body::wrap_stream(stream);
  Ok(reqwest::Response::from(
    http::Response::builder()
      .status(200)
      .header("content-type", "text/event-stream")
      .body(body)
      .expect("failed to build http response"),
  ))
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_chat_completions_handler_stream() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()?;
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
  let mut ctx = MockSharedContext::default();
  ctx
    .expect_forward_request()
    .with(
      eq(LlmEndpoint::ChatCompletions),
      eq(request_value),
      function(
        |alias: &Alias| matches!(alias, Alias::User(u) if u.alias == "testalias-exists:instruct"),
      ),
    )
    .times(1)
    .return_once(move |_, _, _| streamed_response());

  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(Arc::new(router_state));
  let response = app
    .oneshot(Request::post("/v1/chat/completions").json(request)?)
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

fn embeddings_response() -> reqwest::Response {
  let response = json! {{
    "object": "list",
    "data": [
      {
        "object": "embedding",
        "index": 0,
        "embedding": vec![0.1, 0.2, 0.3, 0.4, 0.5]
      }
    ],
    "model": "testalias-exists:instruct",
    "usage": {
      "prompt_tokens": 8,
      "total_tokens": 8
    }
  }};
  mock_response(response.to_string())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_embeddings_handler_non_stream() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_data_service()
    .await
    .build()?;
  let request = CreateEmbeddingRequest {
    model: "testalias-exists:instruct".to_string(),
    input: EmbeddingInput::String("The quick brown fox jumps over the lazy dog".to_string()),
    encoding_format: None,
    user: None,
    dimensions: None,
  };
  let request_value = serde_json::to_value(&request)?;
  let mut ctx = MockSharedContext::default();
  ctx
    .expect_forward_request()
    .with(
      eq(LlmEndpoint::Embeddings),
      eq(request_value),
      function(
        |alias: &Alias| matches!(alias, Alias::User(u) if u.alias == "testalias-exists:instruct"),
      ),
    )
    .times(1)
    .return_once(move |_, _, _| Ok(embeddings_response()));
  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
  let app = Router::new()
    .route("/v1/embeddings", post(embeddings_handler))
    .with_state(Arc::new(router_state));
  let response = app
    .oneshot(Request::post("/v1/embeddings").json(request)?)
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let result: CreateEmbeddingResponse = response.json().await?;
  assert_eq!("list", result.object);
  assert_eq!("testalias-exists:instruct", result.model);
  assert_eq!(1, result.data.len());
  assert_eq!(0, result.data[0].index);
  assert_eq!(vec![0.1, 0.2, 0.3, 0.4, 0.5], result.data[0].embedding);
  assert_eq!(8, result.usage.prompt_tokens);
  assert_eq!(8, result.usage.total_tokens);
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
    .build()?;
  let ctx = MockSharedContext::default();
  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(Arc::new(router_state));

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
    .build()?;
  let ctx = MockSharedContext::default();
  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(Arc::new(router_state));

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
    .build()?;
  let ctx = MockSharedContext::default();
  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service));
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(Arc::new(router_state));

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
