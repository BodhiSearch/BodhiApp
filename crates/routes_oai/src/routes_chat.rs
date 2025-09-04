use crate::ENDPOINT_OAI_CHAT_COMPLETIONS;
use async_openai::types::CreateChatCompletionRequest;
use axum::{body::Body, extract::State, response::Response, Json};
use axum_extra::extract::WithRejection;
use objs::{ApiError, AppError, ErrorType, OpenAIApiError, API_TAG_OPENAI};
use server_core::RouterState;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HttpError {
  #[error("http_error")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),
}

/// Create a chat completion
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_CHAT_COMPLETIONS,
    tag = API_TAG_OPENAI,
    operation_id = "createChatCompletion",
    request_body(
        content = serde_json::Value,
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
         body = serde_json::Value,
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
         body = serde_json::Value,
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
        (status = 400, description = "Invalid request parameters", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Invalid model specified",
                 "type": "invalid_request_error",
                 "code": "model_not_found"
             }
         })),
        (status = 401, description = "Invalid authentication", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Invalid authentication token",
                 "type": "invalid_request_error",
                 "code": "invalid_api_key"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
      ("bearer_auth" = []),
    ),
)]
pub async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<CreateChatCompletionRequest>, ApiError>,
) -> Result<Response, ApiError> {
  let response = state.chat_completions(request).await?;
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
  use crate::routes_chat::chat_completions_handler;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    CreateChatCompletionStreamResponse,
  };
  use axum::{extract::Request, routing::post, Router};
  use futures_util::StreamExt;
  use llama_server_proc::test_utils::mock_response;
  use mockall::predicate::eq;
  use objs::UserAlias;
  use reqwest::StatusCode;
  use rstest::rstest;
  use serde_json::json;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    ContextError, DefaultRouterState, MockSharedContext,
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
    let alias = UserAlias::testalias_exists();
    let mut ctx = MockSharedContext::default();
    ctx
      .expect_chat_completions()
      .with(eq(request.clone()), eq(alias))
      .times(1)
      .return_once(move |_, _| Ok(non_streamed_response()));
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
    let alias = UserAlias::testalias_exists();
    let mut ctx = MockSharedContext::default();
    ctx
      .expect_chat_completions()
      .with(eq(request.clone()), eq(alias))
      .times(1)
      .return_once(move |_, _| streamed_response());

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
}
