# PACKAGE.md - auth_middleware

This document provides detailed technical information for the `auth_middleware` crate, focusing on BodhiApp's HTTP authentication and authorization middleware architecture, sophisticated JWT token management, and multi-layered security implementation patterns.

## HTTP Authentication Middleware Architecture

The `auth_middleware` crate serves as BodhiApp's **HTTP security orchestration layer**, implementing advanced authentication flows, JWT token management, and comprehensive authorization controls with cross-service coordination.

### Dual Authentication Middleware Implementation
Sophisticated middleware architecture supporting multiple authentication patterns:

```rust
// Session-based authentication with same-origin validation - see crates/auth_middleware/src/auth_middleware.rs
pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);
  
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let token_service = DefaultTokenService::new(
    app_service.auth_service(),
    secret_service.clone(),
    app_service.cache_service(),
    app_service.db_service(),
    app_service.setting_service(),
  );

  let token_service = DefaultTokenService::new(/* service dependencies */);
  
  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    // Bearer token authentication takes precedence
    let (access_token, resource_scope) = token_service.validate_bearer_token(header).await?;
    req.headers_mut().insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
    req.headers_mut().insert(KEY_RESOURCE_SCOPE, resource_scope.to_string().parse().unwrap());
  } else if is_same_origin(&headers) {
    // Session-based authentication with CSRF protection
    let access_token = session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await?;
    let (validated_token, role) = token_service.get_valid_session_token(session, access_token).await?;
    req.headers_mut().insert(KEY_RESOURCE_TOKEN, validated_token.parse().unwrap());
    req.headers_mut().insert(KEY_RESOURCE_ROLE, role.to_string().parse().unwrap());
  }
  
  Ok(next.run(req).await)
}

// Optional authentication injection for non-protected routes - see crates/auth_middleware/src/auth_middleware.rs
pub async fn inject_optional_auth_info(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);
  // Similar logic but continues on authentication failure
  // Enables optional authentication for routes that can work with or without auth
  Ok(next.run(req).await)
}
```

**Key Authentication Features**:
- Bearer token authentication takes precedence over session-based authentication for API compatibility
- Same-origin validation using Sec-Fetch-Site headers prevents CSRF attacks on session-based requests
- Internal header management prevents header injection attacks while providing validated information to routes
- Dual middleware approach supports both strict authentication and optional authentication patterns

### API Authorization Middleware Implementation
Fine-grained authorization middleware with configurable role and scope requirements:

```rust
// Role and scope-based authorization - see crates/auth_middleware/src/api_auth_middleware.rs
pub async fn api_auth_middleware(
  required_role: Role,
  required_token_scope: Option<TokenScope>,
  required_user_scope: Option<UserScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  let role_header = req.headers().get(KEY_RESOURCE_ROLE);
  let scope_header = req.headers().get(KEY_RESOURCE_SCOPE);

  match (role_header, scope_header) {
    // Role header takes precedence - hierarchical authorization
    (Some(role_header), _) => {
      let user_role = role_header.to_str()?.parse::<Role>()?;
      if !user_role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }
    
    // Scope-based authorization when no role header present
    (None, Some(scope_header)) => {
      let resource_scope = ResourceScope::try_parse(scope_header.to_str()?)?;
      match resource_scope {
        ResourceScope::Token(token_scope) => {
          if let Some(required_token_scope) = required_token_scope {
            if !token_scope.has_access_to(&required_token_scope) {
              return Err(ApiAuthError::Forbidden);
            }
          }
        }
        ResourceScope::User(user_scope) => {
          if let Some(required_user_scope) = required_user_scope {
            if !user_scope.has_access_to(&required_user_scope) {
              return Err(ApiAuthError::Forbidden);
            }
          }
        }
      }
    }
    
    _ => return Err(ApiAuthError::MissingAuth),
  }

  Ok(next.run(req).await)
}
```

**Authorization Architecture Features**:
- Role-based authorization follows hierarchical ordering with has_access_to() validation from objs crate
- ResourceScope union type seamlessly handles both TokenScope and UserScope authorization contexts
- Configurable authorization requirements allow different endpoints to specify different access levels
- Authorization precedence rules ensure role-based authorization takes precedence over scope-based when both are present

## JWT Token Management Architecture

### DefaultTokenService Implementation
Comprehensive token management service coordinating multiple authentication flows:

