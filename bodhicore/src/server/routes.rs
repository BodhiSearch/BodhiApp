use super::{
  super::{db::DbServiceFn, service::AppServiceFn, SharedContextRwFn},
  router_state::RouterState,
  routes_chat::chat_completions_handler,
  routes_models::ui_models_handler,
  routes_ui::chats_router,
};
use axum::{
  routing::{get, post},
  Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_routes(
  ctx: Arc<dyn SharedContextRwFn>,
  app_service: Arc<dyn AppServiceFn>,
  db_service: Arc<dyn DbServiceFn>,
) -> Router {
  let state = RouterState::new(ctx, app_service, db_service);
  let api_router = Router::new().route("/models", get(ui_models_handler));
  Router::new()
    .route("/ping", get(|| async { "pong" }))
    .nest("/api/ui", api_router)
    .route("/v1/chat/completions", post(chat_completions_handler))
    .merge(chats_router())
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
