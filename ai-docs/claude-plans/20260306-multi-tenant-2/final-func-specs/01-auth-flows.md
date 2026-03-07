# Multi-Tenant Functional Specification — Auth Flows

> **Scope**: Login (standalone + multi-tenant two-phase), session architecture, middleware token lookup, logout
> **Related specs**: [Index](00-index.md) · [Tenant Management](02-tenant-management.md) · [Info Endpoint](03-info-endpoint.md)
> **Decisions**: D21, D23, D25, D54, D56, D63, D68, D74, D77, D80, D105

---

## Standalone Login

Single-phase resource OAuth. User authenticates directly against the single tenant's Keycloak client.

```
1. GET /bodhi/v1/info → status: "setup" | "resource_admin" | "ready"
2. If setup → setup wizard (POST /bodhi/v1/setup) → creates tenant
3. POST /bodhi/v1/auth/initiate { client_id }  ← client_id from /info response (D68)
   → Generates PKCE challenge + random state
   → Stores in session: auth_client_id, oauth_state, pkce_verifier, callback_url
   → Returns { location: "keycloak_auth_url" } (201)
4. User authenticates at Keycloak → redirected to /ui/auth/callback
5. Frontend extracts code + state from URL params
6. POST /bodhi/v1/auth/callback { code, state }
   → Validates state (CSRF), exchanges code for tokens
   → Stores namespaced session keys (D56)
   → If ResourceAdmin: calls make_resource_admin + set_client_ready
   → Redirects to /ui/chat
7. GET /bodhi/v1/info → status: "ready", client_id: "bodhi-resource-<UUID>"
```

---

## Multi-Tenant Login

Two-phase: dashboard OAuth (Phase 1) → tenant selection → resource OAuth (Phase 2).

### Phase 1: Dashboard Authentication

```
1. GET /bodhi/v1/info → status: "tenant_selection", deployment: "multi_tenant"
2. Frontend shows "Login to Bodhi Platform" button
3. POST /bodhi/v1/auth/dashboard/initiate
   → Checks for existing valid dashboard token (if valid → return 200)
   → Generates PKCE + state for BODHI_MULTITENANT_CLIENT_ID
   → Stores: dashboard_oauth_state, dashboard_pkce_verifier, dashboard_callback_url
   → Returns { location: "keycloak_auth_url" } (201)
4. User authenticates at Keycloak → redirected to /ui/auth/dashboard/callback (D77)
5. Frontend extracts code + state
6. POST /bodhi/v1/auth/dashboard/callback { code, state }
   → Exchanges code using multi-tenant client credentials
   → Stores: session["dashboard:access_token"], session["dashboard:refresh_token"]
   → Returns redirect to /ui/login
```

### Phase 2: Tenant Selection + Resource OAuth

```
7. /ui/login calls GET /bodhi/v1/user → has_dashboard_session: true
8. Frontend calls GET /bodhi/v1/tenants → list of user's resource-clients

   Case A: 0 tenants → redirect to /ui/setup/tenants/ (registration)
   Case B: 1 tenant → auto-initiate resource OAuth (seamless)
   Case C: N tenants → tenant selector dropdown

9. POST /bodhi/v1/auth/initiate { client_id: "bodhi-tenant-<UUID>" }
   → Same flow as standalone auth_initiate
10. Keycloak SSO session reused → instant redirect (no password re-entry)
11. POST /bodhi/v1/auth/callback { code, state }
    → Exchanges code, stores namespaced tokens
    → Sets active_client_id
    → Redirects to /ui/chat
12. GET /bodhi/v1/info → status: "ready", client_id: "bodhi-tenant-<UUID>"
```

---

## Session Architecture

### Session Key Layout

**Multi-tenant session:**
```
dashboard:access_token      ← Dashboard client JWT
dashboard:refresh_token     ← Dashboard client refresh token
active_client_id            ← Currently selected resource-client ID
{client_id_A}:access_token  ← Resource-client A's JWT
{client_id_A}:refresh_token ← Resource-client A's refresh token
{client_id_B}:access_token  ← Resource-client B's JWT (if switched to)
{client_id_B}:refresh_token ← Resource-client B's refresh token
user_id                     ← Keycloak user ID (same across all tenants)
```

**Standalone session:**
```
active_client_id            ← Always the single tenant's client_id
{client_id}:access_token    ← The single resource-client JWT
{client_id}:refresh_token   ← The single resource-client refresh token
user_id                     ← Keycloak user ID
```

