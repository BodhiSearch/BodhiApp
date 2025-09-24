# PACKAGE.md - routes_all

This document provides detailed technical information for the `routes_all` crate, focusing on BodhiApp's HTTP route composition architecture, sophisticated middleware orchestration, and comprehensive authentication integration patterns.

## Route Composition Architecture

The `routes_all` crate serves as BodhiApp's **HTTP route composition and middleware orchestration layer**, implementing advanced route unification, multi-layered authentication, and comprehensive API documentation with dynamic UI serving capabilities.

### Router Composition Implementation
Sophisticated route integration with hierarchical authorization and middleware orchestration:

```rust
// Pattern structure (see src/routes.rs for complete implementation)
pub fn build_routes(
  ctx: Arc<dyn SharedContext>,
  app_service: Arc<dyn AppService>,
  static_router: Option<Router>,
) -> Router {
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(ctx, app_service.clone()));

  // Public APIs (no auth required)
  let mut public_apis = Router::new()
    .route(ENDPOINT_PING, get(ping_handler))
    .route(ENDPOINT_HEALTH, get(health_handler))
    .route(ENDPOINT_APP_INFO, get(app_info_handler))
    .route(ENDPOINT_APP_SETUP, post(setup_handler))
    .route(ENDPOINT_LOGOUT, post(logout_handler));

  // Optional authentication layer with session injection
  let optional_auth = Router::new()
    .route(ENDPOINT_USER_INFO, get(user_info_handler))
    .route(ENDPOINT_AUTH_INITIATE, post(auth_initiate_handler))
    .route(ENDPOINT_AUTH_CALLBACK, post(auth_callback_handler))
    .route(ENDPOINT_APPS_REQUEST_ACCESS, post(request_access_handler))
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(user_request_access_handler))
    .route(ENDPOINT_USER_REQUEST_STATUS, get(request_status_handler))
    .route_layer(from_fn_with_state(state.clone(), inject_optional_auth_info));

  // User level APIs with role and scope authorization
  let user_apis = Router::new()
    .route(ENDPOINT_OAI_MODELS, get(oai_models_handler))
    .route(ENDPOINT_OAI_CHAT_COMPLETIONS, post(chat_completions_handler))
    .route(ENDPOINT_OLLAMA_TAGS, get(ollama_models_handler))
    .route(ENDPOINT_OLLAMA_CHAT, post(ollama_model_chat_handler))
    .route(ENDPOINT_MODELS, get(list_local_aliases_handler))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(
          Role::User,
          Some(TokenScope::User),
          Some(UserScope::User),
          state,
          req,
          next,
        )
      },
    ));
}
```

**Key Route Composition Features**:
- Hierarchical route organization with public, optional auth, user, power user, and admin layers
- Role-based authorization with User/PowerUser/Admin hierarchy enforcement
- Scope-based authorization supporting both TokenScope and UserScope validation
- Middleware composition with proper ordering for authentication and authorization flow

### Multi-Layer Authentication Implementation
Advanced authentication architecture supporting multiple authentication flows:

```rust
// Hierarchical authorization pattern (see src/routes.rs for complete implementation)
let power_user_apis = Router::new()
  .route(ENDPOINT_MODELS, post(create_alias_handler))
  .route(&format!("{ENDPOINT_MODELS}/{{id}}"), put(update_alias_handler))
  .route(ENDPOINT_MODEL_PULL, get(list_downloads_handler))
  .route(ENDPOINT_MODEL_PULL, post(create_pull_request_handler))
  .route_layer(from_fn_with_state(
    state.clone(),
    move |state, req, next| {
      api_auth_middleware(
        Role::PowerUser,
        Some(TokenScope::PowerUser),
        Some(UserScope::PowerUser),
        state,
        req,
        next,
      )
    },
  ));

// Session-only APIs for token management
let power_user_session_apis = Router::new()
  .route(ENDPOINT_TOKENS, post(create_token_handler))
  .route(ENDPOINT_TOKENS, get(list_tokens_handler))
  .route(&format!("{ENDPOINT_TOKENS}/{{token_id}}"), put(update_token_handler))
  .route_layer(from_fn_with_state(
    state.clone(),
    move |state, req, next| api_auth_middleware(Role::PowerUser, None, None, state, req, next),
  ));
```

