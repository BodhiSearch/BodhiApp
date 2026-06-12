# Issue 03: Missing MockSessionService in Tests

**Category**: Tests creating real `DefaultSessionService` when mock would suffice
**Severity**: 🔴 Critical
**Crates affected**: `auth_middleware`, `routes_app`
**Tests affected**: ~500 tests (virtually all tests in these two crates)
**Relationship**: Overlaps with Issue 02 — this is the application-level fix, Issue 02 is the structural fix

---

## Problem Summary

`AppServiceStubBuilder::default().build()` silently creates a real `DefaultSessionService` with 2 SQLite connection pools whenever the caller doesn't provide an explicit session service. Most tests in `auth_middleware` and `routes_app` don't need real session storage — they test role/scope authorization, route handlers, and business logic. But they all pay the 42–900s shutdown penalty from the 3 real SQLite pools.

The fix is to change `AppServiceStubBuilder`'s default behavior to use `MockSessionService` (no SQLite pools, no background tasks, instant shutdown).

---

## Root Cause

### The Default Builder Behavior

`crates/services/src/test_utils/app.rs` — `build()` (line ~91):
```rust
pub async fn build(&mut self) -> Result<AppServiceStub, AppServiceStubBuilderError> {
    if !matches!(&self.db_service, Some(Some(_))) {
        self.with_db_service().await;     // creates TestDbService (1 pool)
    }
    if !matches!(&self.session_service, Some(Some(_))) {
        self.with_session_service().await; // creates DefaultSessionService (2 pools)
    }
    if !matches!(&self.app_instance_service, Some(Some(_))) {
        self.with_app_instance_service().await;
    }
    self.fallback_build()
}
```

**The `if !matches!` condition means: "if the caller didn't set a session service, create a real one."**

This design assumes that a real session service is always needed. But for the vast majority of tests, it's not.

### The Contrast Case

In `crates/routes_app/src/routes_users/test_management_crud.rs`:

```rust
// 79 seconds — uses explicit mock session service
async fn test_change_user_role_clears_sessions() {
    let mock_session = MockSessionService::new();
    // ... setup mock expectations ...
    let app_service = AppServiceStubBuilder::default()
        .auth_service(Arc::new(mock_auth))
        .session_service(Arc::new(mock_session))  // ← explicit mock = FAST
        .build().await.unwrap();
}

// 900+ seconds — no explicit session service = real DefaultSessionService = SLOW
async fn test_list_users_handler_success() {
    let app_service = AppServiceStubBuilder::default()
        .auth_service(Arc::new(mock_auth))
        // ← NO session_service() call = builds real DefaultSessionService = SLOW
        .build().await.unwrap();
}
```

The fix requires either:
1. Changing `AppServiceStubBuilder`'s default to use `MockSessionService`
2. OR manually adding `.session_service(Arc::new(MockSessionService::new()))` to each slow test

---

## auth_middleware Tests Analysis

### File: `crates/auth_middleware/src/api_auth_middleware.rs`

This file has two test router helper functions (lines ~148 and ~175):

```rust
// Helper A — used by role tests
async fn test_router_with_auth_context(
    auth_context: AuthContext,
    required_role: ResourceRole,
    token_scope_config: Option<TokenScopeConfig>,
) -> (Router, Arc<AppServiceStub>) {
    let app_service = AppServiceStubBuilder::default()
        .build()     // ← creates 3 real pools, session layer NOT used in this router
        .await
        .unwrap();
    let app_service = Arc::new(app_service);
    // ... builds test router WITHOUT session layer ...
    (router, app_service)
}

// Helper B — used by user_scope tests
async fn test_router_user_scope_with_auth_context(
    auth_context: AuthContext,
    required_scope: UserScope,
) -> (Router, Arc<AppServiceStub>) {
    let app_service = AppServiceStubBuilder::default()
        .build()     // ← same issue
        .await
        .unwrap();
    // ...
}
```

**Key insight**: Neither helper attaches a session layer to the router. `DefaultSessionService` is created but NEVER USED. It's pure waste.

**Tests using Helper A** (role tests, ~30 tests, 74-79s each):
- All `test_api_auth_role_*` parameterized test cases

**Tests using Helper B** (user_scope tests, ~20 tests, 938-940s each):
- All `test_api_auth_user_scope_*` parameterized test cases

