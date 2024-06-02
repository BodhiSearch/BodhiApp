use super::{
  router_state::RouterState,
  routes_chat::chat_completions_handler,
  routes_models::ui_models_handler,
  routes_ui::{
    ui_chat_delete_handler, ui_chat_handler, ui_chat_update_handler, ui_chats_delete_handler,
    ui_chats_handler,
  },
};
use crate::{service::AppServiceFn, SharedContextRwFn};
use axum::{
  routing::{delete, get, post},
  Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_routes(ctx: Arc<dyn SharedContextRwFn>, app_service: Arc<dyn AppServiceFn>) -> Router {
  let state = RouterState::new(ctx, app_service);
  let api_router = Router::new()
    .route("/chats", get(ui_chats_handler))
    .route("/chats", delete(ui_chats_delete_handler))
    .route("/chats/:id", get(ui_chat_handler))
    .route("/chats/:id", post(ui_chat_update_handler))
    .route("/chats/:id", delete(ui_chat_delete_handler))
    .route("/models", get(ui_models_handler));
  Router::new()
    .route("/ping", get(|| async { "pong" }))
    .nest("/api/ui", api_router)
    .route("/v1/chat/completions", post(chat_completions_handler))
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false),
    )
    .layer(TraceLayer::new_for_http())
    .with_state(Arc::new(state))
}
