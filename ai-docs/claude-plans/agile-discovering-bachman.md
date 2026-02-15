# E2E Test Cleanup: Migration to routes_app & server_app

## Context

BodhiApp's E2E Playwright suite has ~20 spec files with 50+ tests. Many test API contracts and auth rejection at the E2E level when they belong lower in the test pyramid. This plan covers 4 phases:

1. Move API token auth rejection tests to `routes_app` (fast unit tests)
2. Remove E2E tests already covered by existing unit/Vitest tests
3. Eliminate per-test Keycloak resource client creation in `server_app`
4. Explore cache bypass for testing 3rd party OAuth flows in `server_app`

Each phase is independently committable with all tests passing.

---

## Phase 1: Move API Token Blocking Tests to routes_app

### What moves

From `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`, the **"API Token Blocking - Toolset Endpoints"** describe block (5 tests):

| # | E2E Test | Endpoint | Assertion |
|---|----------|----------|-----------|
| 1 | GET /toolsets with API token | `GET /bodhi/v1/toolsets` | 401 |
| 2 | GET /toolsets/{id} with API token | `GET /bodhi/v1/toolsets/{id}` | 401 |
| 3 | PUT /toolsets/{id} with API token | `PUT /bodhi/v1/toolsets/{id}` | 401 |
| 4 | DELETE /toolsets/{id} with API token | `DELETE /bodhi/v1/toolsets/{id}` | 401 |
| 5 | POST /toolsets/{id}/execute/{method} with API token | `POST /bodhi/v1/toolsets/{id}/execute/{method}` | 401 |

### Why they can move

These tests make a single HTTP request with a `Bearer bodhiapp_*` token and assert 401. They don't use a browser, don't test UI, and don't require Keycloak OAuth flows. They test that session-only endpoints reject API tokens - a pure authorization layer concern.

### What stays in E2E

The remaining tests in `toolsets-auth-restrictions.spec.mjs` stay:
- "GET /toolsets with session auth returns toolset_types field" (1 test) - needs configured Exa toolset
- "OAuth Token + Toolset Scope Combinations" (4 tests) - need real Keycloak OAuth flows with scope configuration
- "OAuth Token - Toolset CRUD Endpoints (Session-Only)" (2 tests) - need real OAuth tokens; can move later with cache bypass (Phase 4)

### Implementation

**Destination file:** `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs`

**Pattern:** Create a real API token in the database, send as `Authorization: Bearer` header, assert 401.

```rust
// New test function to add:
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
  let (router, app_service, _temp) = build_test_router().await?;
  // 1. Create API token in database
  let token = create_test_api_token(app_service.db_service().as_ref()).await?;
  // 2. Build request with Bearer token
  let request = api_token_request(method, path, &token);
  // 3. Assert 401
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status(),
    "API token should be rejected on session-only endpoint {method} {path}");
  Ok(())
}
```

**New test utilities needed** in `crates/routes_app/src/test_utils/`:

1. `create_test_api_token(db_service) -> String` - creates a `bodhiapp_*` token in DB, returns the raw token string
2. `api_token_request(method, path, token) -> Request<Body>` - builds request with `Authorization: Bearer {token}` and `Host: localhost:1135` headers

**Existing utilities to reuse:**
- `build_test_router()` from `crates/routes_app/src/test_utils/router.rs`
- `DbService::create_api_token()` for token insertion
- Token hash computation via `sha2::Sha256`

### E2E cleanup

Remove the "API Token Blocking - Toolset Endpoints" describe block from `toolsets-auth-restrictions.spec.mjs`. The remaining 7 tests stay in the file.

### Files modified
- `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs` - add test
- `crates/routes_app/src/test_utils/mod.rs` - add `create_test_api_token`, `api_token_request` helpers
- `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` - remove 5 tests

### Verification
```bash
cargo test -p routes_app -- toolset   # new tests pass
cd crates/lib_bodhiserver_napi && npm run test  # remaining E2E tests pass
```

