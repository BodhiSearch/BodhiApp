# Plan: Merge routes_all into routes_app + Migrate Tests + Update Skills

## Context

The `routes_all` crate is a thin composition layer (~4 source files) that imports ~100 handlers from `routes_app` and composes them into a full Router with 5-tier auth middleware. This separation forces route tests to use ad-hoc routers that bypass auth. By merging into `routes_app`, ALL tests use the fully-composed router with real authentication, creating a single test layer that covers auth + business logic together, reducing duplication and making tests more realistic.

**Current pain**: ~149 handler-level tests bypass auth middleware via `X-BodhiApp-*` header injection (`RequestAuthExt`). Auth tier coverage only exists in stashed/uncommitted `routes_all/tests/` files. No consistency in what gets tested where.

**Goal**: One crate (`routes_app`), one router (`build_routes()`), one test pattern (router.oneshot with real auth). Per-module auth test files replace centralized `test_auth_tiers.rs`. Test-utils grow incrementally. Skills updated after migration.

## Decisions Made

| Decision | Choice |
|----------|--------|
| Test boundary | ALL tests use `build_routes()` router. Handler-level ONLY for impossible-state/defensive-security edge cases |
| Services in tests | Real for majority: in-memory SQLite (TestDbService, SqliteSessionService), OfflineHubService, SecretServiceStub. MockAuthService/MockToolService/MockQueueProducer only where external I/O unavoidable |
| Auth in tests | Inject tokens directly into SessionService/DbService/CacheService. Session = JWT in session store. API token = record in DB. No Keycloak calls for session/API token validation |
| Router fixture | Single `build_test_router()` with ALL services wired. Not configurable per test |
| Auth test org | Separate `*_auth_test.rs` per module. SCRAP centralized test_auth_tiers.rs |
| Migration order | Simple-first: setup → settings → models → users → api_token → auth → toolsets → oai → ollama → api_models |
| Test setup DRY | rstest fixtures |
| Test-utils growth | Incremental - add `#[cfg(test)]` handles on services as needed per module |
| Skill strategy | Replace `test-routes-app` with router-first unified skill. Create `bodhi-app-e2e` skill. AFTER migration |
| E2E skill | Document patterns for existing infra (no new test dir). Focus: complex multi-screen user flows, real server + test Keycloak |
| Stashed work | Reference only (keep in stash), start fresh |

## Auth Middleware Paths (Critical for Test Setup)

Understanding these paths determines what to pre-populate per test:

### Path A: Session Cookie
- Request: `Cookie: bodhiapp_session_id={id}` + `Sec-Fetch-Site: same-origin`
- Flow: session store lookup → JWT extraction → **local validation only** (no Keycloak if not expired)
- Sets: `X-BodhiApp-Role`, `X-BodhiApp-Token`, `X-BodhiApp-Username`, `X-BodhiApp-User-Id`
- **Test setup**: `create_authenticated_session(session_service, &["resource_user"])` → stores JWT in session store

### Path B: API Token (`bodhiapp_*` prefix)
- Request: `Authorization: Bearer bodhiapp_XXXXXXXX...`
- Flow: DB lookup by prefix → SHA-256 hash comparison → scope extraction
- Sets: `X-BodhiApp-Scope` (TokenScope), `X-BodhiApp-Token`, `X-BodhiApp-User-Id`
- **Test setup**: Create `ApiToken` record in TestDbService with `token_prefix`, `token_hash`, `scopes`, `status=Active`

### Path C: External OAuth Token (3rd party JWT)
- Request: `Authorization: Bearer <jwt>`
- Flow: cache check → issuer/audience validation → `AuthService.exchange_app_token()` (hits Keycloak)
- **Test setup**: Pre-populate `MokaCacheService` with cached exchange result, OR use MockAuthService for exchange

## Key Files

### Source (routes_all → routes_app)
- `crates/routes_all/src/routes.rs` → `crates/routes_app/src/routes.rs` — `build_routes()`, `apply_ui_router()`, `make_ui_endpoint!` (~380 lines)
- `crates/routes_all/src/routes_proxy.rs` → `crates/routes_app/src/routes_proxy.rs` — `proxy_router()`, `proxy_handler()` (~122 lines)
- `crates/routes_all/src/test_utils/mod.rs` → `crates/routes_app/src/test_utils/router.rs` — `build_test_router()`, `create_authenticated_session()` (~72 lines)

