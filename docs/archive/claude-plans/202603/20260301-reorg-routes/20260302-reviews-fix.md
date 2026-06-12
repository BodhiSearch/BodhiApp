# Fix Plan: Review Findings from Routes Reorg + Auth-Scoped Services

## Context

Commits `d3279cf84..6d559f7b9` (Phase 0 + Phase 2) introduced AuthScope extractor, migrated tokens/ handlers, reorganized services by domain, and added AuthScopedTokenService. A thorough 8-agent review identified 33 findings across 4 priority levels. This plan fixes all of them across 5 focused commits following the layered development methodology (upstream first).

**Review source**: `ai-docs/claude-plans/20260301-reorg-routes/reviews/index.md`

---

## Commit 1: services crate -- Token service refactor + error normalization

**Crate**: `services` | **Test**: `cargo test -p services`

### P1-1: Move token generation logic into DefaultTokenService
- **File**: `crates/services/src/tokens/token_service.rs`
  - Add `create_token(user_id, name, scope)` method to `TokenService` trait with full generation logic (random bytes, SHA-256 hashing, ULID, `bodhiapp_` prefix, base64 encoding)
  - Move logic FROM `crates/services/src/app_service/auth_scoped_tokens.rs::create_token` INTO `DefaultTokenService::create_token`
  - `DefaultTokenService` needs access to `TimeService` (inject via constructor or accessor)
- **File**: `crates/services/src/app_service/auth_scoped_tokens.rs`
  - `AuthScopedTokenService::create_token` now: auth check (`require_user_id()`) + delegate to `self.app_service.token_service().create_token(user_id, name, scope)`
  - `list_tokens`, `get_token`, `update_token` -- delegate through `token_service()` instead of `db_service()` directly
- **File**: `crates/services/src/tokens/mod.rs` -- ensure new trait method is re-exported

### P2-2: Remove raw service passthroughs from AuthScopedAppService
- **File**: `crates/services/src/app_service/auth_scoped.rs`
  - Remove: `mcp_service()`, `tool_service()`, `token_service()` passthrough accessors
  - Keep: `db_service()`, `auth_service()`, `session_service()`, `secret_service()`, `hub_service()`, `data_service()`, `setting_service()` -- needed by auth_middleware and infrastructure
  - Add `get_type()` delegation to `AuthScopedToolService` (for `toolset_to_response` use case)
- **File**: `crates/services/src/app_service/auth_scoped_tools.rs`
  - Add `pub fn get_type(&self, type_name: &str) -> Option<ToolTypeDef>` that delegates to `self.app_service.tool_service().get_type(type_name)`
- Fix compile errors in routes_app from removed passthroughs (handled in commit 3/4)

### P2-3: Normalize `DbError` variant -> `Db` across all domain error enums
- **Files to change** (variant `DbError` -> `Db`):
  - `crates/services/src/mcps/error.rs` -- `McpError::DbError` -> `Db`
  - `crates/services/src/toolsets/error.rs` -- `ToolsetError::DbError` -> `Db`
  - `crates/services/src/mcps/mcp_server_error.rs` -- `McpServerError::DbError` -> `Db`
  - `crates/services/src/app_access_requests/error.rs` -- `AccessRequestError::DbError` -> `Db`
- **No change needed**: `TokenServiceError::Db` and `AuthScopedUserError::Db` already use `Db`
- Update test assertions for changed error codes (`*-db_error` -> `*-db`)

### P2-5: Update stale comment in lib.rs
- **File**: `crates/services/src/lib.rs` line 75
  - Change `"axum/serde-dependent"` to `"serde/validator-dependent error types"`

### P3-2: Add Anonymous factory in test_utils
- **File**: `crates/services/src/test_utils/auth_context.rs`
  - Add `test_anonymous()` and `test_anonymous_with_client_id(client_id)` factory methods

---

