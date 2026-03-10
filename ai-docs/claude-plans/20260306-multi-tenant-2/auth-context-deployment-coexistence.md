# AuthContext Domain Model & Deployment Coexistence

> **Purpose**: Cross-cutting analysis of how BodhiApp's AuthContext enum and middleware chain enable standalone and multi-tenant deployments to share unified code paths.
> **Audience**: AI coding assistants and developers extending auth, middleware, or deployment features.

---

## Design Problem

BodhiApp ships as a single binary that runs in two fundamentally different deployment modes:

- **Standalone**: A personal AI server with one OAuth2 client registration (one "tenant"). The user who sets it up is the admin. There is no concept of choosing which tenant to work with.
- **Multi-tenant**: A SaaS deployment where many users each register their own OAuth2 client. Users authenticate against a "dashboard" client first, then select (or create) a resource tenant to work within.

The naive solution would be to branch on deployment mode throughout the codebase: `if multi_tenant { ... } else { ... }` in every route handler, middleware function, and service method. This creates a combinatorial explosion of code paths that are difficult to test and easy to break.

The design goal was to make standalone behave as a degenerate case of multi-tenant (one tenant, no dashboard token, no tenant selection) so that the vast majority of code paths are shared. Deployment-specific behavior is confined to two narrow places: the middleware layer (which constructs the right AuthContext variant) and a handful of route handlers that must present different UI state (like `/info`).

---

## The MultiTenantSession Variant

The core insight (from the architecture refactor plan) is that **the AuthContext variant type itself encodes the deployment mode**. There is no need for a `deployment: DeploymentMode` field on most variants because the variant name carries the information.

`MultiTenantSession` exists to model a state that has no standalone equivalent: **a user who is authenticated at the platform level (via a dashboard JWT) but may or may not have an active resource tenant**. This is a "partially authenticated" state -- the user has proven their identity, but the system does not yet know which tenant's data they want to access.

The variant's fields reflect this progressive authentication:

- `dashboard_token: String` -- always present; this is what proves the user is who they claim to be. Validated and refreshed by middleware before the handler ever sees it.
- `client_id: Option<String>` and `tenant_id: Option<String>` -- `None` when the user is in "dashboard-only" mode (browsing tenants, creating a new one). `Some` when they have selected and authenticated against a specific resource tenant.
- `token: Option<String>` -- the resource tenant's access token. `None` in dashboard-only mode, `Some` after resource-client OAuth login.
- `role: Option<ResourceRole>` -- `None` in dashboard-only mode (no tenant context means no role), `Some` when working within a tenant.

By contrast, `Session` models the standalone case where `client_id`, `tenant_id`, and `token` are always present (non-Optional). There is no dashboard concept, so there is no partial state.

**Why not unify Session and MultiTenantSession with Options everywhere?** Because that would weaken the type system. In standalone mode, a `Session` variant guarantees that `client_id` and `token` exist -- no handler needs to check. `MultiTenantSession` explicitly communicates "these might be absent" and forces handlers to account for it. The match arms in Rust make this safe and exhaustive.

---

## Unified Code Path Philosophy

Decision D23 establishes the principle: **never branch on deployment mode in middleware**. The middleware always resolves tenant from JWT claims (`azp` for sessions, `aud` for external tokens, `client_id` suffix for API tokens). This works identically whether there is one tenant or a hundred.

The places where standalone and multi-tenant converge:

1. **Tenant resolution**: All authenticated requests resolve the tenant from the token itself, not from a "get the one tenant" lookup. `get_tenant_by_client_id(azp)` works the same way with one row or many rows in the tenants table.

2. **Role-based authorization** (`api_auth_middleware`): The authorization middleware pattern-matches both `Session { role: Some(role) }` and `MultiTenantSession { role: Some(role) }` in the same match arm. From the authorization layer's perspective, the deployment mode is invisible -- it only cares about the role.

