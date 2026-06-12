# OAuth API Test Migration: E2E → server_app

## Context

The `ExternalTokenSimulator` (commit `0ba7e77d`) proved that OAuth token flows can be tested without Keycloak by seeding the `MokaCacheService` cache directly. The POC (`test_external_token_poc.rs`) demonstrates 3 scenarios with `router.oneshot()`.

**Problem**: The E2E test `toolsets-auth-restrictions.spec.mjs` tests OAuth scope combinations via Playwright + Keycloak. These tests are slow (~minutes), require external Keycloak, and test API-level assertions that don't need a browser. The `oauth2-token-exchange.spec.mjs` similarly has an API assertion (GET /user → 200) that should also exist as a fast server_app test.

**Outcome**: Move API-level OAuth tests to server_app (live TCP server, real services, no Keycloak). Keep E2E tests only where browser/Keycloak behavior is the thing being tested. Add documentation codifying the testing philosophy.

---

## Key Decisions (from user interview)

| Decision | Choice |
|----------|--------|
| server_app test transport | Always live TCP server (never `router.oneshot()`) |
| Data setup approach | Session-authenticated API calls through the live server |
| Exa API in tests | Real Exa API; fail test if `INTEG_TEST_EXA_API_KEY` missing |
| ExternalTokenSimulator user_id | Make configurable, use realistic values (UUID, email) |
| Access request DB setup | New `AccessRequestTestBuilder` |
| POC refactoring | Refactor to live server, graduate from "POC" naming |
| E2E tests for Keycloak errors | Stay in E2E (simulator can't replicate) |
| E2E tests for UI wiring | Stay in E2E (validates UI plugged in properly) |
| Token + chat test | Separate effort, track in tech-debt.md |
| toolsets-auth-restrictions holistic review | Separate phase, discussed later closer to implementation |

**Testing philosophy**: E2E tests external auth service (Keycloak) behavior. server_app tests OUR code's behavior given auth state. Don't test mock service responses in server_app — that tests our own mock, not real behavior.

---

## Phase 1: Infrastructure — Live Server + Enhanced Simulator

**Goal**: Build the foundational test infrastructure. No tests deleted.

### 1A. New `setup_test_app_service()` function

**File**: `crates/server_app/tests/utils/live_server_utils.rs`

Extract the POC's inline service setup into a reusable function. Builds `DefaultAppService` with real services but **no Keycloak dependency**:

- Real: `SqliteDbService`, `SqliteSessionService`, `DefaultToolService`, `DefaultExaService`, `MokaCacheService`, `DefaultAccessRequestService`
- Mock/Stub: `MockAuthService` (from `test_auth_service()`), `StubQueue`, `StubNetworkService`, `OfflineHubService`
- Key difference from existing `setup_minimal_app_service()`: No `INTEG_TEST_AUTH_URL` or `INTEG_TEST_RESOURCE_CLIENT_*` env vars

**Reuse**: Model after `setup_minimal_app_service()` (lines 30-254 of `live_server_utils.rs`), but replace Keycloak env vars with test defaults. Use `test_auth_service()` from `services::test_utils` for auth service.

### 1B. New `start_test_live_server()` function

**File**: `crates/server_app/tests/utils/live_server_utils.rs`

```rust
pub struct TestLiveServer {
  pub _temp_dir: TempDir,
  pub base_url: String,
  pub app_service: Arc<dyn AppService>,
  pub handle: ServerShutdownHandle,
}
```

- Calls `setup_test_app_service()`
- Uses `ServeCommand::ByParams { host, port }` + `.get_server_handle()` (same pattern as existing `live_server` fixture, line 263-285)
- Port: Use `TcpListener::bind("127.0.0.1:0")` to get OS-assigned port, extract it, drop listener, then pass port to `ServeCommand`
- Returns `TestLiveServer` with `base_url = format!("http://127.0.0.1:{port}")`

### 1C. ExternalTokenSimulator enhancements

**File**: `crates/server_app/tests/utils/external_token.rs`

Add new method with configurable fields:

```rust
pub fn create_token_with_scope_and_user(
  &self,
  scope: &str,
  azp: &str,
  user_id: &str,
  username: &str,
  access_request_id: Option<&str>,
) -> anyhow::Result<String>
```

Changes to exchange JWT claims (line 61-67):
- `"sub"`: use `user_id` parameter (not hardcoded `"test-external-user"`)
- Add `"preferred_username"`: use `username` parameter
- Add `"given_name"`: `"Test"`
- Add `"family_name"`: `"User"`
- Add `"access_request_id"`: from parameter (needed by `ScopeClaims` struct at `services/src/token.rs:88`)

Also update bearer JWT claims (line 47-52) to use `user_id` parameter for `"sub"`.

Refactor existing `create_token_with_scope()` to delegate with defaults:
```rust
pub fn create_token_with_scope(&self, scope: &str, azp: &str) -> anyhow::Result<String> {
  let user_id = Uuid::new_v4().to_string();
  self.create_token_with_scope_and_user(scope, azp, &user_id, "user@test.com", None)
}
```

### 1D. Session helper for live server tests

**File**: `crates/server_app/tests/utils/live_server_utils.rs`

Add function that creates an authenticated session for HTTP client use:

```rust
pub async fn create_test_session_for_live_server(
  app_service: &Arc<dyn AppService>,
  roles: &[&str],
) -> anyhow::Result<(String, String)>  // returns (session_cookie, user_id)
```

Reuses logic from `routes_app::test_utils::create_authenticated_session()` (lines 71-99 of `router.rs`): builds JWT via `access_token_claims()` + `build_token()`, creates session `Record`, saves to session store. Returns cookie string + the `sub` claim (user_id) from the JWT for coordinating with `ExternalTokenSimulator`.

### 1E. Refactor POC to live server

**Rename**: `test_external_token_poc.rs` → `test_oauth_external_token.rs`

Refactor all 3 tests to use `start_test_live_server()` + `reqwest::Client`:

| Old name | New name |
|----------|----------|
| `test_external_token_cache_bypass_toolsets_list` | `test_oauth_token_with_scope_can_list_toolsets` |
| `test_external_token_cache_bypass_missing_scope_rejected` | `test_oauth_token_without_scope_is_rejected` |
| `test_external_token_rejected_on_session_only_endpoint` | `test_oauth_token_rejected_on_session_only_get` |

Each test: start server → create ExternalTokenSimulator → create token → HTTP call via reqwest → assert status → shutdown.

**Verification**: `cargo test -p server_app test_oauth_external_token -- --nocapture`

---

## Phase 2: AccessRequestTestBuilder + Toolset Setup Helper

**Goal**: Build data setup utilities needed for toolset auth tests.

### 2A. AccessRequestTestBuilder

**File**: `crates/server_app/tests/utils/access_request_builder.rs` (new)

Builder that creates `AppAccessRequestRow` records directly in the DB via `AccessRequestRepository::create()`.

```rust
pub struct AccessRequestTestBuilder<'a> {
  db_service: &'a dyn DbService,
}

impl<'a> AccessRequestTestBuilder<'a> {
  pub fn new(app_service: &'a Arc<dyn AppService>) -> Self { ... }

  /// Create an approved access request with toolset entries
  pub async fn create_approved_with_toolsets(
    &self,
    user_id: &str,
    app_client_id: &str,
    toolset_entries: &[(&str, &str)],  // (toolset_type, instance_id)
  ) -> anyhow::Result<String>  // returns access_request_id

  /// Create an approved access request WITHOUT toolsets (auto-approve scenario)
  pub async fn create_auto_approved(
    &self,
    user_id: &str,
    app_client_id: &str,
  ) -> anyhow::Result<String>
}
```

The `create_approved_with_toolsets` method builds an `AppAccessRequestRow` with:
- `id`: `Uuid::new_v4().to_string()`
- `app_client_id`: from parameter
- `flow_type`: `"redirect"`
- `status`: `"approved"`
- `requested`: `{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}`
- `approved`: `{"toolset_types":[{"toolset_type":"builtin-exa-search","status":"approved","instance_id":"<uuid>"}]}`
- `user_id`: from parameter
- `resource_scope`: `"scope_user_user"`
- `access_request_scope`: `"scope_ar_<access_request_id>"`
- `expires_at`: 1 hour from now

Reference: `AppAccessRequestRow` struct at `services/src/db/objs.rs:220-237`.

### 2B. ToolsetSetup helper

**File**: `crates/server_app/tests/utils/toolset_setup.rs` (new)

Helper for setting up toolsets via HTTP API calls against the live server:

```rust
pub struct ToolsetSetup {
  client: reqwest::Client,
  base_url: String,
  session_cookie: String,
}

impl ToolsetSetup {
  pub fn new(base_url: &str, session_cookie: &str) -> Self { ... }

  /// PUT /bodhi/v1/toolset_types/{type}/app-config
  pub async fn enable_toolset_type(&self, toolset_type: &str) -> anyhow::Result<()>

  /// POST /bodhi/v1/toolsets
  pub async fn create_toolset(&self, toolset_type: &str, api_key: &str) -> anyhow::Result<String>  // returns UUID

  /// GET /bodhi/v1/toolsets
  pub async fn list_toolsets(&self) -> anyhow::Result<serde_json::Value>
}
```

All requests include `Cookie: <session_cookie>`, `Sec-Fetch-Site: same-origin`, `Content-Type: application/json` headers.

### 2C. Wire up mod.rs

**File**: `crates/server_app/tests/utils/mod.rs`

Add:
```rust
mod access_request_builder;
mod toolset_setup;
pub use access_request_builder::*;
pub use toolset_setup::*;
```

**Verification**: `cargo check -p server_app --tests`

---

## Phase 3: Migrate Toolset Auth Tests (DELETE from E2E)

**Goal**: Move all non-Keycloak OAuth tests from `toolsets-auth-restrictions.spec.mjs` to server_app.

### 3A. New test file: `crates/server_app/tests/test_oauth_toolset_auth.rs`

All tests follow this pattern:
1. `start_test_live_server()`
2. `create_test_session_for_live_server()` → get `(session_cookie, user_id)`
3. `ToolsetSetup::new()` → enable type + create toolset → get `toolset_id`
4. `AccessRequestTestBuilder::create_approved_with_toolsets()` → get `access_request_id`
5. `ExternalTokenSimulator::create_token_with_scope_and_user()` with matching `user_id`, `azp`, `access_request_id`
6. HTTP calls with Bearer token via `reqwest`
7. Assert response
8. `handle.shutdown()`

**Tests (from E2E):**

| E2E test | server_app test | Key assertion |
|----------|----------------|---------------|
| Case 1: App WITH scope + OAuth WITH scope | `test_oauth_approved_toolset_list_and_execute` | GET /toolsets → 200 with toolset; POST /execute/search → 200 with Exa result |
| Case 2: App WITH scope + OAuth WITHOUT scope | `test_oauth_without_access_request_scope_execute_denied` | POST /execute → non-200 (no access_request_id in token) |
| Case 4: No scope at all | `test_oauth_without_toolset_scope_empty_list` | GET /toolsets → 200, empty toolsets array |
| Session-only GET | `test_oauth_rejected_on_session_only_get_toolset` | GET /toolsets/{id} → 401 (with real toolset UUID) |
| Session-only PUT | `test_oauth_rejected_on_session_only_put_toolset` | PUT /toolsets/{id} → 401 (with real toolset UUID) |
| Session auth toolset_types | `test_session_list_toolsets_returns_toolset_types` | GET /toolsets via session cookie → 200, `toolset_types` array present |

**Case 1 note**: Requires `INTEG_TEST_EXA_API_KEY` env var for real Exa search. Test must **fail** (not skip) if missing.

### 3B. Delete from E2E

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`

**Delete**: All test blocks EXCEPT Case 3 (`App WITHOUT toolset scope + OAuth WITH toolset scope returns invalid_scope error`).

Case 3 tests Keycloak's `invalid_scope` response — impossible to simulate without real Keycloak.

Simplify the file to contain only Case 3 with its minimal setup (no Exa API key needed for this test).

**Verification**:
- `cargo test -p server_app test_oauth_toolset_auth -- --nocapture` (6 tests pass)
- E2E: Case 3 still passes with Keycloak (separate CI run)

---

## Phase 4: Copy OAuth User Info Test to server_app (KEEP E2E)

**Goal**: Add GET /user with OAuth token test to server_app. E2E version stays.

### 4A. New test file: `crates/server_app/tests/test_oauth_user_info.rs`

```rust
#[tokio::test]
async fn test_oauth_token_returns_full_user_info() -> anyhow::Result<()>
```

Test flow:
1. `start_test_live_server()`
2. Create OAuth token with `create_token_with_scope_and_user()`:
   - `scope: "scope_user_user offline_access"`
   - `user_id: <uuid>`
   - `username: "user@email.com"`
3. GET `/bodhi/v1/user` with Bearer token
4. Assert response:
   - `auth_status: "logged_in"`
   - `username: "user@email.com"` (from `Claims.preferred_username`)
   - `role: "scope_user_user"` (from `UserScope` via `AppRole::ExchangedToken`)

**Note**: The `/user` handler extracts `preferred_username`, `given_name`, `family_name` from the access token JWT via `extract_claims::<Claims>()` (at `services/src/token.rs:106-121`). The exchange JWT in the cache must include these fields — this was addressed in Phase 1C.

**Verification**: `cargo test -p server_app test_oauth_user_info -- --nocapture`

---

## Phase 5: Documentation + Tech Debt

### 5A. Update server_app CLAUDE.md

**File**: `crates/server_app/CLAUDE.md`

Add section under "Live Integration Test Architecture":

```
### OAuth Test Infrastructure (No Keycloak)

server_app OAuth tests use `ExternalTokenSimulator` to bypass Keycloak by seeding
the MokaCacheService cache directly. These tests validate OUR code's behavior
given auth state — they do NOT test Keycloak behavior.

Key utilities:
- `start_test_live_server()` — live TCP server with real services, no Keycloak env vars
- `ExternalTokenSimulator::create_token_with_scope_and_user()` — configurable OAuth tokens
- `AccessRequestTestBuilder` — creates access_request DB records for toolset auth middleware
- `ToolsetSetup` — HTTP-based toolset creation via session-authenticated API calls

Testing boundary:
- server_app: Tests OUR code given auth state (ExternalTokenSimulator)
- E2E: Tests external auth service behavior (Keycloak errors, consent, redirect flows)
- Don't test mock auth service responses in server_app — that validates our mock, not real behavior
```

### 5B. Update E2E CLAUDE.md

**File**: `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md`

Add section:

```
### E2E vs server_app Testing Boundary

E2E tests validate:
- External auth service (Keycloak) behavior (error responses, consent, scope validation)
- UI wiring — that the UI is plugged in properly for user journeys
- Browser-dependent flows (background tab token refresh, multi-user contexts)

Tests that only validate our code's behavior given auth state have been migrated to
server_app using ExternalTokenSimulator. The remaining E2E tests in this directory
specifically require either browser interaction or real Keycloak responses.

Example: toolsets-auth-restrictions.spec.mjs only retains Case 3 (Keycloak invalid_scope)
because it tests Keycloak's scope validation, not our middleware.
```

### 5C. Tech debt file

**File**: `ai-docs/claude-plans/20260215-e2e-cleanup/tech-debt.md` (new)

Track:
1. **Reintroduce E2E happy path after oauth-test-app refactor**: Once oauth-test-app is refactored, add back a smoke E2E test for the full OAuth + toolset access user journey
2. **Token + chat API test in server_app**: Add server_app test combining API token creation with real LLM chat completion (needs llama.cpp, builds on existing `test_live_agentic_chat_with_exa.rs` pattern)
3. **Holistic review of toolsets-auth-restrictions.spec.mjs**: Deeper restructuring — remove stale scope_toolset-* terminology, adopt first-party access-request semantics, consolidate test setup. Separate discussion phase.

**Verification**: `cargo test -p server_app -- --nocapture` (all server_app tests pass)

---

## Critical Files Reference

| File | Role |
|------|------|
| `crates/server_app/tests/utils/live_server_utils.rs` | New `setup_test_app_service()` + `start_test_live_server()` |
| `crates/server_app/tests/utils/external_token.rs` | Enhanced `ExternalTokenSimulator` |
| `crates/server_app/tests/utils/access_request_builder.rs` | New `AccessRequestTestBuilder` |
| `crates/server_app/tests/utils/toolset_setup.rs` | New `ToolsetSetup` HTTP helper |
| `crates/server_app/tests/test_oauth_external_token.rs` | Refactored POC (renamed from `test_external_token_poc.rs`) |
| `crates/server_app/tests/test_oauth_toolset_auth.rs` | Migrated toolset auth tests |
| `crates/server_app/tests/test_oauth_user_info.rs` | Copied user info test |
| `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Pruned to Case 3 only |
| `services/src/token.rs:80-89` | `ScopeClaims` struct (exchange JWT must match) |
| `services/src/token.rs:106-121` | `Claims` struct (exchange JWT must include `preferred_username`, `given_name`, `family_name`) |
| `auth_middleware/src/toolset_auth_middleware.rs` | 5-field validation chain that `AccessRequestTestBuilder` must satisfy |
| `services/src/db/objs.rs:220-237` | `AppAccessRequestRow` schema |
| `routes_app/src/test_utils/router.rs:71-99` | `create_authenticated_session()` pattern to reuse |

## Dependency Graph

```
Phase 1 (Infrastructure)
  ├── 1A: setup_test_app_service()
  ├── 1B: start_test_live_server()  ← depends on 1A
  ├── 1C: ExternalTokenSimulator enhancements
  ├── 1D: Session helper
  └── 1E: Refactor POC             ← depends on 1A-1D

Phase 2 (Builders)                  ← depends on Phase 1
  ├── 2A: AccessRequestTestBuilder
  ├── 2B: ToolsetSetup helper
  └── 2C: Wire mod.rs

Phase 3 (Migrate Tests)             ← depends on Phase 2
  ├── 3A: New server_app tests
  └── 3B: Delete E2E tests          ← depends on 3A passing

Phase 4 (Copy Tests)                ← depends on Phase 1 only
  └── 4A: OAuth user info test

Phase 5 (Documentation)             ← depends on all above
  ├── 5A: server_app CLAUDE.md
  ├── 5B: E2E CLAUDE.md
  └── 5C: tech-debt.md
```

Phase 4 can run in parallel with Phases 2-3.

## Verification

After all phases:
```bash
# server_app tests (no Keycloak needed)
cargo test -p server_app test_oauth -- --nocapture

# Full backend test suite
make test.backend

# E2E Case 3 still passes (requires Keycloak)
# cd crates/lib_bodhiserver_napi && npm run test -- --grep "invalid_scope"
```

---

## Phase 0 (Pre-implementation): Extract Context Document

Before starting implementation, extract the appendix below to:
`ai-docs/claude-plans/20260215-e2e-cleanup/20260216-e2e-to-server-app-ctx.md`

This preserves the exploration context for future reference.

---

# APPENDIX: Exploration Context Document

*Extract to `ai-docs/claude-plans/20260215-e2e-cleanup/20260216-e2e-to-server-app-ctx.md`*

# OAuth API Test Migration: Exploration Context & User Preferences

This document captures the full context from the discussion that led to the OAuth API test migration plan. It includes user preferences, reasoning behind decisions, constraints discovered during exploration, and nuances that may not be fully captured in the plan itself.

---

## Background & Motivation

The `ExternalTokenSimulator` POC (commit `0ba7e77d`) proved that OAuth token flows can be tested by seeding the `MokaCacheService` cache directly, bypassing Keycloak entirely. The E2E test `toolsets-auth-restrictions.spec.mjs` tests OAuth scope combinations via Playwright + Keycloak — these are slow (~minutes), require external Keycloak, and test API-level assertions that don't need a browser.

The user's vision: E2E tests should validate **external auth service behavior** (Keycloak) and **UI wiring**. server_app tests should validate **our code's behavior given auth state**. Testing mock auth service responses in server_app provides no value — it validates our own mock, not real behavior.

---

## Key Decisions with Full Reasoning

### 1. server_app test transport: "Always live TCP server"

User's exact words: "server_app tests should always have the live server, routes_app can use router.oneshot"

The POC used `router.oneshot()` which contradicts this. User directed refactoring POC to live server pattern and graduating from "POC" naming.

### 2. Data setup approach: "Router API calls with session auth"

User explicitly chose session-authenticated API calls over direct DB insertion. Reasoning: higher fidelity — tests the full write path through session auth + API handlers, not just DB state.

User's clarification: "server_app is fast, allowing us to make direct api calls skipping brittleness of UI. But e2e adds the validation for UI being plugged in properly. So we need to have the user journey happy path at least in e2e test."

### 3. E2E vs server_app boundary: "Copy, not just move"

Critical nuance from user: "we should not be moving the complete user journey to server_app, as then it skips validating the UI plugged in properly, add the above to CLAUDE.md of crates/lib_bodhiserver_napi/tests-js/ and crates/server_app/ before your write your plan, and also include this in your plan as well for analysis of what test should be migrated, and what should be copied, so kept both in server_app and e2e"

This led to the classification:
- **Move (delete from E2E)**: Tests with NO UI interaction, only API calls (toolset auth cases 1,2,4, session-only 401s)
- **Copy (keep both)**: Tests that validate a user journey AND have an API assertion worth fast-testing (oauth2-token-exchange /user endpoint)
- **Stay E2E only**: Tests that exercise Keycloak behavior or browser interaction

### 4. Exa API: "Real API, fail if missing"

User: "real exa api, if not set, fail the test" — not skip, fail. This matches the existing `test_live_agentic_chat_with_exa.rs` pattern.

### 5. ExternalTokenSimulator user_id: "Configurable, realistic values"

User: "make externaltokensimulator configurable, and as close to real world data values as possible, for e.g. if user id is uuid, then lets use uuid, username is an email, generally user@email.com, then lets use it properly"

Current hardcoded `"test-external-user"` must become a configurable UUID. Important because:
- Toolsets are owned by `user_id` — session-created toolsets won't match simulator's token without coordination
- `toolset_auth_middleware` validates `user_id` matches between access_request record and token
- `ScopeClaims.access_request_id` field is needed for execute endpoint auth

### 6. AccessRequestTestBuilder: "Encapsulate the 5-field validation chain"

The `toolset_auth_middleware` validates: exists → status approved → azp matches → user_id matches → instance in approved list. User chose a dedicated builder over inline setup or extending ExternalTokenSimulator.

### 7. Toolset scope test cases: "Move all to server_app"

User: "none of them are having any ui interactions, only api interactions, for now, lets move all of them to server_app, once we have refactored oauth-test-app, we can introduce a happy path test, make a note of it in tech-debt.md"

Case 3 (Keycloak invalid_scope) stays E2E because it tests Keycloak's scope validation, not our middleware.

### 8. Token + chat test: "Intended pattern, separate effort"

User explained the server_app testing philosophy: "this is intended pattern, we want to use real llm, the session tokens are locally minted and stored in DB, not validated by keycloak auth server, so server_app have most of the real services, including chat, only mocks keycloak auth service and helps us have high fidelity testing for our APIs"

This confirms: server_app = real everything except Keycloak. Tracked as tech debt item.

### 9. Holistic review of toolsets-auth-restrictions.spec.mjs: "Separate phase, discussed later"

User: "needs deeper restructuring, keep it separate phase, discussed later closer to implementation"

### 10. Error tests (oauth2-token-exchange.spec.mjs): "All stay in E2E"

User: "all stay in e2e, we have mock auth service in server_app, so we will be testing mock service responses instead of testing actual auth service behaviour. Have this criteria where we are testing the external behaviour of auth service, have those tests in e2e, instead of having test in server_app that actually test our own written mock behaviour and does not provide any value"

---

## E2E Test Inventory & Migration Classification

### toolsets-auth-restrictions.spec.mjs (620 lines)

| Test | Classification | Reasoning |
|------|---------------|-----------|
| Session auth: GET /toolsets returns toolset_types | **MOVE** to server_app | Pure API assertion, no UI interaction |
| Case 1: OAuth WITH scope → list + execute | **MOVE** to server_app | API calls only (despite complex setup, no browser interaction in assertion) |
| Case 2: OAuth WITHOUT scope → execute denied | **MOVE** to server_app | API-level scope enforcement |
| Case 3: Keycloak invalid_scope error | **STAY** E2E only | Tests Keycloak's actual scope validation — cannot simulate |
| Case 4: No scope → empty list | **MOVE** to server_app | API-level scope filtering |
| GET/PUT /toolsets/{id} with OAuth → 401 | **MOVE** to server_app | Session-only endpoint enforcement |

### oauth2-token-exchange.spec.mjs (131 lines)

| Test | Classification | Reasoning |
|------|---------------|-----------|
| Full OAuth2 flow + GET /user | **COPY** (keep E2E + add server_app) | E2E validates browser OAuth flow; server_app validates /user response fields |
| Error handling (unauthenticated → logged_out) | **STAY** E2E only | Tests external auth service behavior |

### Other E2E tests (NOT in scope)

| File | Classification | Reasoning |
|------|---------------|-----------|
| token-refresh-integration.spec.mjs | **STAY** E2E | Browser background tab behavior |
| multi-user-request-approval-flow.spec.mjs | **STAY** E2E | Multi-browser contexts, role hierarchy UI |
| api-tokens.spec.mjs | **STAY** E2E | UI lifecycle + chat integration |

---

## Infrastructure Deep Dive

### ExternalTokenSimulator (current state)

**File**: `crates/server_app/tests/utils/external_token.rs`

- Creates bearer JWT (external token) and exchange JWT (Keycloak response)
- Seeds `MokaCacheService` with cache key `exchanged_token:{sha256(bearer)[0..12]}`
- `CachedExchangeResult { token, azp }` — serialization-compatible with auth_middleware
- Bearer JWT claims: `jti`, `sub`, `exp`, `scope`
- Exchange JWT claims: `iss`, `sub`, `azp`, `exp`, `scope`
- **Missing fields needed**: `preferred_username`, `given_name`, `family_name`, `access_request_id`
- **Hardcoded user_id**: `"test-external-user"` — must become configurable

### JWT Claims Structs (what the middleware expects)

**`ScopeClaims`** (`services/src/token.rs:80-89`):
```
iss, sub, azp, aud?, exp, scope, access_request_id?
```
Used by token_service.rs to extract scope and access_request_id from exchanged token.

**`Claims`** (`services/src/token.rs:106-121`):
```
exp, iat, jti, iss, sub, typ, azp, aud?, scope, preferred_username, given_name?, family_name?, resource_access
```
Used by user_info handler to extract user profile from access token.

**`UserIdClaims`** (`services/src/token.rs:73-77`):
```
jti, sub, preferred_username
```
Used to extract user identity.

### Auth Middleware Header Flow

When an external bearer token hits the server:
1. `auth_middleware.rs` extracts bearer token from `Authorization` header
2. Checks `MokaCacheService` for `exchanged_token:{digest}` — **ExternalTokenSimulator seeds this**
3. Parses exchange JWT to extract `ScopeClaims` (sub, azp, scope, access_request_id)
4. Sets headers: `X-BodhiApp-User-Id`, `X-BodhiApp-Scope`, `X-BodhiApp-AZP`, `X-BodhiApp-Access-Request-Id`
5. Downstream handlers/middleware read these headers via typed extractors

### toolset_auth_middleware Validation Chain

For OAuth execute requests (`POST /toolsets/{id}/execute/{method}`):
1. Extract `access_request_id` from `X-BodhiApp-Access-Request-Id` header
2. Fetch `AppAccessRequestRow` from DB by `id`
3. Validate `status == "approved"`
4. Validate `app_client_id == X-BodhiApp-AZP`
5. Validate `user_id == X-BodhiApp-User-Id`
6. Parse `approved` JSON, check `toolset_types` array for `instance_id` match with `status == "approved"`

**`AccessRequestTestBuilder` must satisfy all 6 checks.**

### AppAccessRequestRow Schema

**`services/src/db/objs.rs:220-237`**:
```
id: String (UUID)
app_client_id: String
app_name: Option<String>
app_description: Option<String>
flow_type: String ("redirect"|"popup")
redirect_uri: Option<String>
status: String ("draft"|"approved"|"denied"|"failed")
requested: String (JSON)
approved: Option<String> (JSON)
user_id: Option<String>
resource_scope: Option<String>
access_request_scope: Option<String>
error_message: Option<String>
expires_at: i64
created_at: i64
updated_at: i64
```

### Existing Live Server Infrastructure

**`setup_minimal_app_service()`** (`live_server_utils.rs:30-254`):
- Requires Keycloak env vars: `INTEG_TEST_AUTH_URL`, `INTEG_TEST_AUTH_REALM`, `INTEG_TEST_RESOURCE_CLIENT_ID/SECRET/SCOPE`
- Uses `test_auth_service()` for auth service (MockAuthService with configured URL)
- Builds `DefaultAppService` with all real services

**`live_server` fixture** (`live_server_utils.rs:263-285`):
- Uses `ServeCommand::ByParams { host, port }` + `.get_server_handle()`
- Returns `TestServerHandle` with `temp_cache_dir`, `host`, `port`, `handle`, `app_service`
- Fixed port 51135

**New `setup_test_app_service()`** will mirror this but remove all `INTEG_TEST_*` env var requirements.

### Session Creation Pattern

**`routes_app::test_utils::create_authenticated_session()`** (`router.rs:71-99`):
- Builds JWT with `access_token_claims()` + role injection
- Creates session `Record` with `access_token` key
- Saves to session store via `SessionStore::save()`
- Returns cookie string: `bodhiapp_session_id={session_id}`
- Session JWT `sub` field = UUID (from `access_token_with_exp()` at `auth.rs:98`)

The `sub` from this JWT becomes the `user_id` for toolset ownership. Same `user_id` must be passed to `ExternalTokenSimulator::create_token_with_scope_and_user()` for OAuth token coordination.

### routes_app Existing Test Coverage

**API Token CRUD** (`routes_api_token/tests/api_token_test.rs`):
- Comprehensive: create, list (pagination), update, auth rejection, privilege escalation
- No DELETE tests (handler not implemented)

**Toolset CRUD** (`routes_toolsets/tests/toolsets_test.rs`):
- Comprehensive: list, create, get, update, delete, execute, type management
- Auth tests: unauthenticated rejection, insufficient role, API token rejection
- Note: Uses `MockToolService` which panics without expectations — can't add allow tests

### Commit 56fb064e Changes (during planning)

The external commit `56fb064e` (refactor: migrate E2E tests to pre-configured OAuth clients) changed:
- E2E test files: migrated from dynamic client creation to pre-configured `getPreConfiguredResourceClient()` / `getPreConfiguredAppClient()`
- Created `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md` with pre-configured credentials docs
- **No changes to**: server_app tests, routes_app, auth_middleware, services

This commit does NOT affect the migration plan — all server_app infrastructure files are unchanged.

---

## Risks and Gotchas

1. **Cache format coupling**: `CachedExchangeResult` struct in `external_token.rs` must stay in sync with the real struct in `auth_middleware/token_service.rs`. A structural change to the cache format would silently break all simulator tests.

2. **`extract_claims()` doesn't verify signatures**: The entire simulator approach depends on this. If JWT signature verification is ever added, the simulator needs to use the same signing key the server expects.

3. **`access_request_id` in exchange JWT**: The `ScopeClaims` struct has `access_request_id: Option<String>`. The `toolset_auth_middleware` extracts this from `X-BodhiApp-Access-Request-Id` header. The auth middleware sets this header from the scope claims. So the exchange JWT MUST include `access_request_id` for execute endpoint tests to work.

4. **Port conflicts**: Current live tests use fixed port 51135. New tests should use OS-assigned ports (`TcpListener::bind("127.0.0.1:0")`) to avoid conflicts with other test groups.

5. **Session JWT `sub` = user_id**: The `access_token_claims()` function generates a random UUID for `sub`. This UUID becomes the user_id for toolset ownership. The same UUID must be passed to `ExternalTokenSimulator` so the OAuth token references the same user who owns the toolsets.

6. **`Sec-Fetch-Site: same-origin` header**: Session-authenticated requests to the live server must include this header (required by auth middleware for session auth path). Without it, the server rejects session cookies.

---

## ADDENDUM: Phases 3-4 Reverted (2026-02-16)

### What was reverted

Phases 3 and 4 (toolset auth tests + user info test migration) were reverted. The following files were removed:

- `crates/server_app/tests/test_oauth_toolset_auth.rs` — all 6 migrated toolset auth tests
- `crates/server_app/tests/test_oauth_user_info.rs` — copied user info test
- `crates/server_app/tests/utils/access_request_builder.rs` — `AccessRequestTestBuilder` utility
- `crates/server_app/tests/utils/toolset_setup.rs` — `ToolsetSetup` HTTP helper

The E2E test `toolsets-auth-restrictions.spec.mjs` was restored to its full pre-migration state (all Cases 1-4, session auth, session-only CRUD tests).

### What was kept

Phase 1 infrastructure remains intact:
- `start_test_live_server()` and `setup_test_app_service()` in `live_server_utils.rs`
- `ExternalTokenSimulator` enhancements (`create_token_with_scope_and_user`)
- `create_test_session_for_live_server()` session helper
- `test_oauth_external_token.rs` (refactored from POC, 3 tests)
- `routes_app/src/routes.rs` change (moved `list_toolset_types_handler` to session-only)

### Why

The stubbed token approach via `ExternalTokenSimulator` hides the complexity of real Keycloak token exchange. For 3rd-party app OAuth token behavior, actual Keycloak token exchange is needed to validate the full claims pipeline. Specifically:

1. **Token exchange claim fidelity**: The simulator creates JWT claims directly, but real Keycloak token exchange may add, strip, or transform claims differently. The `access_request_id` claim, `preferred_username`, and scope claims all flow through Keycloak's token exchange logic which has its own validation and transformation rules.

2. **Execute endpoint coverage gap**: The migrated tests validated toolset list and session-only endpoint access (already accessible via `scope_user_user`), but the critical path — `POST /toolsets/{slug}/execute/{method}` — requires the full `toolset_auth_middleware` validation chain with `access_request_id`. Stubbed tokens pass this middleware because we control the claims, but real tokens through Keycloak may fail if the token exchange doesn't preserve the `access_request_id` claim correctly.

3. **Testing boundary violation**: The migration moved tests that validate the interaction between Keycloak token exchange and our middleware into server_app, where Keycloak is absent. This means the tests validate our mock's behavior rather than real auth service behavior — exactly the pattern the testing philosophy warns against.

### Path forward

When re-attempting the migration:
- Focus on the `toolsets/{slug}/execute/{method}` endpoint specifically — this is where stubbed tokens diverge most from real tokens
- Keep E2E tests for any flow that depends on Keycloak's token exchange behavior
- Consider migrating only the session-only CRUD tests (GET/PUT /toolsets/{id} → 401) as those don't depend on token exchange at all
- The Phase 1 infrastructure (`start_test_live_server`, `ExternalTokenSimulator`, `create_test_session_for_live_server`) is ready and validated for future use
