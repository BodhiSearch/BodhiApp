# Cross-Crate Analysis: Error Chains and Framework Leakage

Review of commits `d3279cf84..6d559f7b9` (Phase 0 + Phase 2).

## Findings

| # | Priority | File | Location | Issue | Recommendation |
|---|----------|------|----------|-------|----------------|
| 1 | P1 | `crates/routes_app/src/tokens/error.rs` | `TokenRouteError::Token` (line 8) | Dead `#[from]` variant: `Token(#[from] TokenError)` is never triggered. After migration to `AuthScopedTokenService`, handlers return `TokenServiceError` which converts to `ApiError` directly via the blanket impl. `TokenError` (JWT parsing from `shared_objs/token.rs`) is never returned by any handler in `routes_tokens.rs`. | Remove `Token(#[from] TokenError)` variant. If JWT-related errors are needed later, wrap `TokenServiceError` instead. |
| 2 | P1 | `crates/routes_app/src/tokens/error.rs` | `TokenRouteError::AuthService` (line 19) | Dead `#[from]` variant: `AuthService(#[from] AuthServiceError)` is never triggered. No handler in `routes_tokens.rs` returns `AuthServiceError`. Token creation previously called `auth_service` directly; now it uses `AuthScopedTokenService`. | Remove `AuthService(#[from] AuthServiceError)` variant. |
| 3 | P2 | `crates/routes_app/src/tokens/error.rs` | `TokenRouteError::AppRegMissing`, `RefreshTokenMissing`, `InvalidScope`, `InvalidRole` | Dead variants: `AppRegMissing`, `RefreshTokenMissing`, `InvalidScope`, and `InvalidRole(String)` are never constructed in any handler in `routes_tokens.rs`. These appear to be leftover from the old `routes_api_token` module that was deleted. | Remove dead variants. Only `AccessTokenMissing` and `PrivilegeEscalation` are actually used. |
| 4 | P2 | `crates/routes_app/src/routes_dev.rs` | `dev_secrets_handler` (line 30) | Uses old `Extension(auth_context): Extension<AuthContext>` + `State(state): State<Arc<dyn RouterState>>` pattern instead of the new `AuthScope` extractor. This file was touched in the diff (import path fixes) but not migrated. | Migrate to `AuthScope` extractor in a follow-up phase. `routes_dev.rs` is dev-only so lower priority, but it was touched and left inconsistent. |
| 5 | P2 | `crates/routes_app/src/mcps/routes_mcps_auth.rs` | `mcp_auth_configs_create` (line 38), `mcp_auth_configs_destroy` (line 114), `mcp_oauth_token_exchange` (line 248) | Uses old `Extension(auth_context): Extension<AuthContext>` + `State(state)` pattern. These handlers were created/modified in the diff but still use the legacy extractor pattern. | Migrate to `AuthScope` extractor. These are new handler code added in this diff, so they should follow the new convention. |
| 6 | P2 | `crates/routes_app/src/mcps/routes_mcps_servers.rs` | Lines 35, 102 | Uses old `Extension(auth_context)` + `State(state)` pattern. File was modified in this diff. | Migrate to `AuthScope` extractor. |
| 7 | P2 | `crates/routes_app/src/users/routes_users_access_request.rs` | `users_request_access` (line 42), `users_request_status` (line 103), `users_access_request_approve` (line 222), `users_access_request_reject` (line 311) | Uses old `Extension(auth_context)` + `State(state)` pattern. File was modified in this diff. | Migrate to `AuthScope` extractor in the next migration phase. |
| 8 | P2 | `crates/routes_app/src/users/routes_users_info.rs` | Line 39 | Uses old `Extension(auth_context)` + `State(state)` pattern. File was modified in this diff. | Migrate to `AuthScope` extractor. |
| 9 | P2 | `crates/routes_app/src/mcps/routes_mcps_oauth.rs` | Lines 182, 211 | Uses old `Extension(auth_context)` pattern. File was modified in this diff. | Migrate to `AuthScope` extractor. |
| 10 | P3 | `crates/routes_app/src/tokens/test_tokens_security.rs` | `test_create_token_privilege_escalation_user` (line 37) | New test only asserts HTTP status code (400), does not assert the error code (`"token_route_error-privilege_escalation"`). The codebase convention (per `services/CLAUDE.md`) is to assert error codes via `.code()` or response body `code` field. | Add assertion on the response body `error.code` field to verify the correct error variant is returned. |