**Fix for auth_middleware**: Change both helpers to inject `MockSessionService`:
```rust
async fn test_router_with_auth_context(...) -> (Router, Arc<AppServiceStub>) {
    let app_service = AppServiceStubBuilder::default()
        .session_service(Arc::new(MockSessionService::new()))  // ← add this line
        .build()
        .await
        .unwrap();
    // ...
}
```

### Other auth_middleware tests (fast — not affected)

Tests in these modules are already fast (<1s) and do NOT use `AppServiceStubBuilder`:
- `auth_context::*` — tests `AuthContext` methods directly
- `access_request_auth::*` — builds minimal request with `.with_auth_context()` extension
- `toolset_auth::*` — same pattern

These tests use `RequestAuthContextExt::with_auth_context()` to inject `AuthContext` directly into request extensions, bypassing any service setup entirely. This is the ideal pattern for testing middleware authorization logic.

---

## routes_app Tests Analysis

### File: `crates/routes_app/src/test_utils/router.rs` — `build_test_router()`

```rust
pub async fn build_test_router(
    /* various mock service overrides */
) -> Router {
    let mut builder = AppServiceStubBuilder::default();
    builder
        .with_hub_service()
        .with_data_service().await
        .with_db_service().await
        .with_session_service().await   // ← explicitly creates DefaultSessionService
        .with_app_instance(AppInstance::test_default()).await
        // ...
}
```

`build_test_router()` explicitly calls `.with_session_service().await` which creates a real `DefaultSessionService`. This function is used by ~300+ tests across routes_app.

**Why session service is in `build_test_router()`**: The routes_app test router needs to attach the tower_sessions layer to handle `SessionManager` middleware used by auth endpoints (login/logout/callback). Routes that deal with session creation/destruction legitimately need a real session store.

**But**: Most routes_app tests test routes that DON'T manage sessions — model routes, alias routes, toolset routes, MCP routes. These don't need real session storage.

**Proposed solution**: Make `build_test_router()` accept an optional session service parameter, defaulting to `MockSessionService`:

```rust
pub async fn build_test_router(
    session_service: Option<Arc<dyn SessionService>>,  // ← new param
    /* ... other params ... */
) -> Router {
    let session_svc = session_service
        .unwrap_or_else(|| Arc::new(MockSessionService::new()));  // ← mock by default
    builder
        .session_service(session_svc)
        // ... rest of setup
}
```

OR add a second helper `build_test_router_with_session()` for tests that need real sessions.

### Specific near-deadlocked tests in routes_app

**File**: `crates/routes_app/src/routes_users/test_management_crud.rs`

5 tests at 900s+ (under parallel contention):
```
test_list_users_handler_success
test_list_users_handler_auth_error
test_list_users_handler_pagination_parameters
test_change_user_role_handler_auth_error
test_change_user_role_session_clear_failure_still_succeeds
```

These tests build `AppServiceStub` directly (not via `build_test_router()`):
```rust
let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    // ← missing: .session_service(Arc::new(MockSessionService::new()))
    .build().await?;
```

**Immediate fix**: Add `.session_service(Arc::new(MockSessionService::new()))` to each.

### Tests that DO need real session service

The login/logout/OAuth callback routes in `routes_app` legitimately use HTTP sessions. These tests should keep real `DefaultSessionService` (or use a specialized fast-closing test session service). After Fix 02 (short `idle_timeout`), even these will be fast.

Likely in:
- `routes_app/src/routes_login/` — OAuth login flow
- `routes_app/src/routes_users/` (some) — session clearing on role change

---

## Fix Strategy

### Option A: Change AppServiceStubBuilder Default (Highest Impact)

Modify `crates/services/src/test_utils/app.rs`:

```rust
pub async fn build(&mut self) -> Result<AppServiceStub, AppServiceStubBuilderError> {
    if !matches!(&self.db_service, Some(Some(_))) {
        self.with_db_service().await;
    }
    if !matches!(&self.session_service, Some(Some(_))) {
        // BEFORE: self.with_session_service().await;  // creates real DefaultSessionService
        // AFTER: use MockSessionService as default:
        let mock_session = Arc::new(MockSessionService::new());
        self.session_service = Some(Some(mock_session));
    }
    if !matches!(&self.app_instance_service, Some(Some(_))) {
        self.with_app_instance_service().await;
    }
    self.fallback_build()
}
```

Tests that need real session storage must now explicitly call `.with_session_service().await`.

