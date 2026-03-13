use crate::middleware::{
  access_request_auth_middleware, api_auth_middleware, auth_middleware, canonical_url_middleware,
  optional_auth_middleware, AccessRequestValidator, McpAccessRequestValidator,
  ToolsetAccessRequestValidator,
};
use crate::proxy_router;
use crate::{
  api_models_create, api_models_destroy, api_models_fetch_models, api_models_formats,
  api_models_index, api_models_show, api_models_sync, api_models_test, api_models_update,
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
  ENDPOINT_ACCESS_REQUESTS_PENDING, ENDPOINT_ACCESS_REQUESTS_REVIEW, ENDPOINT_API_MODELS,
  ENDPOINT_API_MODELS_API_FORMATS, ENDPOINT_API_MODELS_FETCH_MODELS, ENDPOINT_API_MODELS_TEST,
  ENDPOINT_APPS_ACCESS_REQUESTS_ID, ENDPOINT_APPS_REQUEST_ACCESS, ENDPOINT_APP_INFO,
  ENDPOINT_APP_SETUP, ENDPOINT_AUTH_CALLBACK, ENDPOINT_AUTH_INITIATE,
  ENDPOINT_DASHBOARD_AUTH_CALLBACK, ENDPOINT_DASHBOARD_AUTH_INITIATE, ENDPOINT_DEV_CLIENTS_DAG,
  ENDPOINT_DEV_DB_RESET, ENDPOINT_DEV_ENVS, ENDPOINT_DEV_SECRETS, ENDPOINT_DEV_TENANTS_CLEANUP,
  ENDPOINT_HEALTH, ENDPOINT_LOGOUT, ENDPOINT_MODELS, ENDPOINT_MODELS_REFRESH, ENDPOINT_MODEL_FILES,
  ENDPOINT_MODEL_PULL, ENDPOINT_PING, ENDPOINT_QUEUE, ENDPOINT_SETTINGS, ENDPOINT_TENANTS,
  ENDPOINT_TOKENS, ENDPOINT_TOOLSETS, ENDPOINT_TOOLSET_TYPES, ENDPOINT_USERS, ENDPOINT_USER_INFO,
  ENDPOINT_USER_REQUEST_ACCESS, ENDPOINT_USER_REQUEST_STATUS,
};
use crate::{
  chat_completions_handler, embeddings_handler, oai_model_handler, oai_models_handler,
  ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS, ENDPOINT_OAI_MODELS,
};
use crate::{
  mcp_auth_configs_create, mcp_auth_configs_destroy, mcp_auth_configs_index, mcp_auth_configs_show,
  mcp_oauth_discover_as, mcp_oauth_discover_mcp, mcp_oauth_dynamic_register, mcp_oauth_login,
  mcp_oauth_token_exchange, mcp_oauth_tokens_destroy, mcp_oauth_tokens_show, mcp_servers_create,
  mcp_servers_index, mcp_servers_show, mcp_servers_update, mcps_create, mcps_destroy,
  mcps_execute_tool, mcps_fetch_tools, mcps_index, mcps_refresh_tools, mcps_show, mcps_update,
  settings_destroy, settings_index, settings_update, setup_create, setup_show,
  toolset_types_disable, toolset_types_enable, toolset_types_index, toolsets_create,
  toolsets_destroy, toolsets_execute, toolsets_index, toolsets_show, toolsets_update,
  ENDPOINT_MCPS, ENDPOINT_MCPS_AUTH_CONFIGS, ENDPOINT_MCPS_FETCH_TOOLS,
  ENDPOINT_MCPS_OAUTH_DISCOVER_AS, ENDPOINT_MCPS_OAUTH_DISCOVER_MCP,
  ENDPOINT_MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE, ENDPOINT_MCP_SERVERS,
};
use crate::{
  ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler,
  ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS,
};
use axum::{
  middleware::from_fn_with_state,
  routing::{delete, get, post, put},
  Router,
};
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
  static_router: Option<Router>,
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
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
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
    .route(ENDPOINT_MODEL_FILES, get(modelfiles_index))
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

  // Overlapping session APIs — session-only auth but permissive CORS
  // These paths have both session methods and non-session methods on the same URL,
  // so they can't be in the restrictive CORS group (would conflict with Axum merge).
  // See TECHDEBT.md "Overlapping CORS path structure" for details.
  let overlapping_session_apis = Router::new()
    .route(ENDPOINT_TOOLSETS, post(toolsets_create))
    .route(ENDPOINT_MCPS, post(mcps_create))
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}"),
      put(mcps_update).delete(mcps_destroy),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| api_auth_middleware(ResourceRole::User, None, None, state, req, next),
    ));

  // Toolset instance CRUD APIs (session-only, no OAuth or API tokens)
  let user_session_apis = Router::new()
    // Toolset types listing
    .route(ENDPOINT_TOOLSET_TYPES, get(toolset_types_index))
    .route(&format!("{ENDPOINT_TOOLSETS}/{{id}}"), get(toolsets_show))
    .route(&format!("{ENDPOINT_TOOLSETS}/{{id}}"), put(toolsets_update))
    .route(
      &format!("{ENDPOINT_TOOLSETS}/{{id}}"),
      delete(toolsets_destroy),
    )
    // MCP CRUD (session-only)
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
    .route(ENDPOINT_API_MODELS, get(api_models_index))
    .route(ENDPOINT_API_MODELS, post(api_models_create))
    .route(ENDPOINT_API_MODELS_API_FORMATS, get(api_models_formats))
    .route(ENDPOINT_API_MODELS_TEST, post(api_models_test))
    .route(
      ENDPOINT_API_MODELS_FETCH_MODELS,
      post(api_models_fetch_models),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}"),
      get(api_models_show),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}"),
      put(api_models_update),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}"),
      delete(api_models_destroy),
    )
    .route(
      &format!("{ENDPOINT_API_MODELS}/{{id}}/sync-models"),
      post(api_models_sync),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| api_auth_middleware(ResourceRole::User, None, None, state, req, next),
    ));

  // Toolset and MCP list API (session and OAuth, no API tokens)
  let user_oauth_apis = Router::new()
    .route(ENDPOINT_TOOLSETS, get(toolsets_index))
    .route(ENDPOINT_MCPS, get(mcps_index))
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

  // Toolset execute API with access request middleware - session and OAuth tokens, NOT API tokens
  let toolset_validator: Arc<dyn AccessRequestValidator> = Arc::new(ToolsetAccessRequestValidator);
  let toolset_exec_apis = Router::new()
    .route(
      &format!("{ENDPOINT_TOOLSETS}/{{id}}/tools/{{tool_name}}/execute"),
      post(toolsets_execute),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        let v = toolset_validator.clone();
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

  // MCP exec APIs with access request middleware - session and OAuth tokens, NOT API tokens
  let mcp_validator: Arc<dyn AccessRequestValidator> = Arc::new(McpAccessRequestValidator);
  let mcp_exec_apis = Router::new()
    .route(&format!("{ENDPOINT_MCPS}/{{id}}"), get(mcps_show))
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}/tools/refresh"),
      post(mcps_refresh_tools),
    )
    .route(
      &format!("{ENDPOINT_MCPS}/{{id}}/tools/{{tool_name}}/execute"),
      post(mcps_execute_tool),
    )
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        let v = mcp_validator.clone();
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

  // Power user APIs (role=power_user or scope=scope_token_power_user)
  let power_user_apis = Router::new()
    .route(ENDPOINT_MODELS, post(models_create))
    .route(
      &format!("{ENDPOINT_MODELS}/{{id}}"),
      put(models_update).delete(models_destroy),
    )
    .route(&format!("{ENDPOINT_MODELS}/{{id}}/copy"), post(models_copy))
    .route(ENDPOINT_MODEL_PULL, get(models_pull_index))
    .route(ENDPOINT_MODEL_PULL, post(models_pull_create))
    .route(
      &format!("{ENDPOINT_MODEL_PULL}/{{id}}"),
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
    // Toolset type enable/disable (admin only)
    .route(
      "/bodhi/v1/toolset_types/{toolset_type}/app-config",
      put(toolset_types_enable),
    )
    .route(
      "/bodhi/v1/toolset_types/{toolset_type}/app-config",
      delete(toolset_types_disable),
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
    .merge(user_session_apis)
    .merge(power_user_session_apis)
    .merge(admin_session_apis)
    .merge(manager_session_apis)
    .route_layer(from_fn_with_state(state.clone(), auth_middleware))
    .layer(restrictive_cors());

  // API-protected routes (PERMISSIVE CORS — external tools/apps need access)
  let api_protected = Router::new()
    .merge(user_apis)
    .merge(user_oauth_apis)
    .merge(overlapping_session_apis)
    .merge(toolset_exec_apis)
    .merge(mcp_exec_apis)
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

  let router = apply_ui_router(
    &app_service.setting_service(),
    router,
    static_router,
    proxy_router("http://localhost:3000".to_string()),
  )
  .await;
  router
    .layer(app_service.session_service().session_layer())
    .layer(from_fn_with_state(
      app_service.setting_service(),
      canonical_url_middleware,
    ))
    .layer(info_trace)
}

async fn apply_ui_router(
  setting_service: &Arc<dyn SettingService>,
  router: Router,
  static_router: Option<Router>,
  proxy_router: Router,
) -> Router {
  let proxy_ui = setting_service
    .get_dev_env(BODHI_DEV_PROXY_UI)
    .await
    .map(|val| val.parse::<bool>().unwrap_or_default())
    .unwrap_or_default();

  match setting_service.is_production().await {
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
#[path = "test_routes.rs"]
mod test_routes;

#[cfg(test)]
#[path = "test_cors.rs"]
mod test_cors;