```rust
// Core token service architecture - see crates/auth_middleware/src/token_service.rs
pub struct DefaultTokenService {
  auth_service: Arc<dyn AuthService>,
  secret_service: Arc<dyn SecretService>,
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  setting_service: Arc<dyn SettingService>,
}

impl DefaultTokenService {
  pub async fn validate_bearer_token(&self, header: &str) -> Result<(String, ResourceScope), AuthError> {
    // 1. Extract and validate bearer token format
    let bearer_token = header.strip_prefix(BEARER_PREFIX)?.trim();
    
    // 2. Check database for API token status
    if let Ok(Some(api_token)) = self.db_service.get_api_token_by_token_id(bearer_token).await {
      if api_token.status == TokenStatus::Inactive {
        return Err(AuthError::TokenInactive);
      }
      return self.handle_internal_api_token(bearer_token, &api_token).await;
    }
    
    // 3. Handle external client token with exchange
    self.handle_external_client_token(bearer_token).await
  }
}
```

### Token Validation Workflow Implementation
Sophisticated token validation with caching and external client support:

```rust
// Internal API token handling with caching - see crates/auth_middleware/src/token_service.rs
async fn handle_internal_api_token(&self, bearer_token: &str, api_token: &ApiToken) -> Result<(String, ResourceScope)> {
  // Check cache for validated access token
  let cache_key = format!("token:{}", api_token.token_id);
  if let Some(access_token) = self.cache_service.get(&cache_key) {
    // Validate cached token expiration without signature verification
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    if let Ok(token_data) = jsonwebtoken::decode::<ExpClaims>(&access_token, &DecodingKey::from_secret(&[]), &validation) {
      let scope = TokenScope::from_scope(&token_data.claims.scope)?;
      return Ok((access_token, ResourceScope::Token(scope)));
    }
  }
  
  // Cache miss or expired - perform full token exchange
  let app_reg_info = self.secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?;
  let claims = extract_claims::<OfflineClaims>(bearer_token)?;
  self.validate_token_claims(&claims, &app_reg_info.client_id)?;
  
  let (access_token, _) = self.auth_service.refresh_token(
    &app_reg_info.client_id,
    &app_reg_info.client_secret,
    bearer_token,
  ).await?;
  
  // Cache the new access token
  self.cache_service.set(&cache_key, &access_token);
  let scope_claims = extract_claims::<ScopeClaims>(&access_token)?;
  let token_scope = TokenScope::from_scope(&scope_claims.scope)?;
  Ok((access_token, ResourceScope::Token(token_scope)))
}

// External client token exchange - see crates/auth_middleware/src/token_service.rs
async fn handle_external_client_token(&self, external_token: &str) -> Result<(String, ResourceScope)> {
  // Validate external token issuer and audience
  let claims = extract_claims::<ScopeClaims>(external_token)?;
  if claims.iss != self.setting_service.auth_issuer() {
    return Err(TokenError::InvalidIssuer(claims.iss))?;
  }
  
  let app_reg_info = self.secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?;
  if claims.aud.as_ref() != Some(&app_reg_info.client_id) {
    return Err(TokenError::InvalidAudience(claims.aud.unwrap_or_default()))?;
  }
  
  // Extract user scopes and exchange token
  let scopes: Vec<&str> = claims.scope.split_whitespace()
    .filter(|s| s.starts_with("scope_user_"))
    .collect();
  let mut exchange_scopes = scopes.iter().map(|s| s.to_string()).collect::<Vec<_>>();
  exchange_scopes.extend(["openid", "email", "profile", "roles"].iter().map(|s| s.to_string()));
  
  let (access_token, _) = self.auth_service.exchange_app_token(
    &app_reg_info.client_id,
    &app_reg_info.client_secret,
    external_token,
    exchange_scopes,
  ).await?;
  
  let scope_claims = extract_claims::<ScopeClaims>(&access_token)?;
  let user_scope = UserScope::from_scope(&scope_claims.scope)?;
  Ok((access_token, ResourceScope::User(user_scope)))
}
```

**Token Management Features**:
- Multi-tier token validation with database lookup, cache checking, and service coordination
- External client token support with RFC 8693 token exchange and proper issuer/audience validation
- Performance optimization through intelligent caching with automatic expiration handling
- Comprehensive claims validation with leeway handling for clock skew and security requirements

## Session Management Integration

### Session Token Lifecycle Management
Sophisticated session-based authentication with automatic token refresh:

