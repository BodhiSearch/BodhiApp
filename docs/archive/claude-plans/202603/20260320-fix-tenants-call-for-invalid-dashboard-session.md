# Fix: Prevent /bodhi/v1/tenants call when user is not logged in

## Context

On multi-tenant deployments (e.g. `dev-multi-tenant.getbodhi.app/ui/login/`), the login page makes a `/bodhi/v1/tenants` API call that returns 403 when the user is not authenticated. This happens because:

1. **Backend bug**: `/bodhi/v1/user` returns `has_dashboard_session: true` by checking raw session key existence — a stale/expired dashboard cookie causes this to be `true` even when the auth middleware rejects the token and creates `AuthContext::Anonymous`.
2. **Frontend gap**: The `needsTenantSelection` condition (`page.tsx:96`) trusts `has_dashboard_session` without checking `auth_status`, enabling the tenants query for unauthenticated users.

**Observed on production**: `GET /bodhi/v1/info` (200) → `GET /bodhi/v1/user` (200, `auth_status: "logged_out"`, `has_dashboard_session: true`) → `GET /bodhi/v1/tenants` (403). Also causes a brief "Loading..." flash before the login button appears.

**User confirmed**: After valid dashboard OAuth, `auth_status` becomes `'logged_in'`, so it can be used as a guard.

---

## Changes

### 1. Backend: Derive `has_dashboard_session` from AuthContext (not raw session)

**File**: `crates/routes_app/src/users/routes_users_info.rs`

Replace lines 46-50:
```rust
let has_dashboard_session = session
    .get::<String>(DASHBOARD_ACCESS_TOKEN_KEY)
    .await
    .unwrap_or(None)
    .is_some();
```

With:
```rust
let has_dashboard_session = auth_scope.auth_context().dashboard_token().is_some();
```

`AuthContext::dashboard_token()` (defined in `services/src/auth/auth_context.rs:133-140`) returns `Some` only for `MultiTenantSession` — which is only created when the middleware has validated the dashboard token. For `Anonymous` (stale/expired token), it returns `None`.

**Also remove**:
- `session: Session` parameter from the handler (line 44)
- `use tower_sessions::Session;` import (line 9)
- `use crate::tenants::DASHBOARD_ACCESS_TOKEN_KEY;` import (line 1)

### 2. Frontend: Add `auth_status` guard to `needsTenantSelection`

**File**: `crates/bodhi/src/app/ui/login/page.tsx`

Change line 96:
```typescript
const needsTenantSelection = !!userInfo?.has_dashboard_session && !appInfo?.client_id;
```

To:
```typescript
const isAuthenticated = userInfo?.auth_status === 'logged_in';
const needsTenantSelection = isAuthenticated && !!userInfo?.has_dashboard_session && !appInfo?.client_id;
```

This is defense-in-depth: even if the backend returns incorrect `has_dashboard_session`, the frontend won't call tenants for unauthenticated users.

**Note on stale session cleanup**: We intentionally do NOT clean up stale `dashboard:access_token` session keys in the middleware. If we did, a transient upstream auth server outage would cause mass permanent logouts. Instead, stale keys are left in the session — they're now harmless since `has_dashboard_session` derives from `AuthContext` (which the middleware validates). When the upstream recovers, the token refreshes on next request and the user is seamlessly re-authenticated.

### 3. Backend test updates

**File**: `crates/routes_app/src/users/test_user_info.rs`

Update `test_user_info_handler_with_dashboard_session()` (lines 340-400):
- Currently uses `build_test_router()` with raw session `DASHBOARD_ACCESS_TOKEN_KEY` insertion (unvalidated string)
- After the fix, `has_dashboard_session` derives from `AuthContext`, not raw session
- Change to use `test_router()` pattern (like other tests) with `AuthContext::MultiTenantSession` injected directly:

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_with_dashboard_session(
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  let claims = services::extract_claims::<services::Claims>(&token)?;

  let auth_context = AuthContext::MultiTenantSession {
    client_id: None,
    tenant_id: None,
    user_id: claims.sub.clone(),
    username: "testuser@email.com".to_string(),
    role: None,
    token: None,
    dashboard_token: token.clone(),
  };

  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;
  assert_eq!(true, response_json.has_dashboard_session);
  match &response_json.user {
    UserResponse::LoggedIn(info) => {
      assert_eq!("testuser@email.com", info.username);
    }
    other => panic!("Expected LoggedIn, got {:?}", other),
  }
  Ok(())
}
```

Add a new test verifying `has_dashboard_session: false` for `Anonymous` in multi-tenant mode (stale session scenario):

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_anonymous_multi_tenant_no_dashboard_session() -> anyhow::Result<()> {
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
          deployment: services::DeploymentMode::MultiTenant,
        }),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;
  assert_eq!(false, response_json.has_dashboard_session);
  assert_eq!(UserResponse::LoggedOut, response_json.user);
  Ok(())
}
```

