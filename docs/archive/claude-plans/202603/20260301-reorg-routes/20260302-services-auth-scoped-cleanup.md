# Auth Scoped Services Error Architecture Cleanup

## Context

The `services` crate's auth scoped services (`AuthScopedTokenService`, `AuthScopedMcpService`, `AuthScopedToolService`, `AuthScopedUserService`) return `ApiError` -- an API/HTTP concern that couples the service layer to web framework types. Additionally, `ApiError`, `OpenAIApiError`, `ErrorBody`, `JsonRejectionError`, and their `IntoResponse` impls live in `services` despite being API-layer concerns. The `auth_middleware` crate also depends on `ApiError` for middleware return types.

**Goal**: Decouple the services layer from API/HTTP concerns by:
1. Creating `AuthContextError` for auth state errors at the service level
2. Changing auth scoped services to return domain service errors (not `ApiError`)
3. Moving `ApiError` + related types to `routes_app`
4. Having `auth_middleware` use its own `MiddlewareError` wrapper

## Phase 1: Create AuthContextError (services crate)

**Files:**
- `crates/services/src/auth/auth_context.rs`

**Changes:**

1. Replace `AnonymousNotAllowedError` struct (lines 107-124) with:
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthContextError {
  #[error("Authentication required. Anonymous access not allowed.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AnonymousNotAllowed,

  #[error("Client ID is required but not present.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  MissingClientId,

  #[error("Authentication token required to perform this operation.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingToken,
}
```

2. Change `require_user_id()` return: `Result<&str, AuthContextError>` (was `ApiError`)
   - Return `Err(AuthContextError::AnonymousNotAllowed)` instead of `Err(ApiError::from(AnonymousNotAllowedError))`
3. Change `require_client_id()` return: `Result<&str, AuthContextError>` (was `ApiError`)
   - Return `Err(AuthContextError::MissingClientId)` instead of `Err(ApiError::from(AnonymousNotAllowedError))`
4. Remove `use crate::ApiError` from imports (keep `ErrorType`)
5. Delete `AnonymousNotAllowedError` struct

**Tests:** Update `test_require_user_id_anonymous_returns_403` and `test_require_client_id_anonymous_returns_403` to use `err.status()` method (AppError trait) instead of `err.status` field (ApiError struct). Error code `auth_context_error-anonymous_not_allowed` stays the same (ErrorMeta generates same code). New code: `auth_context_error-missing_client_id`.

**Note:** This phase compiles independently -- downstream code returning `Result<_, ApiError>` still works because `AuthContextError: AppError`, and the blanket `From<T: AppError> for ApiError` auto-converts via `?`.

**Verify:** `cargo test -p services`

---

## Phase 2: Auth Scoped Services Return Domain Errors (services crate)

### 2a. Add Auth variant to service error enums

**`crates/services/src/tokens/error.rs`** -- add two variants:
```rust
#[error(transparent)]
Auth(#[from] AuthContextError),

#[error(transparent)]
Entity(#[from] EntityError),  // for update_token's NotFound case
```

**`crates/services/src/mcps/error.rs`** -- add to `McpError`:
```rust
#[error(transparent)]
Auth(#[from] AuthContextError),
```

**`crates/services/src/toolsets/error.rs`** -- add to `ToolsetError`:
```rust
#[error(transparent)]
Auth(#[from] AuthContextError),
```

### 2b. Expand AuthScopedUserError

**`crates/services/src/app_service/auth_scoped_users.rs`**:

Replace current `AuthScopedUserError` (single `MissingToken` variant) with:
```rust
pub enum AuthScopedUserError {
  #[error(transparent)]
  Auth(#[from] AuthContextError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  AuthService(#[from] AuthServiceError),

  #[error(transparent)]
  Session(#[from] SessionServiceError),
}
```
`MissingToken` moves to `AuthContextError::MissingToken` (Phase 1).

### 2c. Change auth scoped service return types

| File | Return type change |
|---|---|
| `auth_scoped.rs` | `require_user_id()`, `require_client_id()` -> `Result<&str, AuthContextError>` |
| `auth_scoped_tokens.rs` | All methods -> `Result<..., TokenServiceError>` |
| `auth_scoped_mcps.rs` | All methods -> `Result<..., McpError>` |
| `auth_scoped_tools.rs` | All methods -> `Result<..., ToolsetError>` |
| `auth_scoped_users.rs` | All methods -> `Result<..., AuthScopedUserError>`, `require_token()` returns `Result<&str, AuthContextError>` |

Remove `use crate::ApiError` from each file. The `?` operator works because:
- `AuthContextError` converts to each service error via `#[from]`
- `DbError` converts to `TokenServiceError` via `#[from]`
- `McpService` methods already return `McpError`
- `ToolService` methods already return `ToolsetError`
- `AuthServiceError`/`SessionServiceError` convert to `AuthScopedUserError` via `#[from]`

**Note:** Downstream `routes_app` handlers still compile -- they return `Result<..., ApiError>` and the blanket `From<T: AppError>` converts `TokenServiceError`, `McpError`, etc. to `ApiError` automatically via `?`.

**Verify:** `cargo test -p services`

---

## Phase 3: MiddlewareError in auth_middleware

**Goal:** Decouple auth_middleware from `ApiError`.

### 3a. Create MiddlewareError

**New file:** `crates/auth_middleware/src/middleware_error.rs`

```rust
pub struct MiddlewareError {
  name: String,
  error_type: String,
  status: u16,
  code: String,
  args: HashMap<String, String>,
}
```

- Blanket `From<T: AppError + 'static>` captures error metadata (same pattern as current ApiError)
- `IntoResponse` constructs the OpenAI JSON format: `{"error": {"message", "type", "code", "param"}}` using `services::OpenAIApiError` and `services::ErrorBody` (pure data types, no axum dep, stay in services)

### 3b. Update middleware functions

| File | Function | Change |
|---|---|---|
| `auth_middleware/middleware.rs` | `auth_middleware()` | `Result<Response, MiddlewareError>` |
| `auth_middleware/middleware.rs` | `optional_auth_middleware()` | `Result<Response, MiddlewareError>` |
| `api_auth_middleware.rs` | `api_auth_middleware()` | `Result<Response, MiddlewareError>` |
| `access_request_auth_middleware/middleware.rs` | `access_request_auth_middleware()` | `Result<Response, MiddlewareError>` |

Remove `use services::ApiError` from each. The `?` operator works identically -- domain errors (`AuthError`, `ApiAuthError`, `AccessRequestAuthError`) all implement `AppError`, and the blanket `From` converts them to `MiddlewareError`.

Register module in `crates/auth_middleware/src/lib.rs`.

**Verify:** `cargo test -p auth_middleware`

---

## Phase 4: Move ApiError + Related Types to routes_app

### 4a. Move to routes_app

**New file:** `crates/routes_app/src/shared/api_error.rs`

Move from `services/shared_objs/error_api.rs`:
- `ApiError` struct
- `impl Display for ApiError`
- `impl<T: AppError + 'static> From<T> for ApiError` (blanket conversion)
- `impl From<ApiError> for OpenAIApiError`
- `impl IntoResponse for ApiError`
- `impl IntoResponse for JsonRejectionError`
- Tests (to `routes_app/src/shared/test_api_error.rs`)

Register in `crates/routes_app/src/shared/mod.rs`.

### 4b. Move OpenAIApiError + ErrorBody to routes_app

**New file:** `crates/routes_app/src/shared/error_oai.rs`

Move from `services/shared_objs/error_oai.rs`:
- `OpenAIApiError` struct
- `ErrorBody` struct
- `impl Display for OpenAIApiError`

These have `utoipa::ToSchema` for OpenAPI registration which is routes_app's concern.

**Update `MiddlewareError`** (Phase 3) to NOT use `services::OpenAIApiError/ErrorBody`. Instead construct JSON inline with `serde_json::json!()` to avoid depending on routes_app types.

### 4c. Remove from services

- Delete `crates/services/src/shared_objs/error_api.rs`
- Delete `crates/services/src/shared_objs/error_oai.rs`
- Update `crates/services/src/shared_objs/mod.rs` -- remove `mod error_api; mod error_oai;` and re-exports
- Update `crates/services/src/lib.rs` -- remove `ApiError`, `OpenAIApiError`, `ErrorBody` from exports

### 4d. Update imports

**routes_app (7 files):** Change `use services::ApiError` to `use crate::ApiError`:
- `routes_models_metadata.rs`, `routes_users_info.rs`, `routes_toolsets.rs`
- `routes_mcps.rs`, `routes_mcps_oauth.rs`, `routes_mcps_auth.rs`, `routes_mcps_servers.rs`

**routes_app (many files):** Change `use services::{OpenAIApiError, ErrorBody}` to `use crate::{OpenAIApiError, ErrorBody}`:
- All route files referencing these for utoipa schema registration

**services/bin/setup_client.rs:** Refactor to not use `ApiError` (it's a dev tool).

**services/settings/test_setting_objs.rs:** Refactor test to verify `AppError` trait methods directly instead of going through `ApiError` -> `OpenAIApiError`.

**Verify:** `cargo test -p services && cargo test -p auth_middleware && cargo test -p routes_app`

---

## Phase 5: Clean Up and Regression

### 5a. Remove unused axum imports from services

After removing error_api.rs, the remaining axum usage in services is:
- `shared_objs/error_wrappers.rs` -- `JsonRejectionError` wraps `axum::extract::rejection::JsonRejection` (stays -- still in services)
- `ai_apis/ai_api_service.rs` -- `axum::body::Body`, `axum::response::Response` for AI API forwarding (stays)
- Test files (`test_ai_api_service.rs`, `error_wrappers.rs` tests, `test_utils/http.rs`) -- stay

### 5b. Full regression

```bash
cargo test -p services
cargo test -p auth_middleware
cargo test -p routes_app
cargo test -p server_app
make test.backend
```

### 5c. Update crate CLAUDE.md / PACKAGE.md

- `services/CLAUDE.md`: Remove ApiError/OpenAIApiError/ErrorBody from shared_objs docs, add AuthContextError docs
- `routes_app/CLAUDE.md`: Add ApiError location and error conversion docs
- `auth_middleware/CLAUDE.md`: Add MiddlewareError docs

---

## Summary

| Component | Before | After |
|---|---|---|
| `AuthContext::require_user_id()` | `Result<&str, ApiError>` | `Result<&str, AuthContextError>` |
| `AuthContext::require_client_id()` | `Result<&str, ApiError>` | `Result<&str, AuthContextError>` |
| `AuthScopedAppService::require_*()` | `Result<&str, ApiError>` | `Result<&str, AuthContextError>` |
| `AuthScopedTokenService` methods | `Result<..., ApiError>` | `Result<..., TokenServiceError>` |
| `AuthScopedMcpService` methods | `Result<..., ApiError>` | `Result<..., McpError>` |
| `AuthScopedToolService` methods | `Result<..., ApiError>` | `Result<..., ToolsetError>` |
| `AuthScopedUserService` methods | `Result<..., ApiError>` | `Result<..., AuthScopedUserError>` |
| Middleware functions (4) | `Result<Response, ApiError>` | `Result<Response, MiddlewareError>` |
| Route handlers | `Result<..., ApiError>` | `Result<..., ApiError>` (unchanged) |
| `ApiError` location | `services::shared_objs` | `routes_app::shared` |
| `OpenAIApiError`/`ErrorBody` location | `services::shared_objs` | `routes_app::shared` |

**New types:** `AuthContextError` (services), `MiddlewareError` (auth_middleware)
**Removed:** `AnonymousNotAllowedError` struct, `AuthScopedUserError::MissingToken` variant
**New variants:** `Auth(AuthContextError)` added to `TokenServiceError`, `McpError`, `ToolsetError`; `Entity(EntityError)` added to `TokenServiceError`; `Db`, `AuthService`, `Session` added to `AuthScopedUserError`
