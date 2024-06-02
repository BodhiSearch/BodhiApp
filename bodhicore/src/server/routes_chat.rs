use super::{router_state::RouterState, RouterStateFn};
use crate::oai::{OAIResponse, OpenAIApiError};
use anyhow::Context;
use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
use axum::{
  body::Body,
  extract::State,
  http::{header, HeaderValue, StatusCode},
  response::{sse::Event, IntoResponse, Response, Sse},
  Json,
};
use futures_util::StreamExt;
use serde_json::Value;
use std::{convert::Infallible, sync::Arc};
use tokio_stream::wrappers::ReceiverStream;

pub(crate) async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  Json(request): Json<CreateChatCompletionRequest>,
) -> Result<Response, OpenAIApiError> {
  let stream = request.stream.unwrap_or(false);
  let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
  let handle = tokio::spawn(async move { state.chat_completions(request, tx).await });
  if !stream {
    if let Some(message) = rx.recv().await {
      drop(rx);
      _ = handle.await;
      let response = Response::builder()
        .status(StatusCode::OK)
        .header(
          header::CONTENT_TYPE,
          HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )
        .body(Body::from(message))
        .map_err(|err| OpenAIApiError::InternalServer(err.to_string()))?;
      Ok(response)
    } else {
      Err(OpenAIApiError::InternalServer(
        "receiver stream abruptly closed".to_string(),
      ))
    }
  } else {
    let stream = ReceiverStream::new(rx).map::<Result<Event, Infallible>, _>(move |msg| {
      let data = if msg.starts_with("data: ") {
        msg
          .strip_prefix("data: ")
          .unwrap()
          .strip_suffix("\n\n")
          .unwrap()
      } else if msg.starts_with("error: ") {
        msg
          .strip_prefix("error: ")
          .unwrap()
          .strip_suffix("\n\n")
          .unwrap()
      } else {
        tracing::error!(msg, "unknown event type raised from bodhi_server");
        &msg
      };
      Ok(Event::default().data(data))
    });
    Ok(Sse::new(stream).into_response())
  }
}

async fn chat_completions_stream_handler(
  state: RouterState,
  input: Value,
  chat_template: String,
) -> Response<Body> {
  let input = serde_json::to_string(&input)
    .context("converting request to string to pass to bodhi_server")
    .unwrap();
  let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
  // tokio::spawn(async move {
  //   let lock = state.ctx.ctx.read().await;
  //   let Some(ctx) = lock.as_ref() else {
  //     tracing::warn!("context is not laoded");
  //     return;
  //   };
  //   let result = ctx.completions(
  //     &input,
  //     &chat_template,
  //     Some(server_callback_stream),
  //     &tx as *const _ as *mut c_void,
  //   );
  //   if let Err(err) = result {
  //     tracing::warn!(err = format!("{}", err), "error while streaming completion")
  //   }
  //   drop(tx);
  // });
  // let stream = ReceiverStream::new(rx).map::<Result<Event, Infallible>, _>(move |msg| {
  //   let data = if msg.starts_with("data: ") {
  //     msg
  //       .strip_prefix("data: ")
  //       .unwrap()
  //       .strip_suffix("\n\n")
  //       .unwrap()
  //   } else if msg.starts_with("error: ") {
  //     msg
  //       .strip_prefix("error: ")
  //       .unwrap()
  //       .strip_suffix("\n\n")
  //       .unwrap()
  //   } else {
  //     tracing::error!(msg, "unknown event type raised from bodhi_server");
  //     &msg
  //   };
  //   Ok(Event::default().data(data))
  // });
  // Sse::new(stream).into_response()
  todo!()
}

