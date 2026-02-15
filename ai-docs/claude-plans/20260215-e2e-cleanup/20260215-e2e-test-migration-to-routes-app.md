# E2E Test Cleanup: Migration to routes_app & server_app

## Context

BodhiApp's E2E Playwright suite has ~20 spec files with 50+ tests. Many test API contracts and auth rejection at the E2E level when they belong lower in the test pyramid. This plan covers 4 phases:

1. Move API token auth rejection tests to `routes_app` (fast unit tests)
2. Remove E2E tests already covered by existing unit/Vitest tests
3. ~~Eliminate per-test Keycloak resource client creation in `server_app`~~ **COMPLETED** (commit `c3348f40`)
4. Explore cache bypass for testing 3rd party OAuth flows in `server_app`

Each phase is independently committable with all tests passing.

---

## Phase 1: Move API Token Blocking Tests to routes_app — COMPLETED

> **Implemented** in commit `b95df54a` (`refactor: move API token blocking tests from E2E to routes_app unit tests`)

### What moved

From `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`, the **"API Token Blocking - Toolset Endpoints"** describe block (5 tests):

| # | E2E Test | Endpoint | Assertion |
|---|----------|----------|-----------|
| 1 | GET /toolsets with API token | `GET /bodhi/v1/toolsets` | 401 |
| 2 | GET /toolsets/{id} with API token | `GET /bodhi/v1/toolsets/{id}` | 401 |
| 3 | PUT /toolsets/{id} with API token | `PUT /bodhi/v1/toolsets/{id}` | 401 |
| 4 | DELETE /toolsets/{id} with API token | `DELETE /bodhi/v1/toolsets/{id}` | 401 |
| 5 | POST /toolsets/{id}/execute/{method} with API token | `POST /bodhi/v1/toolsets/{id}/execute/{method}` | 401 |

### Why they moved

These tests make a single HTTP request with a `Bearer bodhiapp_*` token and assert 401. They don't use a browser, don't test UI, and don't require Keycloak OAuth flows. They test that session-only endpoints reject API tokens - a pure authorization layer concern.

### What stays in E2E

The remaining tests in `toolsets-auth-restrictions.spec.mjs` stay:
- "GET /toolsets with session auth returns toolset_types field" (1 test) - needs configured Exa toolset
- "OAuth Token + Toolset Scope Combinations" (4 tests) - need real Keycloak OAuth flows with scope configuration
- "OAuth Token - Toolset CRUD Endpoints (Session-Only)" (2 tests) - need real OAuth tokens; can move later with cache bypass (Phase 4)

### Implementation

**Destination file:** `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs`

**Pattern:** Create a real API token in the database via `create_test_api_token()`, send as `Authorization: Bearer` header via `api_token_request()`, assert 401 via `router.oneshot()`.

```rust
#[anyhow_trace]
#[rstest]
#[case::list("GET", "/bodhi/v1/toolsets")]
#[case::get("GET", "/bodhi/v1/toolsets/some-id")]
#[case::update("PUT", "/bodhi/v1/toolsets/some-id")]
#[case::delete("DELETE", "/bodhi/v1/toolsets/some-id")]
#[case::execute("POST", "/bodhi/v1/toolsets/some-id/execute/some-method")]
#[tokio::test]
async fn test_toolset_endpoints_reject_api_token(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{api_token_request, build_test_router, create_test_api_token};
  let (router, app_service, _temp) = build_test_router().await?;
  let token = create_test_api_token(app_service.db_service().as_ref()).await?;
  let response = router
    .oneshot(api_token_request(method, path, &token))
    .await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status(),
    "API token should be rejected on session-only endpoint {method} {path}");
  Ok(())
}
```

**New test utilities added** to `crates/routes_app/src/test_utils/router.rs`:

1. `create_test_api_token(db_service: &dyn DbService) -> anyhow::Result<String>` — creates a `bodhiapp_testtoken_{uuid}` token in DB with SHA-256 hash, returns the raw token string
2. `api_token_request(method, path, token) -> Request<Body>` — builds request with `Authorization: Bearer {token}` and `Host: localhost:1135` headers