**Breaking migration (D56)**: Existing flat keys (`access_token`, `refresh_token`) were replaced with namespaced keys. Existing sessions are treated as unauthenticated — users re-login once.

### Session Key Constants

```rust
// crates/services/src/session_keys.rs
pub const SESSION_KEY_USER_ID: &str = "user_id";
pub const SESSION_KEY_ACTIVE_CLIENT_ID: &str = "active_client_id";
pub const DASHBOARD_ACCESS_TOKEN_KEY: &str = "dashboard:access_token";
pub const DASHBOARD_REFRESH_TOKEN_KEY: &str = "dashboard:refresh_token";

pub fn access_token_key(client_id: &str) -> String {
  format!("{client_id}:access_token")
}
pub fn refresh_token_key(client_id: &str) -> String {
  format!("{client_id}:refresh_token")
}
```

---

## Middleware Token Lookup

The auth middleware resolves `AuthContext` from incoming requests. Three token types supported.

### Session Tokens (2-step)

```
1. Read session["active_client_id"]
   → Missing: Anonymous (optional middleware) or 401 (strict middleware)
2. Read session["{active_client_id}:access_token"]
   → Missing: Anonymous or 401
3. Decode JWT, extract azp claim
4. Resolve tenant: get_tenant_by_client_id(azp)
5. Validate token expiry via get_valid_session_token(&tenant)
   → Expired: attempt refresh using session["{active_client_id}:refresh_token"]
   → Refresh success: update session tokens, continue
   → Refresh fail: Anonymous or 401
6. Extract user_id, username, role from JWT claims
7. Build AuthContext::Session { client_id, tenant_id, user_id, username, role, token }
```

### API Tokens (prefix hash + suffix)

```
1. Extract Bearer token from Authorization header
2. Split by "." → prefix part + client_id suffix
3. Lookup token by prefix: get_api_token_by_prefix(prefix_hash)
4. SHA-256 verify full token against stored hash
5. Parse client_id from suffix → get_tenant_by_client_id(client_id)
6. Build AuthContext::ApiToken { client_id, tenant_id, user_id, role, token }
```

### External App Tokens (aud claim)

```
1. Extract Bearer token (JWT format, not bodhiapp_ prefix)
2. Verify JWT issuer matches BODHI_AUTH_ISSUER (D24)
3. Extract aud claim → get_tenant_by_client_id(aud)
4. Use tenant's client_secret for RFC 8693 token exchange validation
5. Extract access_request_id from JWT claims
6. Build AuthContext::ExternalApp { client_id, tenant_id, user_id, role, ... }
```

### AuthContext Enum

```rust
pub enum AuthContext {
  Anonymous { client_id: Option<String>, tenant_id: Option<String> },
  Session { client_id: String, tenant_id: String, user_id: String, username: String, role: Option<ResourceRole>, token: String },
  ApiToken { client_id: String, tenant_id: String, user_id: String, role: TokenScope, token: String },
  ExternalApp { client_id: String, tenant_id: String, user_id: String, role: Option<UserScope>, token: String, external_app_token: String, app_client_id: String, access_request_id: Option<String> },
}
```

---

## Dashboard Token Refresh

Dashboard tokens are transparently refreshed before SPI proxy calls (D53, D105).

```rust
// crates/routes_app/src/tenants/dashboard_helpers.rs
pub async fn ensure_valid_dashboard_token(
  session: &Session,
  auth_service: &dyn AuthService,
  setting_service: &dyn SettingService,
  time_service: &dyn TimeService,   // D105: uses TimeService, not SystemTime
) -> Result<String, DashboardAuthRouteError>
```

Logic:
1. Read `dashboard:access_token` from session
2. Decode JWT, check `exp` claim against `time_service.utc_now().timestamp()`
3. If valid → return token
4. If expired → read `dashboard:refresh_token` from session
5. Call `auth_service.refresh_token(multi_tenant_client_id, secret, refresh_token)`
6. Update session with new tokens
7. Return new access token
8. If refresh fails → return error (frontend redirects to dashboard re-login)

---

## Logout

### Desired End-State

**Selective logout** (per-tenant):
- Clear `{active_client_id}:access_token` and `{active_client_id}:refresh_token`
- Clear `active_client_id`
- Dashboard token preserved → user returns to tenant selector

**Full logout**:
- Clear all session data including dashboard tokens
- User returns to initial login screen

### Current Implementation

`session.delete()` clears ALL tokens — equivalent to full logout always.

