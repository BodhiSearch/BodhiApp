# Multi-Tenant Middleware Refactor -- Kickoff

> **Created**: 2026-03-06
> **Status**: ✅ COMPLETED
> **Prior work**: `ai-docs/claude-plans/20260303-multi-tenant/` (Phase 1-2, 4-5 complete -- 48 isolation tests across 7 domains)
> **Decisions from interview**: `ai-docs/claude-plans/20260306-multi-tenant-2/decisions.md` (D21-D28)

---

## Context

We have been implementing multi-tenant support for BodhiApp. The prior plan (see `ai-docs/claude-plans/20260303-multi-tenant/`) completed infrastructure and isolation tests for data-layer domains (tokens, toolsets, MCPs, API models, MCP servers, downloads). Those tests work because they inject `AuthContext` directly via `RequestAuthContextExt::with_auth_context()`, bypassing the auth middleware entirely.

Phase 3 -- middleware integration tests -- is **blocked**. The root cause: the auth middleware calls `tenant_service.get_standalone_app()` which internally calls `db_service.get_tenant()`. When more than one tenant row exists in the database, this returns `Err(TenantError::MultipleTenant)`. Any test that creates two tenants (required for isolation testing) cannot exercise the middleware.

## Problem Statement

The middleware currently assumes a single-tenant world. It needs to resolve the correct tenant from the incoming request's authentication tokens rather than looking up "the one tenant" in the database. Once this is done, the middleware works identically for standalone (1 tenant) and multi-tenant (N tenants) deployments.

## Key Files to Explore

Read these files to understand the current implementation:

- `crates/routes_app/src/middleware/auth/auth_middleware.rs` -- the two middleware functions (`auth_middleware` and `optional_auth_middleware`)
- `crates/routes_app/src/middleware/token_service/token_service.rs` -- `validate_bearer_token`, `handle_external_client_token`, `get_valid_session_token`
- `crates/routes_app/src/middleware/utils.rs` -- `app_status_or_default` helper
- `crates/routes_app/src/middleware/CLAUDE.md` and `crates/routes_app/src/middleware/PACKAGE.md` -- middleware documentation
- `crates/services/src/shared_objs/token.rs` -- JWT claim structs (`Claims`, `ScopeClaims`, `UserIdClaims`, `ExpClaims`) and `extract_claims` function
- `crates/services/src/tenants/tenant_service.rs` -- `TenantService` trait, especially `get_standalone_app()` vs `get_tenant_by_client_id()`
- `crates/routes_app/src/middleware/auth/test_auth_middleware.rs` -- existing middleware tests
- `crates/routes_app/src/middleware/token_service/test_token_service.rs` -- existing token service tests

For test patterns from the prior phase, reference:
- `crates/routes_app/src/tokens/test_tokens_isolation.rs` -- template isolation test pattern with `isolation_router()`

## What Already Works (Do Not Break)

- The **API token path** (`bodhiapp_*` prefix) in `validate_bearer_token` already resolves tenant from the token's `client_id` suffix via `get_tenant_by_client_id()`. This is the model to follow.
- All 48 isolation tests from Phase 2/4/5 must continue passing.
- All existing middleware tests must continue passing (with necessary adaptations).

## Architectural Decisions (D21-D28)

These were established through an interview process. See `decisions.md` for full rationale.

**D21 -- JWT-only tenant resolution**: Resolve tenant from JWT claims. No cookie-based tenant switching for now.

**D22 -- Token-based resolution strategy**:
- Session auth: JWT `azp` (authorized party) claim identifies the tenant's `client_id`
- External app (3rd party JWT): JWT `aud` (audience) claim identifies the target tenant's `client_id`
- API token (`bodhiapp_*`): suffix after last `.` is the `client_id` (already implemented)
- No auth: `AuthContext::Anonymous { client_id: None, tenant_id: None }`

**D23 -- Unified code path**: No branching on `BODHI_DEPLOYMENT` setting in middleware. The token-based resolution works identically for standalone (1 tenant) and multi-tenant (N tenants). Exception: when tenant resolution fails (client_id not found in DB), standalone mode may need to handle the "no tenants yet" (Setup) case differently from multi-tenant.

