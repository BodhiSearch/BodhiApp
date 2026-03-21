# Multi-Tenant Isolation Tests — server_app Integration Kickoff

## Goal

Add end-to-end multi-tenant isolation tests at the `crates/server_app/` level. These tests spin up a real HTTP server (TCP listener on port 51135), make real HTTP requests via `reqwest`, and verify cross-tenant data isolation through the full stack: HTTP → middleware → auth resolution → auth-scoped services → DB.

This is the highest-fidelity test layer — it proves the entire pipeline works, including middleware tenant resolution from bearer tokens and session cookies.

See also:
- `kickoff-multi-tenant-rs-test.md` — services crate (auth-scoped service isolation)
- `kickoff-multi-tenant-routes-app-test.md` — routes_app HTTP handler isolation

---

## Context: server_app Test Architecture

### How Integration Tests Work
- Real TCP server on `127.0.0.1:51135` via `ServeCommand::get_server_handle()`
- HTTP requests via `reqwest::Client`
- `#[serial_test::serial(live)]` — tests run sequentially (shared port)
- `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`, return `-> anyhow::Result<()>`
- Server shutdown via `handle.shutdown().await?` at end of test

### Two Bootstrap Variants

**A. `setup_minimal_app_service()`** — requires real Keycloak:
- Loads `tests/resources/.env.test` (gitignored) with `INTEG_TEST_AUTH_URL`, `INTEG_TEST_*` credentials
- Real OAuth token exchange via Keycloak
- Creates tenant via `tenant_service.create_tenant(client_id, client_secret, AppStatus::Ready)`
- Gets real `(access_token, refresh_token)` via password grant
- Used by: `test_live_tool_calling_*.rs`, `test_live_mcp.rs`, `test_live_agentic_chat_*.rs`

**B. `setup_test_app_service()`** — no Keycloak needed:
- Uses fake `INTEG_TEST_*` defaults (auth URL = `"https://test-id.getbodhi.app"`)
- Tenant created with dummy credentials
- Works with `ExternalTokenSimulator` that seeds token validation cache directly
- Used by: `test_oauth_external_token.rs`

### ExternalTokenSimulator (Keycloak Bypass)
**File**: `crates/server_app/tests/utils/external_token.rs`
- Bypasses real OAuth by pre-seeding `MokaCacheService` with `CachedExchangeResult`
- `create_token_with_role(role, azp)` → generates fake bearer JWT, seeds cache, returns token
- Cache key = `exchanged_token:{sha256(bearer)[0..32]}`
- Token service checks cache before calling auth server → finds pre-seeded result
- **This is the key pattern for testing multi-tenant without Keycloak**

### Session-Based Auth (for tests)
**`create_test_session_for_live_server(app_service, roles)`**:
- Builds JWT claims with specified roles under `resource_access[client_id]["roles"]`
- Signs token via `build_token(claims)`
- Creates session record in session store
- Returns `(session_cookie: String, user_id: String)`

### AppService Build (Multi-Tenant Decision Point)
**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`
```
let is_multi_tenant = setting_service.is_multi_tenant().await;

let data_service = if is_multi_tenant {
  MultiTenantDataService::new(db_service)    // API aliases only, no local models
} else {
  LocalDataService::new(hub_service, db_service)  // Full standalone
};

