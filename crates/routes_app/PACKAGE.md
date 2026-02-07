# PACKAGE.md - routes_app

This document provides the implementation index and navigation guide for the `routes_app` crate, covering application API endpoints, domain error types, typed extractors, and testing patterns.

## Module Structure

Entry point: `crates/routes_app/src/lib.rs` -- Re-exports all public modules with conditional `test_utils` compilation.

### Route Modules (one per API domain)

| Module | File | Purpose |
|--------|------|---------|
| `routes_login` | `crates/routes_app/src/routes_login.rs` | OAuth2 initiate/callback, logout, app-to-app access |
| `routes_access_request` | `crates/routes_app/src/routes_access_request.rs` | User access request workflow (request, status, approve, reject) |
| `routes_users_list` | `crates/routes_app/src/routes_users_list.rs` | Admin user listing, role changes, user removal |
| `routes_api_token` | `crates/routes_app/src/routes_api_token.rs` | API token create, update, list with privilege escalation prevention |
| `routes_models` | `crates/routes_app/src/routes_models.rs` | Model alias listing (discriminated union), model file listing, alias details |
| `routes_models_metadata` | `crates/routes_app/src/routes_models_metadata.rs` | Metadata refresh (sync/async), queue status |
| `routes_create` | `crates/routes_app/src/routes_create.rs` | Model alias create and update via CreateCommand |
| `routes_pull` | `crates/routes_app/src/routes_pull.rs` | Model download via PullCommand with progress tracking |
| `routes_toolsets` | `crates/routes_app/src/routes_toolsets.rs` | Toolset CRUD, execution, type management |
| `routes_settings` | `crates/routes_app/src/routes_settings.rs` | Settings list, update, reset to default |
| `routes_setup` | `crates/routes_app/src/routes_setup.rs` | App setup, ping, health, app info |
| `routes_user` | `crates/routes_app/src/routes_user.rs` | Current user info (discriminated union response) |
| `routes_dev` | `crates/routes_app/src/routes_dev.rs` | Development/debug endpoints |

### Supporting Modules

| Module | File | Purpose |
|--------|------|---------|
| `error` | `crates/routes_app/src/error.rs` | `LoginError` enum (shared across login/callback) |
| `common` | `crates/routes_app/src/common.rs` | `RedirectResponse` DTO |
| `api_dto` | `crates/routes_app/src/api_dto.rs` | Pagination params, paginated response types, endpoint constants |
| `api_models_dto` | `crates/routes_app/src/api_models_dto.rs` | Model alias response DTOs (AliasResponse, UserAliasResponse, etc.) |
| `toolsets_dto` | `crates/routes_app/src/toolsets_dto.rs` | Toolset request/response DTOs |
| `openapi` | `crates/routes_app/src/openapi.rs` | OpenAPI spec generation with Utoipa |
| `utils` | `crates/routes_app/src/utils.rs` | Helper functions (e.g., `extract_request_host`) |

## Domain Error Enums

Each route module defines its own error enum. All use the `errmeta_derive::ErrorMeta` derive macro.

### LoginError (`crates/routes_app/src/error.rs`)

```rust
pub enum LoginError {
  AppRegInfoNotFound,          // ErrorType::InvalidAppState
  AppStatusInvalid(AppStatus), // ErrorType::InvalidAppState
  SecretServiceError(..),      // transparent
  SessionError(..),            // ErrorType::Authentication
  SessionInfoNotFound,         // ErrorType::InternalServer
  OAuthError(String),          // ErrorType::BadRequest
  AuthServiceError(..),        // transparent
  ParseError(..),              // ErrorType::InternalServer
  JsonWebToken(..),            // transparent
  StateDigestMismatch,         // ErrorType::BadRequest
  MissingState,                // ErrorType::BadRequest
  MissingCode,                 // ErrorType::BadRequest
}
```

### AccessRequestError (`crates/routes_app/src/routes_access_request.rs`)

```rust
pub enum AccessRequestError {
  AlreadyPending,              // ErrorType::Conflict
  AlreadyHasAccess,            // ErrorType::UnprocessableEntity
  PendingRequestNotFound,      // ErrorType::NotFound
  RequestNotFound(i64),        // ErrorType::NotFound
  InsufficientPrivileges,      // ErrorType::BadRequest
  FetchFailed(String),         // ErrorType::InternalServer
}
```

### ApiTokenError (`crates/routes_app/src/routes_api_token.rs`)

```rust
pub enum ApiTokenError {
  Token(..),                   // transparent from TokenError
  AppRegMissing,               // ErrorType::InternalServer
  AccessTokenMissing,          // ErrorType::BadRequest
  RefreshTokenMissing,         // ErrorType::BadRequest
  AuthService(..),             // transparent from AuthServiceError
  PrivilegeEscalation,         // ErrorType::BadRequest
  InvalidScope,                // ErrorType::BadRequest
  InvalidRole(String),         // ErrorType::BadRequest
}
```

### UserManagementError (`crates/routes_app/src/routes_users_list.rs`)

```rust
pub enum UserManagementError {
  ListFailed(String),          // ErrorType::InternalServer
  RoleChangeFailed(String),    // ErrorType::InternalServer
  RemoveFailed(String),        // ErrorType::InternalServer
}
```

### MetadataError (`crates/routes_app/src/routes_models_metadata.rs`)

```rust
pub enum MetadataError {
  InvalidRepoFormat(String),   // ErrorType::BadRequest
  ListAliasesFailed,           // ErrorType::InternalServer
  AliasNotFound { repo, filename, snapshot }, // ErrorType::NotFound
  ExtractionFailed(String),    // ErrorType::BadRequest
  EnqueueFailed,               // ErrorType::BadRequest
}
```

