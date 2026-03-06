# Multi-Tenant Middleware Refactor -- JWT-Based Tenant Resolution

> **Created**: 2026-03-06
> **Prior work**: `ai-docs/claude-plans/20260303-multi-tenant/` (Phase 1-2, 4-5), `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff.md`

## Context

The auth middleware calls `get_standalone_app()` which returns `Err(TenantError::MultipleTenant)` when >1 tenant exists, blocking Phase 3 middleware integration tests. The fix: resolve tenant from JWT claims (`azp` for session, `aud` for external tokens) instead of looking up "the one tenant." This makes middleware work identically for standalone and multi-tenant deployments.

## Decisions (from interview)

| ID | Decision |
|----|----------|
| Setup check | **Remove entirely** from both middleware functions. Middleware does authentication only. Setup routes already gate via `app_status_or_default()`. |
| Bearer status | **No status check** for bearer tokens either. Middleware trusts resolved auth. |
| `get_valid_session_token` | Accept `&Tenant` parameter. Caller resolves tenant from JWT `azp`. |
| Claim extraction | Accept double extraction (middleware extracts `azp`, function extracts claims again internally). |
| Error variants | Reuse `TokenError::InvalidAudience` for both `aud=None` and tenant-not-found. No new variants. |
| Mock strategy | `MockAuthService` with flexible expectations for isolation tests. |
| Test scope | Session + External + cross-tenant rejection. 3 test groups. Use `#[values("sqlite","postgres")]` x `#[values("standalone","multi-tenant")]`. |
| Anonymous | Always `{ client_id: None, tenant_id: None }` when no auth present (D26). |

## Existing Functions Reused (no changes needed)

| Function | Location |
|----------|----------|
| `TenantService::get_tenant_by_client_id()` | `services/src/tenants/tenant_service.rs` |
| `extract_claims::<Claims>()` | `services/src/shared_objs/token.rs` — extracts `azp` from session JWT |
| `extract_claims::<ScopeClaims>()` | same — has `aud` field for external tokens |
| `Tenant::test_default()` / `Tenant::test_tenant_b()` | `services/src/test_utils/fixtures.rs` |
| `validate_bearer_token()` | `token_service.rs` — API token path already uses `get_tenant_by_client_id` |
| `sea_context(db_type)` | `services/src/test_utils/sea.rs` — dual SQLite/PostgreSQL test DB |
| `AppServiceStubBuilder::with_tenant()` | `services/src/test_utils/app.rs` — can be called multiple times for multi-tenant |

---

## Phase 1: Migrate middleware to JWT-based tenant resolution

> **Goal**: Remove all `get_standalone_app()` calls from middleware, resolve tenant from JWT claims instead, update all existing tests. App compiles, all existing tests pass, local commit.
>
> **Sub-agent**: Implement all steps below, then run gate checks.

### Step 1: Remove `AppStatusInvalid` variant and Setup check

**`crates/routes_app/src/middleware/auth/error.rs`:**
- Remove `AppStatusInvalid(AppStatus)` variant (lines 47-49).
- Remove `AppStatus` from `use services::{...}` import (line 2) if unused.

**`crates/routes_app/src/middleware/auth/auth_middleware.rs` — `auth_middleware` (lines 103-113):**
- Delete the status check block (lines 104-110): the `let status = ...` extraction and the `if status == AppStatus::Setup` return.
- Keep the `get_standalone_app()` call and `instance_client_id`/`instance_tenant_id` extraction for now — Step 2 replaces them.

**`crates/routes_app/src/middleware/auth/auth_middleware.rs` — `optional_auth_middleware` (lines 181-192):**
- Delete the status check block (lines 182-192): the `let status = ...` extraction and the `if status == AppStatus::Setup` early return with Anonymous.
- Keep the `get_standalone_app()` call and `anon` closure for now — Step 3 replaces them.

### Step 2: Refactor `get_valid_session_token` — accept `&Tenant`

**`crates/routes_app/src/middleware/token_service/token_service.rs` — `get_valid_session_token` (line 412):**

Signature change:
```rust
// Before:
pub async fn get_valid_session_token(&self, session: Session, access_token: String)
// After:
pub async fn get_valid_session_token(&self, session: Session, access_token: String, tenant: &Tenant)
```