### 4. Frontend: New tenants MSW handler

**New file**: `crates/bodhi/src/test-utils/msw-v2/handlers/tenants.ts`

Create MSW v2 handlers for the tenants endpoint:
- `mockTenantsList(tenants)` — returns 200 with `TenantListResponse`
- `mockTenantsListError({ status })` — returns error (403 for auth failures)

Register in the handlers index.

### 5. Frontend: Extend `mockUserLoggedOut()` for `has_dashboard_session`

**File**: `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts`

Extend `mockUserLoggedOut()` to accept optional extra fields (including `has_dashboard_session`) for testing the buggy state:

```typescript
export function mockUserLoggedOut({ stub, ...rest }: { stub?: boolean; has_dashboard_session?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_USER_INFO, ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200 as const).json({
        auth_status: 'logged_out',
        ...rest,
      });
    }),
  ];
}
```

### 6. Frontend: Add MultiTenantLoginContent tests

**File**: `crates/bodhi/src/app/ui/login/page.test.tsx`

Add new `describe` blocks for `MultiTenantLoginContent` via `LoginPage`:

**Bug fix test** (critical):
- Mock: `mockUserLoggedOut({ has_dashboard_session: true })` + `mockAppInfo({ status: 'tenant_selection', deployment: 'multi_tenant' })`
- Assert: Login button ("Login to Bodhi Platform") is rendered
- Assert: No network call to `/bodhi/v1/tenants` (tenants query disabled)

**State A: No dashboard session**:
- Mock: `mockUserLoggedOut()` + `mockAppInfo({ status: 'tenant_selection', deployment: 'multi_tenant' })`
- Assert: `data-test-state="login"` card with "Login to Bodhi Platform" button

**State B: Tenant selection**:
- Mock: `mockUserLoggedIn({ has_dashboard_session: true })` + `mockAppInfo({ status: 'tenant_selection', deployment: 'multi_tenant' })` + `mockTenantsList([tenant1, tenant2])`
- Assert: `data-test-state="select"` card with tenant buttons

**State C: Fully authenticated**:
- Mock: `mockUserLoggedIn({ has_dashboard_session: true })` + `mockAppInfo({ status: 'ready', deployment: 'multi_tenant', client_id: 'test-client' })` + `mockTenantsList([activeTenant])`
- Assert: `data-test-state="welcome"` card with username and active workspace

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/routes_app/src/users/routes_users_info.rs` | Derive `has_dashboard_session` from AuthContext, remove `Session` param |
| `crates/routes_app/src/users/test_user_info.rs` | Update dashboard session test, add anonymous multi-tenant test |
| `crates/bodhi/src/app/ui/login/page.tsx` | Add `auth_status` guard to `needsTenantSelection` |
| `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts` | Extend `mockUserLoggedOut()` for `has_dashboard_session` |
| `crates/bodhi/src/test-utils/msw-v2/handlers/tenants.ts` | **New**: Tenants endpoint MSW handlers |
| `crates/bodhi/src/app/ui/login/page.test.tsx` | Add MultiTenantLoginContent tests |

---

## Verification

### Backend
```bash
cargo check -p routes_app 2>&1 | tail -5
cargo test -p routes_app --lib -- test_user_info 2>&1 | grep -E "test result|FAILED"
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
```

### Frontend
```bash
cd crates/bodhi && npm test -- --run page.test 2>&1 | tail -20
cd crates/bodhi && npm test 2>&1 | tail -20
```

### E2E (browser verification)
Navigate to `https://dev-multi-tenant.getbodhi.app/ui/login/` and verify:
1. No `/bodhi/v1/tenants` network call when not logged in
2. Login button appears immediately (no "Loading..." flash)
3. After clicking login and completing dashboard OAuth, tenants load correctly
