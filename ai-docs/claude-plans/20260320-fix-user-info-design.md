# Refactor: Replace `has_dashboard_session` with `dashboard: Option<DashboardUser>`

## Context

The `/bodhi/v1/user` endpoint's `UserInfoEnvelope` currently has `has_dashboard_session: bool` — a flat flag that only indicates dashboard session presence. This has two problems:

1. **Semantic mixing**: When `MultiTenantSession` has `token: None` (dashboard authenticated but no resource login), the handler returns `UserResponse::LoggedIn` with partial user info from the dashboard context. This conflates dashboard identity with resource/tenant identity — `UserResponse` should purely represent tenant login state.

2. **No dashboard user info**: The `has_dashboard_session` flag carries no user identity. Future features need dashboard user details (name, email) without a separate API call.

**Goal**: Replace `has_dashboard_session: bool` with `dashboard: Option<DashboardUser>`, cleanly separate dashboard vs resource auth in the response, and make `UserResponse` purely represent tenant/resource login.

## Design Decisions (confirmed with user)

- **Identity**: Dashboard and resource user are always the same identity
- **Frontend**: Use `!!userInfo?.dashboard` as boolean guard only — no UI display of dashboard user info yet
- **DashboardUser fields**: `user_id`, `username`, `first_name: Option<String>`, `last_name: Option<String>` (OIDC claims are genuinely optional)
- **No deployment_mode** in user response — frontend already gets it from `/bodhi/v1/info`
- **No expiry info** in DashboardUser
- **MultiTenantSession with token: None** → return `UserResponse::LoggedOut` (not `LoggedIn` with partial info)
- **Auth guard**: `!!userInfo?.dashboard` is sufficient (no `isAuthenticated` check needed — dashboard presence implies validated session)

---

## Changes

### 1. Backend: Add `DashboardUser` struct and update `UserInfoEnvelope`

**File**: `crates/routes_app/src/users/users_api_schemas.rs`

Add `DashboardUser` struct:
```rust
/// Dashboard user information from a validated dashboard session token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DashboardUser {
  pub user_id: String,
  pub username: String,
  pub first_name: Option<String>,
  pub last_name: Option<String>,
}
```

Replace `has_dashboard_session: bool` with `dashboard: Option<DashboardUser>` on `UserInfoEnvelope`:
```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserInfoEnvelope {
  #[serde(flatten)]
  pub user: UserResponse,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub dashboard: Option<DashboardUser>,
}
```

Remove the `is_false` helper function (no longer needed).

### 2. Backend: Refactor handler to decode dashboard token and separate concerns

**File**: `crates/routes_app/src/users/routes_users_info.rs`

Change the handler to return `(UserResponse, Option<DashboardUser>)` tuple from the match:

- **`MultiTenantSession` with `token: Some`**: Decode resource token → `LoggedIn`, decode dashboard token → `Some(DashboardUser)`
- **`MultiTenantSession` with `token: None`**: `LoggedOut`, decode dashboard token → `Some(DashboardUser)`
- **All other variants**: Same user response as before, `None` for dashboard

Key change in `MultiTenantSession` arm:
```rust
AuthContext::MultiTenantSession {
  ref token,
  ref role,
  ref dashboard_token,
  ..
} => {
  let dashboard_claims: Claims = extract_claims::<Claims>(dashboard_token)?;
  let dashboard = DashboardUser {
    user_id: dashboard_claims.sub,
    username: dashboard_claims.preferred_username,
    first_name: dashboard_claims.given_name,
    last_name: dashboard_claims.family_name,
  };
  let user = if let Some(ref token) = token {
    let claims: Claims = extract_claims::<Claims>(token)?;
    UserResponse::LoggedIn(services::UserInfo {
      user_id: claims.sub,
      username: claims.preferred_username,
      first_name: claims.given_name,
      last_name: claims.family_name,
      role: role.map(AppRole::Session),
    })
  } else {
    UserResponse::LoggedOut
  };
  (user, Some(dashboard))
}
```

Update OpenAPI doc examples to reflect the new `dashboard` field shape.

### 3. Backend: Register `DashboardUser` in OpenAPI

**File**: `crates/routes_app/src/shared/openapi.rs`