```rust
// Session token validation and refresh - see crates/auth_middleware/src/token_service.rs
pub async fn get_valid_session_token(&self, session: Session, access_token: String) -> Result<(String, Role)> {
  let claims = extract_claims::<Claims>(&access_token)?;
  let now = Utc::now().timestamp();
  
  // Check if current token is still valid
  if now < claims.exp as i64 {
    let app_reg_info = self.secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?;
    let roles = claims.resource_access.get(&app_reg_info.client_id).ok_or(AuthError::MissingRoles)?;
    let role = Role::from_resource_role(&roles.roles)?;
    return Ok((access_token, role));
  }
  
  // Token expired - attempt refresh
  let refresh_token = session.get::<String>("refresh_token").await?.ok_or(AuthError::RefreshTokenNotFound)?;
  let (new_access_token, new_refresh_token) = self.auth_service.refresh_token(
    &app_reg_info.client_id,
    &app_reg_info.client_secret,
    &refresh_token,
  ).await?;
  
  // Update session with new tokens
  session.insert("access_token", &new_access_token).await?;
  if let Some(refresh_token) = new_refresh_token.as_ref() {
    session.insert("refresh_token", refresh_token).await?;
  }
  
  // Extract role from new token
  let claims = extract_claims::<Claims>(&new_access_token)?;
  let resource_claims = claims.resource_access.get(&app_reg_info.client_id).ok_or(AuthError::MissingRoles)?;
  let role = Role::from_resource_role(&resource_claims.roles)?;
  Ok((new_access_token, role))
}
```

**Session Management Features**:
- Automatic token refresh with seamless session updates and error recovery
- Role extraction from JWT claims with hierarchical validation from objs crate
- Session state management with Tower Sessions integration and secure cookie handling
- Atomic session updates ensure consistency during token refresh operations

## Security Infrastructure Implementation

### Token Digest Security Architecture
Secure token storage and lookup using SHA-256 digests:

```rust
// Token digest generation for secure storage - see crates/auth_middleware/src/token_service.rs
pub fn create_token_digest(bearer_token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bearer_token.as_bytes());
  format!("{:x}", hasher.finalize())[0..12].to_string()
}

// Database token lookup using digests
let token_digest = create_token_digest(bearer_token);
if let Ok(Some(api_token)) = self.db_service.get_api_token_by_token_id(bearer_token).await {
  // Token found in database - validate status and proceed
}
```

### Same-Origin Validation Implementation
CSRF protection through security header validation:

```rust
// Same-origin request validation - see crates/auth_middleware/src/auth_middleware.rs
fn is_same_origin(headers: &HeaderMap) -> bool {
  let host = headers.get(axum::http::header::HOST).and_then(|v| v.to_str().ok());
  let sec_fetch_site = headers.get(SEC_FETCH_SITE_HEADER).and_then(|v| v.to_str().ok());
  evaluate_same_origin(host, sec_fetch_site)
}

fn evaluate_same_origin(host: Option<&str>, sec_fetch_site: Option<&str>) -> bool {
  if let Some(host) = host {
    if host.starts_with("localhost:") {
      return matches!(sec_fetch_site, Some("same-origin"));
    }
  }
  true // Allow non-localhost requests (production handles this differently)
}
```

### Canonical URL Middleware Architecture
SEO and security benefits through canonical URL redirection:

```rust
// Canonical URL redirection middleware - see crates/auth_middleware/src/canonical_url_middleware.rs
pub async fn canonical_url_middleware(
  headers: HeaderMap,
  State(setting_service): State<Arc<dyn SettingService>>,
  request: Request,
  next: Next,
) -> Response {
  // Skip redirect if canonical redirect is disabled or public_host not set
  if !setting_service.canonical_redirect_enabled() || setting_service.get_public_host_explicit().is_none() {
    return next.run(request).await;
  }
  
  // Only redirect GET and HEAD requests to avoid breaking forms and APIs
  if !matches!(request.method().as_str(), "GET" | "HEAD") {
    return next.run(request).await;
  }
  
  // Skip redirects for health check and special endpoints
  if is_exempt_path(request.uri().path()) {
    return next.run(request).await;
  }
  
  // Extract request components and check if redirect needed
  let request_scheme = extract_scheme(&headers, request.uri());
  let request_host = headers.get(header::HOST)?.to_str()?;
  
  if should_redirect_to_canonical(setting_service.as_ref(), &request_scheme, request_host) {
    let canonical_url = build_canonical_url(&setting_service.public_server_url(), request.uri());
    return (StatusCode::MOVED_PERMANENTLY, [("location", &canonical_url)]).into_response();
  }
  
  next.run(request).await
}
```

**Security Infrastructure Features**:
- Token digest-based database storage prevents token exposure in database logs and queries
- Same-origin validation provides CSRF protection for session-based authentication flows
- Canonical URL redirection improves SEO while providing consistent security boundaries
- Comprehensive security header management with automatic injection and removal patterns