3. **Auth-scoped services**: `AuthScopedAppService` wraps `AuthContext` and auto-injects `tenant_id` and `user_id` into service calls. Whether the context is `Session` or `MultiTenantSession`, `require_tenant_id()` and `require_user_id()` work the same way. The auth-scoped pattern means route handlers never need to know which deployment mode they are in for standard CRUD operations.

4. **Bearer token paths**: `ApiToken` and `ExternalApp` variants are deployment-agnostic. API tokens and external app tokens work identically in both modes because they always carry a resolved `client_id` and `tenant_id`.

The places where they diverge:

1. **Middleware variant construction**: After resolving the session token, the middleware checks `is_multi_tenant` to decide whether to construct `Session` or `MultiTenantSession`. This is the primary branching point and it is confined to exactly one place in the codebase.

2. **Dashboard token handling**: Only multi-tenant mode has a dashboard token. The middleware's `try_resolve_dashboard_token()` helper is only called when `is_multi_tenant` is true. In strict auth_middleware, a missing dashboard token is a hard 401. In optional_auth_middleware, it falls back to Anonymous.

3. **`/info` endpoint**: Must return different `AppStatus` values depending on the AuthContext variant. A `MultiTenantSession` with no `client_id` triggers a local membership check to decide between `TenantSelection` and `Setup`. A standalone `Anonymous` looks up the single tenant's status. This is one of the few handlers that explicitly matches on all variants.

4. **Setup flow**: `setup_create` is explicitly standalone-only (`is_multi_tenant -> return error`). Multi-tenant tenants are created through the tenant management endpoints instead.

---

## Middleware Architecture

The middleware chain has three layers, applied in a specific order in `routes.rs`:

### Layer 1: Authentication (auth_middleware / optional_auth_middleware)

These two functions share the same logic but differ in failure behavior:

- `auth_middleware` (applied to `protected_apis`): Returns `401 AuthError::InvalidAccess` when no valid auth is found. Used for all CRUD and API endpoints.
- `optional_auth_middleware` (applied to `optional_auth` routes): Falls back to `AuthContext::Anonymous` on any auth failure. Used for `/info`, `/user/info`, auth initiate/callback, tenant listing, and dev endpoints.

Both follow the same resolution order:
1. Check for `Authorization` header (bearer token) -> delegates to `DefaultTokenService::validate_bearer_token()`
2. Check for same-origin session -> two-step lookup: read `active_client_id`, then read namespaced `{client_id}:access_token`
3. (Multi-tenant only, optional middleware only) Check for dashboard-only session -> `try_resolve_dashboard_token()`
4. Fallback: reject (strict) or Anonymous (optional)

The two-step session lookup is a key multi-tenant enabler. Session keys are namespaced by client_id (`{client_id}:access_token`, `{client_id}:refresh_token`), so a single browser session can hold tokens for multiple tenants. The `active_client_id` marker says which one is "current". This namespacing is invisible in standalone mode -- there is only one client_id, so there is only one set of keys.

### Layer 2: Authorization (api_auth_middleware)

Applied per-route-group via `route_layer`. Reads the `AuthContext` from request extensions (populated by Layer 1) and checks the role hierarchy. The key design choice: `Session` and `MultiTenantSession` are handled in the same match arm because both carry `Option<ResourceRole>`. This means adding a new session-like variant does not require changing authorization logic, as long as it provides `role`.

Authorization is parameterized per route group: `ResourceRole::User` for read endpoints, `ResourceRole::PowerUser` for model management, `ResourceRole::Admin` for settings, `ResourceRole::Manager` for user management. Some route groups also specify `TokenScope` and `UserScope` for API token and external app access control.

### Layer 3: Entity-level access (access_request_auth_middleware)

Applied only to toolset and MCP execution endpoints. Validates that external apps have approved access to the specific entity being accessed. Session users pass through unconditionally.

### The cooperation pattern

