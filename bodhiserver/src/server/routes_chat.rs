use super::routes::{ApiError, RouterState};
use super::routes_ui::save_stream;
use anyhow::{anyhow, Context};
use async_openai::types::{
  CreateChatCompletionRequest, CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
};
use axum::{
  body::Body,
  extract::{Query, State},
  response::{sse::Event, IntoResponse, Response, Sse},
  Json,
};
use serde::Deserialize;
use std::{
  convert::Infallible,
  ffi::{c_char, c_void},
  slice,
};
use tokio::sync::mpsc::{unbounded_channel, Sender, UnboundedSender};
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

#[derive(Deserialize)]
pub(crate) struct Params {
  id: Option<String>,
}

pub(crate) async fn chat_completions_handler(
  State(state): State<RouterState>,
  Query(Params { id }): Query<Params>,
  Json(request): Json<CreateChatCompletionRequest>,
) -> Response<Body> {
  if request.stream.unwrap_or(false) {
    return chat_completions_stream_handler(state, id, request).await;
  }
  let bodhi_ctx = state.bodhi_ctx.lock().unwrap();
  let input = serde_json::to_string(&request).unwrap();
  let userdata = String::with_capacity(2048);
  bodhi_ctx
    .ctx
    .as_ref()
    .unwrap()
    .completions(
      &input,
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
  id: Option<String>,
  request: CreateChatCompletionRequest,
) -> Response<Body> {
  let input = serde_json::to_string(&request)
    .context("converting request to string to pass to bodhi_server")
    .unwrap();
  let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
  tokio::spawn(async move {
    let bodhi_ctx = state
      .bodhi_ctx
      .lock()
      .map_err(|e| anyhow!("{:?}", e))
      .context("unable to get the lock")
      .unwrap();
    let result = bodhi_ctx
      .ctx
      .as_ref()
      .ok_or_else(|| anyhow!("bodhi_ctx is not initialized"))
      .unwrap()
      .completions(
        &input,
        Some(server_callback_stream),
        &tx as *const _ as *mut c_void,
      );
    if let Err(err) = result {
      tracing::warn!(err = format!("{}", err), "error while streaming completion")
    }
    drop(tx);
  });
  let save_tx = if id.is_some() {
    let (save_tx, save_rx) = unbounded_channel::<String>();
    tokio::spawn(async move {
      let result = save_stream(id, request, save_rx).await;
      if let Err(err) = result {
        tracing::warn!(err = format!("{}", err), "error saving chat");
      }
    });
    Some(save_tx)
  } else {
    None
  };
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
    let _ = save_response(save_tx.as_ref(), data);
    Ok(Event::default().data(data))
  });
  Sse::new(stream).into_response()
}

fn save_response(tx: Option<&UnboundedSender<String>>, data: &str) -> Option<()> {
  let tx = tx?;
  let delta = serde_json::from_str::<CreateChatCompletionStreamResponse>(data).ok()?;
  let content = (&delta.choices.first()?.delta.content.as_ref()?).to_string();
  tx.send(content).ok()?;
  Some(())
}