---

## Phase 2: Remove Duplicate E2E Tests

### 2a: Delete canonical-redirect.spec.mjs

**Why:** The canonical redirect middleware already has **15 comprehensive rstest unit tests** in `crates/auth_middleware/src/canonical_url_middleware.rs:236-516` covering:
- Wrong scheme, wrong host, wrong port redirects
- Non-standard port handling
- Query parameter preservation
- POST method exemption
- Health/ping path exemptions
- Canonical disabled setting
- Missing public host setting

The E2E spec tests only 2 scenarios (enabled/disabled) - a strict subset of the existing coverage. The redirect is HTTP-level (301 MOVED_PERMANENTLY at line 81) and fully testable without a browser.

**Action:** Delete `crates/lib_bodhiserver_napi/tests-js/specs/settings/canonical-redirect.spec.mjs`

### 2b: Delete app-initializer.spec.mjs

**Why:** The 4 E2E tests are covered by other tests:

| E2E Test | Already Covered By |
|----------|-------------------|
| Redirect to /ui/setup/ when status=setup | `AppInitializer.test.tsx` (25+ Vitest tests with MSW) |
| Redirect to /ui/login/ when unauthenticated | `AppInitializer.test.tsx` (role-based routing tests) |
| OAuth flow from intercepted route | Other E2E: `public-host-auth.spec.mjs`, `setup-flow.spec.mjs`, every test using `LoginPage` |
| OAuth flow from protected route | Same coverage as above |

**Coverage verification:**
- `crates/bodhi/src/components/AppInitializer.test.tsx` - 25+ tests covering loading states, error handling, status-based routing, role-based access control, authentication behavior
- OAuth login flow is exercised by ~15 other E2E specs that use `LoginPage.performLogin()`

**Action:** Delete `crates/lib_bodhiserver_napi/tests-js/specs/auth/app-initializer.spec.mjs`

### Files modified
- Delete `crates/lib_bodhiserver_napi/tests-js/specs/settings/canonical-redirect.spec.mjs`
- Delete `crates/lib_bodhiserver_napi/tests-js/specs/auth/app-initializer.spec.mjs`
- May need to clean up imports in any shared fixtures/page objects if they were only used by these files

### Verification
```bash
cd crates/lib_bodhiserver_napi && npm run test  # all remaining E2E tests pass
cargo test -p auth_middleware  # canonical redirect unit tests still pass
cd crates/bodhi && npm test  # AppInitializer Vitest tests still pass
```

---

## Phase 3: Server_app Resource Client Reuse

### Problem

`setup_minimal_app_service()` in `crates/server_app/tests/utils/live_server_utils.rs:33-275` makes **3 Keycloak API calls per test**:

1. `create_resource_client("integration_test")` - POST `/realms/{realm}/bodhi/resources` (line 158-160)
2. `get_resource_service_token(&resource_client)` - POST token endpoint (line 161-163)
3. `make_first_resource_admin(&resource_token, &test_user_id)` - POST make-admin endpoint (line 164-166)

With 3 serial tests, that's 9 unnecessary Keycloak API calls per test run.

### Solution

Pre-register a resource client in Keycloak. Add its credentials to `.env.test`. Restructure `setup_minimal_app_service()` to read from env vars instead of dynamic creation.

### Implementation

**1. Add new env vars to `.env.test`:**
```bash
# Pre-registered resource client (created once in Keycloak, reused across tests)
INTEG_TEST_RESOURCE_CLIENT_ID=bodhi-test-resource-client
INTEG_TEST_RESOURCE_CLIENT_SECRET=<secret>
INTEG_TEST_RESOURCE_CLIENT_SCOPE=scope_bodhi-test-resource-client
```

**2. Restructure `setup_minimal_app_service()`:**

Remove lines 144-178 (AuthServerTestClient creation, resource client creation, service token, make admin). Replace with reading env vars:

