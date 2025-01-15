use crate::proxy_router;
use auth_middleware::{auth_middleware, optional_auth_middleware};
use axum::{
  body::Body,
  http::StatusCode,
  middleware::from_fn_with_state,
  response::Response,
  routing::{get, post},
  Router,
};
use routes_app::{
  app_info_handler, chats_router, create_router, create_token_handler, dev_secrets_handler,
  list_tokens_handler, login_callback_handler, login_handler, logout_handler, pull_router,
  setup_handler, user_info_handler,
};
use routes_oai::{
  chat_completions_handler, models_router, oai_model_handler, oai_models_handler,
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
};
use serde_json::json;
use server_core::{DefaultRouterState, RouterState, SharedContext};
use services::{AppService, EnvService, BODHI_DEV_PROXY_UI};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{debug, info, Level};

pub fn build_routes(
  ctx: Arc<dyn SharedContext>,
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
    .route("/app/login", get(login_handler))
    .route("/app/login/", get(login_handler))
    .route("/api/ui/user", get(user_info_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware));
  let protected_apis = Router::new()
    .route("/api/tags", get(ollama_models_handler))
    .route("/api/show", post(ollama_model_show_handler))
    .route("/api/chat", post(ollama_model_chat_handler))
    .nest("/api/ui", api_ui_router)
    .route("/api/tokens", post(create_token_handler))
    .route("/api/tokens", get(list_tokens_handler))
    .route("/api/tokens/", get(list_tokens_handler))
    .route("/v1/models", get(oai_models_handler))
    .route("/v1/models/:id", get(oai_model_handler))
    .route("/v1/chat/completions", post(chat_completions_handler))
    .route_layer(from_fn_with_state(state.clone(), auth_middleware));

  let info_trace = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO));
  let router = Router::<Arc<dyn RouterState>>::new()
    .merge(public_apis)
    .merge(optional_auth)
    .merge(protected_apis)
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false),
    )
    .with_state(state)
    .layer(info_trace);
  let router = apply_ui_router(
    &app_service.env_service(),
    router,
    static_router,
    proxy_router("http://localhost:3000".to_string()),
  );
  router.layer(app_service.session_service().session_layer())
}

fn apply_ui_router(
  env_service: &Arc<dyn EnvService>,
  router: Router,
  static_router: Option<Router>,
  proxy_router: Router,
) -> Router {
  let proxy_ui = env_service
    .get_dev_env(BODHI_DEV_PROXY_UI)
    .map(|val| val.parse::<bool>().unwrap_or_default())
    .unwrap_or_default();

  match env_service.is_production() {
    true => {
      if let Some(static_router) = static_router {
        debug!("serving ui from embedded assets");
        router.merge(static_router)
      } else {
        router
      }
    }
    false if proxy_ui => {
      info!("proxying the ui to localhost:3000");
      router.merge(proxy_router)
    }
    false => {
      if let Some(static_router) = static_router {
        info!("serving ui from embedded assets");
        router.merge(static_router)
      } else {
        router
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::apply_ui_router;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use mockall::predicate::*;
  use rstest::{fixture, rstest};
  use server_core::test_utils::ResponseTestExt;
  use services::{EnvService, MockEnvService, BODHI_DEV_PROXY_UI};
  use std::sync::Arc;
  use tower::ServiceExt;

  // Helper to create a stub router that returns a specific path
  fn create_stub_router(path: &'static str) -> Router {
    let result = path.to_string();
    Router::new().route(path, get(|| async { result }))
  }

  // Helper to make a test request to the router
  async fn test_request(router: Router, path: &str) -> (StatusCode, String) {
    let response = router
      .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
      .await
      .unwrap();

    let status = response.status();
    let body = response.text().await.unwrap();
    (status, body)
  }

  #[fixture]
  fn base_router() -> Router {
    create_stub_router("/api")
  }

  #[fixture]
  fn static_router() -> Router {
    create_stub_router("/static")
  }

  #[fixture]
  fn proxy_router() -> Router {
    create_stub_router("/proxy")
  }

  struct EnvConfig {
    is_production: bool,
    proxy_ui: Option<String>,
  }

  fn env_service(config: EnvConfig) -> Arc<dyn EnvService> {
    let mut mock_env = MockEnvService::new();

    mock_env
      .expect_is_production()
      .return_const(config.is_production);

    mock_env
      .expect_get_dev_env()
      .with(eq(BODHI_DEV_PROXY_UI))
      .return_const(config.proxy_ui);

    Arc::new(mock_env)
  }

  #[rstest]
  #[case::production_with_static(
    EnvConfig {
      is_production: true,
      proxy_ui: None
    },
    Some(static_router()),
    vec![
      ("/api", true),
      ("/static", true),
      ("/proxy", false),
    ]
  )]
  #[case::production_without_static(
    EnvConfig {
      is_production: true,
      proxy_ui: None
    },
    None,
    vec![
      ("/api", true),
      ("/static", false),
      ("/proxy", false),
    ]
  )]
  #[case::dev_with_proxy(
    EnvConfig {
      is_production: false,
      proxy_ui: Some("true".to_string())
    },
    Some(static_router()),
    vec![
      ("/api", true),
      ("/static", false),
      ("/proxy", true),
    ]
  )]
  #[case::dev_with_static(
    EnvConfig {
      is_production: false,
      proxy_ui: Some("false".to_string())
    },
    Some(static_router()),
    vec![
      ("/api", true),
      ("/static", true),
      ("/proxy", false),
    ]
  )]
  #[case::dev_without_static(
    EnvConfig {
      is_production: false,
      proxy_ui: Some("false".to_string())
    },
    None,
    vec![
      ("/api", true),
      ("/static", false),
      ("/proxy", false),
    ]
  )]
  #[tokio::test]
  async fn test_ui_router_scenarios(
    #[case] config: EnvConfig,
    #[case] static_router: Option<Router>,
    #[case] test_paths: Vec<(&str, bool)>,
  ) {
    let env_service = env_service(config);
    let router = apply_ui_router(&env_service, base_router(), static_router, proxy_router());

    for (path, should_exist) in test_paths {
      let (status, body) = test_request(router.clone(), path).await;

      if should_exist {
        assert_eq!(status, StatusCode::OK, "Path {} should exist", path);
        assert_eq!(body, path);
      } else {
        assert_eq!(
          status,
          StatusCode::NOT_FOUND,
          "Path {} should not exist",
          path
        );
      }
    }
  }
}