**Existing utilities reused:**
- `build_test_router()` from `crates/routes_app/src/test_utils/router.rs`
- `DbService::create_api_token()` for token insertion
- Token hash computation via `sha2::Sha256`

### E2E cleanup

Removed the "API Token Blocking - Toolset Endpoints" describe block from `toolsets-auth-restrictions.spec.mjs`:
- Removed `TokensPage` import (no longer needed)
- Removed `apiToken` and `toolsetUuid` variables
- Simplified `beforeAll` to remove API token creation and UUID extraction
- Removed all 5 `test('... with API token returns 401 ...')` blocks
- Kept the `test('GET /toolsets with session auth returns toolset_types field')` test
- Added comment noting API token tests moved to routes_app

### Files modified
- `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs` — added `test_toolset_endpoints_reject_api_token` (5 rstest cases)
- `crates/routes_app/src/test_utils/router.rs` — added `create_test_api_token()`, `api_token_request()` helpers
- `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` — removed 5 tests, simplified beforeAll

### Verification
```bash
cargo test -p routes_app  # 437 passed, 0 failed
```

### Deviation from plan
- Test utilities added to `router.rs` directly (not `mod.rs`) — simpler, co-located with `build_test_router()`
- Import path: `services::db::{ApiToken, DbService, TokenStatus}` (not `services::DbService`) — `DbService` lives in the `db` submodule

---

## Phase 2: Remove Duplicate E2E Tests — COMPLETED

> **Implemented** in commit `9dd90d2f` (`refactor: remove duplicate E2E tests covered by unit tests`)

### 2a: Deleted canonical-redirect.spec.mjs

**Why:** The canonical redirect middleware already has **15 comprehensive rstest unit tests** in `crates/auth_middleware/src/canonical_url_middleware.rs:236-516` covering:
- Wrong scheme, wrong host, wrong port redirects
- Non-standard port handling
- Query parameter preservation
- POST method exemption
- Health/ping path exemptions
- Canonical disabled setting
- Missing public host setting

The E2E spec tested only 2 scenarios (enabled/disabled) — a strict subset of the existing coverage. The redirect is HTTP-level (301 MOVED_PERMANENTLY) and fully testable without a browser.

**Action:** Deleted `crates/lib_bodhiserver_napi/tests-js/specs/settings/canonical-redirect.spec.mjs` (94 lines, 2 tests)

### 2b: Deleted app-initializer.spec.mjs

**Why:** The 4 E2E tests are covered by other tests:

| E2E Test | Already Covered By |
|----------|-------------------|
| Redirect to /ui/setup/ when status=setup | `AppInitializer.test.tsx` (25+ Vitest tests with MSW) |
| Redirect to /ui/login/ when unauthenticated | `AppInitializer.test.tsx` (role-based routing tests) |
| OAuth flow from intercepted route | Other E2E: `public-host-auth.spec.mjs`, `setup-flow.spec.mjs`, every test using `LoginPage` |
| OAuth flow from protected route | Same coverage as above |

**Action:** Deleted `crates/lib_bodhiserver_napi/tests-js/specs/auth/app-initializer.spec.mjs` (171 lines, 4 tests)

### Files modified
- Deleted `crates/lib_bodhiserver_napi/tests-js/specs/settings/canonical-redirect.spec.mjs`
- Deleted `crates/lib_bodhiserver_napi/tests-js/specs/auth/app-initializer.spec.mjs`
- No shared fixture cleanup was needed — no imports were exclusive to these files

### Verification
```bash
cargo test -p auth_middleware  # canonical redirect unit tests pass
cargo test -p routes_app      # 437 passed
```

### Deviation from plan
- None — executed as planned

---

## Phase 3: Server_app Resource Client Reuse — COMPLETED

> **Already implemented** in commit `c3348f40` (`refactor: migrate live server tests to pre-configured resource client`).
> Full plan details: [`ai-docs/claude-plans/20260215-e2e-cleanup/20260215-server-app-remove-register-resource.md`](ai-docs/claude-plans/20260215-e2e-cleanup/20260215-server-app-remove-register-resource.md)

