# Two-Phase Login & Session Architecture

> **Audience**: AI coding assistants working on BodhiApp authentication
> **Scope**: Multi-tenant stage 2 implementation (commits 8aa9acae2..172555fab)
> **Related decisions**: D52-D106 in `decisions.md`

---

## The Problem: Multi-Tenant Authentication

In standalone mode, a single Keycloak resource-client exists. The user logs in, gets tokens for that client, and is done. One user, one client, one pair of tokens in the session.

Multi-tenant mode breaks this model. A single user can belong to multiple tenants (workspaces), each backed by its own Keycloak resource-client. The system needs to answer three questions that did not exist before:

1. **Who is this user?** (platform identity -- independent of any tenant)
2. **Which tenants can they access?** (membership query -- requires platform auth)
3. **Which tenant are they currently using?** (active workspace -- requires resource-client tokens)

A single OAuth flow cannot answer all three. The user's identity must be established before tenant selection can happen, and tenant selection must happen before resource tokens can be obtained.

This led to the two-phase design: one OAuth flow for platform identity (dashboard), another for tenant-scoped authorization (resource-client).

---

## Phase 1: Dashboard Authentication

**Purpose**: Establish who the user is at the platform level. The dashboard OAuth flow authenticates against a special multi-tenant Keycloak client (`BODHI_MULTITENANT_CLIENT_ID`) that has no resource-level permissions.

**What it establishes**:
- The user's identity (`sub` claim from JWT)
- A dashboard access token stored at `dashboard:access_token`
- A dashboard refresh token stored at `dashboard:refresh_token`

**What it does NOT establish**:
- No tenant context (no `active_client_id`)
- No resource-level role (`resource_admin`, `resource_user`, etc.)
- No authorization to access any tenant's data

**Backend flow** (`routes_dashboard_auth.rs`):
1. `POST /bodhi/v1/auth/dashboard/initiate` -- checks multi-tenant mode, generates PKCE + state, stores them in session under `dashboard_`-prefixed keys, returns Keycloak authorization URL using the multi-tenant client
2. Keycloak authenticates the user (password, SSO, etc.) and redirects to `/ui/auth/dashboard/callback`
3. Frontend callback page extracts `code`+`state` from URL, posts to `POST /bodhi/v1/auth/dashboard/callback`
4. Backend validates state, exchanges code for tokens using multi-tenant client credentials, stores tokens under `dashboard:access_token` and `dashboard:refresh_token`, cleans up transient OAuth state, redirects to `/ui/login`

**Early exit**: If a valid `dashboard:access_token` already exists in the session (JWT parses without error), the initiate endpoint returns 200 with a redirect to `/ui/login` instead of starting a new OAuth flow. This prevents unnecessary re-authentication when the user refreshes the page.

**Scope difference**: The dashboard OAuth request asks for `openid email profile` (no `roles` scope). Resource-client OAuth requests `openid email profile roles`. The dashboard token carries identity claims only.

---

## Phase 2: Tenant Selection & Resource OAuth

After Phase 1, the user lands on `/ui/login` with a dashboard session but no active tenant. The system now needs to connect them to a specific workspace.

### The `/info` resolution logic

The `/info` endpoint (`setup_show` in `routes_setup.rs`) is the single source of truth for what the frontend should do. In multi-tenant mode, it pattern-matches on `AuthContext`:

| AuthContext variant | Condition | Status returned |
|---|---|---|
| `MultiTenantSession { client_id: Some(_) }` | Resource token active | `ready` + `client_id` |
| `MultiTenantSession { client_id: None }` | Dashboard only, has local memberships | `tenant_selection` |
| `MultiTenantSession { client_id: None }` | Dashboard only, no memberships | `setup` |
| `Anonymous { deployment: MultiTenant }` | No session at all | `tenant_selection` |

The `setup` status for zero-membership users routes them to tenant registration (`/ui/setup/tenants`). All other dashboard-only states route to `/ui/login` for tenant selection.

### SSO session reuse (why Phase 2 is instant)

This is the key insight that makes the two-phase design feel seamless rather than burdensome. All tenants share a single Keycloak realm (D91). When the user authenticated with the dashboard client in Phase 1, Keycloak created an SSO session cookie in the browser.

When Phase 2 redirects the user to Keycloak for a resource-client OAuth flow, Keycloak sees the existing SSO session cookie and recognizes the user is already authenticated. It skips the login form entirely and immediately issues an authorization code for the resource-client. The browser redirect happens, but the user never sees a login screen.

From the user's perspective: they click a tenant name, there is a brief redirect flash, and they are in. The OAuth redirect chain (`app -> Keycloak -> app`) typically completes in under a second.