Body change (lines 420-427) — replace `get_standalone_app()`:
```rust
// Before:
let instance = self.tenant_service.get_standalone_app().await?.ok_or(TenantError::NotFound)?;
let instance_client_id = instance.client_id.clone();
let instance_client_secret = instance.client_secret.clone();
// After:
let instance_client_id = tenant.client_id.clone();
let instance_client_secret = tenant.client_secret.clone();
```

Add `use services::Tenant;` if not already imported.

### Step 3: Refactor `auth_middleware` — resolve tenant from JWT `azp`

**`crates/routes_app/src/middleware/auth/auth_middleware.rs` — `auth_middleware`:**

Remove `get_standalone_app()` call (line 103) and `instance_client_id`/`instance_tenant_id` (lines 111-113). Replace the session branch (lines 122-155) with:

```rust
} else if is_same_origin(req.headers()) {
  if let Some(access_token) = session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await.map_err(AuthError::from)? {
    // Resolve tenant from JWT azp claim
    let claims = extract_claims::<Claims>(&access_token)?;
    let tenant = tenant_service
      .get_tenant_by_client_id(&claims.azp).await?
      .ok_or(TenantError::NotFound)?;

    let (access_token, role) = token_service
      .get_valid_session_token(session, access_token, &tenant).await?;
    let role = role.ok_or(AuthError::MissingRoles)?;
    let user_claims = extract_claims::<UserIdClaims>(&access_token)?;

    let auth_context = AuthContext::Session {
      client_id: tenant.client_id,
      tenant_id: tenant.id,
      user_id: user_claims.sub.clone(),
      username: user_claims.preferred_username,
      role: Some(role),
      token: access_token,
    };
    req.extensions_mut().insert(auth_context);
    Ok(next.run(req).await)
  } else {
    Err(AuthError::InvalidAccess)?
  }
}
```

Bearer path (line 115-121) unchanged — `validate_bearer_token` resolves tenant internally.

Add `Claims` to imports from `services` if not already present. Remove `AppStatus` import if unused.

**`crates/routes_app/src/middleware/auth/auth_middleware.rs` — `optional_auth_middleware`:**

Remove `get_standalone_app()` call (line 181), instance extraction (lines 194-196), and `anon` closure (lines 199-202). Replace with:

```rust
let anon = || AuthContext::Anonymous { client_id: None, tenant_id: None };
```

Replace session branch (lines 224-269):
```rust
Ok(Some(access_token)) => {
  let result: Result<AuthContext, AuthError> = async {
    let claims = extract_claims::<Claims>(&access_token)?;
    let tenant = tenant_service
      .get_tenant_by_client_id(&claims.azp).await?
      .ok_or(TenantError::NotFound)?;
    let (validated_token, role) = token_service
      .get_valid_session_token(session.clone(), access_token, &tenant).await?;
    let user_claims = extract_claims::<UserIdClaims>(&validated_token)?;
    Ok(AuthContext::Session {
      client_id: tenant.client_id,
      tenant_id: tenant.id,
      user_id: user_claims.sub.clone(),
      username: user_claims.preferred_username,
      role,
      token: validated_token,
    })
  }.await;

  match result {
    Ok(auth_context) => { req.extensions_mut().insert(auth_context); }
    Err(err) => {
      if should_clear_session(&err) { clear_session_auth_data(&session).await; }
      req.extensions_mut().insert(anon());
    }
  }
}
```

Bearer fallback (lines 217, 222) also changes from `anon()` with tenant IDs → `anon()` with `None`/`None`.

### Step 4: Refactor `handle_external_client_token` — resolve tenant from JWT `aud`

**`crates/routes_app/src/middleware/token_service/token_service.rs` — `handle_external_client_token` (lines 198-223):**

Replace `get_standalone_app()` (lines 202-206) + claim extraction (line 208-209) + issuer check (line 211-213) + audience check (lines 215-223) with:

```rust
let claims = extract_claims::<ScopeClaims>(external_token)?;
let original_azp = claims.azp.clone();

if claims.iss != self.setting_service.auth_issuer().await {
  return Err(TokenError::InvalidIssuer(claims.iss))?;
}

let aud = claims.aud.as_ref()
  .ok_or_else(|| TokenError::InvalidAudience("missing audience".to_string()))?;
let instance = self.tenant_service
  .get_tenant_by_client_id(aud).await?
  .ok_or_else(|| TokenError::InvalidAudience(aud.clone()))?;
```