The layers compose cleanly because each one reads from and writes to request extensions:
- Layer 1 **writes** `AuthContext` to extensions
- Layer 2 **reads** `AuthContext` from extensions, passes request through or rejects
- Layer 3 **reads** `AuthContext` from extensions, validates entity access

No layer needs to know how the previous one worked. The `AuthContext` enum is the contract between them.

### Dashboard token refresh in middleware

A significant design choice: dashboard tokens are validated and refreshed **in the middleware**, not in route handlers. Before the architecture refactor, route handlers called `ensure_valid_dashboard_token()` themselves, which meant every multi-tenant handler had to deal with token refresh logic. Now, by the time a handler receives a `MultiTenantSession`, the `dashboard_token` field is guaranteed to be valid (or the request would have been rejected/downgraded to Anonymous).

This uses the same distributed lock pattern as resource token refresh: `DefaultTokenService::get_valid_dashboard_token()` checks expiry, acquires a per-session lock, double-checks (another concurrent request may have refreshed), refreshes if needed, and updates the session.

---

## DeploymentMode & Anonymous Variant

`DeploymentMode` is a simple two-value enum (`Standalone` | `MultiTenant`) defined in `tenant_objs.rs`, resolved from the `BODHI_DEPLOYMENT` setting. It appears in exactly one AuthContext variant: `Anonymous`.

Why Anonymous carries `deployment` (D26): Anonymous is the "no auth information" state. Route handlers that receive Anonymous need to know the deployment mode to present the right UI state. For example, `/info` returns `AppStatus::TenantSelection` for an anonymous multi-tenant request (prompting login) but looks up the standalone tenant's status for an anonymous standalone request. Without the `deployment` field, the handler would need to query `SettingService` asynchronously, adding complexity and a service dependency to what should be a simple status check.

The `is_multi_tenant()` method on `AuthContext` demonstrates the pattern:
- `MultiTenantSession` -> always `true` (the variant name encodes it)
- `Anonymous { deployment }` -> checks the `deployment` field
- All others -> `false`

This means route handlers can call `auth_context.is_multi_tenant()` synchronously instead of `settings.is_multi_tenant().await`. The async call is made once, in the middleware, and the result is encoded into the AuthContext. This is the "mode awareness without branching" pattern: the information flows through the type system, not through if/else checks scattered across handlers.

For authenticated variants (`Session`, `ApiToken`, `ExternalApp`), deployment mode is not stored because it is not needed. These variants always have a resolved tenant, so the handler can operate on that tenant regardless of whether the deployment has one tenant or many. The variant type (`Session` vs `MultiTenantSession`) provides the deployment signal when handlers need it (which is rare outside of `/info` and tenant management).

---

## AppStatus State Machine

`AppStatus` has four values that represent the application lifecycle:

| Status | Standalone meaning | Multi-tenant meaning |
|--------|-------------------|---------------------|
| `Setup` | No tenant registered yet. Show setup wizard. | Dashboard-only user with no memberships. Show "create tenant" flow. |
| `ResourceAdmin` | Tenant registered, awaiting first admin login. | Not used (tenants created Ready). |
| `Ready` | Fully operational. | User has selected and authenticated against a tenant. |
| `TenantSelection` | Not used. | User is authenticated (dashboard) but must choose a tenant. |

`TenantSelection` is the multi-tenant-only state that bridges the gap between "platform-authenticated" and "tenant-authenticated". It is returned by `/info` when the AuthContext is either `MultiTenantSession { client_id: None }` with existing memberships, or `Anonymous { deployment: MultiTenant }`.

The status drives frontend routing:
- `Setup` -> setup wizard or create-tenant page
- `ResourceAdmin` -> login prompt
- `TenantSelection` -> tenant picker
- `Ready` -> main application

The `/info` handler resolves status purely from local state (AuthContext + local DB membership check). This was a deliberate move away from the earlier design that called the auth server's SPI to list tenants on every `/info` request -- too expensive for a bootstrap endpoint that the frontend polls.

---

## Extension Points