### Consumers to Update
- `crates/server_app/src/serve.rs:8` — `use routes_all::build_routes` → `use routes_app::build_routes`
- `crates/server_app/Cargo.toml` — remove routes_all dep
- `crates/lib_bodhiserver/Cargo.toml` — remove routes_all dep (never actually imported in code)
- `Cargo.toml` (workspace) — remove routes_all from members and workspace deps

### Dependencies to Merge (routes_all → routes_app Cargo.toml)
**Production (not already in routes_app)**: `include_dir`, `tower-http` (cors, trace features), `hyper-util`, `utoipa-swagger-ui` (axum, vendored)
**Test-utils feature additions**: `tempfile`, `tower-sessions`, `time`, `maplit`, `anyhow` (as optional deps); `services/test-utils`, `server_core/test-utils`
**Dev deps additions**: `sqlx`, `anyhow_trace`, `pretty_assertions`, `tower`

---

## Execution Plan

### Phase 1: Merge routes_all into routes_app (Commits 1-3)

#### Commit 1: Move routes_proxy.rs + routes.rs into routes_app

1. Add dependencies to `crates/routes_app/Cargo.toml`:
   - Production: `include_dir`, `tower-http` {cors, trace}, `hyper-util`, `utoipa-swagger-ui` {axum, vendored}
   - Optional (test-utils): `anyhow`, `maplit`, `tempfile`, `time`, `tower-sessions`
   - Dev: `sqlx`, `anyhow_trace`, `pretty_assertions`, `tower` {util}
   - Feature `test-utils` additions: the optional deps above + `services/test-utils`, `server_core/test-utils`
2. Create `crates/routes_app/src/routes_proxy.rs` — copy from routes_all, adjust imports to `use crate::*`
3. Create `crates/routes_app/src/routes.rs` — copy from routes_all, change `use routes_app::*` → `use crate::*`
4. Update `crates/routes_app/src/lib.rs` — add `mod routes;`, `mod routes_proxy;`, `pub use routes::*;`, `pub use routes_proxy::*;`
5. Inline tests from routes.rs (UI router tests) and routes_proxy.rs move with their modules

**Verify**: `cargo check -p routes_app`, `cargo test -p routes_app -- routes::tests`, `cargo test -p routes_app -- routes_proxy::tests`

#### Commit 2: Redirect consumers, remove routes_all

1. `crates/server_app/src/serve.rs` — change `use routes_all::build_routes` → `use routes_app::build_routes`
2. `crates/server_app/Cargo.toml` — remove `routes_all` from deps, dev-deps, features
3. `crates/lib_bodhiserver/Cargo.toml` — remove `routes_all` dep
4. Workspace `Cargo.toml` — remove `crates/routes_all` from members, remove `routes_all` from `[workspace.dependencies]`
5. Delete `crates/routes_all/` entirely

**Verify**: `cargo check --workspace`, `cargo test -p routes_app`, `cargo test -p server_app`

---

### Phase 2: Build Test Infrastructure (Commit 3)

#### Commit 3: Add router test infrastructure to routes_app/test_utils

Create `crates/routes_app/src/test_utils/router.rs`:

```rust
// Core fixtures
pub async fn build_test_router() -> (Router, Arc<dyn AppService>, Arc<SqliteSessionService>, Arc<TempDir>)
// Wires: TestDbService, SqliteSessionService, OfflineHubService, SecretServiceStub,
//        MockSharedContext, MockAuthService, MockToolService, MockQueueProducer,
//        MokaCacheService, LocalConcurrencyService, FrozenTimeService

pub async fn create_authenticated_session(
    session_service: &SqliteSessionService, roles: &[&str]
) -> anyhow::Result<String>
// Creates JWT with roles → stores in session store → returns session_id

// Request helpers
pub fn session_request(method: &str, path: &str, session_id: &str) -> Request<Body>
// Sets Cookie + Sec-Fetch-Site: same-origin

pub fn unauth_request(method: &str, path: &str) -> Request<Body>
// No auth headers
```