**What was done:**
- Removed dynamic `AuthServerTestClient` creation (3 Keycloak API calls per test → 0)
- `setup_minimal_app_service()` now reads pre-configured `INTEG_TEST_RESOURCE_CLIENT_ID`, `INTEG_TEST_RESOURCE_CLIENT_SECRET`, `INTEG_TEST_RESOURCE_CLIENT_SCOPE` from `.env.test`
- Fixed port `51135` (was random via `rand`), removed `rand` dev-dependency
- Removed `auth_middleware` `test-utils` feature dependency from `server_app`
- Removed dev console credentials (`INTEG_TEST_DEV_CONSOLE_CLIENT_ID`, `INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET`)
- `get_oauth_tokens()` still makes 1 password grant call per test (acceptable)

---

## Phase 4: Cache Bypass for 3rd Party OAuth in server_app — COMPLETED

> **Implemented** in commit `27774045` (`feat: add ExternalTokenSimulator for testing OAuth flows without Keycloak`)

### Goal

Enable `server_app` tests to simulate 3rd party OAuth API calls without browser-based Keycloak flows, by seeding the token validation cache.

### Investigation findings

**Critical question: Does `extract_claims()` verify JWT signatures?**

**Answer: NO.** `extract_claims()` in `crates/services/src/token.rs:134-148` uses `jsonwebtoken::decode()` with `Validation { validate_signature: false, .. }`. It only base64-decodes the JWT payload and checks the `exp` claim. This means any well-formed JWT with a future `exp` timestamp passes validation — no Keycloak signing keys required.

### How token validation works (from `crates/auth_middleware/src/token_service.rs:51-171`)

```
Bearer token arrives (non-bodhiapp_ prefix)
  → extract_claims::<ExpClaims>(bearer_token)     // Parse JWT, check exp (NO signature verification)
  → SHA-256 digest → first 12 hex chars             // Cache key
  → cache_service.get("exchanged_token:{digest}")   // Cache lookup
  → If HIT:
      → Deserialize CachedExchangeResult { token, azp }
      → extract_claims::<ScopeClaims>(&cached.token)
      → Check exp, extract UserScope from scope claim
      → Return (token, ResourceScope::User(scope), azp)   // Skip auth server!
  → If MISS:
      → handle_external_client_token() → auth server call
```

### Implementation

**Test utility:** `crates/server_app/tests/utils/external_token.rs`

```rust
pub struct ExternalTokenSimulator {
  cache_service: Arc<dyn CacheService>,
}

impl ExternalTokenSimulator {
  pub fn new(app_service: &Arc<dyn AppService>) -> Self { ... }

  /// Creates a fake external bearer token and seeds the cache so requests
  /// with this token bypass Keycloak and resolve to the given scope.
  pub fn create_token_with_scope(&self, scope: &str, azp: &str) -> anyhow::Result<String> {
    // 1. Build bearer JWT with build_token() (RSA-signed test keys)
    //    Claims: { jti, sub: "test-external-user", exp: now+1h, scope }
    // 2. Compute cache key: SHA-256(bearer_jwt)[0..12]
    // 3. Build exchange result JWT (simulates Keycloak response)
    //    Claims: { iss, sub, azp, exp: now+1h, scope }
    // 4. Seed cache: "exchanged_token:{digest}" → JSON { token, azp }
    // 5. Return bearer JWT for use in Authorization header
  }
}
```

Key design decisions:
- Uses `services::test_utils::build_token()` to create RSA-signed JWTs (RS256, kid="test-kid") — though signature verification is disabled, the JWTs are structurally valid
- `CachedExchangeResult` struct is redefined locally (serialization-compatible with `auth_middleware/token_service.rs`) to avoid coupling to `auth_middleware` internals
- Returns `anyhow::Result<String>` (plan proposed bare `String`) for proper error propagation

**POC test file:** `crates/server_app/tests/test_external_token_poc.rs` (3 tests)

| # | Test | What it proves |
|---|------|---------------|
| 1 | `test_external_token_cache_bypass_toolsets_list` | OAuth token with `scope_user_user` → 200 OK on GET /bodhi/v1/toolsets |
| 2 | `test_external_token_cache_bypass_missing_scope_rejected` | OAuth token without `scope_user_user` → 401 on GET /bodhi/v1/toolsets |
| 3 | `test_external_token_rejected_on_session_only_endpoint` | OAuth token with `scope_user_user` → 401 on GET /bodhi/v1/toolsets/{id} (session-only) |