## Commit 2: auth_middleware crate -- Fix redundant DB call + silent empty client_id

**Crate**: `auth_middleware` | **Test**: `cargo test -p auth_middleware`

### P2-7: Eliminate redundant get_instance() call
- **File**: `crates/auth_middleware/src/auth_middleware/middleware.rs`
  - In `auth_middleware()`: call `get_instance()` once at function scope, extract both `AppStatus` and `client_id` from it
  - Remove the second `get_instance()` call at line 146
  - Pass `client_id` to the session branch; bearer path still does its own internal lookup (acceptable for now)

### P2-8: Fall back to Anonymous when instance lookup fails in optional_auth_middleware
- **File**: `crates/auth_middleware/src/auth_middleware/middleware.rs`
  - In `optional_auth_middleware()` around line 265: when `instance_client_id` is `None`, return `AuthContext::Anonymous` instead of creating a Session with empty `client_id`

### P3-3: Remove unnecessary borrow() in MiddlewareError::From
- **File**: `crates/auth_middleware/src/middleware_error.rs`
  - Remove `use std::borrow::Borrow` import
  - Remove `value.borrow()` call, access fields directly on `&value`

### P3-4: Add fallback for unwrap() on Response builder
- **File**: `crates/auth_middleware/src/middleware_error.rs` line 43
  - Replace `.unwrap()` with `.unwrap_or_else(|_| { /* 500 fallback */ })`

---

## Commit 3: routes_app -- AuthScope migration (P0 + P1-7)

**Crate**: `routes_app` | **Test**: `cargo test -p routes_app`

### P0-1: Migrate mcps/ handlers to AuthScope (14 handlers)

**File**: `crates/routes_app/src/mcps/routes_mcps_auth.rs`
- `mcp_auth_configs_create`: `Extension<AuthContext>` + `State(state)` -> `AuthScope`
- `mcp_auth_configs_destroy`: same migration
- `mcp_oauth_token_exchange`: same migration (also uses `Session` -- keep Session extractor)
- `mcp_auth_configs_index`: `State(state)` -> `AuthScope`
- `mcp_auth_configs_show`: same

**File**: `crates/routes_app/src/mcps/routes_mcps_oauth.rs`
- `mcp_oauth_tokens_show`: `Extension<AuthContext>` + `State(state)` -> `AuthScope`
- `mcp_oauth_tokens_destroy`: same
- `mcp_oauth_discover_as`: `State(state)` -> `AuthScope` (public endpoint, falls back to Anonymous)
- `mcp_oauth_discover_mcp`: same
- `mcp_oauth_dynamic_register`: same

**File**: `crates/routes_app/src/mcps/routes_mcps_servers.rs`
- `mcp_servers_create`: `Extension<AuthContext>` + `State(state)` -> `AuthScope`
- `mcp_servers_update`: same
- `mcp_servers_show`: `State(state)` -> `AuthScope`
- `mcp_servers_index`: same

**File**: `crates/routes_app/src/mcps/routes_mcps.rs`
- `mcps_fetch_tools`: `State(state)` -> `AuthScope`

**Pattern for migration**: Replace `Extension(auth_context): Extension<AuthContext>` + `State(state): State<Arc<dyn RouterState>>` with `auth_scope: AuthScope`. Access auth context via `auth_scope.auth_context()`. Access services via `auth_scope.mcps()` (auth-scoped) or `auth_scope.db_service()` (infrastructure).

### P0-2: Migrate users/ handlers to AuthScope (7 handlers)

**File**: `crates/routes_app/src/users/routes_users_access_request.rs`
- `users_request_access`, `users_request_status`, `users_access_requests_pending`, `users_access_requests_index`, `users_access_request_approve`, `users_access_request_reject`
- All: `Extension<AuthContext>` + `State(state)` -> `AuthScope`

**File**: `crates/routes_app/src/users/routes_users_info.rs`
- `users_info`: `Extension(auth_context)` -> `AuthScope`