Key design: `build_test_router()` also returns `Arc<dyn AppService>` so tests can access service handles (e.g., `app_service.db_service()` to insert test data, `app_service.data_service()` to create aliases).

Update `crates/routes_app/src/test_utils/mod.rs` — add `mod router; pub use router::*;`

**Verify**: `cargo check -p routes_app --features test-utils`. Write smoke test: build router → hit `/api/ui/ping` → assert 200.

---

### Phase 3: Module Migrations (Commits 4-13, simple-first)

For each module: (a) migrate business logic tests to router-level where possible, (b) create `*_auth_test.rs` for per-module auth tier tests.

**Auth test template** (each `*_auth_test.rs`):
- `test_*_reject_unauthenticated` — rstest #[case] per endpoint, assert 401
- `test_*_reject_insufficient_role` — rstest #[case], assert 403
- `test_*_allow_authorized` — assert NOT 401/403

#### Commit 4: Migrate routes_setup + routes_setup public endpoints

**Changes**:
- `routes_setup_test.rs` — Keep ALL 9 tests handler-level (all need MockAuthService with specific `expect_register_client` expectations)
- NEW `tests/routes_setup_auth_test.rs` — Auth tests for public endpoints:
  - `test_public_endpoints_accessible_without_auth` — /ping, /health, /app/info, /logout all return non-401/403
  - `test_setup_accessible_without_auth` — POST /app/setup (public endpoint)

**test_utils additions**: None beyond commit 3.

#### Commit 5: Migrate routes_settings

**Changes**:
- `routes_settings_test.rs` — Keep handler-level (need custom `DefaultSettingService` with specific env/settings configs)
- NEW `tests/routes_settings_auth_test.rs`:
  - `test_settings_endpoints_reject_unauthenticated` — GET/PUT/DELETE /settings → 401
  - `test_settings_endpoints_reject_power_user` — Admin-tier, power_user → 403
  - `test_settings_endpoints_allow_admin` — Admin → not 401/403
  - `test_toolset_types_endpoints_reject_unauthenticated` — Admin-tier toolset type endpoints

#### Commit 6: Migrate routes_models (aliases, metadata, pull)

**Changes**:
- `routes_models/tests/aliases_test.rs` — Migrate 12 tests to router-level: list, get, create, update, delete, copy all use real DataService (already wired in build_test_router). Setup: seed aliases via `app_service.data_service()`. Auth: PowerUser session for writes, User for reads.
- `routes_models/tests/metadata_test.rs` — Migrate where possible (5 tests)
- `routes_models/tests/pull_test.rs` — Keep handler-level for tests needing MockQueueProducer expectations (6 tests)
- NEW `routes_models/tests/aliases_auth_test.rs`:
  - Read endpoints (User-tier): GET /models, GET /models/{id}
  - Write endpoints (PowerUser-tier): POST/PUT/DELETE /models, POST /models/{id}/copy
- NEW `routes_models/tests/metadata_auth_test.rs` — PowerUser session-only endpoints
- NEW `routes_models/tests/pull_auth_test.rs` — PowerUser-tier endpoints

**test_utils additions**: Helper to seed test aliases via DataService handle. Potentially `create_authenticated_session_with_claims()` for custom JWT claims (e.g., specific `sub` user_id).

#### Commit 7: Migrate routes_users (access_request, management, user_info)

**Changes**:
- `routes_users/tests/access_request_test.rs` (11 tests) — Migrate to router-level where possible. Tests needing specific user_id in session need custom session claims.
- `routes_users/tests/management_test.rs` (9 tests) — Manager-tier. Migrate DB-backed tests. Keep mock-heavy ones handler-level.
- `routes_users/tests/user_info_test.rs` (10 tests) — Optional-auth endpoints. Migrate.
- NEW auth test files per sub-module:
  - `user_info_auth_test.rs` — optional-auth tier tests
  - `access_request_auth_test.rs` — Manager-tier for approve/reject, optional for request
  - `management_auth_test.rs` — Manager-tier for list/change_role/remove

**test_utils additions**: `create_authenticated_session_with_user_id(session_service, roles, user_id)` — for tests that need a specific `sub` claim.