### AppServiceError (`crates/routes_app/src/routes_setup.rs`)

```rust
pub enum AppServiceError {
  AlreadySetup,                // ErrorType::BadRequest
  ServerNameTooShort,          // ErrorType::BadRequest
  SecretServiceError(..),      // transparent
  AuthServiceError(..),        // transparent
}
```

### Other Error Enums

- `ModelError` in `crates/routes_app/src/routes_models.rs` -- `MetadataFetchFailed`
- `ToolsetValidationError` in `crates/routes_app/src/routes_toolsets.rs` -- `Validation(String)`
- `SettingsError` in `crates/routes_app/src/routes_settings.rs` -- `NotFound`, `BodhiHome`, `Unsupported`
- `LogoutError` in `crates/routes_app/src/routes_login.rs` -- `SessionDelete`
- `UserInfoError` in `crates/routes_app/src/routes_user.rs` -- `InvalidHeader`, `EmptyToken`
- `CreateAliasError` in `crates/routes_app/src/routes_create.rs` -- `AliasNotPresent`, transparent wrappers

## Typed Extractor Usage Pattern

Handlers use typed extractors from `auth_middleware` for identity, with optional `HeaderMap` for auxiliary inspection:

```rust
// Extractors for identity + HeaderMap for auxiliary logic
pub async fn list_toolsets_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsetsResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();
  let toolsets = tool_service.list(&user_id).await?;
  // HeaderMap used for is_oauth_auth() filtering, not identity
  let filtered = if is_oauth_auth(&headers) {
    // ...filter by scope
  } else {
    toolsets
  };
  // ...
}
```

Handlers that need multiple identity fields:

```rust
pub async fn approve_request_handler(
  ExtractRole(approver_role): ExtractRole,
  ExtractUsername(approver_username): ExtractUsername,
  ExtractToken(token): ExtractToken,
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<i64>,
  Json(request): Json<ApproveUserAccessRequest>,
) -> Result<StatusCode, ApiError> { /* ... */ }
```

## API Token Privilege Matrix

Defined in `crates/routes_app/src/routes_api_token.rs`, the `create_token_handler` enforces:

```rust
let token_scope = match (user_role, &payload.scope) {
  (ResourceRole::User, TokenScope::User) => Ok(payload.scope),
  (ResourceRole::User, _) => Err(ApiTokenError::PrivilegeEscalation),
  (_, TokenScope::User | TokenScope::PowerUser) => Ok(payload.scope),
  (_, _) => Err(ApiTokenError::InvalidScope),
}?;
```

## OAuth2 Callback URL Detection

In `crates/routes_app/src/routes_login.rs` and `crates/routes_app/src/routes_setup.rs`, the callback URL strategy depends on configuration:

- **Explicit public host** (`BODHI_PUBLIC_HOST` set): Uses only the configured callback URL
- **Local/network mode** (no explicit host): Extracts `Host` header from the request to construct the callback URL, enabling network access from different machines

Setup handler registers multiple redirect URIs for local mode: all loopback hosts (localhost, 127.0.0.1, 0.0.0.0), the request host if different, and the server's detected IP.

## Toolset Dual-Auth Model

In `crates/routes_app/src/routes_toolsets.rs`:

```rust
fn is_oauth_auth(headers: &HeaderMap) -> bool {
  !headers.contains_key(KEY_HEADER_BODHIAPP_ROLE)
    && headers.get(KEY_HEADER_BODHIAPP_SCOPE)
      .and_then(|v| v.to_str().ok())
      .map(|s| s.starts_with("scope_user_"))
      .unwrap_or(false)
}
```

Session auth (has `ROLE` header): full access to all toolset types and configs.
OAuth auth (has `SCOPE` header, no `ROLE`): filtered by `scope_toolset-*` scopes in token.

## OpenAPI Specification

`crates/routes_app/src/openapi.rs` defines `BodhiOpenAPIDoc` with:
- All endpoint paths registered via `#[openapi(paths(...))]`
- All request/response schemas via `#[openapi(components(schemas(...)))]`
- `OpenAPIEnvModifier` for environment-specific server URL and bearer auth security scheme

## Testing Patterns

### Router Construction in Tests
Each route module constructs test routers with mocked services:

```rust
fn test_router(mock_tool_service: MockToolService) -> Router {
  let app_service = AppServiceStubBuilder::default()
    .with_tool_service(Arc::new(mock_tool_service))
    .build().unwrap();
  let state: Arc<dyn RouterState> = Arc::new(
    DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    )
  );
  routes_toolsets(state)
}
```

### Header-Based Auth Simulation
Tests simulate authentication by setting internal headers directly (bypassing middleware):

```rust
Request::builder()
  .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
  .header(KEY_HEADER_BODHIAPP_TOKEN, "admin-token")
  .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::Admin.to_string())
```

### Integration Tests with Real Database
Access request and session tests use real SQLite databases via `test_db_service` fixtures. Session clearing tests create real session records and verify they are removed after role changes.

### Test Utilities

- `crates/routes_app/src/test_utils/alias_response.rs` -- Builder-based test fixtures for `AliasResponse`
- `crates/routes_app/src/test_utils/mod.rs` -- Test helper macros and constants

## Commands

**Run all tests**: `cargo test -p routes_app`
**Run specific module tests**: `cargo test -p routes_app routes_toolsets` (or any module name)
**Run with test-utils feature**: `cargo test -p routes_app --features test-utils`