### P1-7: Migrate apps/ unauthenticated handlers to AuthScope (2 handlers)

**File**: `crates/routes_app/src/apps/routes_apps.rs`
- `apps_create_access_request`: `State(state)` -> `AuthScope` (unauthenticated, falls back to Anonymous)
- `apps_get_access_request_status`: same

### P2-14: Migrate routes_dev.rs to AuthScope

**File**: `crates/routes_app/src/routes_dev.rs`
- `dev_secrets_handler`: `Extension<AuthContext>` + `State(state)` -> `AuthScope`
- `dev_db_reset_handler`: `State(state)` -> `AuthScope`
- `envs_handler`: `State(state)` -> `AuthScope`

### P2-11: Replace expect() panics with require_user_id()

**File**: `crates/routes_app/src/users/routes_users.rs`
- `users_change_role` (lines 75-79): replace `auth_scope.auth_context().token().expect(...)` + `extract_claims` with `auth_scope.require_user_id()?`. Log `user_id` instead of `username`.
- `users_destroy` (lines 139-143): same pattern

---

## Commit 4: routes_app -- Error cleanup + business logic moves

**Crate**: `routes_app` | **Test**: `cargo test -p routes_app`

### P1-2: Remove dead TokenRouteError variants
- **File**: `crates/routes_app/src/tokens/error.rs`
  - Remove: `AppRegMissing`, `RefreshTokenMissing`, `InvalidScope`, `InvalidRole`
  - Remove: `Token(#[from] TokenError)`, `AuthService(#[from] AuthServiceError)` (dead `#[from]` conversions)
  - Keep: `AccessTokenMissing`, `PrivilegeEscalation`

### P1-3: Fix ErrorType::BadRequest -> Forbidden
- **File**: `crates/routes_app/src/tokens/error.rs`
  - `PrivilegeEscalation`: `ErrorType::BadRequest` -> `ErrorType::Forbidden`
  - `AccessTokenMissing`: `ErrorType::BadRequest` -> `ErrorType::Forbidden`
  - Update test assertions in `test_tokens_security.rs` (status 400 -> 403)

### P1-4: Rename LoginError -> AuthRouteError
- **File**: `crates/routes_app/src/auth/error.rs`
  - Rename enum `LoginError` -> `AuthRouteError`
  - Update all references in `routes_auth.rs` and test files
  - Error codes change: `login_error-*` -> `auth_route_error-*`
  - Update test assertions for new error codes

### P1-5: Restore args_delegate=false on auth/error.rs
- **File**: `crates/routes_app/src/auth/error.rs`
  - Add `args_delegate = false` to: `SessionError`, `ParseError`, `SessionDelete`

### P1-6: Remove duplicate test module declarations
- **File**: `crates/routes_app/src/settings/routes_settings.rs` -- remove lines 215-217 (`#[cfg(test)]` block)
- **File**: `crates/routes_app/src/setup/routes_setup.rs` -- remove lines 138-140 (`#[cfg(test)]` block)

### P2-9: Move ExternalApp filtering into AuthScopedMcpService
- **File**: `crates/services/src/app_service/auth_scoped_mcps.rs`
  - Modify `list()` method to handle ExternalApp filtering internally
  - Move `extract_approved_mcp_ids` helper from handler into service
- **File**: `crates/routes_app/src/mcps/routes_mcps.rs`
  - Simplify `mcps_index` handler to just call `auth_scope.mcps().list().await?` + map to response

### P2-10: Move ownership check into AuthScopedMcpService
- **File**: `crates/services/src/app_service/auth_scoped_mcps.rs`
  - Add ownership/privilege validation to `delete()` or new `delete_auth_config()` method
  - `is_owner`/`is_privileged` logic moves from handler to service
- **File**: `crates/routes_app/src/mcps/routes_mcps_auth.rs`
  - Simplify `mcp_auth_configs_destroy` handler