#### Commit 8: Migrate routes_api_token

**Changes**:
- `routes_api_token_test.rs` (12 tests) — Migrate privilege escalation and pagination tests to router-level. Tests use real DB via AppServiceStub. Auth: sessions with specific roles.
- NEW `tests/routes_api_token_auth_test.rs` — PowerUser session-only tier

**test_utils additions**: Helper to create API token in DB for Path B testing (if needed).

#### Commit 9: Migrate routes_auth (login, request_access)

**Changes**:
- `routes_auth/tests/login_test.rs` (12 tests) — **Complex**: OAuth2 flows need MockAuthService or mockito Keycloak mock. Keep handler-level for now. May evolve to router-level with mockito backend as test-utils mature.
- `routes_auth/tests/request_access_test.rs` (5 tests) — Similar complexity.
- NEW `routes_auth/tests/login_auth_test.rs` — Optional-auth tier for auth_initiate/auth_callback
- NEW `routes_auth/tests/request_access_auth_test.rs` — Optional-auth tier

#### Commit 10: Migrate routes_toolsets

**Changes**:
- `routes_toolsets/tests/toolsets_test.rs` (13 tests) — Complex: dual auth model (session vs OAuth), needs MockToolService. Keep mock-heavy tests handler-level. Migrate CRUD tests that work with real services.
- NEW `routes_toolsets/tests/toolsets_auth_test.rs`:
  - User session-only: POST/GET/PUT/DELETE toolsets
  - User OAuth: GET toolsets (list)
  - Toolset execute: custom middleware chain

#### Commit 11: Migrate routes_oai + routes_ollama

**Changes**:
- `routes_oai/tests/chat_test.rs` (6 tests) — Keep handler-level (need MockSharedContext with LLM proxy expectations)
- `routes_oai/tests/models_test.rs` (7 tests) — Migrate to router-level (use real DataService)
- `routes_ollama/tests/handlers_test.rs` (2 tests) — Migrate where possible
- NEW `routes_oai/tests/oai_auth_test.rs` — User-tier for all OAI endpoints
- NEW `routes_ollama/tests/ollama_auth_test.rs` — User-tier for Ollama endpoints

**test_utils additions**: May need MockSharedContext expectations helper if chat tests move to router-level later.

#### Commit 12: Migrate routes_api_models

**Changes**:
- `routes_api_models/tests/api_models_test.rs` (21 tests) — Migrate DB-backed tests to router-level. Keep MockAiApiService-dependent tests handler-level.
- NEW `routes_api_models/tests/api_models_auth_test.rs` — PowerUser-tier for all 10+ endpoints

---

### Phase 4: Update Skills (Commits 13-14)

#### Commit 13: Replace test-routes-app skill with unified router-first skill

Rewrite `.claude/skills/test-routes-app/`:
- `SKILL.md` — Router-first testing is default. Handler-level only for impossible-state edge cases.
- `fixtures.md` — Document `build_test_router()`, service handles, data seeding patterns
- `requests.md` — Document `session_request()`, `unauth_request()`, API token request patterns
- `assertions.md` — Document auth tier assertion patterns, business logic assertions
- `advanced.md` — Document per-module auth test file pattern, incremental test-utils growth

#### Commit 14: Create bodhi-app-e2e skill

