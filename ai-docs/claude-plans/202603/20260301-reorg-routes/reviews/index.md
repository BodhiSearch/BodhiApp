# Consolidated Review: HEAD~2..HEAD (Routes Reorg + Auth-Scoped Services)

**Date**: 2026-03-02
**Commits**: `d3279cf84..6d559f7b9` (182 files)
**Reviewers**: 8 parallel agents

## Summary

| Priority | Count | Theme |
|----------|-------|-------|
| P0 | 2 | Extension<AuthContext> legacy pattern not migrated (14 handlers) |
| P1 | 7 | Dead code, wrong error types, naming violations, args_delegate regression, duplicate test decls |
| P2 | 14 | Auth gaps, stale docs, business logic in handlers, naming inconsistencies |
| P3 | 10 | Minor cosmetic/convention issues |
| **Total** | **33** | |

---

## P0 -- Critical (must fix before merge)

### P0-1: mcps/ handlers use legacy Extension<AuthContext> pattern
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #1-7, [cross-cutting.md](cross-cutting.md) #5-6,9
**Files**: `crates/routes_app/src/mcps/routes_mcps_auth.rs`, `routes_mcps_oauth.rs`, `routes_mcps_servers.rs`, `routes_mcps.rs`
**Handlers** (13 total):
- `routes_mcps_auth.rs`: `mcp_auth_configs_create`, `mcp_auth_configs_destroy`, `mcp_oauth_token_exchange`, `mcp_auth_configs_index`, `mcp_auth_configs_show`
- `routes_mcps_oauth.rs`: `mcp_oauth_tokens_show`, `mcp_oauth_tokens_destroy`, `mcp_oauth_discover_as`, `mcp_oauth_discover_mcp`, `mcp_oauth_dynamic_register`
- `routes_mcps_servers.rs`: `mcp_servers_create`, `mcp_servers_update`, `mcp_servers_show`, `mcp_servers_index`
- `routes_mcps.rs`: `mcps_fetch_tools`
**Issue**: All use `Extension(auth_context): Extension<AuthContext>` and/or `State(state): State<Arc<dyn RouterState>>` instead of `AuthScope`. None call `forward_request()`, so they don't qualify for the RouterState exception.
**Recommendation**: Migrate all to `AuthScope` extractor.

### P0-2: users/ handlers use legacy Extension<AuthContext> pattern
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #1-2, [cross-cutting.md](cross-cutting.md) #7-8
**Files**: `crates/routes_app/src/users/routes_users_access_request.rs`, `routes_users_info.rs`
**Handlers** (7 total):
- `routes_users_access_request.rs`: `users_request_access`, `users_request_status`, `users_access_requests_pending`, `users_access_requests_index`, `users_access_request_approve`, `users_access_request_reject`
- `routes_users_info.rs`: `users_info`
**Issue**: Same as P0-1 -- legacy `Extension<AuthContext>` + `State(state)` pattern.
**Recommendation**: Migrate all to `AuthScope`. `AuthScope` falls back to `Anonymous` for unauthenticated endpoints.

---

## P1 -- High (should fix)

### P1-1: AuthScopedTokenService bypasses TokenService trait
**Source**: [services-auth-scope.md](services-auth-scope.md) #1, [services-domain.md](services-domain.md) #3
**File**: `crates/services/src/app_service/auth_scoped_tokens.rs`
**Location**: `create_token`, `list_tokens`, `get_token`, `update_token`
**Issue**: Calls `db_service()` directly and inlines token generation logic (SHA-256 hashing, ULID, random bytes). The existing `TokenService` trait has matching methods but is never used. Breaks the service abstraction that mcps/tools/users follow correctly.
**Recommendation**: Move token generation logic into `TokenService::create_token(user_id, name, scope)`. Change `AuthScopedTokenService` to delegate to `app_service.token_service()`.

### P1-2: TokenRouteError has 6 dead variants + 2 dead #[from] conversions
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #8, [cross-cutting.md](cross-cutting.md) #1-3
**File**: `crates/routes_app/src/tokens/error.rs`
**Dead variants**: `AppRegMissing`, `RefreshTokenMissing`, `InvalidScope`, `InvalidRole`
**Dead #[from]**: `Token(#[from] TokenError)`, `AuthService(#[from] AuthServiceError)`
**Issue**: Leftover from deleted `routes_api_token` module. Never constructed by any handler.
**Recommendation**: Remove all 6 dead variants and the 2 dead `#[from]` conversions. Keep only `AccessTokenMissing` and `PrivilegeEscalation`.

