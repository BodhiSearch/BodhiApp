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
