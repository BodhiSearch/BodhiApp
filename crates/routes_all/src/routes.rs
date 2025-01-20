#[macro_export]
macro_rules! make_ui_endpoint {
  ($name:ident, $path:expr) => {
    pub const $name: &str = concat!("/api/ui/", $path);
  };
}

use crate::proxy_router;
use auth_middleware::{api_auth_middleware, auth_middleware, inject_session_auth_info};
use axum::{
  body::Body,
  http::StatusCode,
  middleware::from_fn_with_state,
  response::Response,
  routing::{get, post, put},
  Router,
};
use objs::{Role, TokenScope};
use routes_app::{
  app_info_handler, create_alias_handler, create_pull_request_handler, create_token_handler,
  dev_secrets_handler, get_alias_handler, get_download_status_handler, list_chat_templates_handler,
  list_downloads_handler, list_local_aliases_handler, list_local_modelfiles_handler,
  list_tokens_handler, login_callback_handler, login_handler, logout_handler,
  pull_by_alias_handler, setup_handler, update_alias_handler, update_token_handler,
  user_info_handler, ApiDoc, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_CHAT_TEMPLATES,
  ENDPOINT_DEV_SECRETS, ENDPOINT_LOGIN, ENDPOINT_LOGIN_CALLBACK, ENDPOINT_LOGOUT, ENDPOINT_MODELS,
  ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_PULL, ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_MODELS,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS, ENDPOINT_PING, ENDPOINT_TOKENS,
  ENDPOINT_USER_INFO,
};
use routes_oai::{
  chat_completions_handler, oai_model_handler, oai_models_handler, ollama_model_chat_handler,
  ollama_model_show_handler, ollama_models_handler,
};
use serde_json::json;
use server_core::{DefaultRouterState, RouterState, SharedContext};
use services::{AppService, EnvService, BODHI_DEV_PROXY_UI};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{debug, info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn build_routes(
  ctx: Arc<dyn SharedContext>,
  app_service: Arc<dyn AppService>,
  static_router: Option<Router>,
) -> Router {
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(ctx, app_service.clone()));

  // Public APIs (no auth required)
  let mut public_apis = Router::new()
    .route(
      ENDPOINT_PING,
      get(|| async {
        Response::builder()
          .status(StatusCode::OK)
          .body(Body::from(json!({"message": "pong"}).to_string()))
          .unwrap()
      }),
    )
    .route(ENDPOINT_APP_INFO, get(app_info_handler))
    .route(ENDPOINT_APP_SETUP, post(setup_handler))
    .route(ENDPOINT_LOGIN_CALLBACK, get(login_callback_handler))
    // TODO: having as api/ui/logout coz of status code as 200 instead of 302 because of automatic follow redirect by axios
    .route(ENDPOINT_LOGOUT, post(logout_handler));

  // Dev-only admin routes
  if !app_service.env_service().is_production() {
    let dev_apis = Router::new().route(ENDPOINT_DEV_SECRETS, get(dev_secrets_handler));
    public_apis = public_apis.merge(dev_apis);
  }

  let optional_auth = Router::new()
    .route(ENDPOINT_LOGIN, get(login_handler))
    .route(&format!("{ENDPOINT_LOGIN}/"), get(login_handler))
    .route(ENDPOINT_USER_INFO, get(user_info_handler))
    .route_layer(from_fn_with_state(state.clone(), inject_session_auth_info));

  // User level APIs (role=user & scope=scope_token_user)
  let user_apis = Router::new()
    // OpenAI Compatible APIs
    .route(ENDPOINT_OAI_MODELS, get(oai_models_handler))
    .route(
      &format!("{ENDPOINT_OAI_MODELS}/:id"),
      get(oai_model_handler),
    )
    .route(
      ENDPOINT_OAI_CHAT_COMPLETIONS,
      post(chat_completions_handler),
    )
    // Ollama APIs
    .route(ENDPOINT_OLLAMA_TAGS, get(ollama_models_handler))
    .route(ENDPOINT_OLLAMA_SHOW, post(ollama_model_show_handler))
    .route(ENDPOINT_OLLAMA_CHAT, post(ollama_model_chat_handler))
    // Basic Bodhi APIs
    .route(ENDPOINT_MODELS, get(list_local_aliases_handler))
    .route(&format!("{ENDPOINT_MODELS}/:id"), get(get_alias_handler))
    .route(ENDPOINT_MODEL_FILES, get(list_local_modelfiles_handler))
    .route(ENDPOINT_CHAT_TEMPLATES, get(list_chat_templates_handler))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(Role::User, Some(TokenScope::User), state, req, next)
      },
    ));

  // Power user APIs (role=power_user or scope=scope_token_power_user)
  let power_user_apis = Router::new()
    .route(ENDPOINT_MODELS, post(create_alias_handler))
    .route(&format!("{ENDPOINT_MODELS}/:id"), put(update_alias_handler))
    .route(ENDPOINT_MODEL_PULL, get(list_downloads_handler))
    .route(ENDPOINT_MODEL_PULL, post(create_pull_request_handler))
    .route(
      &format!("{ENDPOINT_MODEL_PULL}/:alias"),
      post(pull_by_alias_handler),
    )
    .route(
      &format!("{ENDPOINT_MODEL_PULL}/:id"),
      get(get_download_status_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(
          Role::PowerUser,
          Some(TokenScope::PowerUser),
          state,
          req,
          next,
        )
      },
    ));

  // Session-only power user APIs (token management)
  let power_user_session_apis = Router::new()
    .route(ENDPOINT_TOKENS, post(create_token_handler))
    .route(ENDPOINT_TOKENS, get(list_tokens_handler))
    .route(
      &format!("{ENDPOINT_TOKENS}/:token_id"),
      put(update_token_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| api_auth_middleware(Role::PowerUser, None, state, req, next),
    ));

  // Combine all protected APIs
  let protected_apis = Router::new()
    .merge(user_apis)
    .merge(power_user_apis)
    .merge(power_user_session_apis)
    .route_layer(from_fn_with_state(state.clone(), auth_middleware));

  let info_trace = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO));

  // Build final router
  let router = Router::<Arc<dyn RouterState>>::new()
    .merge(public_apis)
    .merge(optional_auth)
    .merge(protected_apis)
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
