use super::{
  routes_chat::chat_completions_handler,
  routes_models::ui_models_handler,
  routes_ui::{ui_chats_delete_handler, ui_chats_handler},
};
use crate::server::bodhi_ctx::BodhiContextWrapper;
use axum::{
  http::StatusCode,
  response::IntoResponse,
  routing::{delete, get, post},
};
use std::sync::{Arc, Mutex};
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
  pub(crate) bodhi_ctx: Arc<Mutex<BodhiContextWrapper>>,
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
    .route("/api/ui/chats", get(ui_chats_handler))
    .route("/api/ui/chats", delete(ui_chats_delete_handler))
    .route("/api/ui/models", get(ui_models_handler))
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false),
    )
    .layer(TraceLayer::new_for_http())
    .with_state(RouterState::new(bodhi_ctx))
}
