use super::routes::{ApiError, RouterState};
use anyhow::{anyhow, Context};
use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
use axum::body::Body;
use axum::extract::State;
use axum::response::sse::Event;
use axum::response::{Response, Sse};
use axum::{response::IntoResponse, Json};
use std::convert::Infallible;
use std::ffi::{c_char, c_void};
use std::slice;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

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
  if request.stream.unwrap_or(false) {
    return chat_completions_stream_handler(state, request).await;
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

  let stream = ReceiverStream::new(rx).map::<std::result::Result<Event, Infallible>, _>(|msg| {
    if msg.starts_with("data: ") {
      let data = msg
        .strip_prefix("data: ")
        .unwrap()
        .strip_suffix("\n\n")
        .unwrap();
      Ok(Event::default().data(data))
    } else if msg.starts_with("error: ") {
      let data = msg
        .strip_prefix("error: ")
        .unwrap()
        .strip_suffix("\n\n")
        .unwrap();
      Ok(Event::default().data(data))
    } else {
      tracing::error!(msg, "unknown event type raised from bodhi_server");
      Ok(Event::default().data(msg))
    }
  });
  Sse::new(stream).into_response()
}