**When SSO reuse fails**: If the Keycloak SSO session has expired (configurable server-side, typically 30 minutes of inactivity or 10 hours absolute), the user will see the Keycloak login form again. After re-authenticating, both the SSO session and the resource-client tokens are refreshed.

### Tenant selection UX

The login page fetches `GET /bodhi/v1/tenants` to get the user's tenant list. Each item includes `logged_in: bool` (whether a valid token for that tenant exists in the current session) and `is_active: bool` (whether it matches `active_client_id`).

Three paths depending on the tenant count:

1. **Single tenant**: Auto-login fires immediately. If `logged_in` is true, the page calls the activate endpoint. If false, it initiates resource OAuth. A `useRef` guard prevents React StrictMode double-firing.

2. **Multiple tenants**: A tenant selector card is shown. User clicks a workspace name, which triggers either activation (if `logged_in`) or OAuth initiation (if not).

3. **Zero tenants**: The `/info` endpoint returns `setup` status, and `AppInitializer` redirects to the tenant registration page at `/ui/setup/tenants`.

---

## Session Key Architecture

### Why flat keys were replaced

Before multi-tenant, the session stored a single pair of tokens:
```
access_token  = "eyJ..."
refresh_token = "eyR..."
user_id       = "550e8400-..."
```

This breaks when one user has tokens for multiple tenants simultaneously. If they log into Tenant A and then Tenant B, Tenant B's tokens overwrite Tenant A's. Switching back to Tenant A would require a full re-authentication instead of an instant activation.

### Namespaced design

Decision D56 chose a breaking change: replace flat keys with `{client_id}:` prefixed keys. Existing sessions become invalid (users re-login once). No migration layer.

Current session layout for a multi-tenant user with two workspaces:

```
user_id                                = "550e8400-..."
active_client_id                       = "bodhi-tenant-abc123"
dashboard:access_token                 = "eyJ..." (dashboard JWT)
dashboard:refresh_token                = "eyR..." (dashboard refresh)
bodhi-tenant-abc123:access_token       = "eyJ..." (Tenant A resource JWT)
bodhi-tenant-abc123:refresh_token      = "eyR..." (Tenant A refresh token)
bodhi-tenant-def456:access_token       = "eyJ..." (Tenant B resource JWT)
bodhi-tenant-def456:refresh_token      = "eyR..." (Tenant B refresh token)
```

**Global keys** (not namespaced):
- `user_id` -- same user across all tenants (Keycloak `sub` claim is stable)
- `active_client_id` -- which tenant's tokens should be used for API requests

**Dashboard keys** (fixed prefix):
- `dashboard:access_token`, `dashboard:refresh_token` -- platform-level identity

**Tenant keys** (dynamic prefix):
- `{client_id}:access_token`, `{client_id}:refresh_token` -- per-tenant resource tokens

Helper functions in `services::session_keys` generate the namespaced keys:
- `access_token_key("bodhi-tenant-abc123")` returns `"bodhi-tenant-abc123:access_token"`
- `refresh_token_key("bodhi-tenant-abc123")` returns `"bodhi-tenant-abc123:refresh_token"`

### Why `client_id` and not `tenant_id`

The `client_id` (Keycloak client identifier like `bodhi-tenant-abc123`) is used rather than the internal `tenant_id` (ULID) because `client_id` is what appears in JWT `azp` claims. The middleware reads `azp` from the JWT to resolve the tenant, so using `client_id` for session key namespacing avoids an extra lookup.

---

## Token Lifecycle

### Storage

Tokens are stored server-side in the session store (tower-sessions). The browser holds only the session cookie. This means tokens are never exposed to JavaScript and cannot be stolen via XSS.

### Refresh mechanism

The middleware layer (`DefaultTokenService`) handles token refresh transparently:

1. On each authenticated request, the middleware reads the active tenant's access token
2. If the JWT has expired, it attempts a refresh using the tenant's refresh token and client credentials (looked up from the local tenants table)
3. A distributed lock (`{client_id}:{session_id}:refresh_token`) prevents concurrent refresh attempts for the same tenant/session combination
4. On successful refresh, the new tokens replace the old ones in the session
5. On refresh failure, the session's token data for that tenant is cleaned up, and the request proceeds as unauthenticated

Dashboard tokens follow the same pattern via `get_valid_dashboard_token()`, using `dashboard:refresh_token` and the multi-tenant client credentials.

### What happens when tokens go stale

| Scenario | Behavior |
|---|---|
| Access token expired, refresh token valid | Transparent refresh -- user sees no interruption |
| Access token expired, refresh token expired | Middleware cleans up token data for that tenant. `/info` returns `tenant_selection`. Frontend redirects to login page |
| Dashboard token expired, refresh valid | Middleware refreshes transparently via `try_resolve_dashboard_token()` |
| Dashboard token expired, refresh expired | `AuthContext` becomes `Anonymous`. Frontend shows "Login to Bodhi Platform" button |
| Session cookie expired/missing | Entirely new session. Start from Phase 1 |

