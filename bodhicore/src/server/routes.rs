use super::{
  router_state::RouterState,
  routes_chat::chat_completions_handler,
  routes_models::{oai_model_handler, oai_models_handler},
  routes_ollama::{ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler},
  routes_setup::{app_info_handler, setup_handler},
  routes_ui::chats_router,
  RouterStateFn,
};
use crate::{
  service::{auth_middleware, AppServiceFn},
  SharedContextRwFn,
};
use axum::{
  middleware::from_fn_with_state,
  routing::{get, post},
  Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_routes(
  ctx: Arc<dyn SharedContextRwFn>,
  app_service: Arc<dyn AppServiceFn>,
  static_router: Option<Router>,
) -> Router {
  let state: Arc<dyn RouterStateFn> = Arc::new(RouterState::new(ctx, app_service.clone()));
  let public_apis = Router::new()
    .route("/ping", get(|| async { "pong" }))
    .route("/app/info", get(app_info_handler))
    .route("/app/setup", post(setup_handler));
  let api_router = Router::new().merge(chats_router());
  let protected_apis = Router::new()
    .route("/api/tags", get(ollama_models_handler))
    .route("/api/show", post(ollama_model_show_handler))
    .route("/api/chat", post(ollama_model_chat_handler))
    .nest("/api/ui", api_router)
    .route("/v1/models", get(oai_models_handler))
    .route("/v1/models/:id", get(oai_model_handler))
    .route("/v1/chat/completions", post(chat_completions_handler))
    .route_layer(from_fn_with_state(state.clone(), auth_middleware));

  let router = Router::<Arc<dyn RouterStateFn>>::new()
    .merge(public_apis)
    .merge(protected_apis)
    // TODO: check CORS
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false),
    )
    .layer(app_service.session_service().session_layer())
    .layer(TraceLayer::new_for_http())
    .with_state(state);
  if let Some(static_router) = static_router {
    router.merge(static_router)
  } else {
    router
  }
}
