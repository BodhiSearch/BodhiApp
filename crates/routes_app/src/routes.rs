use crate::middleware::{
  access_request_auth_middleware, api_auth_middleware, auth_middleware, canonical_url_middleware,
  optional_auth_middleware, AccessRequestValidator, McpAccessRequestValidator,
};
use crate::{
  api_models_create, api_models_destroy, api_models_fetch_models, api_models_formats,
  api_models_show, api_models_sync, api_models_test, api_models_update,
  apps_approve_access_request, apps_create_access_request, apps_deny_access_request,
  apps_get_access_request_review, apps_get_access_request_status, auth_callback, auth_initiate,
  auth_logout, dashboard_auth_callback, dashboard_auth_initiate, dev_clients_dag_handler,
  dev_db_reset_handler, dev_secrets_handler, dev_tenants_cleanup_handler, envs_handler,
  health_handler, modelfiles_index, models_copy, models_create, models_destroy, models_index,
  models_pull_create, models_pull_index, models_pull_show, models_show, models_update,
  ping_handler, queue_status_handler, refresh_metadata_handler, tenants_activate, tenants_create,
  tenants_index, tokens_create, tokens_index, tokens_update, users_access_request_approve,
  users_access_request_reject, users_access_requests_index, users_access_requests_pending,
  users_change_role, users_destroy, users_index, users_info, users_request_access,
  users_request_status, BodhiOpenAPIDoc, GlobalErrorResponses, OpenAPIEnvModifier,
  ENDPOINT_ACCESS_REQUESTS_ALL, ENDPOINT_ACCESS_REQUESTS_APPROVE, ENDPOINT_ACCESS_REQUESTS_DENY,
  ENDPOINT_ACCESS_REQUESTS_PENDING, ENDPOINT_ACCESS_REQUESTS_REVIEW,
  ENDPOINT_APPS_ACCESS_REQUESTS_ID, ENDPOINT_APPS_REQUEST_ACCESS, ENDPOINT_APP_INFO,
  ENDPOINT_APP_SETUP, ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE,
  ENDPOINT_DASHBOARD_AUTH_CALLBACK, ENDPOINT_DASHBOARD_AUTH_INITIATE, ENDPOINT_DEV_CLIENTS_DAG,
  ENDPOINT_DEV_DB_RESET, ENDPOINT_DEV_ENVS, ENDPOINT_DEV_SECRETS, ENDPOINT_DEV_TENANTS_CLEANUP,
  ENDPOINT_HEALTH, ENDPOINT_LOGOUT, ENDPOINT_MODELS, ENDPOINT_MODELS_ALIAS, ENDPOINT_MODELS_API,
  ENDPOINT_MODELS_API_FETCH_MODELS, ENDPOINT_MODELS_API_FORMATS, ENDPOINT_MODELS_API_TEST,
  ENDPOINT_MODELS_FILES, ENDPOINT_MODELS_FILES_PULL, ENDPOINT_MODELS_REFRESH, ENDPOINT_PING,
  ENDPOINT_QUEUE, ENDPOINT_SETTINGS, ENDPOINT_TENANTS, ENDPOINT_TOKENS, ENDPOINT_USERS,
  ENDPOINT_USER_INFO, ENDPOINT_USER_REQUEST_ACCESS, ENDPOINT_USER_REQUEST_STATUS,
};
use crate::{
  apps_mcps_execute_tool, apps_mcps_index, apps_mcps_refresh_tools, apps_mcps_show,
  mcp_auth_configs_create, mcp_auth_configs_destroy, mcp_auth_configs_index, mcp_auth_configs_show,
  mcp_oauth_discover_as, mcp_oauth_discover_mcp, mcp_oauth_dynamic_register, mcp_oauth_login,
  mcp_oauth_token_exchange, mcp_oauth_tokens_destroy, mcp_oauth_tokens_show, mcp_proxy_handler,
  mcp_servers_create, mcp_servers_index, mcp_servers_show, mcp_servers_update, mcps_create,
  mcps_destroy, mcps_execute_tool, mcps_fetch_tools, mcps_index, mcps_refresh_tools, mcps_show,
  mcps_update, settings_destroy, settings_index, settings_update, setup_create, setup_show,
  ENDPOINT_APPS_MCPS, ENDPOINT_MCPS, ENDPOINT_MCPS_AUTH_CONFIGS, ENDPOINT_MCPS_FETCH_TOOLS,
  ENDPOINT_MCPS_OAUTH_DISCOVER_AS, ENDPOINT_MCPS_OAUTH_DISCOVER_MCP,
  ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE, ENDPOINT_MCP_SERVERS,
};
use crate::{build_ui_proxy_router, build_ui_spa_router};
use crate::{
  chat_completions_handler, embeddings_handler, oai_model_handler, oai_models_handler,
  ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
};
use crate::{
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};
use axum::{
  middleware::from_fn_with_state,
  response::Redirect,
  routing::{any, delete, get, post, put},
  Router,
};
use include_dir::Dir;
use services::{AppService, SettingService, BODHI_DEV_PROXY_UI};
use services::{ResourceRole, TokenScope, UserScope};
use std::sync::Arc;
use tower_http::{
  cors::{Any, CorsLayer},
  trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, info, Level};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