**Authentication Integration Features**:
- Bearer token authentication for OpenAI/Ollama API endpoints with comprehensive role and scope validation
- Session-based authentication for application management endpoints with secure cookie configuration
- Dual authentication support with proper precedence rules and error handling
- API token management with database-backed validation and status tracking

## UI Serving Architecture Implementation

### Dynamic UI Serving with Environment Configuration
Sophisticated UI serving coordination with development and production modes:

```rust
// UI serving pattern (see src/routes.rs for complete implementation)
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
```

**UI Serving Features**:
- Production mode with embedded static assets for optimized performance
- Development proxy mode for hot reload development workflows with localhost:3000 integration
- Development static mode for testing production builds in development environment
- Environment-specific configuration with graceful fallback handling

### Proxy Router Implementation
HTTP proxy functionality for development workflows:

```rust
// Proxy implementation pattern (see src/routes_proxy.rs for complete implementation)
pub fn proxy_router(backend_url: String) -> Router {
  Router::new().fallback(move |req| proxy_handler(req, backend_url.clone()))
}

async fn proxy_handler(mut req: Request, backend_url: String) -> Response<Body> {
  let client = HttpClient::builder(TokioExecutor::new()).build_http();
  let uri = format!(
    "{backend_url}{}",
    req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("")
  )
  .parse::<Uri>()
  .unwrap();

  *req.uri_mut() = uri;

  match client.request(req).await {
    Ok(res) => res.map(Body::new),
    Err(e) => {
      error!(?e, "error proxying request");
      Response::builder()
        .status(500)
        .body(Body::from("Internal Server Error"))
        .unwrap()
    }
  }
}
```

**Proxy Architecture Features**:
- Fallback proxy handler for unmatched routes with comprehensive error handling
- HTTP client integration with TokioExecutor for async request processing
- URI rewriting and request forwarding with proper error recovery
- Development workflow support for frontend hot reload and testing

## Middleware Orchestration Implementation

### Comprehensive Middleware Stack
Advanced middleware composition with proper ordering and configuration:

```rust
// Middleware stack pattern (see src/routes.rs for complete implementation)
let protected_apis = Router::new()
  .merge(user_apis)
  .merge(power_user_apis)
  .merge(power_user_session_apis)
  .merge(admin_session_apis)
  .route_layer(from_fn_with_state(state.clone(), auth_middleware));

// Tracing configuration with appropriate log levels
let info_trace = TraceLayer::new_for_http()
  .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
  .on_response(DefaultOnResponse::new().level(Level::DEBUG))
  .on_failure(DefaultOnFailure::new().level(Level::ERROR));

// Final router composition with CORS and session management
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
      .allow_credentials(false),
  )
  .with_state(state);

let router = apply_ui_router(&app_service.setting_service(), router, static_router, proxy_router);
router
  .layer(app_service.session_service().session_layer())
  .layer(from_fn_with_state(app_service.setting_service(), canonical_url_middleware))
  .layer(info_trace)
```

**Middleware Stack Features**:
- Layered middleware architecture with proper ordering for authentication and authorization flow
- CORS configuration with comprehensive header and method support for web client integration
- Session management with Tower Sessions and secure cookie configuration
- Tracing middleware with appropriate log levels for production performance optimization
- Canonical URL middleware for SEO optimization and security benefits

## OpenAPI Documentation Integration

### Unified API Documentation
Comprehensive OpenAPI specification generation with environment-specific configuration:

```rust
// OpenAPI integration pattern (see src/routes.rs)
let mut openapi = BodhiOpenAPIDoc::openapi();
OpenAPIEnvModifier::new(app_service.setting_service()).modify(&mut openapi);

let router = Router::<Arc<dyn RouterState>>::new()
  .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
```

**OpenAPI Integration Features**:
- Combined documentation from routes_oai and routes_app with unified specification
- Environment-specific configuration with dynamic server URL and security scheme adaptation
- Interactive Swagger UI interface with authentication flow documentation and endpoint testing
- Comprehensive schema definitions with validation and examples for all request/response types

### Localization Resources
Embedded localization files for multi-language support:

```rust
// Localization pattern (see src/lib.rs)
pub mod l10n {
  use include_dir::Dir;
  
  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
```

**Localization Features**:
- Embedded localization resources using include_dir macro for compile-time resource inclusion
- Support for multiple languages with fluent localization files (en-US/messages.ftl)
- Runtime localization access for error messages and user-facing text across all route handlers
- Integration with error handling system for localized error responses

### UI Endpoint Macro
Utility macro for consistent UI endpoint generation:

```rust
// Macro definition (see src/routes.rs)
#[macro_export]
macro_rules! make_ui_endpoint {
  ($name:ident, $path:expr) => {
    pub const $name: &str = concat!("/api/ui/", $path);
  };
}
```

**Macro Features**:
- Consistent UI endpoint generation with /api/ui/ prefix for all UI-specific endpoints
- Compile-time string concatenation for zero runtime overhead
- Type-safe endpoint constants preventing typos in route definitions
- Integration with route definition patterns for consistent API structure

## Cross-Crate Integration Implementation

### Route Layer Integration Architecture
Route composition coordinates extensively with BodhiApp's HTTP layer:

```rust
// RouterState integration for consistent service access (see src/routes.rs for complete implementation)
let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(ctx, app_service.clone()));

// All routes use consistent state management for service access
let user_apis = Router::new()
  .route(ENDPOINT_OAI_MODELS, get(oai_models_handler))
  .route(ENDPOINT_OAI_CHAT_COMPLETIONS, post(chat_completions_handler))
  .route(ENDPOINT_MODELS, get(list_local_aliases_handler))
  .with_state(state.clone());
```

**Cross-Crate Integration Features**:
- RouterState dependency injection provides consistent AppService and SharedContext access
- **routes_oai Integration**: OpenAI/Ollama API endpoints (chat completions, models, ollama tags/chat/show) with bearer token authentication
- **routes_app Integration**: Application management endpoints (aliases, downloads, API models, tokens, settings, users, access requests) with session authentication  
- **auth_middleware Coordination**: Multi-layer authentication with api_auth_middleware and auth_middleware for different authorization requirements
- **server_core Integration**: DefaultRouterState providing unified AppService and SharedContext access across all route handlers

### Service Layer HTTP Coordination
Route composition coordinates with BodhiApp's service layer through RouterState:

- **AppService Registry Access**: All route handlers access business services through RouterState dependency injection
- **Authentication Service Integration**: OAuth2 flows, session management, and API token validation coordinated across route boundaries
- **Model Management Coordination**: DataService and HubService integration for model operations across OpenAI and application endpoints
- **Configuration Management**: SettingService integration for environment-specific routing and UI serving configuration

## Route Composition Testing Architecture

### Multi-Route Integration Testing
Route composition requires comprehensive testing across route boundaries:

```rust
// UI serving testing pattern (see src/routes.rs for complete test implementation)
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
#[tokio::test]
async fn test_ui_router_scenarios(
  #[case] config: EnvConfig,
  #[case] static_router: Option<Router>,
  #[case] test_paths: Vec<(&str, bool)>,
) {
  let setting_service = test_setting_service(config);
  let router = apply_ui_router(&setting_service, base_router(), static_router, proxy_router());

  for (path, should_exist) in test_paths {
    let (status, body) = test_request(router.clone(), path).await;
    if should_exist {
      assert_eq!(status, StatusCode::OK, "Path {} should exist", path);
      assert_eq!(body, path);
    } else {
      assert_eq!(status, StatusCode::NOT_FOUND, "Path {} should not exist", path);
    }
  }
}
```