**Impact**: Fixes auth_middleware (all tests) and routes_app management_crud tests automatically. `build_test_router()` in routes_app still calls `.with_session_service().await` explicitly, so those tests still get real sessions (but can be fixed separately).

**Risk**: Some tests might currently implicitly rely on real session service behavior even though they don't set it explicitly. Need to audit. Most such tests would fail with meaningful errors if they genuinely need real sessions.

### Option B: Fix Each Test Site Individually

More targeted but tedious for ~500 tests. Suitable for the management_crud near-deadlocked tests as a quick fix:

```rust
// In test_management_crud.rs
let app_service = AppServiceStubBuilder::default()
    .auth_service(Arc::new(mock_auth))
    .session_service(Arc::new(MockSessionService::new()))  // ← add this
    .build().await?;
```

And in auth_middleware's test helpers:
```rust
let app_service = AppServiceStubBuilder::default()
    .session_service(Arc::new(MockSessionService::new()))  // ← add this
    .build()
    .await
    .unwrap();
```

### Option C: Fix build_test_router() in routes_app

```rust
// In crates/routes_app/src/test_utils/router.rs
pub async fn build_test_router(...) -> Router {
    builder
        .with_hub_service()
        .with_data_service().await
        .with_db_service().await
        // REMOVE: .with_session_service().await  ← removes all 3 real pools
        // ADD mock session (or let AppServiceStubBuilder default handle it after Fix A):
        .session_service(Arc::new(MockSessionService::new()))
        .with_app_instance(AppInstance::test_default()).await
}
```

For tests that need real session storage (login tests), add a separate helper.

---

## Expected Improvement

| Scenario | Before | After Fix A | After Fix B/C |
|----------|--------|-------------|---------------|
| auth_middleware role test | 74s | <1s | <1s |
| auth_middleware user_scope test | 940s | <1s | <1s |
| routes_app management_crud deadlocked | 900s+ | <2s | <2s |
| routes_app general tests | 77-97s | <2s | <2s |
| routes_app full suite | >840s killed | ~20s | ~20s |

---

## MockSessionService Interface

The `MockSessionService` (generated by `mockall`) should already exist in `crates/services/src/test_utils/`. Verify:

```bash
grep -rn "MockSessionService" crates/services/src/
grep -rn "MockSessionService" crates/auth_middleware/src/
grep -rn "MockSessionService" crates/routes_app/src/
```

If it exists, importing it in test files should be straightforward:
```rust
use services::test_utils::MockSessionService;
```

---

## Verification Steps

```bash
# After fix, auth_middleware single test should be <2s
time cargo nextest run -p auth_middleware -E 'test(test_api_auth_role_success::case_01)'

# Full auth_middleware suite should be <10s
time cargo nextest run -p auth_middleware

# Near-deadlocked tests should complete in <5s
time cargo nextest run -p routes_app -E 'test(test_list_users_handler_success)'

# Full routes_app should be <30s
time cargo nextest run -p routes_app
```

---

## Key Files to Modify

1. `crates/services/src/test_utils/app.rs` — `AppServiceStubBuilder::build()` default session service
2. `crates/auth_middleware/src/api_auth_middleware.rs` — `test_router_with_auth_context()` and `test_router_user_scope_with_auth_context()` helpers (lines ~148 and ~175)
3. `crates/routes_app/src/test_utils/router.rs` — `build_test_router()` helper
4. `crates/routes_app/src/routes_users/test_management_crud.rs` — 5 near-deadlocked tests

---

## Investigation Commands for Fresh Session

```bash
# Find all AppServiceStubBuilder usages in test files
grep -rn "AppServiceStubBuilder" crates/ --include="*.rs" | grep -v "target/"

# Find all tests that DON'T set explicit session_service
grep -rn "AppServiceStubBuilder" crates/ --include="*.rs" | grep -v "session_service\|target/"

# Verify MockSessionService exists
grep -rn "MockSessionService\|mock_session_service" crates/services/src/

# Check build_test_router implementation
cat crates/routes_app/src/test_utils/router.rs

# Check the near-deadlocked management_crud tests
grep -A 15 "async fn test_list_users_handler_success" crates/routes_app/src/routes_users/test_management_crud.rs
grep -A 15 "async fn test_change_user_role_clears_sessions" crates/routes_app/src/routes_users/test_management_crud.rs
```