```rust
// BEFORE: Dynamic client creation (3 Keycloak API calls)
let auth_client = AuthServerTestClient::new(config);
let resource_client = auth_client.create_resource_client("integration_test").await?;
let resource_token = auth_client.get_resource_service_token(&resource_client).await?;
auth_client.make_first_resource_admin(&resource_token, &test_user_id).await?;

// AFTER: Pre-registered client (0 Keycloak API calls)
let client_id = std::env::var("INTEG_TEST_RESOURCE_CLIENT_ID")?;
let client_secret = std::env::var("INTEG_TEST_RESOURCE_CLIENT_SECRET")?;
let client_scope = std::env::var("INTEG_TEST_RESOURCE_CLIENT_SCOPE")?;
```

**3. Update AppRegInfo construction** (currently line 168-178):

```rust
let app_reg_info = AppRegInfoBuilder::default()
  .client_id(client_id.clone())
  .client_secret(client_secret)
  .scope(client_scope)
  .build()?;
```

**4. Token acquisition stays per-test:**

`get_oauth_tokens()` (lines 316-358) still makes 1 Keycloak call (password grant) per test. This is acceptable - tokens expire, and tests are 5-10 minutes each.

**5. Dev console credentials may become optional:**

The `INTEG_TEST_DEV_CONSOLE_CLIENT_ID` and `INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET` env vars were only needed for `AuthServerTestClient`. If no test uses dynamic client creation anymore, these can be removed from `.env.test`. Check if any test still needs them before removing.

### Files modified
- `crates/server_app/tests/utils/live_server_utils.rs` - restructure `setup_minimal_app_service()`
- `crates/server_app/tests/resources/.env.test` - add resource client credentials
- `crates/server_app/tests/resources/.env.test.example` - add example values

### Verification
```bash
cargo test -p server_app  # all 3 existing tests pass with pre-registered client
```

---

## Phase 4: Cache Bypass Exploration for 3rd Party OAuth in server_app

### Goal

Enable `server_app` tests to simulate 3rd party OAuth API calls without browser-based Keycloak flows, by seeding the token validation cache.

### How token validation works (from `crates/auth_middleware/src/token_service.rs:51-171`)

```
Bearer token arrives (non-bodhiapp_ prefix)
  → extract_claims::<ExpClaims>(bearer_token)     // Parse JWT, check exp (line 120)
  → SHA-256 digest → first 12 hex chars             // Cache key (lines 126-128)
  → cache_service.get("exchanged_token:{digest}")   // Cache lookup (line 133)
  → If HIT:
      → Deserialize CachedExchangeResult { token, azp }  // (line 135)
      → extract_claims::<ScopeClaims>(&cached.token)      // Parse exchange token (line 136)
      → Check exp, extract UserScope from scope claim     // (lines 137-140)
      → Return (token, ResourceScope::User(scope), azp)   // Skip auth server!
  → If MISS:
      → handle_external_client_token() → auth server call // (line 160)
```

### Cache seeding approach

To test a 3rd party OAuth flow in server_app without a browser:

1. **Create a fake external JWT** with valid structure and non-expired `exp` claim
2. **Compute its SHA-256 digest** → first 12 hex chars = cache key
3. **Create a valid exchange result JWT** with the desired `scope` and non-expired `exp`
4. **Serialize `CachedExchangeResult { token, azp }` to JSON**
5. **Seed the cache:** `cache_service.set("exchanged_token:{digest}", json_string)`
6. **Send requests** with the fake JWT as `Authorization: Bearer {fake_jwt}`
7. **The middleware** will find the cache hit and use the seeded scope

### Key investigation items

**Critical question: Does `extract_claims()` verify JWT signatures?**

- `extract_claims` is imported from `services` crate (`crates/services/src/token.rs`)
- If it uses `jsonwebtoken::decode()` with `Validation::default()`, it verifies signatures → can't use fake JWTs
- If it uses `jsonwebtoken::dangerous_insecure_decode()` or `Validation { validate_signature: false, .. }`, any well-formed JWT works
- **Investigation:** Read the `extract_claims()` implementation in `crates/services/src/token.rs`