Create `.claude/skills/bodhi-app-e2e/SKILL.md`:
- **Scope**: Complex multi-screen user flows (not API-level testing — that's covered by router tests)
- **Backend**: Everything real (server, test Keycloak, real DB, real services)
- **Infra**: Existing Playwright in `crates/lib_bodhiserver_napi/`
- **Patterns**: Server startup, OAuth flow testing, session persistence, multi-page navigation
- **When NOT to use**: API validation, auth tier testing, single-endpoint behavior (use router-level tests instead)

Also review and potentially update the existing `playwright` skill to complement (not overlap) the bodhi-app-e2e skill.

---

## Service Wiring in build_test_router()

| Service | Implementation | External I/O? |
|---------|---------------|---------------|
| DbService | TestDbService (real SQLite, TempDir) | No |
| SessionService | SqliteSessionService (real SQLite) | No |
| SecretService | SecretServiceStub (in-memory, AppRegInfo set) | No |
| HubService | OfflineHubService (downloads fail) | No |
| DataService | LocalDataService (real, file-based) | No |
| SettingService | SettingServiceStub (defaults) | No |
| CacheService | MokaCacheService (in-memory) | No |
| TimeService | FrozenTimeService (deterministic) | No |
| ConcurrencyService | LocalConcurrencyService | No |
| AuthService | MockAuthService (no expectations) | Panics if called |
| ToolService | MockToolService (no expectations) | Panics if called |
| QueueProducer | MockQueueProducer (no expectations) | Panics if called |
| SharedContext | MockSharedContext (no expectations) | Panics if called |

Tests that hit endpoints calling AuthService/ToolService/QueueProducer/SharedContext **must** either:
- Stay handler-level (with specific mock expectations), OR
- Evolve build_test_router() to accept overrides for those specific services (future work)

## What Stays Handler-Level (and Why)

| Module | Tests | Reason |
|--------|-------|--------|
| routes_setup | 9 | MockAuthService.expect_register_client() |
| routes_settings | 9 | Custom DefaultSettingService configs |
| routes_auth/login | 12 | MockAuthService for OAuth2 flow |
| routes_auth/access | 5 | MockAuthService for token exchange |
| routes_oai/chat | 5-6 | MockSharedContext for LLM proxy |
| routes_toolsets (some) | ~5 | MockToolService with expectations |
| routes_models/pull | ~4 | MockQueueProducer for download queue |

**Total**: ~50 tests stay handler-level. ~100 tests migrate to router-level + ~30 new auth tests = ~130 router-level tests.

## Sub-Agent Execution Strategy

Each commit is implemented by a dedicated sub-agent. The orchestrator (main agent) manages the chain, passing accumulated context forward.

### General Sub-Agent Protocol

Every sub-agent receives this standard preamble in its prompt:

1. **Explore first**: Read the current state of files you'll modify. Check `git log --oneline -5` to see what previous sub-agents committed.
2. **Implement**: Make the changes specified in the commit description.
3. **Verify**: Run `cargo check -p routes_app` then `cargo test -p routes_app`. For Phase 1 commits, also `cargo check --workspace`.
4. **Commit**: Stage specific files (not `git add -A`), commit with descriptive message.
5. **Report back**: Return a summary with:
   - Files created/modified/deleted
   - Test count changes (before vs after)
   - Any unexpected issues encountered
   - Insights/patterns discovered that help subsequent sub-agents
   - Any test-utils additions made

### Sub-Agent 1: Move code into routes_app (Commit 1)

**Context to pass**:
- This plan file path: `ai-docs/claude-plans/wobbly-marinating-barto.md`
- Read the plan's Commit 1 section
- Source files: `crates/routes_all/src/routes.rs`, `crates/routes_all/src/routes_proxy.rs`, `crates/routes_all/Cargo.toml`
- Destination: `crates/routes_app/src/routes.rs`, `crates/routes_app/src/routes_proxy.rs`, `crates/routes_app/Cargo.toml`
- Key: change `use routes_app::*` → `use crate::*` in the moved files
- Read `crates/routes_app/src/lib.rs` to understand current module structure before adding new modules
- Read `crates/routes_all/Cargo.toml` to get exact dep versions (all use `workspace = true`)

**Verify**: `cargo check -p routes_app`, `cargo test -p routes_app`

**Expected insights to report**: Any import resolution issues, any `pub use` conflicts with existing routes_app exports, whether `make_ui_endpoint!` macro works correctly after move.

### Sub-Agent 2: Redirect consumers, remove routes_all (Commit 2)

**Context to pass**:
- Summary from Sub-Agent 1
- Files to change: `crates/server_app/src/serve.rs`, `crates/server_app/Cargo.toml`, `crates/lib_bodhiserver/Cargo.toml`, workspace `Cargo.toml`
- CRITICAL: Delete `crates/routes_all/` directory entirely (including uncommitted test files in `crates/routes_all/tests/`)
- The stashed work stays in git stash (do NOT drop stashes)

**Verify**: `cargo check --workspace`, `cargo test -p routes_app`, `cargo test -p server_app`

**Expected insights to report**: Whether any other crate had hidden routes_all imports, clean workspace compilation status.

### Sub-Agent 3: Build test infrastructure (Commit 3)

**Context to pass**:
- Summary from Sub-Agents 1-2
- Reference: stashed `crates/routes_all/src/test_utils/mod.rs` content (the `build_test_router()` and `create_authenticated_session()` functions from the uncommitted changes — Sub-Agent 2 would have seen them before deletion)
- Current `crates/routes_app/src/test_utils/mod.rs` content
- The plan's Phase 2 section with function signatures
- Key services: `AppServiceStubBuilder` in `crates/services/src/test_utils/app.rs`, `access_token_claims()` and `build_token()` in `crates/services/src/test_utils/auth.rs`
- Auth middleware path details from this plan's "Auth Middleware Paths" section
- `build_test_router()` must return `(Router, Arc<dyn AppService>, Arc<SqliteSessionService>, Arc<TempDir>)` — the AppService handle is critical for tests to seed data
- Add request helpers: `session_request()`, `unauth_request()`
- Write a smoke test in `crates/routes_app/tests/test_router_smoke.rs` that builds the router and hits `/api/ui/ping`

**Verify**: `cargo check -p routes_app --features test-utils`, `cargo test -p routes_app test_router_smoke`

**Expected insights to report**: Full list of services wired, any issues with service construction, what AppServiceStubBuilder methods were needed, the exact test annotation pattern that works.

### Sub-Agents 4-12: Module Migrations (Commits 4-12)

Each sub-agent receives:
- Summary from ALL previous sub-agents (accumulated)
- The specific commit section from this plan
- Key patterns from Sub-Agent 3's report (test annotation pattern, fixture usage, request helpers)
- Current content of `crates/routes_app/src/test_utils/` (to see what helpers exist)
- Current content of the specific module's test files being migrated

**Per sub-agent instructions**:
1. Read the existing test file(s) for this module thoroughly
2. Read the handler implementation to understand what services each endpoint calls
3. For each test, decide: router-level (can use build_test_router) or handler-level (needs specific mocks)
4. Create the `*_auth_test.rs` file with auth tier tests for all endpoints in this module
5. Migrate business logic tests to router-level where possible
6. Add any needed test_utils helpers (and document what was added in the report)
7. Run `cargo test -p routes_app` to verify ALL tests pass (not just this module)
8. Commit with message: `test(routes_app): migrate routes_<module> to router-level tests`

**Report template for migration sub-agents**:
```
Module: routes_<name>
Tests migrated to router-level: N
Tests kept handler-level: M (reasons: ...)
New auth tests added: K
test_utils additions: [list any new helpers]
Patterns discovered: [anything useful for next modules]
Issues encountered: [any blockers or workarounds]
Service handles used for data setup: [which services and methods]
```

#### Sub-Agent 4: routes_setup (Commit 4)
- Simplest module. Public endpoints + MockAuthService-dependent business logic.
- Expected: 0 business logic tests migrate (all need MockAuthService). ~5 auth tests added for public endpoints.

#### Sub-Agent 5: routes_settings (Commit 5)
- Admin-tier endpoints. Custom SettingService configs.
- Expected: 0 business logic tests migrate. ~6 auth tests added.

#### Sub-Agent 6: routes_models (Commit 6)
- First module with real business logic migration. aliases (12 tests), metadata (5), pull (6).
- Expected: ~12 aliases tests migrate. ~6 pull tests stay handler-level (MockQueueProducer). ~15 auth tests added.
- **Critical**: This is where data seeding patterns emerge. Report exactly how aliases are seeded via DataService handle.

#### Sub-Agent 7: routes_users (Commit 7)
- 30 tests across 3 sub-modules. Manager-tier and optional-auth endpoints.
- Expected: test_utils addition of `create_authenticated_session_with_user_id()`. ~15 tests migrate. ~12 auth tests added.

#### Sub-Agent 8: routes_api_token (Commit 8)
- 12 tests. PowerUser session-only tier. Privilege escalation testing.
- Expected: Most tests migrate (use real DB). ~6 auth tests added.
- May need API token creation helper for Path B testing.

#### Sub-Agent 9: routes_auth (Commit 9)
- **Complex**: 17 tests across login + access. OAuth2 flows with MockAuthService.
- Expected: 0 business logic tests migrate (all need MockAuthService). ~4 auth tests added.
- Report: what would be needed to eventually migrate these (mockito? configurable build_test_router?).

#### Sub-Agent 10: routes_toolsets (Commit 10)
- 13 tests. Dual auth model (session vs OAuth). MockToolService.
- Expected: Some CRUD tests migrate. ~8 auth tests added.

#### Sub-Agent 11: routes_oai + routes_ollama (Commit 11)
- 15 tests total. Chat tests need MockSharedContext, model tests use real DataService.
- Expected: ~9 tests migrate (models + ollama). ~6 auth tests added.

#### Sub-Agent 12: routes_api_models (Commit 12)
- Largest module (21 tests). PowerUser-tier.
- Expected: DB-backed tests migrate. ~10 auth tests added.

### Sub-Agents 13-14: Skills Updates (Commits 13-14)

#### Sub-Agent 13: Replace test-routes-app skill (Commit 13)

**Context to pass**:
- All accumulated insights from Sub-Agents 3-12
- Current skill content: `.claude/skills/test-routes-app/`
- The actual patterns that emerged during migration (from sub-agent reports)
- The actual test_utils API (from current `crates/routes_app/src/test_utils/`)

**Instructions**: Rewrite the skill based on what was actually built, not what was planned. Use real code examples from the migrated tests. Document:
- Router-first as default, handler-level as exception
- build_test_router() fixture and what it returns
- create_authenticated_session() with role arrays
- Per-module auth test file pattern
- Data seeding patterns via service handles
- When to keep tests handler-level (the decision criteria)

#### Sub-Agent 14: Create bodhi-app-e2e skill (Commit 14)

**Context to pass**:
- Summary of all migration work
- Current Playwright config: `crates/lib_bodhiserver_napi/playwright.config.mjs`
- Current e2e test files in `crates/lib_bodhiserver_napi/tests/`
- Integration test patterns from `crates/integration-tests/`
- The existing `playwright` skill content in `.claude/skills/`

**Instructions**: Create skill documenting:
- When to use e2e vs router-level tests (decision criteria)
- Real server setup patterns
- OAuth flow testing with test Keycloak
- Multi-screen user flow patterns
- Session persistence via cookie jar
- What's already covered by router-level tests (don't duplicate)