Rest of function (access request validation, token exchange, role derivation) unchanged — already uses `instance.client_id`, `instance.client_secret`, `instance.id`.

### Step 5: Update existing tests

**`crates/routes_app/src/middleware/auth/test_auth_middleware.rs`:**

**Test: `test_auth_middleware_returns_app_status_invalid_for_app_status_setup_or_missing` (line 147):**
- This test sends requests with NO auth token and NO session cookie to a Setup-status tenant.
- After refactor: no session → "no access token in session" → `InvalidAccess` (401).
- Change: `StatusCode::INTERNAL_SERVER_ERROR` → `StatusCode::UNAUTHORIZED`.
- Change: error code `"auth_error-app_status_invalid"` → `"auth_error-invalid_access"`.
- Change: error body to match `InvalidAccess` format (no `param` field).
- The `assert_optional_auth_passthrough` call still passes (Anonymous with `None`/`None`).
- Consider renaming to `test_auth_middleware_returns_invalid_access_when_no_auth`.

**Test: `test_auth_middleware_with_expired_session_token_and_failed_refresh` (line 468):**
- **BREAKING**: Creates custom `Tenant { client_id: "test_client_id", ... }` but `expired_token` fixture has `azp: TEST_CLIENT_ID` ("test-client"). Middleware will try `get_tenant_by_client_id("test-client")` but tenant has `client_id: "test_client_id"` → `TenantError::NotFound`.
- **Fix**: Change to `Tenant::test_default()` (has `client_id: TEST_CLIENT_ID`). Update mock expectation from `eq("test_client_id"), eq("test_client_secret")` → `eq(TEST_CLIENT_ID), eq(TEST_CLIENT_SECRET)`.

**Tests: valid session, expired session, refresh persist (lines 202, 268, 367):**
- These use `Tenant::test_default()` and JWTs with `azp: TEST_CLIENT_ID`. The new `get_tenant_by_client_id("test-client")` finds the same tenant. Should pass without changes.

**Tests: API token tests (lines 687, 758, 826, 897):**
- Use bearer path → `validate_bearer_token` → already resolves tenant from token suffix. Unchanged.

**Tests: no-token, header stripping, cross-site (lines 578, 614, 646):**
- No session token or bearer involved in tenant resolution. Unchanged.

**`crates/routes_app/src/middleware/token_service/test_token_service.rs`:**
- Find ALL calls to `get_valid_session_token(session, access_token)`.
- Add `&Tenant::test_default()` as third argument (or matching `Tenant` if custom).
- Add `use services::Tenant;` if needed.

### Gate checks
```bash
cargo check -p routes_app
cargo test -p routes_app -- test_auth_middleware 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app -- test_token_service 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app -- middleware 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p services -p server_core -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

### Commit
After all tests pass: `git add -A && git commit -m "refactor: JWT-based tenant resolution in auth middleware"`

---

## Phase 2: Add multi-tenant middleware isolation tests

> **Goal**: Add isolation tests verifying middleware resolves different tenants from different JWT tokens. Use `#[values("sqlite","postgres")]` x `#[values("standalone","multi-tenant")]` for full coverage.
>
> **Sub-agent**: Create new test file, implement all test groups, run gate checks.

### New file: `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs`

**Register in `auth_middleware.rs`:**
```rust
#[cfg(test)]
#[path = "test_auth_middleware_isolation.rs"]
mod test_auth_middleware_isolation;
```

### Test matrix

All tests parameterized with:
- `#[values("sqlite", "postgres")]` — dual DB backend
- `#[values("standalone", "multi-tenant")]` — standalone creates 1 tenant, multi-tenant creates 2

Use `#[serial(pg_app)]` for PostgreSQL serialization.

### Test setup helper

Follow `isolation_router()` pattern from `crates/routes_app/src/tokens/test_tokens_isolation.rs:28-56`:

```rust
async fn isolation_app_service(
  db_type: &str,
  mode: &str,  // "standalone" or "multi-tenant"
  session_service: Arc<dyn SessionService>,
  auth_service: Option<Arc<dyn AuthService>>,
) -> anyhow::Result<(Arc<dyn AppService>, SeaTestContext)> {
  let ctx = sea_context(db_type).await;
  let db_svc: Arc<dyn DbService> = Arc::new(ctx.service.clone());
  let mut builder = AppServiceStubBuilder::default();
  builder
    .db_service(db_svc)
    .with_tenant_service().await;
  // ... set session_service, auth_service, settings (auth_issuer, etc.)

  let app_service: Arc<dyn AppService> = Arc::new(builder.build().await?);

  // Always create tenant A
  app_service.db_service().create_tenant_test(&Tenant::test_default()).await?;
  // Multi-tenant adds tenant B
  if mode == "multi-tenant" {
    app_service.db_service().create_tenant_test(&Tenant::test_tenant_b()).await?;
  }

  Ok((app_service, ctx))
}
```

Build JWT helpers that allow overriding `azp`, `aud`, `resource_access` fields:
```rust
fn session_token_for_tenant(client_id: &str, roles: &[&str]) -> anyhow::Result<String> {
  let mut claims = access_token_claims();
  claims["azp"] = json!(client_id);
  claims["resource_access"] = json!({ client_id: { "roles": roles } });
  let (token, _) = build_token(claims)?;
  Ok(token)
}
```

### Test Group 1: Session token tenant resolution

**`test_session_resolves_tenant_a_from_azp`**
- `#[values("sqlite","postgres")]` x `#[values("standalone","multi-tenant")]`
- Create session with token having `azp: TEST_CLIENT_ID`
- Send same-origin request through `auth_middleware`
- Assert: response 418, `AuthContext::Session { client_id: TEST_CLIENT_ID, tenant_id: TEST_TENANT_ID }`

**`test_session_resolves_tenant_b_from_azp`** (multi-tenant only)
- `#[values("sqlite","postgres")]`, mode = "multi-tenant" only
- Token with `azp: "test-client-b"`, `resource_access: { "test-client-b": { "roles": [...] } }`
- Assert: `AuthContext::Session { client_id: "test-client-b", tenant_id: TEST_TENANT_B_ID }`

### Test Group 2: External token tenant resolution

**`test_external_token_resolves_tenant_from_aud`**
- `#[values("sqlite","postgres")]` x `#[values("standalone","multi-tenant")]`
- External JWT with `aud: TEST_CLIENT_ID`, valid issuer
- `MockAuthService.exchange_app_token` with flexible expectations
- Need to set up access_request in DB for the token exchange flow
- Assert: `AuthContext::ExternalApp { client_id: TEST_CLIENT_ID, tenant_id: TEST_TENANT_ID }`

### Test Group 3: Cross-tenant rejection / fallback

**`test_session_rejects_unknown_azp`**
- `#[values("sqlite","postgres")]` x `#[values("standalone","multi-tenant")]`
- Session token with `azp: "nonexistent-client"`
- Strict middleware: assert 401
- Optional middleware (via `/with_optional_auth`): assert 418 with `is_authenticated: false` (Anonymous)

**`test_external_token_rejects_unknown_aud`**
- `#[values("sqlite","postgres")]` x `#[values("standalone","multi-tenant")]`
- Bearer JWT with `aud: "nonexistent-client"`, valid issuer
- Assert: 401 with `token_error-invalid_audience`