## Localization Infrastructure

### Error Message Localization
Authentication and authorization errors support localization through Fluent resource files:

```rust
// Localization resource inclusion - see crates/auth_middleware/src/lib.rs
pub mod l10n {
  use include_dir::Dir;
  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}

// Error messages with localization support - see crates/auth_middleware/src/resources/en-US/messages.ftl
auth_error-invalid_access = access denied
auth_error-refresh_token_not_found = refresh token not found in session, logout and login again to continue
auth_error-tower_sessions = session is not available, please try again later, error: {$error}
auth_error-token_inactive = API token is inactive
auth_error-app_status_invalid = app status is invalid for this operation: {$var_0}
api_auth_error-forbidden = insufficient privileges to access this resource
api_auth_error-missing_auth = missing authentication header
```

**Localization Features**:
- Fluent-based localization system with parameterized error messages
- Embedded resource files using include_dir for deployment convenience
- Consistent error message formatting across authentication and authorization failures
- Support for dynamic error parameters with proper escaping and formatting

## Utility Functions and Helpers

### Authentication Utility Functions
Core utility functions supporting authentication and security operations:

```rust
// App status retrieval with fallback - see crates/auth_middleware/src/utils.rs
pub fn app_status_or_default(secret_service: &Arc<dyn SecretService>) -> AppStatus {
  secret_service.app_status().unwrap_or_default()
}

// Random string generation for security tokens - see crates/auth_middleware/src/utils.rs
pub fn generate_random_string(length: usize) -> String {
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let mut rng = rand::rng();
  (0..length).map(|_| {
    let idx = rng.random_range(0..CHARSET.len());
    CHARSET[idx] as char
  }).collect()
}

// API error response structure - see crates/auth_middleware/src/utils.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorResponse {
  error: String,
}
```

**Utility Features**:
- App status retrieval with graceful fallback to Setup status when not configured
- Cryptographically secure random string generation for tokens and security identifiers
- Consistent API error response structures for HTTP error handling
- Integration with project-wide error handling patterns and localization system

## Cross-Crate Integration Implementation

### Service Layer Authentication Coordination
Authentication middleware coordinates extensively with BodhiApp's service layer:

```rust
// Service coordination through RouterState - see crates/auth_middleware/src/auth_middleware.rs
let app_service = state.app_service();
let token_service = DefaultTokenService::new(
  app_service.auth_service(),    // OAuth2 flows and token exchange
  app_service.secret_service(),  // JWT signing keys and app registration
  app_service.cache_service(),   // Token validation caching
  app_service.db_service(),      // API token storage and status
  app_service.setting_service(), // Configuration and issuer validation
);

// Cross-service error coordination
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthError {
  #[error(transparent)]
  AuthService(#[from] AuthServiceError),
  #[error(transparent)]
  SecretService(#[from] SecretServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "auth_error-tower_sessions")]
  TowerSession(#[from] tower_sessions::session::Error),
}
```

### Domain Object Integration Patterns
Extensive coordination with objs crate for authentication and authorization:

```rust
// Role and scope integration - see crates/auth_middleware/src/api_auth_middleware.rs
use objs::{Role, ResourceScope, TokenScope, UserScope, RoleError, TokenScopeError, UserScopeError};

// Role hierarchy validation using objs domain logic
let user_role = role_header.to_str()?.parse::<Role>()?;
if !user_role.has_access_to(&required_role) {
  return Err(ApiAuthError::Forbidden);
}

// ResourceScope union type handling
let resource_scope = ResourceScope::try_parse(scope_header.to_str()?)?;
match resource_scope {
  ResourceScope::Token(token_scope) => {
    // Token-based authorization logic
  }
  ResourceScope::User(user_scope) => {
    // User-based authorization logic
  }
}
```

## Testing Infrastructure Architecture

### Authentication Testing Patterns
Comprehensive testing infrastructure for authentication flows:

```rust
// Service mock coordination for authentication testing - see crates/auth_middleware/src/auth_middleware.rs
#[rstest]
#[tokio::test]
async fn test_auth_middleware_with_expired_session_token(
  expired_token: (String, String),
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let (expired_token, _) = expired_token;
  let (exchanged_token, _) = build_token(access_token_claims())?;
  
  // Setup session with expired token
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
  let mut record = Record {
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(expired_token.clone()),
      "refresh_token".to_string() => Value::String("valid_refresh_token".to_string()),
    },
    // ... session setup
  };
  
  // Mock auth service for token refresh
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_refresh_token()
    .with(eq(TEST_CLIENT_ID), eq(TEST_CLIENT_SECRET), eq("valid_refresh_token"))
    .return_once(|_, _, _| Ok((exchanged_token.clone(), Some("new_refresh_token".to_string()))));
  
  // Test complete authentication flow with token refresh
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  
  // Verify session was updated with new tokens
  let updated_record = session_service.session_store.load(&id).await?.unwrap();
  assert_eq!(exchanged_token, updated_record.data.get("access_token").unwrap().as_str().unwrap());
}
```