### Proxy Router Testing
Comprehensive proxy functionality testing:

```rust
// Proxy testing pattern (see src/routes_proxy.rs for complete test implementation)
#[rstest]
#[awt]
#[tokio::test]
async fn test_proxy_handler(#[future] backend_server: (SocketAddr, Sender<()>)) -> anyhow::Result<()> {
  let (socket_addr, shutdown_tx) = backend_server;
  let app = Router::new()
    .route("/test", get(|| async { "Test response" }))
    .merge(proxy_router(format!("http://{socket_addr}")));

  // Test direct route handling
  let res = app.clone().oneshot(
    Request::builder().uri("http://example.com/test").body(Body::empty()).unwrap()
  ).await.unwrap();
  assert_eq!(StatusCode::OK, res.status());
  assert_eq!("Test response", res.text().await.unwrap());

  // Test proxy handling
  let res = app.clone().oneshot(
    Request::builder().uri("http://example.com/proxy-handled").body(Body::empty()).unwrap()
  ).await.unwrap();
  assert_eq!(StatusCode::OK, res.status());
  assert_eq!("Proxied response", res.text().await.unwrap());

  shutdown_tx.send(()).unwrap();
  Ok(())
}
```

**Testing Architecture Features**:
- Environment-specific UI serving testing with different configuration scenarios
- Proxy functionality testing with backend server coordination and graceful shutdown
- Route composition testing with comprehensive middleware and authentication validation
- Integration testing across route boundaries with realistic service interactions

## Extension Guidelines for Route Composition

### Adding New Route Groups
When integrating new route implementations:

1. **Route Integration Design**: Import route handlers and endpoints from new route crates with proper dependency management
2. **Authentication Layer Selection**: Choose appropriate authentication middleware (api_auth_middleware vs auth_middleware) based on endpoint requirements
3. **Authorization Configuration**: Configure role and scope requirements using Role and TokenScope/UserScope enums from objs crate
4. **Middleware Ordering**: Apply middleware in correct order (authentication → authorization → route-specific) for proper request processing
5. **OpenAPI Integration**: Merge OpenAPI documentation from new route crates with environment-specific configuration

### Extending Authentication Patterns
For new authentication and authorization requirements:

1. **Role Hierarchy Integration**: Ensure new authorization logic follows established role hierarchy with has_access_to() validation
2. **Scope Extension**: Extend TokenScope and UserScope enums in objs crate for new authorization contexts
3. **Middleware Composition**: Layer authentication middleware appropriately for different endpoint security requirements
4. **Session Integration**: Coordinate with Tower Sessions for new session-based authentication patterns
5. **Error Handling**: Provide consistent authentication error responses across all route boundaries

### UI Serving Extensions
For new UI serving capabilities and development workflows:

1. **Environment Configuration**: Extend SettingService with new BODHI_DEV_* environment variables for additional UI serving modes
2. **Static Asset Integration**: Configure new static_router patterns for different UI frameworks with proper asset bundling
3. **Proxy Configuration**: Extend proxy_router to support different backend URLs and request transformation patterns
4. **Fallback Handling**: Implement new fallback strategies in apply_ui_router for different deployment scenarios
5. **Performance Optimization**: Add caching layers, asset compression, and CDN integration for production deployments

## Commands

**Testing**: `cargo test -p routes_all` (includes route composition and UI serving tests)  
**Building**: Standard `cargo build -p routes_all`  
**Integration Testing**: `cargo test -p routes_all --features test-utils` (includes comprehensive route testing infrastructure)