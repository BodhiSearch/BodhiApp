use crate::{
  app_info_handler, auth_middleware, chat_completions_handler, chats_router, create_router,
  dev_secrets_handler, login_callback_handler, login_handler, logout_handler, models_router,
  oai_model_handler, oai_models_handler, ollama_model_chat_handler, ollama_model_show_handler,
  ollama_models_handler, optional_auth_middleware, proxy_router, pull_router, setup_handler,
  user_info_handler, DefaultRouterState, RouterState, SharedContextRw,
};
use axum::{
  body::Body,
  http::StatusCode,
  middleware::from_fn_with_state,
  response::Response,
  routing::{get, post},
  Router,
};
use serde_json::json;
use services::AppService;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_routes(
  ctx: Arc<dyn SharedContextRw>,
  app_service: Arc<dyn AppService>,
  static_router: Option<Router>,
) -> Router {
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(ctx, app_service.clone()));
  let mut public_apis = Router::new()
    .route(
      "/ping",
      get(|| async {
        Response::builder()
          .status(StatusCode::OK)
          .body(Body::from(json!({"message": "pong"}).to_string()))
          .unwrap()
      }),
    )
    .route("/app/info", get(app_info_handler))
    .route("/app/setup", post(setup_handler))
    .route("/app/login", get(login_handler))
    .route("/app/login/", get(login_handler))
    .route("/app/login/callback", get(login_callback_handler))
    // TODO: having as api/ui/logout coz of status code as 200 instead of 302 because of automatic follow redirect by axios
    .route("/api/ui/logout", post(logout_handler));

  if !app_service.env_service().is_production() {
    let dev_apis = Router::new().route("/dev/secrets", get(dev_secrets_handler));
    public_apis = public_apis.merge(dev_apis);
  }
  let api_ui_router = Router::new()
    .merge(chats_router())
    .merge(models_router())
    .merge(create_router())
    .merge(pull_router());
  let optional_auth = Router::new()
    .route("/api/ui/user", get(user_info_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware));
  let protected_apis = Router::new()
    .route("/api/tags", get(ollama_models_handler))
    .route("/api/show", post(ollama_model_show_handler))
    .route("/api/chat", post(ollama_model_chat_handler))
    .nest("/api/ui", api_ui_router)
    .route("/v1/models", get(oai_models_handler))
    .route("/v1/models/:id", get(oai_model_handler))
    .route("/v1/chat/completions", post(chat_completions_handler))
    .route_layer(from_fn_with_state(state.clone(), auth_middleware));

  let router = Router::<Arc<dyn RouterState>>::new()
    .merge(public_apis)
    .merge(optional_auth)
    .merge(protected_apis)
    // TODO: check CORS
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false),
    )
    .with_state(state);
  let router = if app_service.env_service().is_production() {
    if let Some(static_router) = static_router {
      router.merge(static_router)
    } else {
      router
    }
  } else {
    router.merge(proxy_router("http://localhost:3000".to_string()))
  };
  router
    .layer(app_service.session_service().session_layer())
    .layer(TraceLayer::new_for_http())
}