> **TECHDEBT** [D63]: Selective resource-client logout not yet implemented. Currently `session.delete()` clears everything. See [TECHDEBT.md](../TECHDEBT.md).

---

## Auth Endpoint API Reference

### `POST /bodhi/v1/auth/initiate` {#resource-oauth}

Start resource-client OAuth flow. Works in both deployment modes.

**Request:**
```json
{ "client_id": "bodhi-resource-<UUID>" }
```

**Response (201):**
```json
{ "location": "https://keycloak.example.com/realms/bodhi/protocol/openid-connect/auth?client_id=...&redirect_uri=...&response_type=code&scope=openid&state=...&code_challenge=...&code_challenge_method=S256" }
```

**Response (200)** — already authenticated:
```json
{ "location": "/ui/chat" }
```

**Side effects:**
- Stores `auth_client_id`, `oauth_state`, `pkce_verifier`, `callback_url` in session

---

### `POST /bodhi/v1/auth/callback` {#resource-oauth-callback}

Complete resource-client OAuth flow.

**Request:**
```json
{
  "code": "authorization_code",
  "state": "random_state",
  "error": null,
  "error_description": null
}
```

**Response (200):**
```json
{ "location": "/ui/chat" }
```

**Side effects:**
- Sets `{client_id}:access_token`, `{client_id}:refresh_token`, `active_client_id`, `user_id` in session (D74)
- If `ResourceAdmin` status: calls `make_resource_admin()` + `set_client_ready()`
- Cleans up `auth_client_id`, `oauth_state`, `pkce_verifier`, `callback_url` from session

**Errors:**
- `400` — OAuth error in response, state mismatch, missing code
- `500` — Token exchange failure, tenant not found

---

### `POST /bodhi/v1/auth/dashboard/initiate` {#dashboard-oauth}

Start dashboard OAuth flow. **Multi-tenant only** — returns error in standalone (D101).

**Request:** empty body

**Response (201):**
```json
{ "location": "https://keycloak.example.com/realms/bodhi/protocol/openid-connect/auth?client_id=<BODHI_MULTITENANT_CLIENT_ID>&..." }
```

**Response (200)** — valid dashboard token already exists:
```json
{ "location": "/ui/login" }
```

**Side effects:**
- Stores `dashboard_oauth_state`, `dashboard_pkce_verifier`, `dashboard_callback_url` in session

**Errors:**
- `500` — `not_multi_tenant` when `BODHI_DEPLOYMENT != "multi_tenant"`

---

### `POST /bodhi/v1/auth/dashboard/callback` {#dashboard-oauth-callback}

Complete dashboard OAuth flow. **Multi-tenant only**.

**Request:**
```json
{
  "code": "authorization_code",
  "state": "random_state",
  "error": null,
  "error_description": null
}
```

**Response (200):**
```json
{ "location": "/ui/login" }
```

**Side effects:**
- Stores `dashboard:access_token`, `dashboard:refresh_token` in session
- Cleans up `dashboard_oauth_state`, `dashboard_pkce_verifier`, `dashboard_callback_url`

---

### `POST /bodhi/v1/logout` {#logout}

Clear session and logout.

**Request:** empty body

**Response:** Redirect or 200

**Side effects:**
- Calls `session.delete()` — clears ALL session data

---

## Frontend Callback Routes

### `/ui/auth/callback` (Resource OAuth)

Existing page. Handles resource-client OAuth callback.

1. Extracts `code` and `state` from URL query params
2. POSTs to `/bodhi/v1/auth/callback`
3. On success → redirects to `/ui/chat`
4. Uses `useRef` to prevent duplicate submissions (React strict mode)

### `/ui/auth/dashboard/callback` (Dashboard OAuth) {#frontend-dashboard-callback}

New page (D77). Handles dashboard OAuth callback.

1. Extracts `code` and `state` from URL query params
2. POSTs to `/bodhi/v1/auth/dashboard/callback`
3. On success → redirects to `/ui/login`
4. Same duplicate-prevention pattern as resource callback

---

## TECHDEBT

> **TECHDEBT** [D63]: Multi-tenant-aware logout — selective resource-client logout vs full logout not yet implemented. `session.delete()` clears everything. See [TECHDEBT.md](../TECHDEBT.md).

> **TECHDEBT** [D80]: Shared code exchange utility — code exchange logic duplicated between `routes_auth.rs` and `routes_dashboard_auth.rs`. Should be extracted into shared parameterized function. See [TECHDEBT.md](../TECHDEBT.md).
