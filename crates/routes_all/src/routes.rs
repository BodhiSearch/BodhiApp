#[macro_export]
macro_rules! make_ui_endpoint {
  ($name:ident, $path:expr) => {
    pub const $name: &str = concat!("/api/ui/", $path);
  };
}

use crate::proxy_router;
use auth_middleware::canonical_url_middleware;
use auth_middleware::{api_auth_middleware, auth_middleware, inject_optional_auth_info};
use axum::{
  middleware::from_fn_with_state,
  routing::{delete, get, post, put},
  Router,
};
use objs::{ResourceRole, TokenScope, UserScope};
use routes_app::{
  app_info_handler, approve_request_handler, auth_callback_handler, auth_initiate_handler,
  change_user_role_handler, create_alias_handler, create_api_model_handler,
  create_pull_request_handler, create_token_handler, delete_api_model_handler,
  delete_setting_handler, dev_secrets_handler, envs_handler, fetch_models_handler,
  get_api_formats_handler, get_api_model_handler, get_download_status_handler,
  get_user_alias_handler, health_handler, list_aliases_handler, list_all_requests_handler,
  list_api_models_handler, list_downloads_handler, list_local_modelfiles_handler,
  list_pending_requests_handler, list_settings_handler, list_tokens_handler, list_users_handler,
  logout_handler, ping_handler, pull_by_alias_handler, queue_status_handler,
  refresh_all_metadata_handler, refresh_single_metadata_handler, reject_request_handler,
  remove_user_handler, request_access_handler, request_status_handler, setup_handler,
  sync_models_handler, test_api_model_handler, update_alias_handler, update_api_model_handler,
  update_setting_handler, update_token_handler, user_info_handler, user_request_access_handler,
  BodhiOpenAPIDoc, GlobalErrorResponses, OpenAPIEnvModifier, ENDPOINT_ACCESS_REQUESTS_ALL,
  ENDPOINT_ACCESS_REQUESTS_PENDING, ENDPOINT_API_MODELS, ENDPOINT_API_MODELS_API_FORMATS,
  ENDPOINT_API_MODELS_FETCH_MODELS, ENDPOINT_API_MODELS_TEST, ENDPOINT_APPS_REQUEST_ACCESS,
  ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE,
  ENDPOINT_DEV_ENVS, ENDPOINT_DEV_SECRETS, ENDPOINT_HEALTH, ENDPOINT_LOGOUT, ENDPOINT_MODELS,
  ENDPOINT_MODELS_REFRESH, ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_PULL, ENDPOINT_PING,
  ENDPOINT_QUEUE, ENDPOINT_SETTINGS, ENDPOINT_TOKENS, ENDPOINT_USERS, ENDPOINT_USER_INFO,
  ENDPOINT_USER_REQUEST_ACCESS, ENDPOINT_USER_REQUEST_STATUS,
};
use routes_oai::{
  chat_completions_handler, embeddings_handler, oai_model_handler, oai_models_handler,
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};
use server_core::{DefaultRouterState, RouterState, SharedContext};
use services::{AppService, SettingService, BODHI_DEV_PROXY_UI};
use std::sync::Arc;
use tower_http::{
  cors::{Any, CorsLayer},
  trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, info, Level};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