### P1-3: TokenRouteError uses BadRequest for auth errors
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #9-10
**File**: `crates/routes_app/src/tokens/error.rs`
**Variants**: `PrivilegeEscalation` (ErrorType::BadRequest), `AccessTokenMissing` (ErrorType::BadRequest)
**Issue**: Privilege escalation and missing auth should be 403 Forbidden, not 400 BadRequest.
**Recommendation**: Change both to `ErrorType::Forbidden`.

### P1-4: LoginError not renamed to AuthRouteError
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #3
**File**: `crates/routes_app/src/auth/error.rs`
**Issue**: All other domains use `<Domain>RouteError` (`ModelRouteError`, `UsersRouteError`, `AppsRouteError`). `LoginError` breaks convention.
**Recommendation**: Rename to `AuthRouteError`. Update error codes in test assertions (`login_error-*` -> `auth_route_error-*`).

### P1-5: args_delegate=false dropped from 3 auth/error.rs variants
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #4-6
**File**: `crates/routes_app/src/auth/error.rs`
**Variants**: `SessionError`, `ParseError`, `SessionDelete`
**Issue**: Old versions had `args_delegate = false`. Without it, the proc macro calls `format!("{}", self.0)` for `args()`, potentially adding an unexpected `param` key to JSON error responses.
**Recommendation**: Add `args_delegate = false` back to all three variants.

### P1-6: Duplicate test module declarations in settings/ and setup/
**Source**: [routes-app-batch-c.md](routes-app-batch-c.md) #1-2
**Files**: `crates/routes_app/src/settings/routes_settings.rs` (lines 215-217), `crates/routes_app/src/setup/routes_setup.rs` (lines 138-140)
**Issue**: Test modules declared in both the handler file AND mod.rs. Compiled twice under different module paths.
**Recommendation**: Remove declarations from handler files, keep only in mod.rs.

### P1-7: apps/ unauthenticated handlers use State(state) instead of AuthScope
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #7
**File**: `crates/routes_app/src/apps/routes_apps.rs`
**Handlers**: `apps_create_access_request` (L55), `apps_get_access_request_status` (L140)
**Issue**: Use `State(state)` directly. CLAUDE.md rule: "All API request flows use AuthScopedAppService." AuthScope falls back to Anonymous for unauthenticated endpoints.
**Recommendation**: Migrate to `AuthScope` for uniform handler signature.

---

## P2 -- Medium (fix in follow-up)

### P2-1: clear_sessions_for_user missing auth gate
**Source**: [services-auth-scope.md](services-auth-scope.md) #2
**File**: `crates/services/src/app_service/auth_scoped_users.rs` (lines 91-101)
**Issue**: Every other method calls `require_token()` or `require_user_id()`. This privileged admin action has no auth check.
**Recommendation**: Add `self.require_token()?;` at method start.

### P2-2: Raw service passthroughs alongside auth-scoped accessors
**Source**: [services-auth-scope.md](services-auth-scope.md) #3
**File**: `crates/services/src/app_service/auth_scoped.rs` (lines 60-128)
**Issue**: `mcp_service()`, `tool_service()`, `token_service()` bypass auth scoping. Handlers can accidentally use unscoped access.
**Recommendation**: Remove passthroughs for domains with auth-scoped counterparts, or document the boundary.

### P2-3: TokenServiceError::Db naming inconsistent
**Source**: [services-domain.md](services-domain.md) #1
**File**: `crates/services/src/tokens/error.rs`
**Issue**: Variant named `Db` while all others use `DbError` (`McpError::DbError`, `ToolsetError::DbError`, etc.). Produces `token_service_error-db` vs `*-db_error`.
**Recommendation**: Rename to `DbError` for consistency, or rename all others to `Db`.

### P2-4: DefaultTokenService pays for unused error variants
**Source**: [services-domain.md](services-domain.md) #2
**File**: `crates/services/src/tokens/token_service.rs`
**Issue**: Returns `TokenServiceError` but only produces `DbError` conversions. `Auth` and `Entity` variants only used by `AuthScopedTokenService`.
**Recommendation**: Accept as trait unification or split return types. Lower priority if P1-1 is fixed (AuthScopedTokenService delegates to TokenService).

### P2-5: Stale comment in services/src/lib.rs
**Source**: [services-infra.md](services-infra.md) #1
**File**: `crates/services/src/lib.rs` (line 75)
**Issue**: Comment says "axum/serde-dependent" but `JsonRejectionError` was moved out.
**Recommendation**: Update to "serde/validator-dependent error types".

### P2-6: CLAUDE.md and PACKAGE.md reference removed JsonRejectionError
**Source**: [services-infra.md](services-infra.md) #2-3
**Files**: `crates/services/CLAUDE.md` (lines 20, 89, 115), `crates/services/PACKAGE.md` (lines 17, 349)
**Issue**: Still list `JsonRejectionError` as living in services.
**Recommendation**: Remove references; update axum dependency justification.

