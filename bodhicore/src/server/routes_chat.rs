use super::{router_state::RouterState, routes::ApiError};
use crate::tokenizer_config::TokenizerConfig;
use anyhow::Context;
use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
use axum::{
  body::Body,
  extract::State,
  response::{sse::Event, IntoResponse, Response, Sse},
  routing::post,
  Json, Router,
};
use serde_json::Value;
use std::{
  convert::Infallible,
  ffi::{c_char, c_void},
  slice,
};
use tokio::sync::mpsc::Sender;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

pub(crate) fn llm_router() -> Router<RouterState> {
  Router::new().route("/v1/chat/completions", post(chat_completions_handler))
}

unsafe extern "C" fn server_callback(
  contents: *const c_char,
  size: usize,
  userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  };
  let user_data_str = unsafe { &mut *(userdata as *mut String) };
  user_data_str.push_str(input_str);
  size
}

unsafe extern "C" fn server_callback_stream(
  contents: *const c_char,
  size: usize,
  userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  }
  .to_owned();
  let sender = unsafe { &mut *(userdata as *mut Sender<String>) }.clone();
  // TODO: handle closed receiver
  tokio::spawn(async move { sender.send(input_str).await.unwrap() });
  size
}

pub(crate) async fn chat_completions_handler(
  State(state): State<RouterState>,
  Json(request): Json<CreateChatCompletionRequest>,
) -> Response<Body> {
  let mut input = serde_json::to_value(&request)
    .context("converting request to string to pass to bodhi_server")
    .unwrap();

  let alias = state.app_service.find_alias(&request.model).unwrap();
  // let config = TokenizerConfig::for_repo(&alias.repo)
  //   .ok()
  //   .unwrap_or_default();
  let config = TokenizerConfig::default();
  let prompt = config.apply_chat_template(&request.messages).unwrap();
  input["prompt"] = Value::String(prompt);
  if request.stream.unwrap_or(false) {
    return chat_completions_stream_handler(state, input, String::from("")).await;
  }
  let input = serde_json::to_string(&input).unwrap();
  let userdata = String::with_capacity(2048);
  state
    .completions(&request.model, &input, "", Some(server_callback), &userdata)
    .await
    .unwrap(); // todo
  serde_json::from_str::<CreateChatCompletionResponse>(&userdata)
    .map(Json)
    .map_err(ApiError::Json)
    .into_response()
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
  tokio::spawn(async move {
    let lock = state.ctx.ctx.read().await;
    let Some(ctx) = lock.as_ref() else {
      tracing::warn!("context is not laoded");
      return;
    };
    let result = ctx.completions(
      &input,
      &chat_template,
      Some(server_callback_stream),
      &tx as *const _ as *mut c_void,
    );
    if let Err(err) = result {
      tracing::warn!(err = format!("{}", err), "error while streaming completion")
    }
    drop(tx);
  });
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
  Sse::new(stream).into_response()
}

#[cfg(test)]
mod test {
  use super::llm_router;
  use crate::bindings::{disable_llama_log, llama_server_disable_logging};
  use crate::test_utils::{app_service_stub, AppServiceTuple, ResponseTestExt};
  use crate::{
    server::router_state::RouterState,
    test_utils::{init_test_tracing, RequestTestExt},
    SharedContextRw, SharedContextRwFn,
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{CreateChatCompletionResponse, CreateChatCompletionStreamResponse};
  use axum::{body::Body, extract::Request};
  use ctor::ctor;
  use llama_server_bindings::GptParams;
  use reqwest::StatusCode;
  use rstest::rstest;
  use serde_json::json;
  use serial_test::serial;
  use std::sync::Arc;
  use tower::ServiceExt;

  #[ctor]
  fn init() {
    init_test_tracing();
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial]
  #[anyhow_trace]
  async fn test_routes_chat_completions(app_service_stub: AppServiceTuple) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let request = serde_json::to_string(&json! {{
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
    let app = llm_router().with_state(RouterState::new(wrapper, Arc::new(service)));
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
    let app = llm_router().with_state(RouterState::new(wrapper, Arc::new(service)));
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
}