---

## Tenant Switching

Two paths, chosen based on whether a valid token already exists in the session for the target tenant.

### Fast path: Activate endpoint

`POST /bodhi/v1/tenants/{client_id}/activate`

When the user has previously logged into a tenant during the same session, the session still holds that tenant's tokens. The activate endpoint simply:
1. Verifies `{client_id}:access_token` exists in the session
2. Sets `active_client_id` to the new `client_id`
3. Returns 200

The next API request will use the newly active tenant's tokens. No OAuth redirect, no Keycloak interaction. This is instant.

The frontend hooks (`useTenantActivate`) invalidate the `tenants`, `appInfo`, and `user` query caches after activation, causing the UI to re-fetch and reflect the new active workspace.

### Slow path: Full re-auth

When the target tenant has no token in the session (`logged_in: false` in the tenant list), the frontend initiates a resource OAuth flow for that tenant's `client_id`. Thanks to SSO session reuse, this typically completes without user interaction (redirect flash only). After the OAuth callback stores the new tokens, the user is in.

### UX flow

The login page presents the currently active tenant and a list of other tenants. Each button is labeled with the tenant name. The `onClick` handler checks `tenant.logged_in`:
- `true` -> call `activateTenant({ client_id })`, then on success initiate OAuth (to ensure token validity and set up the redirect chain)
- `false` -> call `initiateOAuth({ client_id })` directly

---

## Frontend State Machine

The `MultiTenantLoginContent` component manages four visual states. The state is derived from two data sources: `useUser()` (authentication status) and `useTenants()` (tenant membership list).

### State A: No Dashboard Session

**Condition**: `userInfo` does not have `has_dashboard_session`, and no `client_id` in `appInfo`.

**UI**: A single "Login to Bodhi Platform" button that triggers `useDashboardOAuthInitiate`. This starts Phase 1.

### State B1: Single Tenant, Auto-Login Failed

**Condition**: Dashboard session exists, exactly one tenant, but the automatic OAuth initiation failed (error response or network issue).

**UI**: An error message with a manual "Connect to [workspace name]" button. The `autoLoginFailed` state flag distinguishes this from the initial auto-login attempt.

### State B2: Multiple Tenants, Tenant Selector

**Condition**: Dashboard session exists, more than one tenant, no active resource token.

**UI**: A "Select Workspace" card with one button per tenant. Each button triggers either activation (if `logged_in`) or OAuth initiation (if not).

### State C: Fully Authenticated

**Condition**: `userInfo.auth_status === 'logged_in'` AND `appInfo.client_id` is set.

**UI**: "Welcome" card showing the username and active workspace. Buttons for "Go to Home", tenant switching (one per other tenant), and "Log Out".

### How AppInitializer routes to the login page

`AppInitializer` is the gatekeeper. It fetches `/info` and routes based on the `status` field:
- `tenant_selection` -> redirect to `/ui/login`
- `setup` (multi-tenant) -> redirect to `/ui/setup/tenants`
- `ready` -> render children or redirect to default page

The login page itself uses `allowedStatus={['ready', 'tenant_selection']}` so it renders for both states. `MultiTenantLoginContent` then determines which of the four states to show.

### Auto-login guard

When there is exactly one tenant, the login page attempts to log in automatically without showing any UI. A `useRef(false)` guard (`hasAutoLoginTriggered`) ensures this fires only once, even under React StrictMode's double-effect execution. If the auto-login fails, `autoLoginFailed` is set to `true`, and the page falls through to State B1 (manual connect).

---

## Standalone Adaptation

The same session schema works for standalone mode with zero special cases. A standalone deployment has exactly one tenant row (created during the setup flow). The session layout is:

```
user_id                                = "550e8400-..."
active_client_id                       = "bodhi-resource-xyz789"
bodhi-resource-xyz789:access_token     = "eyJ..."
bodhi-resource-xyz789:refresh_token    = "eyR..."
```

No `dashboard:*` keys exist because there is no dashboard OAuth in standalone mode. The middleware's two-step lookup (`active_client_id` -> `{client_id}:access_token`) works identically.

**Unified `POST /auth/initiate`** (D68): The frontend sends `{ client_id }` in the request body for both modes. Standalone gets the `client_id` from the `/info` response. Multi-tenant sends the selected tenant's `client_id`. The backend handler is the same code path -- it looks up the tenant by `client_id`, stores `auth_client_id` in the session for the callback, and generates the OAuth URL.