### P2-12: Fix N+1 query in apps_get_access_request_review
- **File**: `crates/routes_app/src/apps/routes_apps.rs`
  - Hoist `auth_scope.tools().list()` and `auth_scope.mcps().list()` calls BEFORE the loops
  - Filter results in-memory per iteration

### P3-7: Add error code assertion to test_tokens_security.rs
- **File**: `crates/routes_app/src/tokens/test_tokens_security.rs`
  - Assert `error.code` in response body (now `token_route_error-privilege_escalation` with 403 status)

### P3-6: Rename test functions to match handler naming convention
- **File**: `crates/routes_app/src/tokens/test_tokens_crud.rs`
  - `test_create_token_handler_success` -> `test_tokens_create_success` (and similar renames)
- **File**: `crates/routes_app/src/mcps/test_oauth_utils.rs`
  - `test_get_oauth_token_handler_success` -> `test_mcp_oauth_tokens_show_success` (and similar)

---

## Commit 5: Documentation + P3 cosmetics

**Test**: `cargo test -p routes_app && cargo test -p services`

### P2-6: Update CLAUDE.md and PACKAGE.md references
- **File**: `crates/services/CLAUDE.md` (lines 20, 89, 115) -- remove `JsonRejectionError` references
- **File**: `crates/services/PACKAGE.md` (lines 17, 349) -- remove `JsonRejectionError`, update axum dep justification to reference `ai_apis` module

### P2-13: Fix stale comment in openapi.rs
- **File**: `crates/routes_app/src/shared/openapi.rs` line 135
  - `routes_mcp/mod.rs` -> `mcps/mod.rs`

### P3-1: Document bind-to-local pattern for sub-service accessors
- **File**: `crates/services/src/app_service/auth_scoped.rs`
  - Add doc comment on `tokens()`, `mcps()`, `tools()`, `users()` advising callers to bind to a local variable to avoid redundant clones

### P3-5: Move endpoint constants from mcps/mod.rs (optional)
- Consider moving to `constants.rs` -- skip if too disruptive given oai/ollama precedent

### P3-8: Document ListUsersParams pagination rationale
- **File**: `crates/routes_app/src/users/users_api_schemas.rs`
  - Add comment explaining `ListUsersParams` intentionally omits sort fields (unlike `PaginationSortParams`)

### P3-9: Cap page number before cast
- **File**: `crates/routes_app/src/users/routes_users_access_request.rs` line 147
  - Add `.min(u32::MAX as usize)` before `as u32` cast

### P3-10: (Resolved by P2-2) toolset_to_response now uses AuthScopedToolService.get_type()

### P3 cosmetic in ollama
- **File**: `crates/routes_app/src/ollama/ollama_api_schemas.rs` line 154
  - `0 as f64` -> `0.0`

---

## Skipped Items

### P2-1: clear_sessions_for_user auth gate -- SKIP
**Reason**: Auth-scoped services are for user-level resource filtering, NOT authorization. Auth_middleware handles authorization. Adding require_token() here conflates concerns.

### P2-4: DefaultTokenService pays for unused error variants -- RESOLVED
**Reason**: Fixed by P1-1. After token generation logic moves into DefaultTokenService, it will use the full TokenServiceError surface.

---

## Verification

After all 5 commits:
1. `cargo test -p services` -- verify token service refactor, error renames
2. `cargo test -p auth_middleware` -- verify middleware fixes
3. `cargo test -p routes_app` -- verify AuthScope migration, error cleanup
4. `make test.backend` -- full backend regression
5. `cargo clippy --workspace` -- no new warnings
6. Grep for `Extension<AuthContext>` in routes_app -- should only appear in oai/ollama forward_request handlers
7. Grep for `DbError` variant names -- should all be `Db` now
8. `make build.ts-client` -- regenerate TS types if API response shapes changed (403 vs 400)
