use super::{
  context::load_context,
  routes::{ApiError, RouterState},
  utils,
};
use crate::{chat_template::ChatTemplate, hf::find_model, hf_tokenizer::HubTokenizerConfig};
use anyhow::Context;
use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
use axum::{
  body::Body,
  extract::State,
  http::StatusCode,
  response::{sse::Event, IntoResponse, Response, Sse},
  Json,
};
use serde_json::Value;
use std::{
  convert::Infallible,
  ffi::{c_char, c_void},
  slice,
};
use tokio::sync::mpsc::Sender;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

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
  State(mut state): State<RouterState>,
  Json(request): Json<CreateChatCompletionRequest>,
) -> Response<Body> {
  if let Err(err) = load_context(&mut state, &request.model).await {
    return err;
  };
  let mut input = serde_json::to_value(&request)
    .context("converting request to string to pass to bodhi_server")
    .unwrap();

  let model = find_model(&request.model).unwrap();
  let config = HubTokenizerConfig::for_repo(&model.repo).ok().unwrap_or_default();
  let chat_template = ChatTemplate::new(config).unwrap();
  let (chat_template, prompt) = chat_template.apply(input["messages"].clone()).unwrap();
  let prompt = prompt.as_object().unwrap();
  if prompt.contains_key("prompt") {
    let base = input.as_object_mut().unwrap();
    base.insert(
      "prompt".to_string(),
      prompt.get("prompt").unwrap().to_string().into(),
    );
  }

  if request.stream.unwrap_or(false) {
    return chat_completions_stream_handler(state, input, chat_template).await;
  }
  let input = serde_json::to_string(&input).unwrap();
  let userdata = String::with_capacity(2048);
  let lock = state.ctx.read().await;
  let Some(ctx) = lock.as_ref() else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      utils::ApiError::ServerError("context not laoded".to_string()),
    )
      .into_response();
  };
  ctx
    .completions(
      &input,
      &chat_template,
      Some(server_callback),
      &userdata as *const _ as *mut c_void,
    )
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
    let lock = state.ctx.read().await;
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
