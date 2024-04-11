use crate::server::bodhi_ctx::BodhiContextWrapper;
use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
use axum::extract::State;
use axum::{
  http::StatusCode,
  response::IntoResponse,
  routing::{get, post},
  Json,
};
use std::ffi::{c_char, c_void};
use std::slice;
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;

// TODO: serialize error in OpenAI format
#[derive(Debug)]
enum ApiError {
  Json(serde_json::Error),
}

impl IntoResponse for ApiError {
  fn into_response(self) -> axum::response::Response {
    match self {
      ApiError::Json(e) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Error while marshalling response: {e}"),
      )
        .into_response(),
    }
  }
}

type Result<T> = std::result::Result<T, ApiError>;

#[derive(Clone)]
struct RouterState {
  bodhi_ctx: Arc<Mutex<BodhiContextWrapper>>,
}

impl RouterState {
  fn new(bodhi_ctx: Arc<Mutex<BodhiContextWrapper>>) -> Self {
    Self { bodhi_ctx }
  }
}

pub(super) fn build_routes(bodhi_ctx: Arc<Mutex<BodhiContextWrapper>>) -> axum::Router {
  axum::Router::new()
    .route("/ping", get(|| async { "pong" }))
    .route("/v1/chat/completions", post(chat_completions_handler))
    .layer(TraceLayer::new_for_http())
    .with_state(RouterState::new(bodhi_ctx))
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

async fn chat_completions_handler(
  State(state): State<RouterState>,
  Json(request): Json<CreateChatCompletionRequest>,
) -> Result<Json<CreateChatCompletionResponse>> {
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
  serde_json::from_str(&userdata)
    .map(Json)
    .map_err(ApiError::Json)
}