pub fn build_routes(
  ctx: Arc<dyn SharedContext>,
  app_service: Arc<dyn AppService>,
  static_router: Option<Router>,
) -> Router {
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(ctx, app_service.clone()));

  // Public APIs (no auth required)
  let public_apis = Router::new()
    .route(ENDPOINT_PING, get(ping_handler))
    .route(ENDPOINT_HEALTH, get(health_handler))
    .route(ENDPOINT_APP_INFO, get(app_info_handler))
    .route(ENDPOINT_APP_SETUP, post(setup_handler))
    // TODO: having as api/ui/logout coz of status code as 200 instead of 302 because of automatic follow redirect by axios
    .route(ENDPOINT_LOGOUT, post(logout_handler));

  let mut optional_auth = Router::new()
    .route(ENDPOINT_USER_INFO, get(user_info_handler))
    .route(ENDPOINT_AUTH_INITIATE, post(auth_initiate_handler))
    .route(ENDPOINT_AUTH_CALLBACK, post(auth_callback_handler))
    .route(ENDPOINT_APPS_REQUEST_ACCESS, post(request_access_handler))
    .route(
      ENDPOINT_USER_REQUEST_ACCESS,
      post(user_request_access_handler),
    )
    .route(ENDPOINT_USER_REQUEST_STATUS, get(request_status_handler));

  // Dev-only routes with optional auth
  if !app_service.setting_service().is_production() {
    let dev_apis = Router::new()
      .route(ENDPOINT_DEV_SECRETS, get(dev_secrets_handler))
      .route(ENDPOINT_DEV_ENVS, get(envs_handler));
    optional_auth = optional_auth.merge(dev_apis);
  }

  let optional_auth =
    optional_auth.route_layer(from_fn_with_state(state.clone(), inject_optional_auth_info));

  // User level APIs (role=user & scope=scope_token_user)
  let user_apis = Router::new()
    // OpenAI Compatible APIs
    .route(ENDPOINT_OAI_MODELS, get(oai_models_handler))
    .route(
      &format!("{ENDPOINT_OAI_MODELS}/{{id}}"),
      get(oai_model_handler),
    )
    .route(
      ENDPOINT_OAI_CHAT_COMPLETIONS,
      post(chat_completions_handler),
    )
    .route(ENDPOINT_OAI_EMBEDDINGS, post(embeddings_handler))
    // Ollama APIs
    .route(ENDPOINT_OLLAMA_TAGS, get(ollama_models_handler))
    .route(ENDPOINT_OLLAMA_SHOW, post(ollama_model_show_handler))
    .route(ENDPOINT_OLLAMA_CHAT, post(ollama_model_chat_handler))
    // Basic Bodhi APIs
    .route(ENDPOINT_MODELS, get(list_aliases_handler))
    .route(
      &format!("{ENDPOINT_MODELS}/{{id}}"),
      get(get_user_alias_handler),
    )
    .route(ENDPOINT_MODEL_FILES, get(list_local_modelfiles_handler))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(
          ResourceRole::User,
          Some(TokenScope::User),
          Some(UserScope::User),
          state,
          req,
          next,
        )
      },
    ));

  // Power user APIs (role=power_user or scope=scope_token_power_user)
  let power_user_apis = Router::new()
    .route(ENDPOINT_MODELS, post(create_alias_handler))
    .route(
      &format!("{ENDPOINT_MODELS}/{{id}}"),
      put(update_alias_handler),
    )
    .route(ENDPOINT_MODEL_PULL, get(list_downloads_handler))
    .route(ENDPOINT_MODEL_PULL, post(create_pull_request_handler))
    .route(
      &format!("{ENDPOINT_MODEL_PULL}/{{id}}"),
      post(pull_by_alias_handler),
    )
    .route(
      &format!("{ENDPOINT_MODEL_PULL}/{{id}}"),
      get(get_download_status_handler),
    )
    // API Models management
    .route(ENDPOINT_API_MODELS, get(list_api_models_handler))
    .route(ENDPOINT_API_MODELS, post(create_api_model_handler))
    .route(
      ENDPOINT_API_MODELS_API_FORMATS,
      get(get_api_formats_handler),
    )
    .route(ENDPOINT_API_MODELS_TEST, post(test_api_model_handler))
    .route(ENDPOINT_API_MODELS_FETCH_MODELS, post(fetch_models_handler))
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}"),
      get(get_api_model_handler),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}"),
      put(update_api_model_handler),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}"),
      delete(delete_api_model_handler),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}/sync-models"),
      post(sync_models_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(
          ResourceRole::PowerUser,
          Some(TokenScope::PowerUser),
          Some(UserScope::PowerUser),
          state,
          req,
          next,
        )
      },
    ));

  // Session-only power user APIs (token management, metadata refresh, queue status)
  let power_user_session_apis = Router::new()
    .route(ENDPOINT_TOKENS, post(create_token_handler))
    .route(ENDPOINT_TOKENS, get(list_tokens_handler))
    .route(
      &format!("{ENDPOINT_TOKENS}/{{token_id}}"),
      put(update_token_handler),
    )
    .route(ENDPOINT_MODELS_REFRESH, post(refresh_all_metadata_handler))
    .route(
      &format!("{ENDPOINT_MODELS}/{{id}}/refresh"),
      post(refresh_single_metadata_handler),
    )
    .route(ENDPOINT_QUEUE, get(queue_status_handler))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::PowerUser, None, None, state, req, next)
      },
    ));

  let admin_session_apis = Router::new()
    .route(ENDPOINT_SETTINGS, get(list_settings_handler))
    .route(
      &format!("{ENDPOINT_SETTINGS}/{{key}}"),
      put(update_setting_handler),
    )
    .route(
      &format!("{ENDPOINT_SETTINGS}/{{key}}"),
      delete(delete_setting_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::Admin, None, None, state, req, next)
      },
    ));

  // Manager/Admin access request APIs (session-only)
  let manager_session_apis = Router::new()
    .route(
      ENDPOINT_ACCESS_REQUESTS_PENDING,
      get(list_pending_requests_handler),
    )
    .route(ENDPOINT_ACCESS_REQUESTS_ALL, get(list_all_requests_handler))
    .route(
      &format!("{ENDPOINT_ACCESS_REQUESTS_ALL}/{{id}}/approve"),
      post(approve_request_handler),
    )
    .route(
      &format!("{ENDPOINT_ACCESS_REQUESTS_ALL}/{{id}}/reject"),
      post(reject_request_handler),
    )
    .route(ENDPOINT_USERS, get(list_users_handler))
    .route(
      &format!("{ENDPOINT_USERS}/{{user_id}}/role"),
      put(change_user_role_handler),
    )
    .route(
      &format!("{ENDPOINT_USERS}/{{user_id}}"),
      delete(remove_user_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::Manager, None, None, state, req, next)
      },
    ));

  // Combine all protected APIs
  let protected_apis = Router::new()
    .merge(user_apis)
    .merge(power_user_apis)
    .merge(power_user_session_apis)
    .merge(admin_session_apis)
    .merge(manager_session_apis)
    .route_layer(from_fn_with_state(state.clone(), auth_middleware));

  // Reduce verbose middleware logging - only log errors and warnings for better signal-to-noise ratio
  let info_trace = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
    .on_response(DefaultOnResponse::new().level(Level::DEBUG))
    .on_failure(DefaultOnFailure::new().level(Level::ERROR));

  let mut openapi = BodhiOpenAPIDoc::openapi();
  OpenAPIEnvModifier::new(app_service.setting_service()).modify(&mut openapi);
  GlobalErrorResponses.modify(&mut openapi);

  // Build final router
  let router = Router::<Arc<dyn RouterState>>::new()
    .merge(public_apis)
    .merge(optional_auth)
    .merge(protected_apis)
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_private_network(true)
        .allow_credentials(false),
    )
    .with_state(state);

  let router = apply_ui_router(
    &app_service.setting_service(),
    router,
    static_router,
    proxy_router("http://localhost:3000".to_string()),
  );
  router
    .layer(app_service.session_service().session_layer())
    .layer(from_fn_with_state(
      app_service.setting_service(),
      canonical_url_middleware,
    ))
    .layer(info_trace)
}

