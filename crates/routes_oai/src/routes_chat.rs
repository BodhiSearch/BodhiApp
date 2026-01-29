use crate::{ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS};
use async_openai::types::{
  chat::{
    CreateChatCompletionRequest, CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
  },
  embeddings::{CreateEmbeddingRequest, CreateEmbeddingResponse},
};
use axum::{body::Body, extract::State, response::Response, Json};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, BadRequestError, ErrorType, API_TAG_OPENAI};
use server_core::{LlmEndpoint, RouterState};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HttpError {
  #[error("Error constructing HTTP response: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),

  #[error("Response serialization failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Serialization(#[from] serde_json::Error),
}

/// Validates basic structure of chat completion request
fn validate_chat_completion_request(request: &serde_json::Value) -> Result<(), BadRequestError> {
  // Validate model field exists and is a string
  if request.get("model").and_then(|v| v.as_str()).is_none() {
    return Err(BadRequestError::new(
      "'model' field is required and must be a string".to_string(),
    ));
  }

  // Validate messages field exists and is an array
  if !request
    .get("messages")
    .map(|v| v.is_array())
    .unwrap_or(false)
  {
    return Err(BadRequestError::new(
      "'messages' field is required and must be an array".to_string(),
    ));
  }

  // Validate stream field is boolean if present
  if let Some(stream) = request.get("stream") {
    if !stream.is_boolean() {
      return Err(BadRequestError::new(
        "'stream' field must be a boolean".to_string(),
      ));
    }
  }

  Ok(())
}

/// Create a chat completion
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_CHAT_COMPLETIONS,
    tag = API_TAG_OPENAI,
    operation_id = "createChatCompletion",
    summary = "Create Chat Completion (OpenAI Compatible)",
    description = "Creates a chat completion response using the specified model. Supports both streaming and non-streaming responses. Fully compatible with OpenAI's chat completions API format.",
    request_body(
        content = CreateChatCompletionRequest,
        example = json!({
            "model": "llama2:chat",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "temperature": 0.7,
            "max_tokens": 100,
            "stream": false
        })
    ),
    responses(
        (status = 200, description = "Chat completion response",
         content_type = "application/json",
         body = CreateChatCompletionResponse,
         example = json!({
             "id": "chatcmpl-123",
             "object": "chat.completion",
             "created": 1677610602,
             "model": "llama2:chat",
             "choices": [
                 {
                     "index": 0,
                     "message": {
                         "role": "assistant",
                         "content": "Hello! How can I help you today?"
                     },
                     "finish_reason": "stop"
                 }
             ],
             "usage": {
                 "prompt_tokens": 20,
                 "completion_tokens": 10,
                 "total_tokens": 30
             }
         })),
         (status = 201, description = "Chat completion stream, the status is 200, using 201 to avoid OpenAPI format limitation.",
         content_type = "text/event-stream",
         headers(
             ("Cache-Control" = String, description = "No-cache directive")
         ),
         body = CreateChatCompletionStreamResponse,
         example = json!({
             "id": "chatcmpl-123",
             "object": "chat.completion.chunk",
             "created": 1694268190,
             "model": "llama2:chat",
             "choices": [{
                 "delta": {"content": "Hello"},
                 "index": 0,
                 "finish_reason": null
             }]
         })
        ),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, ApiError>,
) -> Result<Response, ApiError> {
  // Validate basic request structure
  validate_chat_completion_request(&request)?;

  // Forward request directly as Value (no re-serialization needed)
  let response = state
    .forward_request(LlmEndpoint::ChatCompletions, request)
    .await?;
  let mut response_builder = Response::builder().status(response.status());
  if let Some(headers) = response_builder.headers_mut() {
    *headers = response.headers().clone();
  }
  let stream = response.bytes_stream();
  let body = Body::from_stream(stream);
  Ok(response_builder.body(body).map_err(HttpError::Http)?)
}