### Gate checks
```bash
cargo test -p routes_app -- test_auth_middleware_isolation 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app -- isolation 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p services -p server_core -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

### Commit
After all tests pass: `git add -A && git commit -m "test: multi-tenant middleware isolation tests (session, external, cross-tenant)"`

---

## Phase 3: E2E Playwright tests (SQLite)

> **Goal**: Rebuild NAPI bindings with middleware changes, run E2E Playwright tests against SQLite, fix any failures.
>
> **Sub-agent 1 of 2**: Sequential — run SQLite E2E first, fix failures one by one.

### Steps

1. Rebuild UI + NAPI bindings:
   ```bash
   make build.ui-rebuild
   ```

2. Run SQLite E2E tests:
   ```bash
   cd crates/lib_bodhiserver_napi && npm run test:playwright:sqlite
   ```

3. If any tests fail, investigate and fix one by one. Failures are likely from:
   - Anonymous context now having `client_id: None` / `tenant_id: None` instead of the standalone tenant's IDs
   - Setup status no longer checked in middleware (routes relying on middleware to block Setup-status requests)

4. Re-run until all pass.

### Commit
`git add -A && git commit -m "fix: E2E playwright fixes for JWT-based tenant resolution (sqlite)"`

---

## Phase 4: E2E Playwright tests (PostgreSQL)

> **Goal**: Run E2E Playwright tests against PostgreSQL, fix any failures.
>
> **Sub-agent 2 of 2**: Sequential — run PostgreSQL E2E after SQLite passes.

### Steps

1. Run PostgreSQL E2E tests:
   ```bash
   cd crates/lib_bodhiserver_napi && npm run test:playwright:postgres
   ```

2. If any tests fail, investigate and fix one by one. PostgreSQL-specific issues may include:
   - RLS policy interactions with the new tenant resolution
   - Transaction isolation differences

3. Re-run until all pass.

### Commit
`git add -A && git commit -m "fix: E2E playwright fixes for JWT-based tenant resolution (postgres)"`

---

## Phase 5: Update docs

> **Sub-agent**: Update middleware documentation to reflect the changes.

### Files to update
- `crates/routes_app/src/middleware/CLAUDE.md` — Remove Setup check mentions, document JWT `azp`/`aud` tenant resolution, update `get_valid_session_token` signature
- `crates/routes_app/src/middleware/PACKAGE.md` — Same updates

### Commit
`git add -A && git commit -m "docs: update middleware docs for JWT-based tenant resolution"`

---

## Files Summary

| File | Change | Phase |
|------|--------|-------|
| `crates/routes_app/src/middleware/auth/error.rs` | Remove `AppStatusInvalid(AppStatus)` | 1 |
| `crates/routes_app/src/middleware/auth/auth_middleware.rs` | Remove `get_standalone_app()` + Setup check, JWT `azp` resolution, add isolation test module | 1, 2 |
| `crates/routes_app/src/middleware/token_service/token_service.rs` | `get_valid_session_token` +`&Tenant`, `handle_external_client_token` aud→tenant | 1 |
| `crates/routes_app/src/middleware/auth/test_auth_middleware.rs` | Fix Setup test → InvalidAccess, fix failed-refresh test tenant mismatch | 1 |
| `crates/routes_app/src/middleware/token_service/test_token_service.rs` | Add `&Tenant` arg to `get_valid_session_token` calls | 1 |
| `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` | **NEW**: isolation tests with sqlite/postgres x standalone/multi-tenant | 2 |
| E2E tests / NAPI bindings | Rebuild + fix any Playwright failures | 3, 4 |
| `crates/routes_app/src/middleware/CLAUDE.md` | Update docs | 5 |
| `crates/routes_app/src/middleware/PACKAGE.md` | Update docs | 5 |

## Existing test impact analysis

| Test (test_auth_middleware.rs) | Lines | Impact |
|------|-------|--------|
| `test_..._app_status_invalid_for_app_status_setup_or_missing` | 147 | **BREAKS** — update to expect InvalidAccess (401) |
| `test_..._with_valid_session_token` | 202 | OK — azp=TEST_CLIENT_ID matches Tenant::test_default() |
| `test_..._with_expired_session_token` | 268 | OK — same azp/tenant match |
| `test_..._token_refresh_persists_to_session` | 367 | OK — same azp/tenant match |
| `test_..._expired_session_token_and_failed_refresh` | 468 | **BREAKS** — custom tenant `client_id: "test_client_id"` ≠ JWT `azp: "test-client"`. Fix: use Tenant::test_default(), update mock expectations |
| `test_..._returns_invalid_access_when_no_token_in_session` | 578 | OK — no session token, no azp lookup |
| `test_..._removes_internal_token_headers` | 614 | OK — optional auth, no session |
| `test_session_ignored_when_cross_site` | 646 | OK — cross-site check before session processing |
| `test_..._bodhiapp_token_*` (4 tests) | 687+ | OK — bearer path, unchanged |
| `test_evaluate_same_origin` | 978 | OK — unit test, no middleware |