### P2-7: Redundant get_instance() call in auth_middleware
**Source**: [auth-middleware.md](auth-middleware.md) #1
**File**: `crates/auth_middleware/src/auth_middleware/middleware.rs` (lines 145-150)
**Issue**: `instance_client_id` fetched unconditionally before bearer-vs-session branch, but bearer path does its own `get_instance()` internally. Doubles DB lookup for bearer requests.
**Recommendation**: Defer `get_instance()` to session-only branch, or pass `client_id` into token service.

### P2-8: Silent empty client_id in optional_auth_middleware
**Source**: [auth-middleware.md](auth-middleware.md) #2
**File**: `crates/auth_middleware/src/auth_middleware/middleware.rs` (line 265)
**Issue**: `unwrap_or_default()` produces empty string `""` for Session client_id when instance lookup fails. `require_client_id()` succeeds but returns `""`.
**Recommendation**: Return MiddlewareError when instance missing, or fall back to Anonymous.

### P2-9: Business logic in mcps_index handler
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #13
**File**: `crates/routes_app/src/mcps/routes_mcps.rs` (lines 36-62)
**Issue**: ExternalApp filtering logic (extract approved MCP IDs from access request JSON) implemented in handler.
**Recommendation**: Move into `AuthScopedMcpService.list()`.

### P2-10: Ownership check inline in mcp_auth_configs_destroy
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #12
**File**: `crates/routes_app/src/mcps/routes_mcps_auth.rs` (lines 114-149)
**Issue**: `is_owner`/`is_privileged` pattern in handler. Should be service-layer responsibility.
**Recommendation**: Move ownership validation into auth-scoped service.

### P2-11: expect() panics for token extraction in users handlers
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #9
**File**: `crates/routes_app/src/users/routes_users.rs` (lines 75-79, 139-143)
**Issue**: `auth_scope.auth_context().token().expect("requires auth middleware")` panics instead of returning error.
**Recommendation**: Use `auth_scope.require_user_id()?` or return proper error.

### P2-12: N+1 query in apps_get_access_request_review
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #10
**File**: `crates/routes_app/src/apps/routes_apps.rs` (lines 212-230)
**Issue**: `auth_scope.tool_service().list(user_id).await?` called per tool type inside a loop.
**Recommendation**: Hoist query before loop, filter in-memory.

### P2-13: Stale comment in openapi.rs
**Source**: [routes-app-batch-c.md](routes-app-batch-c.md) #3
**File**: `crates/routes_app/src/shared/openapi.rs` (line 135)
**Issue**: References `routes_mcp/mod.rs` instead of `mcps/mod.rs`.
**Recommendation**: Update comment.

### P2-14: routes_dev.rs legacy Extension pattern
**Source**: [routes-app-batch-c.md](routes-app-batch-c.md) #4, [cross-cutting.md](cross-cutting.md) #4
**File**: `crates/routes_app/src/routes_dev.rs` (line 30)
**Issue**: `dev_secrets_handler` uses `Extension<AuthContext>`. Dev-only but touched in diff.
**Recommendation**: Migrate in follow-up. Low urgency.

---

## P3 -- Low (optional cleanup)

### P3-1: Redundant clones on sub-service accessors
**Source**: [services-auth-scope.md](services-auth-scope.md) #4
**File**: `crates/services/src/app_service/auth_scoped.rs`
**Issue**: Each `tokens()`/`mcps()`/etc. call clones Arc + AuthContext. Minor perf concern.
**Recommendation**: Cache sub-services or document bind-to-local pattern.

### P3-2: No Anonymous factory in test_utils
**Source**: [services-infra.md](services-infra.md) #4
**File**: `crates/services/src/test_utils/auth_context.rs`
**Issue**: Missing `test_anonymous()` factory method.
**Recommendation**: Add for completeness.

### P3-3: Unnecessary borrow() in MiddlewareError::From
**Source**: [auth-middleware.md](auth-middleware.md) #3
**File**: `crates/auth_middleware/src/middleware_error.rs` (line 19)
**Issue**: `value.borrow()` on owned value adds unnecessary indirection.
**Recommendation**: Remove borrow, access fields directly on `&value`.

### P3-4: unwrap() on Response builder in MiddlewareError
**Source**: [auth-middleware.md](auth-middleware.md) #4
**File**: `crates/auth_middleware/src/middleware_error.rs` (line 43)
**Issue**: Panics if status code invalid. Unlikely but undefensive.
**Recommendation**: Add fallback 500 response.