**D24 -- Trust JWT `aud` after issuer check**: For external app tokens, the `aud` claim is trustworthy after validating the JWT issuer matches our configured Keycloak.

**D25 -- Expired JWT claims are safe for tenant resolution**: `extract_claims` does raw base64 decode without expiry check. Expired tokens are still cryptographically signed.

**D26 -- Anonymous = None/None**: Optional auth middleware falls back to `Anonymous { client_id: None, tenant_id: None }` when no auth is present.

**D27 -- Access request flow unchanged**: Once tenant is resolved from JWT `aud`, the existing access_request lookup by `(tenant_id, scope)` works correctly.

**D28 -- Middleware-only scope**: Only fix `get_standalone_app()` calls within the middleware layer (4 calls in `auth_middleware.rs` and `token_service.rs`, plus `utils.rs` if needed). Other usages in auth/setup/apps/dev routes are out of scope.

## Functional Outcomes

1. **`auth_middleware` (strict)**: Should resolve the correct tenant from the incoming token (session JWT, bearer JWT, or API token) and build `AuthContext` with that tenant's `client_id` and `tenant_id`. Should fail with appropriate errors when no valid auth or tenant not found.

2. **`optional_auth_middleware` (permissive)**: Same tenant resolution logic, but falls back to `Anonymous { client_id: None, tenant_id: None }` on any failure instead of returning errors.

3. **`handle_external_client_token`**: Should resolve tenant from the JWT's `aud` claim instead of `get_standalone_app()`. Uses the resolved tenant's `client_id` and `client_secret` for RFC 8693 token exchange.

4. **`get_valid_session_token`**: Should NOT resolve tenant internally. The caller (middleware) already resolved the tenant from the JWT's `azp` claim -- pass it in so this function can use the tenant's credentials for token refresh.

5. **Setup status check**: Explore how the current Setup status check (pre-auth rejection when tenant is in Setup status) should evolve. The middleware currently checks status BEFORE any token parsing. With token-based resolution, the check can only happen AFTER resolving a tenant. Consider whether the middleware should check status at all, or whether setup routes handle this independently (they already call `app_status_or_default`).

6. **Existing tests**: All existing middleware and token service tests should continue to pass. These tests create a single tenant with `TEST_CLIENT_ID` and use JWTs with `azp: TEST_CLIENT_ID` -- the new lookup path should find the same tenant.

7. **New isolation tests**: Add tests verifying that the middleware correctly resolves different tenants from different tokens. Follow the `isolation_router()` pattern from `test_tokens_isolation.rs`.

## Recommendations and Caveats

- **Explore `app_status_or_default` callers** before deciding whether to change it. It's used in setup and auth routes (out of scope), so changes may have wider blast radius. Consider leaving it as-is if only middleware needs updating.

- **Explore the existing test setup patterns** in `test_auth_middleware.rs`. Tests use `AppServiceStubBuilder` with `with_tenant()` which creates real `DefaultTenantService` backed by a real SQLite DB. The `get_tenant_by_client_id()` method on `DefaultTenantService` should work without any service-layer changes. Verify this.

- **Explore whether `test_auth_middleware_returns_app_status_invalid_for_app_status_setup_or_missing` needs redesign**. This test sends requests with NO auth token. If the Setup check moves to after tenant resolution (which requires a JWT), this test's expectations may change. Use `AskUserQuestion` to clarify if uncertain about the desired behavior.

- **Explore the `handle_external_client_token` audience validation**. Currently it validates `claims.aud == instance.client_id` (equality check against the single tenant). In the new flow, `claims.aud` IS the lookup key for finding the tenant. The equality check becomes implicit in the lookup. Verify no other validation is lost.

- **Use `AskUserQuestion` to clarify any ambiguity** about edge cases, error handling, or test expectations before implementing. The decisions above provide direction but not implementation details for every scenario.

## Gate Checks

After implementation:
1. `cargo check -p routes_app` -- must compile
2. `cargo test -p routes_app -- middleware` -- existing middleware tests pass
3. `cargo test -p routes_app -- isolation` -- all isolation tests pass (existing + new)
4. `cargo test -p routes_app` -- full crate regression
5. `cargo test -p services -p server_core -p routes_app -p server_app --lib` -- cross-crate regression
