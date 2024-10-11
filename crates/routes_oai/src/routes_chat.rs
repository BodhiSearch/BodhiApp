use async_openai::types::CreateChatCompletionRequest;
use axum::{
  body::Body,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiError, InternalServerError};
use server_core::{fwd_sse, RouterState};
use std::sync::Arc;

#[derive(Debug, derive_new::new)]
pub struct ChatCompletionsResponse {
  pub message: String,
}

impl IntoResponse for ChatCompletionsResponse {
  fn into_response(self) -> Response {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::from(self.message))
      .unwrap_or_else(|err| {
        tracing::error!(?err, "error building response");
        Response::builder()
          .status(StatusCode::INTERNAL_SERVER_ERROR)
          .body(Body::empty())
          .unwrap()
      })
  }
}

pub async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<CreateChatCompletionRequest>, ApiError>,
) -> Result<Response, ApiError> {
  let stream = request.stream.unwrap_or(false);
  let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
  let handle = tokio::spawn(async move { state.chat_completions(request, tx).await });
  if !stream {
    if let Some(message) = rx.recv().await {
      drop(rx);
      Ok(ChatCompletionsResponse::new(message).into_response())
    } else if let Ok(Err(e)) = handle.await {
      tracing::warn!(?e, "error while processing reqeust");
      Err(e.into())
    } else {
      Err(InternalServerError::new("receiver stream abruptly closed".to_string()).into())
    }
  } else {
    Ok(fwd_sse(rx))
  }
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
  use mockall::predicate::{always, eq};
  use objs::{Alias, HubFileBuilder};
  use reqwest::StatusCode;
  use rstest::rstest;
  use serde_json::json;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContextRw,
  };
  use services::test_utils::{app_service_stub, AppServiceStub};
  use std::sync::Arc;
  use tokio::sync::mpsc::Sender;
  use tower::ServiceExt;

  async fn non_streamed_response(sender: Sender<String>) -> anyhow::Result<()> {
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
    }}
    .to_string();
    sender.send(response).await?;
    Ok(())
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
    let alias = Alias::testalias_exists();
    let model_file = HubFileBuilder::testalias_exists()
      .hf_cache(app_service_stub.hf_cache())
      .build()?;
    let tokenizer_file = HubFileBuilder::llama3_tokenizer()
      .hf_cache(app_service_stub.hf_cache())
      .build()?;
    let mut ctx = MockSharedContextRw::default();
    ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(alias),
        eq(model_file),
        eq(tokenizer_file),
        always(),
      )
      .return_once(move |_, _, _, _, sender: Sender<String>| {
        tokio::spawn(async move {
          non_streamed_response(sender).await.unwrap();
        });
        Ok(())
      });
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

  async fn streamed_response(sender: Sender<String>) -> anyhow::Result<()> {
    for (i, value) in [
      " ", " After", " Monday", ",", " the", " next", " day", " is", " T", "ues", "day", ".",
    ]
    .into_iter()
    .enumerate()
    {
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
      let response: CreateChatCompletionStreamResponse = serde_json::from_value(response)?;
      let response = serde_json::to_string(&response)?;
      sender.send(format!("data: {response}\n\n")).await?;
    }
    let end_delta = r#"{"choices":[{"finish_reason":"stop","index":0,"delta":{}}],"created":1717317061,"id":"chatcmpl-Twf1ixroh9WzY9Pvm4IGwNF4kB4EjTp4","model":"llama2:chat","object":"chat.completion.chunk","usage":{"completion_tokens":13,"prompt_tokens":15,"total_tokens":28}}"#.to_string();
    sender.send(format!("data: {end_delta}\n\n")).await?;
    Ok(())
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
    let alias = Alias::testalias_exists();
    let model_file = HubFileBuilder::testalias_exists()
      .hf_cache(app_service_stub.hf_cache())
      .build()?;
    let tokenizer_file = HubFileBuilder::llama3_tokenizer()
      .hf_cache(app_service_stub.hf_cache())
      .build()?;
    let mut ctx = MockSharedContextRw::default();
    ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(alias),
        eq(model_file),
        eq(tokenizer_file),
        always(),
      )
      .return_once(move |_, _, _, _, sender: Sender<String>| {
        tokio::spawn(async move {
          let res = streamed_response(sender).await;
          assert!(res.is_ok());
        });
        Ok(())
      });

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