/// Create embeddings
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_EMBEDDINGS,
    tag = API_TAG_OPENAI,
    operation_id = "createEmbedding",
    summary = "Create Embeddings (OpenAI Compatible)",
    description = "Creates embeddings for the input text using the specified model. Fully compatible with OpenAI's embeddings API format.",
    request_body(
        content = CreateEmbeddingRequest,
        example = json!({
            "model": "text-embedding-model",
            "input": "The quick brown fox jumps over the lazy dog"
        })
    ),
    responses(
        (status = 200, description = "Embedding response",
         content_type = "application/json",
         body = CreateEmbeddingResponse,
         example = json!({
             "object": "list",
             "data": [
                 {
                     "object": "embedding",
                     "index": 0,
                     "embedding": [0.1, 0.2, 0.3]
                 }
             ],
             "model": "text-embedding-model",
             "usage": {
                 "prompt_tokens": 8,
                 "total_tokens": 8
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn embeddings_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<CreateEmbeddingRequest>, ApiError>,
) -> Result<Response, ApiError> {
  let request_value = serde_json::to_value(request).map_err(HttpError::Serialization)?;
  let response = state
    .forward_request(LlmEndpoint::Embeddings, request_value)
    .await?;
  let mut response_builder = Response::builder().status(response.status());
  if let Some(headers) = response_builder.headers_mut() {
    *headers = response.headers().clone();
  }
  let stream = response.bytes_stream();
  let body = Body::from_stream(stream);
  Ok(response_builder.body(body).map_err(HttpError::Http)?)
}

#[cfg(test)]
mod test {
  use crate::routes_chat::{chat_completions_handler, embeddings_handler};
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
  use mockall::predicate::eq;
  use objs::{Alias, UserAlias};
  use reqwest::StatusCode;
  use rstest::rstest;
  use serde_json::json;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    ContextError, DefaultRouterState, LlmEndpoint, MockSharedContext,
  };
  use services::test_utils::{app_service_stub, AppServiceStub};
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
  async fn test_routes_chat_completions_non_stream(
    #[future] app_service_stub: AppServiceStub,
  ) -> anyhow::Result<()> {
    let request = CreateChatCompletionRequestArgs::default()
      .model("testalias-exists:instruct")
      .messages(vec![ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
          .content("What day comes after Monday?")
          .build()?,
      )])
      .build()?;
    let user_alias = UserAlias::testalias_exists();
    let alias = Alias::User(user_alias);
    let request_value = serde_json::to_value(&request)?;
    let mut ctx = MockSharedContext::default();
    ctx
      .expect_forward_request()
      .with(
        eq(LlmEndpoint::ChatCompletions),
        eq(request_value),
        eq(alias),
      )
      .times(1)
      .return_once(move |_, _, _| Ok(non_streamed_response()));
    let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service_stub));
    let app = Router::new()
      .route("/v1/chat/completions", post(chat_completions_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::post("/v1/chat/completions").json(request).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let result: CreateChatCompletionResponse = response.json().await.unwrap();
    assert_eq!(
      "The day that comes after Monday is Tuesday.",
      result
        .choices
        .first()
        .unwrap()
        .message
        .content
        .as_ref()
        .unwrap()
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
      let response: CreateChatCompletionStreamResponse = serde_json::from_value(response).unwrap();
      let response = serde_json::to_string(&response).unwrap();
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
        .unwrap(),
    ))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_routes_chat_completions_stream(
    #[future] app_service_stub: AppServiceStub,
  ) -> anyhow::Result<()> {
    let request = CreateChatCompletionRequestArgs::default()
      .model("testalias-exists:instruct")
      .stream(true)
      .messages(vec![ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
          .content("What day comes after Monday?")
          .build()?,
      )])
      .build()?;
    let user_alias = UserAlias::testalias_exists();
    let alias = Alias::User(user_alias);
    let request_value = serde_json::to_value(&request)?;
    let mut ctx = MockSharedContext::default();
    ctx
      .expect_forward_request()
      .with(
        eq(LlmEndpoint::ChatCompletions),
        eq(request_value),
        eq(alias),
      )
      .times(1)
      .return_once(move |_, _, _| streamed_response());

    let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service_stub));
    let app = Router::new()
      .route("/v1/chat/completions", post(chat_completions_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::post("/v1/chat/completions").json(request).unwrap())
      .await?;
    assert_eq!(StatusCode::OK, response.status());
    let response: Vec<CreateChatCompletionStreamResponse> = response.sse().await?;
    let content = response.into_iter().fold(String::new(), |mut f, r| {
      let content = r
        .choices
        .first()
        .unwrap()
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
  async fn test_routes_embeddings_non_stream(
    #[future] app_service_stub: AppServiceStub,
  ) -> anyhow::Result<()> {
    let request = CreateEmbeddingRequest {
      model: "testalias-exists:instruct".to_string(),
      input: EmbeddingInput::String("The quick brown fox jumps over the lazy dog".to_string()),
      encoding_format: None,
      user: None,
      dimensions: None,
    };
    let user_alias = UserAlias::testalias_exists();
    let alias = Alias::User(user_alias);
    let request_value = serde_json::to_value(&request)?;
    let mut ctx = MockSharedContext::default();
    ctx
      .expect_forward_request()
      .with(eq(LlmEndpoint::Embeddings), eq(request_value), eq(alias))
      .times(1)
      .return_once(move |_, _, _| Ok(embeddings_response()));
    let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service_stub));
    let app = Router::new()
      .route("/v1/embeddings", post(embeddings_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::post("/v1/embeddings").json(request).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let result: CreateEmbeddingResponse = response.json().await.unwrap();
    assert_eq!("list", result.object);
    assert_eq!("testalias-exists:instruct", result.model);
    assert_eq!(1, result.data.len());
    assert_eq!(0, result.data[0].index);
    assert_eq!(vec![0.1, 0.2, 0.3, 0.4, 0.5], result.data[0].embedding);
    assert_eq!(8, result.usage.prompt_tokens);
    assert_eq!(8, result.usage.total_tokens);
    Ok(())
  }
}