fn permissive_cors() -> CorsLayer {
  CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
    .expose_headers([
      "mcp-session-id".parse().unwrap(),
      "mcp-protocol-version".parse().unwrap(),
    ])
    .allow_private_network(true)
    .allow_credentials(false)
}

fn restrictive_cors() -> CorsLayer {
  // Defaults: no origins, no methods, no headers, no private network
  // OPTIONS gets 200 but no Access-Control-Allow-Origin → browser blocks
  CorsLayer::new()
}

pub async fn build_routes(
  app_service: Arc<dyn AppService>,
  static_dir: Option<&'static Dir<'static>>,
) -> Router {
  let state = app_service.clone();

  // Public APIs (no auth required)
  let public_apis = Router::new()
    .route(ENDPOINT_PING, get(ping_handler))
    .route(ENDPOINT_HEALTH, get(health_handler))
    .route(ENDPOINT_APP_SETUP, post(setup_create))
    // TODO: having as api/ui/logout coz of status code as 200 instead of 302 because of automatic follow redirect by axios
    .route(ENDPOINT_LOGOUT, post(auth_logout))
    // App access request endpoints (unauthenticated)
    .route(
      ENDPOINT_APPS_REQUEST_ACCESS,
      post(apps_create_access_request),
    )
    .route(
      ENDPOINT_APPS_ACCESS_REQUESTS_ID,
      get(apps_get_access_request_status),
    );

  let mut optional_auth = Router::new()
    .route(ENDPOINT_APP_INFO, get(setup_show))
    .route(ENDPOINT_USER_INFO, get(users_info))
    .route(ENDPOINT_AUTH_INITIATE, post(auth_initiate))
    .route(ENDPOINT_AUTH_CALLBACK, post(auth_callback))
    .route(
      ENDPOINT_DASHBOARD_AUTH_INITIATE,
      post(dashboard_auth_initiate),
    )
    .route(
      ENDPOINT_DASHBOARD_AUTH_CALLBACK,
      post(dashboard_auth_callback),
    )
    .route(ENDPOINT_TENANTS, get(tenants_index))
    .route(ENDPOINT_TENANTS, post(tenants_create))
    .route(
      &format!("{ENDPOINT_TENANTS}/{{client_id}}/activate"),
      post(tenants_activate),
    );

  // Dev-only routes with optional auth
  if !app_service.setting_service().is_production().await {
    let dev_apis = Router::new()
      .route(ENDPOINT_DEV_SECRETS, get(dev_secrets_handler))
      .route(ENDPOINT_DEV_ENVS, get(envs_handler))
      .route(ENDPOINT_DEV_DB_RESET, post(dev_db_reset_handler))
      .route(ENDPOINT_DEV_DB_RESET, get(dev_db_reset_handler))
      .route(ENDPOINT_DEV_CLIENTS_DAG, post(dev_clients_dag_handler))
      .route(
        ENDPOINT_DEV_TENANTS_CLEANUP,
        get(dev_tenants_cleanup_handler),
      )
      .route(
        ENDPOINT_DEV_TENANTS_CLEANUP,
        delete(dev_tenants_cleanup_handler),
      );
    optional_auth = optional_auth.merge(dev_apis);
  }

  let optional_auth =
    optional_auth.route_layer(from_fn_with_state(state.clone(), optional_auth_middleware));

  // Guest endpoints (session auth required, minimum Guest role)
  let guest_endpoints = Router::new()
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::Guest, None, None, state, req, next)
      },
    ));

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
    .route(ENDPOINT_MODELS, get(models_index))
    .route(&format!("{ENDPOINT_MODELS}/{{id}}"), get(models_show))
    .route(ENDPOINT_MODELS_FILES, get(modelfiles_index))
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

  // Session-only APIs (no OAuth or API tokens)
  let user_session_apis = Router::new()
    // MCP CRUD (session-only)
    .route(ENDPOINT_MCPS, get(mcps_index))
    .route(ENDPOINT_MCPS, post(mcps_create))
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}"),
      get(mcps_show).put(mcps_update).delete(mcps_destroy),
    )
    // MCP tool discovery (session-only)
    .route(ENDPOINT_MCPS_FETCH_TOOLS, post(mcps_fetch_tools))
    // Unified auth config endpoints
    .route(ENDPOINT_MCPS_AUTH_CONFIGS, post(mcp_auth_configs_create))
    .route(ENDPOINT_MCPS_AUTH_CONFIGS, get(mcp_auth_configs_index))
    .route(
      &format!("{ENDPOINT_MCPS_AUTH_CONFIGS}/{{id}}"),
      get(mcp_auth_configs_show),
    )
    .route(
      &format!("{ENDPOINT_MCPS_AUTH_CONFIGS}/{{id}}"),
      delete(mcp_auth_configs_destroy),
    )
    // OAuth login and token exchange (nested under auth-configs)
    .route(
      &format!("{ENDPOINT_MCPS_AUTH_CONFIGS}/{{id}}/login"),
      post(mcp_oauth_login),
    )
    .route(
      &format!("{ENDPOINT_MCPS_AUTH_CONFIGS}/{{id}}/token"),
      post(mcp_oauth_token_exchange),
    )
    // OAuth token endpoints
    .route(
      "/bodhi/v1/mcps/oauth-tokens/{token_id}",
      get(mcp_oauth_tokens_show),
    )
    .route(
      "/bodhi/v1/mcps/oauth-tokens/{token_id}",
      delete(mcp_oauth_tokens_destroy),
    )
    // OAuth discovery
    .route(ENDPOINT_MCPS_OAUTH_DISCOVER_AS, post(mcp_oauth_discover_as))
    .route(
      ENDPOINT_MCPS_OAUTH_DISCOVER_MCP,
      post(mcp_oauth_discover_mcp),
    )
    // Standalone dynamic client registration (no server_id)
    .route(
      ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE,
      post(mcp_oauth_dynamic_register),
    )
    // MCP servers (read for all users)
    .route(ENDPOINT_MCP_SERVERS, get(mcp_servers_index))
    .route(
      &format!("{ENDPOINT_MCP_SERVERS}/{{id}}"),
      get(mcp_servers_show),
    )
    // App access request review/approve/deny (session-only)
    .route(
      ENDPOINT_ACCESS_REQUESTS_REVIEW,
      get(apps_get_access_request_review),
    )
    .route(
      ENDPOINT_ACCESS_REQUESTS_APPROVE,
      put(apps_approve_access_request),
    )
    .route(
      ENDPOINT_ACCESS_REQUESTS_DENY,
      post(apps_deny_access_request),
    )
    // API Models management (session-only, user role)
    .route(ENDPOINT_MODELS_API, post(api_models_create))
    .route(ENDPOINT_MODELS_API_FORMATS, get(api_models_formats))
    .route(ENDPOINT_MODELS_API_TEST, post(api_models_test))
    .route(
      ENDPOINT_MODELS_API_FETCH_MODELS,
      post(api_models_fetch_models),
    )
    .route(
      &format!("{ENDPOINT_MODELS_API}/{{id}}"),
      get(api_models_show),
    )
    .route(
      &format!("{ENDPOINT_MODELS_API}/{{id}}"),
      put(api_models_update),
    )
    .route(
      &format!("{ENDPOINT_MODELS_API}/{{id}}"),
      delete(api_models_destroy),
    )
    .route(
      &format!("{ENDPOINT_MODELS_API}/{{id}}/sync-models"),
      post(api_models_sync),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| api_auth_middleware(ResourceRole::User, None, None, state, req, next),
    ));

  // MCP exec APIs with access request middleware - session and OAuth tokens, NOT API tokens
  let mcp_validator_orig: Arc<dyn AccessRequestValidator> = Arc::new(McpAccessRequestValidator);
  let mcp_exec_apis = Router::new()
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}/tools/refresh"),
      post(mcps_refresh_tools),
    )
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}/tools/{{tool_name}}/execute"),
      post(mcps_execute_tool),
    )
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}/mcp"),
      any(mcp_proxy_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        let v = mcp_validator_orig.clone();
        access_request_auth_middleware(v, state, req, next)
      },
    ))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(
          ResourceRole::User,
          None,
          Some(UserScope::User),
          state,
          req,
          next,
        )
      },
    ));

  // External app API endpoints (under /apps/ prefix, OAuth tokens only)
  // Apps list endpoints
  let apps_list_apis = Router::new().route(ENDPOINT_APPS_MCPS, get(apps_mcps_index));

  // Apps MCP show + exec (with McpAccessRequestValidator)
  let mcp_validator: Arc<dyn AccessRequestValidator> = Arc::new(McpAccessRequestValidator);
  let apps_mcp_exec = Router::new()
    .route(&format!("{ENDPOINT_APPS_MCPS}/{{id}}"), get(apps_mcps_show))
    .route(
      &format!("{ENDPOINT_APPS_MCPS}/{{id}}/tools/refresh"),
      post(apps_mcps_refresh_tools),
    )
    .route(
      &format!("{ENDPOINT_APPS_MCPS}/{{id}}/tools/{{tool_name}}/execute"),
      post(apps_mcps_execute_tool),
    )
    .route(
      &format!("{ENDPOINT_APPS_MCPS}/{{id}}/mcp"),
      any(mcp_proxy_handler),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        let v = mcp_validator.clone();
        access_request_auth_middleware(v, state, req, next)
      },
    ));

  // Combine all apps APIs with OAuth-accepting auth
  let apps_apis = Router::new()
    .merge(apps_list_apis)
    .merge(apps_mcp_exec)
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(
          ResourceRole::User,
          None,
          Some(UserScope::User),
          state,
          req,
          next,
        )
      },
    ));

  // Power user APIs (role=power_user or scope=scope_token_power_user)
  let power_user_apis = Router::new()
    .route(ENDPOINT_MODELS_ALIAS, post(models_create))
    .route(
      &format!("{ENDPOINT_MODELS_ALIAS}/{{id}}"),
      put(models_update).delete(models_destroy),
    )
    .route(
      &format!("{ENDPOINT_MODELS_ALIAS}/{{id}}/copy"),
      post(models_copy),
    )
    .route(ENDPOINT_MODELS_FILES_PULL, get(models_pull_index))
    .route(ENDPOINT_MODELS_FILES_PULL, post(models_pull_create))
    .route(
      &format!("{ENDPOINT_MODELS_FILES_PULL}/{{id}}"),
      get(models_pull_show),
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
    .route(ENDPOINT_TOKENS, post(tokens_create))
    .route(ENDPOINT_TOKENS, get(tokens_index))
    .route(
      &format!("{ENDPOINT_TOKENS}/{{token_id}}"),
      put(tokens_update),
    )
    .route(ENDPOINT_MODELS_REFRESH, post(refresh_metadata_handler))
    .route(ENDPOINT_QUEUE, get(queue_status_handler))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::PowerUser, None, None, state, req, next)
      },
    ));

  let admin_session_apis = Router::new()
    .route(ENDPOINT_SETTINGS, get(settings_index))
    .route(
      &format!("{ENDPOINT_SETTINGS}/{{key}}"),
      put(settings_update),
    )
    .route(
      &format!("{ENDPOINT_SETTINGS}/{{key}}"),
      delete(settings_destroy),
    )
    // MCP server create/update (admin only)
    .route(ENDPOINT_MCP_SERVERS, post(mcp_servers_create))
    .route(
      &format!("{ENDPOINT_MCP_SERVERS}/{{id}}"),
      put(mcp_servers_update),
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
      get(users_access_requests_pending),
    )
    .route(
      ENDPOINT_ACCESS_REQUESTS_ALL,
      get(users_access_requests_index),
    )
    .route(
      &format!("{ENDPOINT_ACCESS_REQUESTS_ALL}/{{id}}/approve"),
      post(users_access_request_approve),
    )
    .route(
      &format!("{ENDPOINT_ACCESS_REQUESTS_ALL}/{{id}}/reject"),
      post(users_access_request_reject),
    )
    .route(ENDPOINT_USERS, get(users_index))
    .route(
      &format!("{ENDPOINT_USERS}/{{user_id}}/role"),
      put(users_change_role),
    )
    .route(
      &format!("{ENDPOINT_USERS}/{{user_id}}"),
      delete(users_destroy),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::Manager, None, None, state, req, next)
      },
    ));

  // Session-protected routes (RESTRICTIVE CORS — blocks all cross-origin)
  let session_protected = Router::new()
    .merge(guest_endpoints)
    .merge(user_session_apis)
    .merge(power_user_session_apis)
    .merge(admin_session_apis)
    .merge(manager_session_apis)
    .route_layer(from_fn_with_state(state.clone(), auth_middleware))
    .layer(restrictive_cors());

  // API-protected routes (PERMISSIVE CORS — external tools/apps need access)
  let api_protected = Router::new()
    .merge(user_apis)
    .merge(mcp_exec_apis)
    .merge(apps_apis)
    .merge(power_user_apis)
    .route_layer(from_fn_with_state(state.clone(), auth_middleware))
    .layer(permissive_cors());

  // Public + optional auth (PERMISSIVE CORS)
  let public_router = Router::new()
    .merge(public_apis)
    .merge(optional_auth)
    .layer(permissive_cors());

  // Reduce verbose middleware logging - only log errors and warnings for better signal-to-noise ratio
  let info_trace = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
    .on_response(DefaultOnResponse::new().level(Level::DEBUG))
    .on_failure(DefaultOnFailure::new().level(Level::ERROR));

  let mut openapi = BodhiOpenAPIDoc::openapi();
  OpenAPIEnvModifier::new(app_service.setting_service())
    .modify(&mut openapi)
    .await;
  GlobalErrorResponses.modify(&mut openapi);

  // Build final router — NO global CorsLayer
  let router = Router::<Arc<dyn AppService>>::new()
    .merge(public_router)
    .merge(session_protected)
    .merge(api_protected)
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
    .with_state(state);

  let router = apply_ui_router(&app_service.setting_service(), router, static_dir).await;
  let secure_cookie = app_service.setting_service().is_secure_transport().await;
  router
    .layer(app_service.session_service().session_layer(secure_cookie))
    .layer(from_fn_with_state(
      app_service.setting_service(),
      canonical_url_middleware,
    ))
    .layer(info_trace)
}

async fn apply_ui_router(
  setting_service: &Arc<dyn SettingService>,
  router: Router,
  static_dir: Option<&'static Dir<'static>>,
) -> Router {
  let is_production = setting_service.is_production().await;
  let proxy_ui = setting_service
    .get_dev_env(BODHI_DEV_PROXY_UI)
    .await
    .map(|val| val.parse::<bool>().unwrap_or_default())
    .unwrap_or_default();

  // Root redirect: / → /ui/
  let router = router.route("/", get(|| async { Redirect::temporary("/ui/") }));

  match (is_production, proxy_ui) {
    // Dev with proxy: forward to Vite dev server for HMR
    (false, true) => {
      info!("proxying the ui to localhost:3000");
      router.merge(build_ui_proxy_router("http://localhost:3000".to_string()))
    }
    // Production or dev without proxy: serve from embedded assets
    _ => {
      if let Some(dir) = static_dir {
        debug!("serving ui from embedded assets under /ui");
        router.merge(build_ui_spa_router(dir))
      } else {
        router
      }
    }
  }
}

#[cfg(test)]
#[path = "test_routes.rs"]
mod test_routes;

#[cfg(test)]
#[path = "test_cors.rs"]
mod test_cors;