### P3-5: Endpoint constants in mcps/mod.rs
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #16
**File**: `crates/routes_app/src/mcps/mod.rs`
**Issue**: Convention says mod.rs = declarations + re-exports only. But precedent exists in oai/ollama.
**Recommendation**: Low priority. Consider moving to constants.rs in future cleanup.

### P3-6: Test functions use _handler suffix
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #17-18
**Files**: `crates/routes_app/src/tokens/test_tokens_crud.rs`, `crates/routes_app/src/mcps/test_oauth_utils.rs`
**Issue**: Test names like `test_create_token_handler_success` don't match Rails-style handler naming (no `_handler`).
**Recommendation**: Rename to e.g. `test_tokens_create_success`.

### P3-7: test_tokens_security.rs only asserts HTTP status
**Source**: [cross-cutting.md](cross-cutting.md) #10
**File**: `crates/routes_app/src/tokens/test_tokens_security.rs`
**Issue**: Doesn't assert error code in response body. Convention is to assert `error.code`.
**Recommendation**: Add error code assertion.

### P3-8: ListUsersParams duplicates pagination fields
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #12
**File**: `crates/routes_app/src/users/users_api_schemas.rs`
**Issue**: Duplicates `page`/`page_size` already in `PaginationSortParams`.
**Recommendation**: Consider reusing `PaginationSortParams`.

### P3-9: Uncapped page number cast
**Source**: [routes-app-batch-b.md](routes-app-batch-b.md) #14
**File**: `crates/routes_app/src/users/routes_users_access_request.rs` (line 147)
**Issue**: `params.page as u32` has no cap, could truncate.
**Recommendation**: Add `.min(max)` before cast.

### P3-10: toolset_to_response uses raw ToolService
**Source**: [routes-app-batch-a.md](routes-app-batch-a.md) #14
**File**: `crates/routes_app/src/toolsets/routes_toolsets.rs` (lines 332-353)
**Issue**: Calls `auth_scope.tool_service()` (unscoped) for type definitions.
**Recommendation**: Document exception or add to auth-scoped service.

---

## Fix Iteration Order

Recommended fix order for AI-consumable iteration:

### Phase 1: AuthScope Migration (P0 + P1-7)
1. Migrate mcps/ handlers (P0-1, 15 handlers)
2. Migrate users/ handlers (P0-2, 7 handlers)
3. Migrate apps/ unauthenticated handlers (P1-7, 2 handlers)
4. Migrate routes_dev.rs (P2-14, 1 handler, optional)

### Phase 2: Error Cleanup (P1-2 through P1-6)
5. Remove dead TokenRouteError variants + #[from] (P1-2)
6. Fix ErrorType::BadRequest -> Forbidden (P1-3)
7. Rename LoginError -> AuthRouteError (P1-4)
8. Restore args_delegate=false on auth/error.rs (P1-5)
9. Remove duplicate test module declarations (P1-6)

### Phase 3: Service Layer (P1-1, P2-1 through P2-4)
10. Refactor AuthScopedTokenService to delegate to TokenService (P1-1)
11. Fix clear_sessions_for_user auth gate (P2-1)
12. Remove raw service passthroughs (P2-2)
13. Fix TokenServiceError::Db naming (P2-3)

### Phase 4: Documentation + Minor (P2-5 through P2-13, P3-*)
14. Update stale comments and docs (P2-5, P2-6, P2-13)
15. Fix middleware issues (P2-7, P2-8)
16. Move business logic to service layer (P2-9, P2-10)
17. All P3 items

---

## Review File Index

| Agent | File | Findings |
|-------|------|----------|
| 1. services-auth-scope | [services-auth-scope.md](services-auth-scope.md) | 4 (1 P1, 2 P2, 1 P3) |
| 2. services-domain | [services-domain.md](services-domain.md) | 4 (0 P1, 2 P2, 2 P3) |
| 3. services-infra | [services-infra.md](services-infra.md) | 4 (0 P1, 3 P2, 1 P3) |
| 4. auth-middleware | [auth-middleware.md](auth-middleware.md) | 4 (0 P1, 2 P2, 2 P3) |
| 5. routes-app-batch-a | [routes-app-batch-a.md](routes-app-batch-a.md) | 18 (7 P0, 3 P1, 4 P2, 4 P3) |
| 6. routes-app-batch-b | [routes-app-batch-b.md](routes-app-batch-b.md) | 11 (2 P0, 4 P1, 5 P2, 1 P3) |
| 7. routes-app-batch-c | [routes-app-batch-c.md](routes-app-batch-c.md) | 6 (0 P0, 2 P1, 2 P2, 2 P3) |
| 8. cross-cutting | [cross-cutting.md](cross-cutting.md) | 10 (0 P0, 2 P1, 7 P2, 1 P3) |
| **Raw total** | | **61** |
| **After dedup** | | **33** |