fn apply_ui_router(
  setting_service: &Arc<dyn SettingService>,
  router: Router,
  static_router: Option<Router>,
  proxy_router: Router,
) -> Router {
  let proxy_ui = setting_service
    .get_dev_env(BODHI_DEV_PROXY_UI)
    .map(|val| val.parse::<bool>().unwrap_or_default())
    .unwrap_or_default();

  match setting_service.is_production() {
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
  use objs::EnvType;
  use rstest::{fixture, rstest};
  use server_core::test_utils::ResponseTestExt;
  use services::{
    test_utils::SettingServiceStub, SettingService, BODHI_DEV_PROXY_UI, BODHI_ENV_TYPE,
  };
  use std::{collections::HashMap, sync::Arc};
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

  fn test_setting_service(config: EnvConfig) -> Arc<dyn SettingService> {
    let env_type = if config.is_production {
      EnvType::Production
    } else {
      EnvType::Development
    };
    let mut envs = HashMap::new();
    if let Some(proxy_ui) = config.proxy_ui {
      envs.insert(BODHI_DEV_PROXY_UI.to_string(), proxy_ui);
    }
    let setting_service = SettingServiceStub::with_envs_settings(
      envs,
      HashMap::from([(BODHI_ENV_TYPE.to_string(), env_type.to_string())]),
    );
    Arc::new(setting_service)
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
    let setting_service = test_setting_service(config);
    let router = apply_ui_router(
      &setting_service,
      base_router(),
      static_router,
      proxy_router(),
    );

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
