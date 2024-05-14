use super::{
  routes_chat::chat_completions_handler,
  routes_models::ui_models_handler,
  routes_ui::{
    ui_chat_delete_handler, ui_chat_handler, ui_chat_update_handler, ui_chats_delete_handler,
    ui_chats_handler,
  },
  shared_rw::SharedContextRw,
};
use axum::{
  body::Body,
  http::{self, StatusCode, Uri},
  response::{IntoResponse, Response},
  routing::{delete, get, post},
};
use include_dir::Dir;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// TODO: serialize error in OpenAI format
#[derive(Debug)]
pub(crate) enum ApiError {
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

#[derive(Clone)]
pub(crate) struct RouterState {
  pub(crate) ctx: SharedContextRw,
}

impl RouterState {
  fn new(ctx: SharedContextRw) -> Self {
    Self { ctx }
  }
}

pub fn build_routes(ctx: SharedContextRw) -> axum::Router {
  let state = RouterState::new(ctx);
  axum::Router::new()
    .route("/ping", get(|| async { "pong" }))
    .route("/v1/chat/completions", post(chat_completions_handler))
    .route("/api/ui/chats", get(ui_chats_handler))
    .route("/api/ui/chats", delete(ui_chats_delete_handler))
    .route("/api/ui/chats/:id", get(ui_chat_handler))
    .route("/api/ui/chats/:id", post(ui_chat_update_handler))
    .route("/api/ui/chats/:id", delete(ui_chat_delete_handler))
    .route("/api/ui/models", get(ui_models_handler))
    // .route("/", get(static_handler))
    // .route("/*path", get(static_handler))
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false),
    )
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}

// async fn static_handler(uri: Uri) -> impl IntoResponse {
//   let mut path = uri.path();
//   // coming from route "/"
//   if path.eq("/") {
//     path = "index.html";
//   }
//   // coming from route "/*path", does not contain leading '/' in path, but may be trailing one
//   if path.starts_with('/') {
//     path = &path[1..];
//   }
//   let path = if path.ends_with('/') {
//     format!("{}index.html", path)
//   } else {
//     path.to_owned()
//   };
//   let path = PathBuf::from(path);
//   if let Some(file) = STATIC_DIR.get_file(&path) {
//     let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
//     (
//       StatusCode::OK,
//       [(http::header::CONTENT_TYPE, mime_type.as_ref())],
//       file.contents(),
//     )
//       .into_response()
//   } else {
//     (StatusCode::NOT_FOUND, "Not Found").into_response()
//   }
// }