**Test 1 (200 OK test) uses a custom router setup** with real `DefaultToolService` (backed by real SQLite DB via `AppServiceStubBuilder`), `DefaultExaService`, and real `TimeService`. This was required because `build_test_router()` uses a default `MockToolService` with no expectations, which panics when the toolset list handler calls `tool_service.list()`.

**Tests 2 and 3 (401 tests) use `build_test_router()`** since the auth middleware rejects the request before reaching the tool service handler.

### E2E tests that could move to server_app with this approach

| E2E Test | What it tests | Cache bypass feasibility |
|----------|--------------|-------------------------|
| OAuth token CRUD blocking (2 tests) | OAuth token → 401 on session-only endpoints | **High** — proven by POC test 3 |
| OAuth scope: WITH scope → can list/execute | Scope-based toolset access | **High** — proven by POC test 1 |
| OAuth scope: WITHOUT scope → execute denied | Missing scope rejection | **High** — proven by POC test 2 |
| OAuth scope: empty list without config | No toolset access | **High** — seed empty scope |
| OAuth scope: invalid_scope error | Keycloak rejects bad scope | **Low** — needs real Keycloak rejection |

### Files modified
- `crates/server_app/tests/utils/external_token.rs` — NEW: `ExternalTokenSimulator` utility
- `crates/server_app/tests/utils/mod.rs` — added `mod external_token` and `pub use external_token::*`
- `crates/server_app/tests/test_external_token_poc.rs` — NEW: 3 POC tests
- `crates/server_app/Cargo.toml` — added dev-dependencies: `chrono`, `serde`, `sha2`, `tower`, `uuid`
- `Cargo.lock` — updated for new dependencies

### Verification
```bash
cargo test -p server_app --test test_external_token_poc  # 3 passed, 0 failed
cargo test -p server_app --lib                           # 21 passed, 0 failed
cargo test -p routes_app                                 # 437 passed, 0 failed
```

### Deviations from plan

1. **Return type**: `create_token_with_scope` returns `anyhow::Result<String>` (plan proposed bare `String`) — better error propagation with `build_token()` which can fail
2. **JWT construction**: Uses `build_token()` from `services::test_utils` (RSA-signed) instead of plain `build_test_jwt()` — structurally valid JWTs even though signatures aren't checked
3. **Bearer JWT claims**: Includes `jti`, `sub`, `scope` in addition to `exp` — more realistic token structure
4. **Test 1 required custom router**: `build_test_router()` default `MockToolService` panics on `list()`. Fixed by building router with real `DefaultToolService` wired from `AppServiceStub` fields (`db_service`, `time_service`, `DefaultExaService::new()`)
5. **No `.env.test` changes needed**: The cache bypass approach doesn't require any environment variables — all state is in-memory
6. **Documentation**: Feasibility findings documented in this plan file rather than a separate file in `ai-docs/claude-plans/20260215-e2e-cleanup/`

---

## Summary: E2E Test Impact

| Action | Tests | Net E2E Reduction | Status |
|--------|-------|-------------------|--------|
| Phase 1: Move to routes_app | 5 API token blocking | -5 | **COMPLETED** (`b95df54a`) |
| Phase 2a: Delete canonical-redirect | 2 tests | -2 | **COMPLETED** (`9dd90d2f`) |
| Phase 2b: Delete app-initializer | 4 tests | -4 | **COMPLETED** (`9dd90d2f`) |
| Phase 3: Server_app client reuse | infrastructure | N/A | **COMPLETED** (`c3348f40`) |
| Phase 4: Cache bypass POC | 3 POC tests added | +3 server_app tests | **COMPLETED** (`27774045`) |
| **Total E2E reduction** | | **-11** | |

## Phase Execution Order (as executed)

1. **Phase 1** (`b95df54a`) — routes_app migration: 5 tests moved, 437 routes_app tests pass
2. **Phase 2** (`9dd90d2f`) — E2E deletion: 6 tests removed (2 + 4)
3. **Phase 3** (`c3348f40`) — server_app client reuse: already completed prior
4. **Phase 4** (`27774045`) — cache bypass POC: 3 new server_app tests proving feasibility