**If signatures ARE verified:**
- Use a real Keycloak-signed token from password grant as the "external" bearer token
- The token's `azp` would be the resource client, not a true 3rd party - but for testing scope-based access, this may suffice
- Alternatively, create a 2nd Keycloak client that issues tokens with different `azp`

**If signatures are NOT verified:**
- Create a minimal JWT with `{ "exp": future_timestamp }` for the bearer token
- Create a minimal JWT with `{ "exp": future_timestamp, "scope": "scope_user_user", "azp": "test-app" }` for the exchange result
- This is the ideal scenario for test flexibility

### Proposed test utility

```rust
// In crates/server_app/tests/utils/
pub struct ExternalTokenSimulator {
  cache_service: Arc<dyn CacheService>,
}

impl ExternalTokenSimulator {
  /// Creates a fake external JWT and seeds the cache so requests with this
  /// token bypass Keycloak and resolve to the given scope.
  pub fn create_token_with_scope(&self, scope: &str, azp: &str) -> String {
    // 1. Build fake bearer JWT
    let bearer_jwt = build_test_jwt(&json!({"exp": future_exp()}));
    // 2. Compute cache key
    let digest = sha256_first_12(&bearer_jwt);
    // 3. Build exchange result JWT with desired scope
    let exchange_jwt = build_test_jwt(&json!({
      "exp": future_exp(),
      "scope": scope,
      "azp": azp,
    }));
    // 4. Seed cache
    let cached = CachedExchangeResult { token: exchange_jwt, azp: azp.to_string() };
    self.cache_service.set(
      &format!("exchanged_token:{}", digest),
      &serde_json::to_string(&cached).unwrap(),
    );
    bearer_jwt
  }
}
```

### E2E tests that could move to server_app with this

| E2E Test | What it tests | Cache bypass feasibility |
|----------|--------------|-------------------------|
| OAuth token CRUD blocking (2 tests) | OAuth token → 401 on session-only endpoints | High - just need any non-session auth type |
| OAuth scope: WITH scope → can list/execute | Scope-based toolset access | High - seed scope, test API |
| OAuth scope: WITHOUT scope → execute denied | Missing scope rejection | High - seed without scope |
| OAuth scope: empty list without config | No toolset access | High - seed empty scope |
| OAuth scope: invalid_scope error | Keycloak rejects bad scope | Low - needs real Keycloak rejection |

### Deliverable

- Document feasibility findings in `ai-docs/claude-plans/20260215-e2e-cleanup/`
- If feasible: implement `ExternalTokenSimulator` utility, write 1 POC test
- If not feasible: document blockers and alternative approaches

### Files to investigate/modify
- `crates/services/src/token.rs` - read `extract_claims()` implementation
- `crates/auth_middleware/src/token_service.rs` - understand `CachedExchangeResult` (line 17-21)
- `crates/server_app/tests/utils/` - add new test utility module
- `crates/server_app/tests/resources/.env.test` - may need additional env vars

### Verification
```bash
cargo test -p server_app  # POC test passes
```

---

## Summary: E2E Test Impact

| Action | Tests | Net E2E Reduction |
|--------|-------|-------------------|
| Phase 1: Move to routes_app | 5 API token blocking | -5 |
| Phase 2a: Delete canonical-redirect | 2 tests | -2 |
| Phase 2b: Delete app-initializer | 4 tests | -4 |
| Phase 4 (if feasible): Move OAuth tests | up to 4 tests | -4 |
| **Total** | | **-11 to -15** |

## Phase Execution Order

1. **Phase 1** (routes_app migration) - independent, can run first
2. **Phase 2** (E2E deletion) - independent, can run in parallel with Phase 1
3. **Phase 3** (server_app client reuse) - independent of 1 & 2
4. **Phase 4** (cache bypass) - depends on Phase 3 (needs restructured server_app setup)
