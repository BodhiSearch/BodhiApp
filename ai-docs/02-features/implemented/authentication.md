# Authentication & Authorization Implementation

## Overview

Bodhi App implements OAuth2/OpenID Connect authentication with Keycloak as the identity provider. The system supports both authenticated and non-authenticated modes, with JWT token validation, role-based access control, and API token management.

**Supported Authentication Methods:**
- **Bearer Token Authentication**: JWT tokens in `Authorization: Bearer <token>` header
- **Session-based Authentication**: Session cookies with JWT tokens stored in session
- **No Authentication**: Completely disabled authentication mode

**NOT Supported:**
- Basic Authentication
- Digest Authentication
- API Key headers (X-API-KEY)
- Custom authentication schemes

## Authentication Modes

### 1. Authenticated Mode (`authz: true`)
- **OAuth2 Integration**: Uses Keycloak as external identity provider
- **JWT Tokens**: Access and refresh tokens with role-based claims
- **Role Hierarchy**: Admin → Manager → PowerUser → User with inherited permissions
- **API Tokens**: Long-lived offline tokens for programmatic access
- **First Admin**: First user to authenticate becomes the resource admin

### 2. Non-Authenticated Mode (`authz: false`)
- **No Authentication**: All endpoints publicly accessible
- **Bypass Middleware**: Authorization middleware completely skipped
- **Local Development**: Ideal for single-user scenarios
- **No Token Validation**: All requests proceed without checks

## Core Implementation

### Authentication Middleware

**File**: `crates/auth_middleware/src/auth_middleware.rs`

The main authentication middleware handles two authentication methods:

```rust
// auth_middleware.rs:66-132
pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // Remove any existing auth headers (lines 73-75)
  req.headers_mut().remove(KEY_RESOURCE_TOKEN);
  req.headers_mut().remove(KEY_RESOURCE_ROLE);
  req.headers_mut().remove(KEY_RESOURCE_SCOPE);

  // Check app status - redirect to setup if needed (lines 85-94)
  if app_status_or_default(&secret_service) == AppStatus::Setup {
    return Ok(Redirect::to(&format!("{}/ui/setup", frontend_url)).into_response());
  }

  // Skip auth if disabled (lines 96-99)
  if !authz_status(&secret_service) {
    return Ok(next.run(req).await);
  }

  // Method 1: Bearer token in Authorization header (lines 102-113)
  if let Some(header) = req.headers().get(axum::http::header::AUTHORIZATION) {
    let (access_token, token_scope) = token_service.validate_bearer_token(header).await?;
    req.headers_mut().insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
    req.headers_mut().insert(KEY_RESOURCE_SCOPE, token_scope.to_string().parse().unwrap());
    Ok(next.run(req).await)
  }
  // Method 2: Session token (lines 114-131)
  else if let Some(access_token) = session.get::<String>("access_token").await? {
    let (access_token, role) = token_service.get_valid_session_token(session, access_token).await?;
    req.headers_mut().insert(KEY_RESOURCE_TOKEN, access_token.parse().unwrap());
    req.headers_mut().insert(KEY_RESOURCE_ROLE, role.to_string().parse().unwrap());
    Ok(next.run(req).await)
  } else {
    Err(AuthError::InvalidAccess)?
  }
}
```

### Internal Headers

**File**: `crates/auth_middleware/src/auth_middleware.rs:16-18`

The middleware sets these internal headers for downstream handlers:

```rust
pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";   // Contains validated JWT token
pub const KEY_RESOURCE_ROLE: &str = "X-Resource-Access";   // Contains user role (session auth)
pub const KEY_RESOURCE_SCOPE: &str = "X-Resource-Scope";   // Contains token scope (API auth)
```

**Note**: These are internal headers set by the middleware, NOT headers that clients send.

## Role-Based Access Control

### Role Hierarchy

**File**: Referenced in tests at `crates/auth_middleware/src/api_auth_middleware.rs:145-154`

```rust
// Supported roles (from test cases):
Role::User        // Basic user access
Role::PowerUser   // Enhanced user access
Role::Manager     // Management access
Role::Admin       // Full administrative access

// Hierarchy: Admin > Manager > PowerUser > User
// Higher roles inherit permissions of lower roles
```

### API Authorization Middleware

**File**: `crates/auth_middleware/src/api_auth_middleware.rs:37-94`