### Adding a new deployment mode

1. Add the variant to `DeploymentMode` enum in `tenant_objs.rs`.
2. Update `SettingService::deployment_mode()` to parse the new setting value.
3. Decide: does this mode need its own `AuthContext` variant? If it has distinct authentication state (like multi-tenant's dashboard token), add a new variant. If it is a behavioral variation of an existing mode, add it as a field or configuration.
4. Update the middleware's session branch to construct the appropriate AuthContext for the new mode.
5. Update `is_multi_tenant()` (or add a new predicate) on AuthContext.
6. Update the `/info` handler's match arms.
7. Authorization middleware (`api_auth_middleware`) should NOT need changes if the new variant provides `role` in a matchable pattern.

### Adding a new AuthContext capability

For example, adding a "team" concept within a tenant:

1. If the capability adds new state, decide whether it warrants a new variant or a new field on an existing variant. Prefer fields for additive information; prefer variants for fundamentally different authentication states.
2. Add convenience methods to `AuthContext` (e.g., `team_id() -> Option<&str>`).
3. If the new state comes from a JWT claim or session data, update the middleware to extract and populate it.
4. Auth-scoped services that need the new state can access it through `AuthContext` methods.

### Adding a new auth mechanism

For example, mTLS client certificates:

1. Add a new variant to `AuthContext` with the fields the certificate provides.
2. Add a new branch in `auth_middleware` before the session fallback (the resolution order matters -- bearer tokens are checked before sessions).
3. Add match arms in `api_auth_middleware` for the new variant's authorization model.
4. All existing route handlers that use convenience methods (`require_user_id()`, `require_tenant_id()`) will continue to work if the new variant implements those methods.

---

## Key Invariants

1. **AuthContext is always present after middleware**: Every request that passes through `auth_middleware` or `optional_auth_middleware` has an `AuthContext` in its extensions. Handlers can rely on `AuthScope` extraction succeeding.

2. **Tenant resolution is token-derived, never position-based**: No middleware path calls `get_standalone_app()`. All tenant resolution goes through `get_tenant_by_client_id()` using a claim from the token itself. This guarantees that adding tenants never breaks existing resolution.

3. **Dashboard tokens in MultiTenantSession are valid**: By the time a handler receives a `MultiTenantSession`, the `dashboard_token` has been validated and refreshed by middleware. Handlers never need to refresh it themselves.

4. **Non-Optional fields are guaranteed present**: `Session.client_id`, `Session.token`, `ApiToken.client_id` etc. are non-Optional. Code that receives these variants can use the values directly without unwrapping. `MultiTenantSession` makes the optionality of tenant context explicit at the type level.

5. **Authorization is variant-agnostic for role checks**: `api_auth_middleware` groups `Session` and `MultiTenantSession` in the same match arm for role checking. Any new session-like variant that provides `Option<ResourceRole>` integrates automatically.

6. **Anonymous never has user_id**: `user_id()` returns `None` for `Anonymous`. `require_user_id()` returns `AuthContextError::AnonymousNotAllowed` (403). This is the boundary between "browsing" and "acting".

7. **is_multi_tenant() is synchronous**: Once middleware has run, deployment mode is encoded in the AuthContext. No async settings lookup needed in handlers. This was an explicit design goal to keep handlers simple and testable.

8. **Session keys are namespaced by client_id**: `{client_id}:access_token` and `{client_id}:refresh_token`. This means switching tenants in multi-tenant mode does not destroy the previous tenant's tokens. The `active_client_id` session key marks which tenant is current.

9. **Middleware never checks setup status**: Authentication middleware does authentication only (D28). Setup gating is the responsibility of the setup route handlers themselves, using `standalone_app_status_or_default()`. This separation means middleware changes never accidentally break the setup flow.

10. **MiddlewareError has blanket From<T: AppError>**: Any service error that implements `AppError` can be returned from middleware without explicit conversion. This keeps the middleware code focused on logic rather than error mapping.