#[cfg(test)]
mod test {
  use crate::bindings::{disable_llama_log, llama_server_disable_logging};
  use crate::server::routes_chat::chat_completions_handler;
  use crate::service::MockHubService;
  use crate::test_utils::{
    app_service_stub, AppServiceTuple, MockAppService, MockRouterState, MockSharedContext,
    ResponseTestExt,
  };
  use crate::{
    server::router_state::RouterState,
    test_utils::{init_test_tracing, RequestTestExt},
    SharedContextRw, SharedContextRwFn,
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{
    ChatChoice, ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    ChatCompletionStreamResponseDelta, CreateChatCompletionRequestArgs,
    CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
  };
  use axum::routing::post;
  use axum::Router;
  use axum::{body::Body, extract::Request};
  use ctor::ctor;
  use llama_server_bindings::GptParams;
  use mockall::predicate::{always, eq};
  use reqwest::StatusCode;
  use rstest::{fixture, rstest};
  use serde_json::json;
  use serial_test::serial;
  use std::sync::Arc;
  use tokio::sync::mpsc::Sender;
  use tower::ServiceExt;

  #[fixture]
  fn setup() -> () {
    init_test_tracing();
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_routes_chat_completions_non_stream(
    #[from(setup)] _setup: (),
  ) -> anyhow::Result<()> {
    let mut router_state = MockRouterState::new();
    let request = CreateChatCompletionRequestArgs::default()
      .model("testalias:instruct")
      .messages(vec![ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
          .content("What day comes after Monday?")
          .build()?,
      )])
      .build()?;
    router_state
      .expect_chat_completions()
      .with(always(), always())
      .return_once(|_, sender: Sender<String>| {
        let response = json! {{
          "id": "testid",
          "model": "testalias:instruct",
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
        // .to_string();
        let response: CreateChatCompletionResponse = serde_json::from_value(response).unwrap();
        let response = serde_json::to_string(&response).unwrap();
        tokio::spawn(async move { sender.send(response).await });
        Ok(())
      });
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

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_routes_chat_completions_stream(#[from(setup)] _setup: ()) -> anyhow::Result<()> {
    let mut router_state = MockRouterState::new();
    let request = CreateChatCompletionRequestArgs::default()
      .model("testalias:instruct")
      .stream(true)
      .messages(vec![ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
          .content("What day comes after Monday?")
          .build()?,
      )])
      .build()?;
    router_state
      .expect_chat_completions()
      .with(always(), always())
      .return_once(|_, sender: Sender<String>| {
        tokio::spawn(async move {
          for (i, value) in [
            " ", " After", " Monday", ",", " the", " next", " day", " is", " T", "ues", "day",
            ".",
          ]
          .into_iter()
          .enumerate()
          {
            let response = json! {{
              "id": format!("testid-{i}"),
              "model": "testalias:instruct",
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
              serde_json::from_value(response).unwrap();
            let response = serde_json::to_string(&response).unwrap();
            _ = sender.send(format!("data: {response}\n\n")).await;
          }
          let end_delta = r#"{"choices":[{"finish_reason":"stop","index":0,"delta":{}}],"created":1717317061,"id":"chatcmpl-Twf1ixroh9WzY9Pvm4IGwNF4kB4EjTp4","model":"llama2:chat","object":"chat.completion.chunk","usage":{"completion_tokens":13,"prompt_tokens":15,"total_tokens":28}}"#.to_string();
          let _ = sender.send(format!("data: {end_delta}\n\n")).await;
        });
        Ok(())
      });
    let app = Router::new()
      .route("/v1/chat/completions", post(chat_completions_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::post("/v1/chat/completions").json(request).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let response: Vec<CreateChatCompletionStreamResponse> = response.sse().await.unwrap();
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
  /*
  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial]
  #[anyhow_trace]
  async fn test_routes_chat_completions_stream(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let request = serde_json::to_string(&json! {{
      "stream": true,
      "model": "TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q4_K_M.gguf",
      "seed": 42,
      "messages": [{"role": "user", "content": "You are a helpful assistant. What day comes after Monday?"}]
    }})
    .unwrap();
    let model_path = dirs::home_dir()
      .ok_or_else(|| anyhow!("unable to locate home dir"))?
      .join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf")
      .canonicalize()?
      .to_str()
      .unwrap()
      .to_owned();
    let gpt_params = GptParams {
      model: model_path,
      ..Default::default()
    };
    let AppServiceTuple(_temp_bodhi_home, _temp_hf_home, _, _, service) = app_service_stub;
    let wrapper = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let app = llm_router().with_state(RouterState::new(Arc::new(wrapper), Arc::new(service)));
    let response = app
      .oneshot(Request::post("/v1/chat/completions").json(request).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let response: Vec<CreateChatCompletionStreamResponse> = response.sse().await.unwrap();
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
    assert_eq!("The day that comes after Monday is Tuesday.", content);
    Ok(())
  }
  */
}
