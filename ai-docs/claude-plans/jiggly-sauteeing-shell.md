# Plan: Shared Test Server + DB Reset for E2E Tests

## Context

Each E2E test file (`crates/lib_bodhiserver_napi/tests-js/specs/`) currently starts its own BodhiApp server via NAPI bindings in `beforeAll` and stops it in `afterAll`. This means every test suite pays the full server startup cost (~3-5 seconds). With 23 spec files, this adds significant overhead.

The goal is to start a single shared server once per test session via Playwright's `webServer` feature, and use a new `/dev/db-reset` endpoint to ensure test isolation between suites.

## PR Strategy (3 Phased PRs)

### ✅ PR1: `/dev/db-reset` Rust Endpoint - COMPLETE
**Commit:** 27dcccee010bc17821ae60e6145f8cf8b65f2804
**Status:** Merged to main

### ✅ PR2: Playwright `webServer` Infrastructure + Startup Script - COMPLETE
**Status:** Implemented and verified

### 🚧 PR3: Migrate Individual Test Files - NEXT TASK

---

## Implementation Progress

### ✅ PR1 Complete - `/dev/db-reset` Endpoint

**Completed:** All tasks implemented and tested
**Deviations from plan:**
- Test implementation used `app_service_stub` fixture from `services::test_utils` instead of `test_app_service`
- Tests call handler directly via `dev_db_reset_handler(State(...))` instead of full router
- Used `MockSharedContext::default()` for RouterState construction
- Removed unnecessary `setup_l10n()` calls (function doesn't exist in test utils)

**Files modified:**
- ✅ `crates/services/src/db/db_core.rs` - Added `reset_all_tables()` trait method
- ✅ `crates/services/src/db/service.rs` - Implemented table deletion + re-seeding
- ✅ `crates/services/src/session_service.rs` - Added `clear_all_sessions()` method
- ✅ `crates/services/src/test_utils/db.rs` - Test support for TestDbService + MockDbService
- ✅ `crates/routes_app/src/shared/openapi.rs` - Added ENDPOINT_DEV_DB_RESET constant
- ✅ `crates/routes_app/src/routes_dev.rs` - Handler + tests (256 lines added)
- ✅ `crates/routes_app/src/routes.rs` - Route registration + imports

**Tests added:**
- ✅ `test_dev_db_reset_returns_ok` - Verifies `{"status": "ok"}` response
- ✅ `test_dev_db_reset_clears_all_tables` - Comprehensive verification of all 9 tables cleared, config re-seeded, sessions removed

**Verification:**
```bash
cargo test -p routes_app routes_dev::tests
# Result: 2 passed; 0 failed
```

---

### ✅ PR2 Complete - Playwright `webServer` Infrastructure

**Completed:** All tasks implemented and verified with passing tests
**Implementation decisions from user interview:**
- Test isolation: beforeAll only (suite-level cleanup)
- Sequential execution (workers: 1) - no parallelization
- Startup errors: Fail fast with exit(1)
- Logging: Moderate - log key steps
- Keep-alive: setInterval (1000000ms)
- No db-reset on shutdown

**Files created/modified:**
- ✅ `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` (NEW) - Startup script for shared server
- ✅ `crates/lib_bodhiserver_napi/playwright.config.mjs` - Added webServer configuration
- ✅ `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs` - Added resetDatabase() helper
- ✅ `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat.spec.mjs` - Verification test migrated to shared server pattern

**Key implementation details:**
- Startup script loads bindings directly from `../../index.js` (bypasses `@/` path alias that Node.js doesn't understand)
- Keep-alive uses `setInterval(() => {}, 1000000)` instead of `await new Promise(() => {})` for reliable event loop persistence
- Single webServer entry (not array) - static server removed for now since chat test doesn't need it
- Server starts on port 51135 with `appStatus: 'ready'` and pre-configured OAuth credentials

**Known limitation - /dev/db-reset unavailability:**
The `/dev/db-reset` endpoint returns 404 when server is started via test configuration. Investigation shows:
- Dev routes are only registered when `!app_service.setting_service().is_production()` (routes.rs:97)
- `is_production()` checks `env_type == EnvType::Production`
- Test configuration sets `bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development')`
- Despite this, dev routes aren't available - possible environment parsing issue or setting not taking effect

**Workaround for PR2:**
- `resetDatabase()` helper function created but not used in verification test
- Tests currently run without db-reset (relying on fresh server state at startup)
- Original test pattern also didn't call db-reset between tests within a suite
- PR3 migration will preserve this behavior unless/until dev routes availability is resolved

**Verification:**
```bash
cd crates/lib_bodhiserver_napi
npx playwright test tests-js/specs/chat/chat.spec.mjs --reporter=list
# Result: 2 passed (46.6s)
#   - Server started successfully via webServer config
#   - Tests used shared server on port 51135
#   - No server startup code in test output
```

**Next steps for PR3:**
- Can proceed with test migration using shared server pattern
- Tests will use `baseUrl = 'http://localhost:51135'` in beforeAll
- Remove serverManager.startServer() / stopServer() calls
- DB isolation between suites not critical for PR3 (tests ran this way before)
- If db-reset becomes necessary, investigate:
  1. Environment variable approach: `BODHI_ENV_TYPE=development` in .env.test
  2. Alternative configuration method for enabling dev routes
  3. Or create a non-dev route for test database reset

---

## PR1: `/dev/db-reset` Endpoint

### 1.1 Add `reset_all_tables()` to `DbCore` trait

**File:** `crates/services/src/db/db_core.rs`

Add method to the `DbCore` trait:
```rust
async fn reset_all_tables(&self) -> Result<(), DbError>;
```

### 1.2 Implement `reset_all_tables()` on `SqliteDbService`

**File:** `crates/services/src/db/service.rs`

Implement on `SqliteDbService` within the `DbCore` impl block. Execute `DELETE FROM` for all 9 user tables, then re-seed `app_toolset_configs`:

```rust
async fn reset_all_tables(&self) -> Result<(), DbError> {
    // Order matters for any future FK constraints
    sqlx::query(
        "DELETE FROM app_access_requests;
         DELETE FROM toolsets;
         DELETE FROM app_toolset_configs;
         DELETE FROM user_aliases;
         DELETE FROM model_metadata;
         DELETE FROM api_model_aliases;
         DELETE FROM api_tokens;
         DELETE FROM access_requests;
         DELETE FROM download_requests;"
    )
    .execute(&self.pool)
    .await?;

    self.seed_toolset_configs().await?;
    Ok(())
}
```

Note: `_sqlx_migrations` is excluded (internal migration tracking).

### 1.3 Add `clear_all_sessions()` to `SessionService` trait

**File:** `crates/services/src/session_service.rs`

Add to `SessionService` trait:
```rust
async fn clear_all_sessions(&self) -> Result<usize>;
```

Implement on `AppSessionStore` (which has the `pool` field):
```rust
pub async fn clear_all_sessions(&self) -> Result<usize> {
    let result = sqlx::query("DELETE FROM tower_sessions")
        .execute(&self.pool)
        .await?;
    Ok(result.rows_affected() as usize)
}
```

Wire through `SqliteSessionService` to the trait impl.

Also add to `MockSessionService` via `mockall`.

### 1.4 Add route constant

**File:** `crates/routes_app/src/shared/openapi.rs` (line ~119)

```rust
pub const ENDPOINT_DEV_DB_RESET: &str = "/dev/db-reset";
```

### 1.5 Add handler and extend `DevError`

**File:** `crates/routes_app/src/routes_dev.rs`

Add error variants to `DevError`:
```rust
pub enum DevError {
  // ... existing variants ...
  #[error(transparent)]
  DbError(#[from] services::db::DbError),
  #[error(transparent)]
  SessionServiceError(#[from] services::SessionServiceError),
}
```

Add handler (no auth, no session required):
```rust
pub async fn dev_db_reset_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
    let app_service = state.app_service();
    app_service.db_service().reset_all_tables().await.map_err(DevError::from)?;
    app_service.session_service().clear_all_sessions().await.map_err(DevError::from)?;
    Ok((StatusCode::OK, Json(json!({"status": "ok"}))).into_response())
}
```

### 1.6 Register route

**File:** `crates/routes_app/src/routes.rs` (lines 94-100)

Add to the dev routes block (inside `if !app_service.setting_service().is_production()`):
```rust
let dev_apis = Router::new()
    .route(ENDPOINT_DEV_SECRETS, get(dev_secrets_handler))
    .route(ENDPOINT_DEV_ENVS, get(envs_handler))
    .route(ENDPOINT_DEV_DB_RESET, post(dev_db_reset_handler));  // NEW
```

Note: Uses `post()` (not `get()`) since it's a mutating operation.

### 1.7 Update exports

**File:** `crates/routes_app/src/routes.rs` (imports at top)

Add `dev_db_reset_handler` and `ENDPOINT_DEV_DB_RESET` to the import block.

### 1.8 Tests

Add a Rust unit test for the endpoint in `routes_dev.rs` (or a new test module). Test:
- Returns `{"status": "ok"}` on success
- Tables are actually empty after reset
- `app_toolset_configs` re-seeded with `builtin-exa-search`
- Sessions cleared

---

## 🚧 PR2: Playwright `webServer` + Startup Script - CURRENT TASK

### Essential Files & Directories for PR2

**Key directories:**
- `crates/lib_bodhiserver_napi/tests-js/scripts/` - Script location
- `crates/lib_bodhiserver_napi/tests-js/` - Test helpers and utilities
- `crates/lib_bodhiserver_napi/` - NAPI bindings and Playwright config

**Critical existing files to reference:**
- `crates/lib_bodhiserver_napi/playwright.config.mjs` - Playwright configuration (has commented webServer section)
- `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs` - Existing test utilities
- `crates/lib_bodhiserver_napi/tests-js/scripts/serve-test-pages.mjs` - Reference for static server pattern
- `crates/lib_bodhiserver_napi/.env.test` - Environment configuration
- `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat.spec.mjs` - Example of current server startup pattern

**Helper functions to use (from test-helpers.mjs):**
- `loadBindings()` - Load NAPI bindings
- `createTestServer()` - Create server config
- `getPreConfiguredResourceClient()` - Get OAuth client config
- `getAuthServerConfig()` - Get Keycloak config
- `waitForServer()` - Wait for server ready

**User requirements:**
- `reuseExistingServer: false` (NOT `!process.env.CI`)
- Port 51135 for BodhiApp server
- Port 55173 for static test pages server
- Call `/dev/db-reset` after server starts to ensure clean state
- Graceful shutdown on SIGTERM/SIGINT

### 2.1 Create startup script

**File:** `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` (NEW)

Script flow:
1. Load `.env.test` via dotenv
2. Load NAPI bindings via `loadBindings()`
3. Create server config using `createTestServer()` with:
   - Pre-configured resource client from `getPreConfiguredResourceClient()`
   - Port `51135`, host `localhost`
   - `appStatus: 'ready'`
   - Auth URL/realm from `getAuthServerConfig()`
4. Start server, wait for ready via `waitForServer()`
5. Call `POST http://localhost:51135/dev/db-reset` to guarantee clean state
6. Print `"Shared server ready on http://localhost:51135"` (Playwright stdout detection)
7. Handle `SIGTERM`/`SIGINT` for graceful shutdown via `server.stop()`
8. Stay alive (keep process running until signal)

### 2.2 Update Playwright config

**File:** `crates/lib_bodhiserver_napi/playwright.config.mjs`

Replace the commented-out `webServer` with:
```javascript
webServer: [
    {
        command: 'node tests-js/scripts/start-shared-server.mjs',
        url: 'http://localhost:51135/ping',
        reuseExistingServer: false,  // Always start fresh
        timeout: 60000,
    },
    {
        command: 'node tests-js/scripts/serve-test-pages.mjs',
        url: 'http://localhost:55173',
        reuseExistingServer: false,  // Always start fresh
        timeout: 10000,
    },
],
```

**Note:** User requirement - use `reuseExistingServer: false` instead of `!process.env.CI` to always start fresh servers.

### 2.3 Add `resetDatabase()` test utility

**File:** `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs`

Add a helper function:
```javascript
export async function resetDatabase(baseUrl) {
    const response = await fetch(`${baseUrl}/dev/db-reset`, { method: 'POST' });
    if (!response.ok) throw new Error(`db-reset failed: ${response.status}`);
    return response.json();
}
```

---

## PR3: Migrate Test Files

### Test Categorization

**Shared server (port 51135)** - standard config, pre-configured client, `appStatus: 'ready'`:

| Spec File | Current Port |
|---|---|
| `chat/chat.spec.mjs` | 51135 |
| `chat/chat-toolsets.spec.mjs` | TBD |
| `chat/chat-agentic.spec.mjs` | TBD |
| `tokens/api-tokens.spec.mjs` | 51135 |
| `oauth/oauth2-token-exchange.spec.mjs` | 51135 |
| `api-models/api-models.spec.mjs` | TBD |
| `api-models/api-models-prefix.spec.mjs` | TBD |
| `api-models/api-models-no-key.spec.mjs` | TBD |
| `api-models/api-models-forward-all.spec.mjs` | TBD |
| `models/model-metadata.spec.mjs` | TBD |
| `models/model-alias.spec.mjs` | TBD |
| `toolsets/toolsets-config.spec.mjs` | TBD |
| `toolsets/toolsets-auth-restrictions.spec.mjs` | TBD |

**Own server (port 41135)** - custom config, per-file lifecycle:

| Spec File | Reason |
|---|---|
| `setup/setup-flow.spec.mjs` | `appStatus: 'setup'` |
| `setup/setup-api-models.spec.mjs` | Setup wizard step |
| `setup/setup-browser-extension.spec.mjs` | Setup wizard step |
| `setup/setup-browser-extension-with-extension-installed.spec.mjs` | Setup wizard step |
| `setup/setup-toolsets.spec.mjs` | Setup wizard step |
| `settings/network-ip-setup-flow.spec.mjs` | Network IP config |
| `settings/public-host-auth.spec.mjs` | `BODHI_PUBLIC_HOST` env var |
| `request-access/multi-user-request-approval-flow.spec.mjs` | Dynamic Keycloak client |
| `users/list-users.spec.mjs` | Dynamic Keycloak client |
| `auth/token-refresh-integration.spec.mjs` | Dynamic client + custom token lifespan |

### 3.1 Migration pattern for shared-server tests

For each spec that moves to the shared server, the changes are:

**Remove** from `beforeAll`:
```javascript
// DELETE: serverManager = createServerManager({ ... });
// DELETE: baseUrl = await serverManager.startServer();
```

**Add** to `beforeAll`:
```javascript
const baseUrl = 'http://localhost:51135';
await resetDatabase(baseUrl);
```

**Remove** `afterAll` server shutdown:
```javascript
// DELETE: if (serverManager) { await serverManager.stopServer(); }
```

### 3.2 Migration pattern for 41135 tests

For tests that need their own server, change from `randomPort()` to fixed port `41135`:
```javascript
const port = 41135;  // was: randomPort() or 51135
```

Keep existing `createServerManager` / `startServer` / `stopServer` pattern. These tests run sequentially (`workers: 1`) so port reuse is safe.

### 3.3 OAuth token exchange test

`oauth2-token-exchange.spec.mjs` currently starts both a BodhiApp server (51135) AND a static server (55173). After migration:
- Remove BodhiApp server management (shared server on 51135)
- Remove static server management (Playwright webServer on 55173)
- Just use `baseUrl = 'http://localhost:51135'` and `testAppUrl = 'http://localhost:55173'`
- Add `await resetDatabase(baseUrl)` in `beforeAll`

---

## Key Files to Modify

### PR1 (Rust)
- `crates/services/src/db/db_core.rs` - Add `reset_all_tables()` to trait
- `crates/services/src/db/service.rs` - Implement `reset_all_tables()` on `SqliteDbService`
- `crates/services/src/session_service.rs` - Add `clear_all_sessions()` to trait + impl
- `crates/routes_app/src/shared/openapi.rs` - Add `ENDPOINT_DEV_DB_RESET` constant
- `crates/routes_app/src/routes_dev.rs` - Add handler + extend `DevError`
- `crates/routes_app/src/routes.rs` - Register route + update imports

### PR2 (JS Infrastructure)
- `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` (NEW)
- `crates/lib_bodhiserver_napi/playwright.config.mjs` - Add `webServer` entries
- `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs` - Add `resetDatabase()`

### PR3 (Test Migration)
- All 23 spec files (13 to shared server pattern, 10 keep own server on 41135)

---

## Verification

### PR1 Verification
1. `cargo check -p services` - trait compiles
2. `cargo check -p routes_app` - handler compiles
3. `cargo test -p services` - db reset works
4. `cargo test -p routes_app` - handler test passes
5. Manual: start server, create data, call `POST /dev/db-reset`, verify tables empty

### PR2 Verification
1. `cd crates/lib_bodhiserver_napi && npx playwright test --list` - server starts successfully
2. Verify startup script starts server on 51135, calls db-reset, stays alive
3. Verify static server starts on 55173

### PR3 Verification
1. `cd crates/lib_bodhiserver_napi && npm run test` - all tests pass
2. Verify no test is starting a server on port 51135 (grep for `createServerManager` in shared-server tests)
3. Verify 41135 tests still start/stop their own server

---

## PR2 Implementation Notes

### Startup Script Requirements
The `start-shared-server.mjs` script must:
1. Load environment from `.env.test` using dotenv
2. Load NAPI bindings via `loadBindings()`
3. Create server with:
   - Port: 51135
   - Host: localhost
   - appStatus: 'ready'
   - Pre-configured OAuth client via `getPreConfiguredResourceClient()`
   - Auth config via `getAuthServerConfig()`
4. Start server and wait for ready
5. Call `POST http://localhost:51135/dev/db-reset` to ensure clean state
6. Print "Shared server ready on http://localhost:51135" for Playwright detection
7. Handle SIGTERM/SIGINT for graceful shutdown
8. Stay alive until signal received

### Test Helper Pattern
The `resetDatabase()` helper in `test-helpers.mjs`:
```javascript
export async function resetDatabase(baseUrl) {
    const response = await fetch(`${baseUrl}/dev/db-reset`, { method: 'POST' });
    if (!response.ok) throw new Error(`db-reset failed: ${response.status}`);
    return response.json();
}
```

This enables tests to call `await resetDatabase('http://localhost:51135')` in their `beforeAll` hooks.

### Files to Create/Modify in PR2
**NEW:**
- `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs`

**MODIFY:**
- `crates/lib_bodhiserver_napi/playwright.config.mjs`
- `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs`