---

## Orchestrator Checklist

The main agent follows this checklist:

- [ ] Sub-Agent 1: Move code → committed
- [ ] Sub-Agent 2: Remove routes_all → committed
- [ ] Sub-Agent 3: Test infrastructure → committed
- [ ] Sub-Agent 4: routes_setup auth tests → committed
- [ ] Sub-Agent 5: routes_settings auth tests → committed
- [ ] Sub-Agent 6: routes_models migration → committed
- [ ] Sub-Agent 7: routes_users migration → committed
- [ ] Sub-Agent 8: routes_api_token migration → committed
- [ ] Sub-Agent 9: routes_auth auth tests → committed
- [ ] Sub-Agent 10: routes_toolsets migration → committed
- [ ] Sub-Agent 11: routes_oai + ollama migration → committed
- [ ] Sub-Agent 12: routes_api_models migration → committed
- [ ] Sub-Agent 13: Update test-routes-app skill → committed
- [ ] Sub-Agent 14: Create bodhi-app-e2e skill → committed
- [ ] Final: `make test.backend` passes
- [ ] Final: `grep -r "routes_all" crates/ Cargo.toml` → no results

---

## Verification

**Per commit**:
- `cargo check -p routes_app` (compilation)
- `cargo test -p routes_app` (all tests pass)
- For Phase 1: `cargo check --workspace` and `cargo test -p server_app`

**After all commits**:
- `make test.backend` (full suite)
- `grep -r "routes_all" crates/ Cargo.toml` → no results
- `cargo test -p routes_app -- --list 2>&1 | wc -l` → ~180+ tests
- Verify no test uses `RequestAuthExt::with_user_auth()` in router-level tests (handler-level exceptions allowed)