## Analysis Summary

### 1. Error Chain Tracing (PASS)

The chain `service error -> AppError trait -> ApiError -> HTTP response` is intact:
- **Blanket impl** `impl<T: AppError + 'static> From<T> for ApiError` exists in `crates/routes_app/src/shared/api_error.rs` (line 26).
- `TokenServiceError`, `McpError`, `ToolsetError`, `AuthScopedUserError` all derive `ErrorMeta` with `trait_to_impl = AppError`, so they satisfy the `AppError + 'static` bound.
- `ApiError` converts to `OpenAIApiError` (line 39) and implements `IntoResponse` (line 61) for HTTP response generation.
- `MiddlewareError` in `auth_middleware` has its own parallel blanket `From<T: AppError>` impl (line 17 of `middleware_error.rs`), correctly separated from `ApiError`.

### 2. Axum Leakage into Services (PARTIAL)

The diff **removed** axum from `shared_objs/error_wrappers.rs` (deleted `JsonRejectionError` and its test, which used `axum::extract::rejection::JsonRejection`). This was moved to `routes_app/src/shared/error_wrappers.rs`.

**Pre-existing axum usage** (not introduced by this diff, not flagged):
- `ai_api_service.rs` uses `axum::body::Body` and `axum::response::Response` (returns axum Response for streaming)
- `test_utils/http.rs` uses axum Body/Response for test helpers
- `session_service.rs` uses `tower_sessions::SessionManagerLayer` (tower, not axum directly)

### 3. Extension<AuthContext> Remnants (FINDINGS 4-9)

13 handler functions across 6 files still use the old `Extension(auth_context): Extension<AuthContext>` pattern. All 6 files were modified in this diff. The `tokens/` module was fully migrated to `AuthScope`; the others were not. See findings #4-9 above.

### 4. MiddlewareError vs ApiError Boundary (PASS)

`auth_middleware` does **not** import `ApiError`, `OpenAIApiError`, or `ErrorBody`. The only match found was `ApiErrorResponse` in `utils.rs`, which is a completely unrelated simple struct `{ error: String }` used for middleware-specific error formatting. The name is potentially confusing but it predates this diff and is not an actual boundary violation.

### 5. Error Code Consistency (WARNING)

`TokenRouteError` uses auto-generated codes. Expected codes for used variants:
- `TokenRouteError::AccessTokenMissing` -> `"token_route_error-access_token_missing"` (correct per ErrorMeta convention)
- `TokenRouteError::PrivilegeEscalation` -> `"token_route_error-privilege_escalation"` (correct)

The new test (`test_tokens_security.rs`) only asserts status code, not error code. See finding #10.

### 6. Re-export Chain Integrity (PASS)

- **services re-exports errmeta types**: `services/src/lib.rs` line 74: `pub use errmeta::{impl_error_from, AppError, EntityError, ErrorType, IoError, RwLockReadError};`
- **auth_middleware re-exports AuthContext**: `auth_middleware/src/lib.rs` line 21: `pub use auth_context::AuthContext;` (which is a re-export shim from `services::AuthContext`)
- **routes_app gets ApiError locally**: `routes_app/src/shared/mod.rs` line 15: `pub use api_error::*;` -- ApiError is defined in `shared/api_error.rs`, not imported from services. Confirmed by `routes_dev.rs` change: `use crate::ApiError;` (was `use services::ApiError;`).
- **services no longer exports ApiError**: `shared_objs/error_api.rs` was deleted, `shared_objs/mod.rs` no longer re-exports `error_api::*` or `error_oai::*`.