### OAuth2 Test Infrastructure
The auth_middleware crate includes comprehensive OAuth2 testing infrastructure through the test_utils module:

```rust
// OAuth2 test client configuration - see crates/auth_middleware/src/test_utils/auth_server_test_client.rs
#[derive(Debug, Clone, Builder)]
pub struct AuthServerConfig {
  pub auth_server_url: String,
  pub realm: String,
  pub dev_console_client_id: String,
  pub dev_console_client_secret: String,
}

// OAuth2 integration test client - see crates/auth_middleware/src/test_utils/auth_server_test_client.rs
impl AuthServerTestClient {
  pub async fn setup_dynamic_clients(&self, username: &str, password: &str) -> Result<DynamicClients> {
    // Complete OAuth2 client setup workflow
    let dev_console_token = self.get_dev_console_user_token(username, password).await?;
    let app_client = self.create_app_client(&dev_console_token, "Test App Client").await?;
    let resource_client = self.create_resource_client("Test Resource Server").await?;
    let resource_service_token = self.get_resource_service_token(&resource_client).await?;
    
    // Setup resource admin and request access
    self.make_first_resource_admin(&resource_service_token, username).await?;
    let resource_scope_name = self.request_audience_access(&resource_service_token, &app_client.client_id).await?;
    
    Ok(DynamicClients { app_client, resource_client, resource_scope_name })
  }
  
  async fn get_dev_console_user_token(&self, username: &str, password: &str) -> Result<String>;
  async fn create_app_client(&self, token: &str, name: &str) -> Result<ClientCreateResponse>;  
  async fn create_resource_client(&self, name: &str) -> Result<ClientCreateResponse>;
  async fn get_resource_service_token(&self, resource_client: &ClientCreateResponse) -> Result<String>;
}
```

**Testing Architecture Features**:
- Comprehensive service mock coordination for isolated authentication testing scenarios
- OAuth2 integration test client supporting complete dynamic client setup workflows
- Session management testing with SQLite backend and realistic session lifecycle scenarios
- Token validation testing with expired tokens, refresh flows, and external client scenarios

## Extension Guidelines for Authentication Infrastructure

### Adding New Authentication Middleware
When creating new authentication middleware for specific use cases:

1. **Service Dependency Design**: Use RouterState dependency injection for consistent AppService registry access
2. **Error Handling Architecture**: Create middleware-specific errors implementing AppError trait for HTTP response consistency
3. **Header Management Patterns**: Follow established patterns for internal header injection and removal for security
4. **Token Service Integration**: Leverage DefaultTokenService for consistent token validation and caching strategies
5. **Testing Infrastructure**: Design comprehensive service mocking for isolated middleware testing scenarios

### Extending Token Management Capabilities
For new token validation and management patterns:

1. **Token Type Extensions**: Add support for new JWT token types while maintaining security validation standards
2. **Cache Strategy Design**: Implement appropriate caching strategies with proper TTL and invalidation for performance
3. **Database Integration**: Coordinate with DbService for new token storage and status tracking requirements
4. **External Client Support**: Design external client token patterns with proper issuer and audience validation
5. **Performance Optimization**: Minimize database and service calls through intelligent caching and validation patterns

### Authorization Pattern Extensions
For new authorization requirements and access control patterns:

1. **Role Hierarchy Integration**: Ensure new authorization logic follows established role hierarchy with objs crate integration
2. **Resource Scope Extensions**: Extend ResourceScope union type for new authorization contexts while maintaining precedence
3. **Cross-Service Consistency**: Coordinate with objs crate for new role and scope types to maintain domain consistency
4. **API Middleware Design**: Create configurable authorization middleware supporting different role and scope requirements
5. **Authorization Testing**: Design comprehensive authorization testing scenarios with different role and scope combinations

## Commands

**Testing**: `cargo test -p auth_middleware` (includes authentication flow and middleware testing)  
**Integration Testing**: `cargo test -p auth_middleware --features test-utils` (includes OAuth2 integration test client)  
**Live Testing**: `cargo test -p auth_middleware test_live_auth_middleware` (requires live OAuth2 server configuration)