- Add `DashboardUser` to the import from `crate::` (line 15 area, alongside `UserInfoEnvelope`)
- Add `DashboardUser` to the `schemas(...)` list in the `// auth` section (after `UserInfoEnvelope`, around line 308)

### 4. Backend: Update unit tests

**File**: `crates/routes_app/src/users/test_user_info.rs`

- Import `DashboardUser` from `crate`
- All existing tests asserting `has_dashboard_session: false` → change to `dashboard: None`
  - `test_user_info_handler_anonymous` (line 30)
  - `test_user_info_handler_session_token_with_role` (line 61)
  - `test_user_info_handler_api_token_with_token_scope` (line 108)
  - `test_user_info_handler_bearer_token_with_user_scope` (line 142)
  - `test_user_info_handler_session_without_role` (line 195)
  - `test_user_info_handler_external_app_without_scope` (line 289)
- `test_user_info_handler_with_dashboard_session` (line 340):
  - Currently: `token: None` → asserts `LoggedIn` + `has_dashboard_session: true`
  - Change to: `token: None` → assert `user: LoggedOut` AND `dashboard: Some(DashboardUser { user_id: claims.sub, username: "testuser@email.com", first_name: Some("Test"), last_name: Some("User") })`
  - Add new test case: `token: Some(token)` → assert `user: LoggedIn(UserInfo { ... })` AND `dashboard: Some(DashboardUser { ... })`
- `test_user_info_handler_anonymous_multi_tenant_no_dashboard_session` (line 382):
  - Assert `dashboard: None` and `user: LoggedOut`

### 5. Backend: Update live integration tests

**File**: `crates/routes_app/tests/test_live_multi_tenant.rs`

These tests use real Keycloak tokens + session injection (full middleware stack).

- `test_user_info_has_dashboard_session` (line 447):
  - Currently: Injects session with only `dashboard:access_token` → asserts `body["has_dashboard_session"] == true`
  - Change to: Assert `body["auth_status"] == "logged_out"` (no resource token = not logged into tenant), assert `body["dashboard"]` is a non-null object with `user_id`, `username` fields, assert `body.get("has_dashboard_session").is_none()`

- `test_user_info_no_dashboard_session` (line 489):
  - Currently: Asserts `has_dashboard_session` is absent or false
  - Change to: Assert `body.get("dashboard").is_none()`, assert `body["auth_status"] == "logged_out"`

### 6. Regenerate OpenAPI + TypeScript client

```bash
cargo run --package xtask openapi
make build.ts-client
```

### 7. Frontend: Update `needsTenantSelection` and invite flow

**File**: `crates/bodhi/src/app/ui/login/page.tsx`

Replace:
```typescript
const isAuthenticated = userInfo?.auth_status === 'logged_in';
const needsTenantSelection = isAuthenticated && !!userInfo?.has_dashboard_session && !appInfo?.client_id;
```

With:
```typescript
const needsTenantSelection = !!userInfo?.dashboard && !appInfo?.client_id;
```

The `isAuthenticated` guard is removed because:
- With `MultiTenantSession { token: None }`, `auth_status` is now `'logged_out'` but `dashboard` is present
- `dashboard` presence alone implies a validated dashboard session (backend authority)

Also update invite flow (line ~129):
```typescript
// was: if (!userInfo?.has_dashboard_session)
if (!userInfo?.dashboard) {
```

### 8. Frontend: Update MSW handlers

**File**: `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts`

Update `mockUserLoggedOut()` to accept `dashboard` instead of `has_dashboard_session`:
```typescript
export function mockUserLoggedOut({ stub, ...rest }: { stub?: boolean; dashboard?: object } = {})
```

`mockUserLoggedIn()` already uses `...rest` spread, so passing `dashboard` will work.

### 9. Frontend: Update login page tests

**File**: `crates/bodhi/src/app/ui/login/page.test.tsx`

Update all `MultiTenantLoginContent` tests:

- **Stale session test**: With new backend model this scenario can't happen (dashboard only set from validated AuthContext). Keep as regression test but use `dashboard` field.
- **State A**: `mockUserLoggedOut()` (no dashboard) → login button
- **State B**: `mockUserLoggedOut({ dashboard: { user_id: 'test-id', username: 'test@example.com', first_name: null, last_name: null } })` + tenants mock → tenant selection. Note: `auth_status: 'logged_out'` with `dashboard` present = dashboard-only session.
- **State C**: `mockUserLoggedIn({ role: 'resource_admin', dashboard: { ... } })` + appInfo with client_id → welcome