The `LoginPage` component branches on `appInfo.deployment`: `multi_tenant` renders `MultiTenantLoginContent`, standalone renders `LoginContent`. The standalone `LoginContent` has a simpler flow (no dashboard phase, no tenant selection), but uses the same `useOAuthInitiate` hook with `{ client_id: appInfo.client_id }`.

---

## Security Model

### What the dashboard token grants

The dashboard token authenticates the user's platform identity. It grants access to:
- `GET /bodhi/v1/tenants` -- list tenants the user belongs to
- `POST /bodhi/v1/tenants` -- create a new tenant (one per user, D65)
- `/user/info` -- returns `has_dashboard_session: true` so the frontend knows to show the tenant selector instead of the platform login button

It does NOT grant access to any tenant's data. All data-accessing endpoints (models, chat, tokens, settings, etc.) require a resource-client token via `AuthContext::Session` or `AuthContext::MultiTenantSession { token: Some(_) }`.

### What the resource token grants

The resource-client token carries `resource_access` claims with the user's role for that specific tenant (e.g., `resource_admin`, `resource_user`). This token is required for all tenant-scoped operations. The `api_auth_middleware` checks the role hierarchy before allowing access.

### Principle of least privilege

A user with a dashboard session but no active resource token cannot read or write any tenant data. The `MultiTenantSession { client_id: None, token: None }` auth context only satisfies tenant listing and user info endpoints.

### Session security

- Tokens are stored server-side (session store), never in localStorage or cookies visible to JavaScript
- PKCE is used for all OAuth flows (both dashboard and resource-client)
- CSRF protection via random `state` parameter verified on callback
- Session cookies use `HttpOnly`, `Secure`, and `SameSite` attributes
- Dashboard and resource OAuth use separate callback URLs (`/ui/auth/dashboard/callback` vs `/ui/auth/callback`) and separate session key prefixes, preventing cross-contamination

---

## Edge Cases & Failure Modes

### SSO session timeout

Keycloak SSO sessions have configurable idle and absolute timeouts. If the SSO session expires between Phase 1 and Phase 2 (e.g., user leaves the tab open for hours before selecting a tenant), Phase 2 will show the Keycloak login form. After re-authentication, the flow continues normally. The dashboard token in the session remains valid independently of the SSO session.

### Partial auth states

**Dashboard token but session cleared server-side**: The browser has a session cookie, but the session store no longer has the data (e.g., server restart with in-memory store). The middleware sees no tokens, constructs `Anonymous`, and `/info` returns `tenant_selection`. The user is shown the platform login button.

**Active client_id but missing token**: If `active_client_id` is set but the corresponding `{client_id}:access_token` is missing (session corruption, manual deletion), the middleware falls through to the "no resource token" path. In multi-tenant mode, it checks for a dashboard token and constructs `MultiTenantSession { client_id: None }`. The user sees the tenant selector.

**Token for a deleted tenant**: If the tenant row is removed from the database but the session still holds tokens, `get_tenant_by_client_id(azp)` returns `None`. The middleware treats this as unauthenticated. The user can still see other tenants (if they have a dashboard session) or must re-login.

### Concurrent sessions

Multiple browser tabs share the same session cookie and therefore the same server-side session. If Tab A activates Tenant X while Tab B is using Tenant Y:
- Tab A's `activate` call sets `active_client_id = X`
- Tab B's next API request will use Tenant X's tokens (because `active_client_id` changed)
- Tab B will see data from Tenant X, not Tenant Y

This is a known limitation. The session is a single shared namespace -- there is no per-tab isolation. Future work could use tab-specific session partitioning if this becomes a UX problem.

### Token refresh race conditions

The distributed lock (`{client_id}:{session_id}:refresh_token`) via `ConcurrencyService` prevents two concurrent requests from attempting to refresh the same tenant's tokens simultaneously. One request wins the lock and refreshes; others wait for the lock to release and then read the updated token from the session. This is critical for PostgreSQL deployments where multiple server instances may serve the same session.

### Logout scope

Logout (`POST /bodhi/v1/logout`) destroys the entire session (`session.delete()`). This clears all dashboard tokens, all resource-client tokens for all tenants, and the `active_client_id`. There is no selective logout (e.g., "log out of Tenant A but stay in Tenant B"). D63 deferred selective logout to a future milestone.

### First-time user with zero tenants

A new user who has never created a workspace authenticates via the dashboard, then `GET /bodhi/v1/tenants` returns an empty list. The `/info` endpoint checks `has_memberships()` on the local DB, returns `setup` status, and `AppInitializer` redirects to `/ui/setup/tenants`. After tenant creation, the page automatically initiates resource OAuth with the new `client_id`, completing the entire onboarding in one flow.
