# Multi-Tenant Functional Specification — Info Endpoint

> **Scope**: `/info` and `/user/info` state machines, status resolution logic
> **Related specs**: [Index](00-index.md) · [Auth Flows](01-auth-flows.md) · [Tenant Management](02-tenant-management.md)
> **Decisions**: D26, D54, D67, D70, D81, D100, D105

---

## `GET /bodhi/v1/info` {#info}

Returns application status, deployment mode, and active tenant's client_id.

**Auth:** Optional (runs without auth middleware; `AuthScope` falls back to `Anonymous`)

**Response shape:**
```json
{
  "version": "1.0.0",
  "commit_sha": "abc1234",
  "status": "ready",
  "deployment": "multi_tenant",
  "client_id": "bodhi-tenant-a1b2c3d4-..."
}
```

**TypeScript type:**
```typescript
interface AppInfo {
  version: string;
  commit_sha: string;
  status: AppStatus;           // "setup" | "resource_admin" | "ready" | "tenant_selection"
  deployment: string;          // "standalone" | "multi_tenant" (D67)
  client_id?: string | null;   // Active tenant's client_id (D70)
}
```

---

## Standalone Status Resolution

DB-based — determined by tenant count and status in the database.

```
┌─────────────────────────┐
│ Count tenants in DB     │
└────────┬────────────────┘
         │
    ┌────┴────┐
    │ 0 rows  │──────────────────▶ status: Setup
    └────┬────┘                    client_id: null
         │
    ┌────┴────┐
    │ 1 row   │──▶ tenant.status ──▶ status: ResourceAdmin | Ready
    └─────────┘                      client_id: from AuthContext (if authenticated)
```

**Note**: In standalone, `client_id` in the response comes from `AuthContext`. Since `/info` is not behind auth middleware (D54), `AuthContext` is always `Anonymous` in standalone → `client_id` is always `None`.

> The current implementation works correctly for the frontend because standalone uses a single-phase login where `client_id` is obtained from the setup flow, not from `/info`. The `/info` endpoint's `client_id` field is primarily useful in multi-tenant mode.

---

## Multi-Tenant Status Resolution

Session-based — determined by session state (dashboard token, active tenant token).

### `resolve_multi_tenant_status()` Decision Tree

```
┌──────────────────────────────────────────┐
│ Check session["active_client_id"]        │
│ + session["{active_client_id}:access_token"] │
└────────┬─────────────────────────────────┘
         │
    ┌────┴──────────────┐
    │ Both exist?       │
    │ YES               │──────────────────▶ status: Ready
    └────┬──────────────┘                    client_id: active_client_id
         │ NO
    ┌────┴──────────────────────────┐
    │ Check dashboard:access_token  │
    └────────┬──────────────────────┘
             │
        ┌────┴──────────┐
        │ Missing?      │
        │ YES           │──────────────────▶ status: TenantSelection
        └────┬──────────┘                    client_id: null
             │ NO
        ┌────┴──────────────────────────┐
        │ ensure_valid_dashboard_token() │
        │ (refresh if expired) (D105)    │
        └────────┬──────────────────────┘
                 │
            ┌────┴──────────┐
            │ Refresh       │
            │ failed?       │──────────────▶ status: TenantSelection
            └────┬──────────┘                client_id: null
                 │ OK
            ┌────┴──────────────────┐
            │ SPI: list_tenants()   │
            └────────┬──────────────┘
                     │
                ┌────┴──────────┐
                │ 0 tenants     │──────────▶ status: Setup
                └────┬──────────┘            client_id: null
                     │
                ┌────┴──────────┐
                │ 1+ tenants    │──────────▶ status: TenantSelection
                └────┬──────────┘            client_id: null
                     │
                ┌────┴──────────┐
                │ SPI error     │──────────▶ status: TenantSelection
                └───────────────┘            client_id: null
```

**Key behaviors:**
- `Setup` means "user has dashboard session, but zero tenants" → frontend routes to `/ui/setup/tenants/`
- `TenantSelection` means "user needs to select or login to a tenant" → frontend routes to `/ui/login`
- `Ready` means "user has an active, authenticated tenant" → frontend routes to app
- SPI errors gracefully degrade to `TenantSelection` (user can retry)

---

## `GET /bodhi/v1/user` {#user-info}

Returns current user's authentication state plus dashboard session indicator.

**Auth:** Optional

**Response shapes:**

**Logged out (Anonymous):**
```json
{
  "status": "logged_out",
  "has_dashboard_session": true
}
```
Note: `has_dashboard_session` is only present when `true` (via `skip_serializing_if`). When `false`, the field is omitted for backward compatibility (D100).

**Logged in (Session):**
```json
{
  "status": "logged_in",
  "user_id": "keycloak-user-uuid",
  "username": "john@example.com",
  "role": "resource_power_user",
  "has_dashboard_session": true
}
```

**API token:**
```json
{
  "status": "api_token",
  "role": "scope_token_power_user"
}
```

### TypeScript types

```typescript
type UserResponse =
  | { status: "logged_out" }
  | { status: "logged_in"; user_id: string; username: string; role: string }
  | { status: "api_token"; role: string };

interface UserInfoEnvelope extends UserResponse {
  has_dashboard_session?: boolean;  // Only present when true
}
```

### Rust types

```rust
// crates/routes_app/src/users/routes_users_info.rs
pub struct UserInfoEnvelope {
  #[serde(flatten)]
  pub user: UserResponse,
  #[serde(default, skip_serializing_if = "is_false")]
  pub has_dashboard_session: bool,
}

pub enum UserResponse {
  #[serde(rename = "logged_out")]
  LoggedOut,
  #[serde(rename = "logged_in")]
  LoggedIn(UserInfo),
  #[serde(rename = "api_token")]
  Token(TokenInfo),
}
```

### Dashboard session detection

The handler reads `dashboard:access_token` from session independently of `AuthContext` (D81). This means:
- A user can be `logged_out` (no active resource token) but have `has_dashboard_session: true`
- The frontend uses this to decide whether to show "Login to Bodhi Platform" (no dashboard session) vs tenant selector (has dashboard session)

---

## Frontend Consumption

### `useAppInfo()` Hook

```typescript
const { data: appInfo } = useAppInfo();
// appInfo.status: AppStatus
// appInfo.deployment: "standalone" | "multi_tenant"
// appInfo.client_id: string | null
```

### `useUser()` Hook

```typescript
const { data: userInfo } = useUser();
// userInfo.status: "logged_out" | "logged_in" | "api_token"
// userInfo.has_dashboard_session?: boolean
```

### AppInitializer Routing Logic

The `AppInitializer` component gates page access by status:

```typescript
<AppInitializer allowedStatus="ready">        // Most pages
<AppInitializer allowedStatus="setup">         // Setup pages
<AppInitializer allowedStatus="resource_admin"> // Resource admin page
<AppInitializer allowedStatus={['ready', 'tenant_selection']}>  // Login page
```

**Redirect logic when status doesn't match `allowedStatus`:**

| Status | Deployment | Redirect to |
|--------|-----------|-------------|
| `setup` | `standalone` | `/ui/setup` (setup wizard) |
| `setup` | `multi_tenant` | `/ui/setup/tenants` (tenant registration) |
| `resource_admin` | Any | `/ui/setup/resource-admin` |
| `tenant_selection` | `multi_tenant` | `/ui/login` |
| `ready` | Any | `/ui/chat` (or current page) |