---

## Expected JSON responses

**Anonymous (no session)**:
```json
{ "auth_status": "logged_out" }
```

**Dashboard only (no resource login)**:
```json
{
  "auth_status": "logged_out",
  "dashboard": {
    "user_id": "550e8400-...",
    "username": "user@example.com",
    "first_name": "Test",
    "last_name": "User"
  }
}
```

**Fully authenticated (dashboard + resource)**:
```json
{
  "auth_status": "logged_in",
  "user_id": "550e8400-...",
  "username": "user@example.com",
  "role": "resource_admin",
  "dashboard": {
    "user_id": "550e8400-...",
    "username": "user@example.com",
    "first_name": "Test",
    "last_name": "User"
  }
}
```

**Standalone session / API token / External app**:
```json
{ "auth_status": "logged_in", "user_id": "...", ... }
```
(no `dashboard` field)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/routes_app/src/users/users_api_schemas.rs` | Add `DashboardUser`, replace `has_dashboard_session` with `dashboard: Option<DashboardUser>`, remove `is_false` |
| `crates/routes_app/src/users/routes_users_info.rs` | Decode dashboard token, return `LoggedOut` for token-less MultiTenantSession, update OpenAPI doc |
| `crates/routes_app/src/shared/openapi.rs` | Register `DashboardUser` schema |
| `crates/routes_app/src/users/test_user_info.rs` | Update all assertions from `has_dashboard_session` to `dashboard`, add token:Some case |
| `crates/routes_app/tests/test_live_multi_tenant.rs` | Update 2 live integration tests to assert `dashboard` object instead of `has_dashboard_session` bool |
| `openapi.json` | Regenerated |
| `ts-client/` | Regenerated |
| `crates/bodhi/src/app/ui/login/page.tsx` | `needsTenantSelection` uses `!!userInfo?.dashboard`, update invite flow |
| `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts` | Replace `has_dashboard_session` with `dashboard` in mock params |
| `crates/bodhi/src/app/ui/login/page.test.tsx` | Update test mocks to use `dashboard` object |

### Files NOT modified (verified no changes needed)

| File | Reason |
|------|--------|
| `crates/lib_bodhiserver_napi/tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs` | Tests via UI state (`[data-test-state]`), not API field assertions. Implicitly validates after `build.ui-rebuild`. |
| `crates/lib_bodhiserver_napi/tests-js/pages/MultiTenantLoginPage.mjs` | Page object uses `[data-test-state]` selectors, no `has_dashboard_session` references. |
| Other E2E test files | No direct references to `has_dashboard_session` or `dashboard` field. |

---

## Verification

### 1. Backend compile + unit tests
```bash
cargo check -p routes_app 2>&1 | tail -5
cargo test -p routes_app --lib -- test_user_info 2>&1 | grep -E "test result|FAILED"
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
```

### 2. Backend live integration tests (requires Keycloak)
```bash
cargo test -p routes_app --test test_live_multi_tenant 2>&1 | grep -E "test result|FAILED"
```

### 3. OpenAPI + TypeScript regeneration
```bash
cargo run --package xtask openapi
make build.ts-client
```

### 4. Frontend unit tests
```bash
cd crates/bodhi && npm test -- --run page.test 2>&1 | tail -20
cd crates/bodhi && npm test 2>&1 | tail -20
```

### 5. Rebuild embedded UI for E2E
```bash
make build.ui-rebuild
```

### 6. E2E Playwright tests (validates full stack)
```bash
cd crates/lib_bodhiserver_napi && npm run test:playwright
```
The multi-tenant-lifecycle spec implicitly validates: dashboard login → tenant selection → chat → logout. No spec file changes needed — the UI behavior is unchanged, only the API field shape changes under the hood.

### 7. Manual E2E (browser verification)
Navigate to `https://dev-multi-tenant.getbodhi.app/ui/login/` and verify:
1. No `/bodhi/v1/tenants` network call when not logged in
2. Login button appears immediately (no "Loading..." flash)
3. After dashboard OAuth, tenant selection appears (auth_status: "logged_out", dashboard object present)
4. After selecting tenant and completing resource OAuth, welcome screen shows