```rust
pub async fn api_auth_middleware(
  required_role: Role,
  required_scope: Option<TokenScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // Skip if authorization disabled (lines 54-58)
  if !state.app_service().secret_service().authz()? {
    return Ok(next.run(req).await);
  }

  let role_header = req.headers().get(KEY_RESOURCE_ROLE);
  let scope_header = req.headers().get(KEY_RESOURCE_SCOPE);

  match (role_header, scope_header, required_scope) {
    // Role-based access (session tokens) - lines 65-75
    (Some(role_header), _, _) => {
      let user_role = role_header.to_str()?.parse::<Role>()?;
      if !user_role.has_access_to(&required_role) {
        return Err(ApiAuthError::Forbidden);
      }
    }
    // Scope-based access (API tokens) - lines 77-87
    (None, Some(scope_header), Some(required_scope)) => {
      let user_scope = scope_header.to_str()?.parse::<TokenScope>()?;
      if !user_scope.has_access_to(&required_scope) {
        return Err(ApiAuthError::Forbidden);
      }
    }
    // No valid authorization - line 90
    _ => return Err(ApiAuthError::MissingAuth),
  }

  Ok(next.run(req).await)
}
```

## Token Scopes

### Supported Scopes

**File**: Referenced in tests at `crates/auth_middleware/src/token_service.rs:278-284`

```rust
// API Token Scopes (from test cases):
TokenScope::User        // scope_token_user
TokenScope::PowerUser   // scope_token_power_user
TokenScope::Manager     // scope_token_manager
TokenScope::Admin       // scope_token_admin

// Hierarchy: Admin > Manager > PowerUser > User
```

### Bearer Token Validation

**File**: `crates/auth_middleware/src/token_service.rs:39-121`

```rust
pub async fn validate_bearer_token(
  &self,
  header: &str,
) -> Result<(String, TokenScope), AuthError> {
  // Extract Bearer token (lines 44-52)
  let offline_token = header.strip_prefix(BEARER_PREFIX)
    .ok_or_else(|| TokenError::InvalidToken("authorization header is malformed".to_string()))?;

  // Check token exists and is active in database (lines 55-67)
  let api_token = self.db_service.get_api_token_by_token_id(offline_token).await?;
  if api_token.status == TokenStatus::Inactive {
    return Err(AuthError::TokenInactive);
  }

  // Check cache for valid access token (lines 69-92)
  if let Some(access_token) = self.cache_service.get(&format!("token:{}", api_token.token_id)) {
    // Validate expiration and return cached token
  }

  // Exchange offline token for access token (lines 104-112)
  let (access_token, _) = self.auth_service.refresh_token(
    &app_reg_info.client_id,
    &app_reg_info.client_secret,
    offline_token,
  ).await?;

  // Cache the access token (lines 115-120)
  self.cache_service.set(&format!("token:{}", api_token.token_id), &access_token);

  Ok((access_token, token_scope))
}
```

## Error Handling

### Authentication Errors

**File**: `crates/auth_middleware/src/auth_middleware.rs:20-64`

```rust
pub enum AuthError {
  Token(TokenError),                    // Invalid token format/content
  Role(RoleError),                     // Invalid role
  TokenScope(TokenScopeError),         // Invalid token scope
  MissingRoles,                        // No roles in token
  InvalidAccess,                       // No valid authentication
  TokenInactive,                       // Token disabled in database
  TokenNotFound,                       // Token not found in database
  AuthService(AuthServiceError),       // External auth service error
  AppRegInfoMissing,                   // App not registered
  RefreshTokenNotFound,                // No refresh token in session
  SecretService(SecretServiceError),   // Secret service error
  TowerSession(tower_sessions::session::Error), // Session error
  SignatureKey(String),                // JWT signature error
  InvalidToken(String),                // Generic token error
  SignatureMismatch(String),           // JWT signature mismatch
}
```

### API Authorization Errors

**File**: `crates/auth_middleware/src/api_auth_middleware.rs:12-35`

```rust
pub enum ApiAuthError {
  SecretService(SecretServiceError),   // Secret service error
  Forbidden,                           // Insufficient permissions
  MissingAuth,                         // No authentication headers
  MalformedRole(String),              // Invalid role header format
  MalformedScope(String),             // Invalid scope header format
  InvalidRole(RoleError),             // Invalid role value
  InvalidScope(TokenScopeError),      // Invalid scope value
}
```

## Related Documentation

- **[App Status System](../app-status.md)** - Application state machine and setup flow
- **[Backend Integration](../../01-architecture/backend-integration.md)** - API integration patterns
- **[App Overview](../../01-architecture/app-overview.md)** - High-level system architecture