let inference_service = if is_multi_tenant {
  MultitenantInferenceService::new(ai_api_service)  // Remote-only
} else {
  StandaloneInferenceService::new(shared_context, ai_api_service)  // Local LLM
};
```
**Note**: Current test setup always uses standalone mode. Multi-tenant bootstrap path is untested.

### Test Files
| File | Auth Method | Keycloak? |
|------|-------------|-----------|
| `test_live_tool_calling_non_streamed.rs` | Session cookie (real OAuth) | Yes |
| `test_live_tool_calling_streamed.rs` | Session cookie (real OAuth) | Yes |
| `test_live_mcp.rs` | Session cookie (real OAuth) | Yes |
| `test_live_agentic_chat_with_exa.rs` | Session cookie (real OAuth) | Yes |
| `test_oauth_external_token.rs` | Bearer token (ExternalTokenSimulator) | No |

---

## What to Test

### Part A: Multi-Tenant Bootstrap

First, verify the multi-tenant bootstrap path works:
- Set `BODHI_DEPLOYMENT=multi` in app settings
- Verify `MultiTenantDataService` and `MultitenantInferenceService` are wired
- Verify LLM routes return appropriate errors (501 or 404)
- This is currently untested at the integration level

### Part B: Cross-Tenant Isolation via ExternalTokenSimulator

Using the no-Keycloak path (`setup_test_app_service()` + `ExternalTokenSimulator`):

**Test pattern**:
1. Start test server with `start_test_live_server()`
2. Create two tenants in the DB (tenant A and tenant B with different client_ids)
3. Use `ExternalTokenSimulator::new_with_client_id(app_service, client_id_a)` for tenant A
4. Use `ExternalTokenSimulator::new_with_client_id(app_service, client_id_b)` for tenant B
5. Create resources under both tenants via direct service calls
6. Make HTTP requests with tenant A's bearer token → verify only tenant A's data
7. Make HTTP requests with tenant B's bearer token → verify only tenant B's data

**Key question**: Does `ExternalTokenSimulator` resolve the correct tenant_id when two tenants exist? It seeds the cache with a `CachedExchangeResult` containing the `azp` claim — the auth middleware then looks up the tenant by client_id. Verify this pipeline works with multiple tenants.

### Part C: Cross-Tenant Isolation via API Tokens

API tokens embed the client_id: `bodhiapp_<random>.<client_id>`

**Test pattern**:
1. Create two tenants with different client_ids
2. Create API tokens for each tenant (token suffix = respective client_id)
3. Make requests with tenant A's API token → verify tenant A's data only
4. Make requests with tenant B's API token → verify tenant B's data only

This tests the token validation middleware path: parse suffix → `get_tenant_by_client_id()` → resolve tenant_id → build `AuthContext::ApiToken`.

### Part D: Cross-Tenant Write Isolation via HTTP

- POST to create a resource as tenant A → verify it's stored under tenant A
- GET as tenant B → verify tenant A's resource is invisible
- PUT/DELETE tenant A's resource as tenant B → verify 404 (not 403)

---

## Key Files to Read First

### Test Infrastructure
- `crates/server_app/tests/utils/live_server_utils.rs` — `setup_test_app_service()`, `start_test_live_server()`, `TestLiveServer`, `create_test_session_for_live_server()`
- `crates/server_app/tests/utils/external_token.rs` — `ExternalTokenSimulator`
- `crates/server_app/tests/utils/mod.rs` — module structure

### Representative Test (study this first)
- `crates/server_app/tests/test_oauth_external_token.rs` — simplest integration test, no Keycloak, shows the full pattern

### Bootstrap
- `crates/lib_bodhiserver/src/app_service_builder.rs` — `build_app_service()`, multi-tenant branching
- `crates/server_app/src/serve.rs` — `ServeCommand::get_server_handle()`

### Auth Pipeline
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — tenant resolution from JWT/token
- `crates/routes_app/src/middleware/token_service/token_service.rs` — API token → tenant lookup

### Plan Files
- `ai-docs/claude-plans/20260303-multi-tenant/20260303-multi-tenant-rls.md` — sections C (Token Identity) and E (Inference Routing)
- `ai-docs/claude-plans/20260303-multi-tenant/decisions.md` — D5 (BODHI_DEPLOYMENT), D10 (conditional routes), D13 (session-based routing)
- `ai-docs/claude-plans/20260303-multi-tenant/TECHDEBT.md` — P0-12 (anonymous tenant resolution), P0-15 (cross-tenant JWT), P0-16 (middleware branching), P2-8 (cross-tenant integration tests)

---

## Open Questions (explore, don't prescribe)

1. **Multi-tenant bootstrap test feasibility**: Can we set `BODHI_DEPLOYMENT=multi` in `setup_test_app_service()` and verify the bootstrap path? What breaks? Does `MultiTenantDataService` work with the test infrastructure?

2. **ExternalTokenSimulator with two tenants**: The simulator takes `app_service` and a `client_id`. When two tenants exist, does `get_tenant_by_client_id()` in the auth middleware correctly resolve each? Need to verify the full pipeline: cache hit → extract `azp` → tenant lookup.

3. **Port sharing**: All tests use port 51135 with `#[serial_test::serial(live)]`. Is this sufficient for multi-tenant tests, or do we need a way to run two servers (one per tenant) in parallel?

4. **Session-based multi-tenant**: The session path calls `get_standalone_app()` (assumes one tenant). Per TECHDEBT P0-16, this needs to branch for multi-tenant. Should integration tests expose this gap, or should we fix the middleware first?

5. **PostgreSQL integration**: Should any tests here verify RLS enforcement on real PostgreSQL? The Docker-based PostgreSQL tests in `services/src/db/test_rls.rs` test at the raw SQL level. Integration tests could verify the full stack with RLS as defense-in-depth.

6. **Test placement**: Should new multi-tenant integration tests go in existing test files or new ones like `test_multi_tenant_isolation.rs`?
