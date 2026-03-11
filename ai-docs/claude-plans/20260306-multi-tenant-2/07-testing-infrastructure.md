# Testing Infrastructure for Multi-Tenant

## Overview

Multi-tenant stage 2 required testing changes at every level of the stack: unit tests gained `MultiTenantSession` AuthContext fixtures and dual-variant parameterization (`session` / `multi_tenant`); repository and route-handler isolation tests were added for all tenant-scoped domains on both SQLite and PostgreSQL; Rust integration tests exercise real Keycloak at two levels (oneshot and live TCP); and Playwright E2E tests run against both a standalone (SQLite, port 51135) and a multi-tenant (PostgreSQL, port 41135) server with pre-seeded tenants. The testing infrastructure also includes dev-only endpoints (`POST /dev/clients/{client_id}/dag`, `DELETE /dev/tenants/cleanup`) and an idempotent `create_tenant_test` to support repeatable test runs against PostgreSQL.

## Functional Behavior

### Test Levels and What Changed

| Level | Crate / Location | What Changed for Multi-Tenant |
|-------|-----------------|-------------------------------|
| **Unit tests** | `services`, `routes_app` | `MultiTenantSession` AuthContext factories; `make_auth_with_role`/`make_auth_no_role` helper functions enabling `#[values("session", "multi_tenant")]` parameterization; dashboard auth route tests; `/info` endpoint tests for all AuthContext variants |
| **Repository isolation tests** | `crates/services/src/<domain>/test_*_isolation.rs` | 10 domain-specific isolation test files verifying cross-tenant and intra-user isolation on both SQLite and PostgreSQL |
| **Route-handler isolation tests** | `crates/routes_app/src/<domain>/test_*_isolation.rs` | 6 domain test files + 1 middleware isolation test; use `AuthContext` injection (`.with_auth_context()`) to bypass middleware and test handler-level isolation |
| **Integration tests (oneshot)** | `crates/routes_app/tests/test_live_multi_tenant.rs` | 7 tests using `tower::oneshot()` with real Keycloak tokens |
| **Integration tests (live TCP)** | `crates/server_app/tests/test_live_multi_tenant.rs` | 3 multi-turn tests on port 51135; full flow (dashboard login -> tenant creation -> DAG -> resource login -> activate -> info) |
| **E2E (Playwright)** | `crates/lib_bodhiserver_napi/tests-js/` | Dual-project setup: `standalone` (SQLite) and `multi_tenant` (PostgreSQL); tests use API models (OpenAI `gpt-4.1-nano`) for multi-tenant compatibility; GGUF testing isolated to `local-models.spec.mjs` (standalone-only) |
| **Component tests (Vitest)** | `crates/bodhi/src/` | Existing standalone tests updated for `client_id` parameter; MSW handlers extended with `deployment` and `client_id` params |

### Test Counts (Final State)

| Crate | Tests | Notes |
|-------|-------|-------|
| `services` | 832 passed | All dual-db (SQLite + PostgreSQL) |
| `routes_app` | 656 passed | 647 unit + 2 live auth + 7 live multi-tenant |
| `server_app` | 8 unit + 3 integration | Integration tests compile-verified (require real Keycloak) |
| E2E (multi_tenant project) | 30 passed, 3 skipped, 16 failed | 16 failures are pre-existing (same on standalone), not MT-related |

## Architecture & Data Model

### MultiTenantSession AuthContext Test Factories

Defined in `crates/services/src/test_utils/auth_context.rs`. Three factory methods cover all multi-tenant test scenarios:

| Factory | Fields | Use Case |
|---------|--------|----------|
| `test_multi_tenant_session(user_id, username)` | `client_id: None`, `tenant_id: None`, `dashboard_token: "test-dashboard-token"` | Dashboard-only session (no active tenant) |
| `test_multi_tenant_session_no_role(user_id, username)` | `client_id: Some(DEFAULT_CLIENT_ID)`, `tenant_id: Some(TEST_TENANT_ID)`, `role: None` | Active tenant but no role assigned |
| `test_multi_tenant_session_full(user_id, username, client_id, tenant_id, role, token)` | All fields populated | Fully authenticated multi-tenant session |

Chainable builder methods: `.with_tenant_id()`, `.with_user_id()`, `.with_deployment()`, `.with_dashboard_token()`.

### Dual-Variant Test Pattern (routes_app)

Helper functions in `crates/routes_app/src/test_utils/auth_context.rs` enable tests parameterized across standalone and multi-tenant AuthContext variants:

```rust
// Creates AuthContext matching the variant string
pub fn make_auth_with_role(variant: &str, user_id, username, role, token) -> AuthContext
pub fn make_auth_no_role(variant: &str, user_id, username) -> AuthContext

// Usage in tests:
#[values("session", "multi_tenant")] auth_variant: &str
```

Used in token CRUD tests, user management tests, and user access request tests. The `RequestAuthContextExt` trait provides `.with_auth_context()` for injecting `AuthContext` directly into request extensions (bypassing middleware).

### Repository Isolation Test Pattern

All use `#[values("sqlite", "postgres")]` + `#[serial(pg_app)]` + `sea_context(db_type)`:

```
1. Create resource in TEST_TENANT_ID
2. Create resource in TEST_TENANT_B_ID (same user)
3. List/Get per tenant -> only that tenant's resources visible
4. Cross-tenant Get by ID -> None
```

Constants from `crates/services/src/test_utils/db.rs`: `TEST_TENANT_ID`, `TEST_TENANT_B_ID`, `TEST_USER_ID`, `TEST_TENANT_A_USER_B_ID`.

**Services layer coverage:**

| Domain | Test File | Cross-Tenant | Intra-User |
|--------|-----------|:------------:|:----------:|
| Tokens | `crates/services/src/tokens/test_token_repository_isolation.rs` | Yes | Yes |
| MCPs (instances) | `crates/services/src/mcps/test_mcp_repository_isolation.rs` | Yes | Yes |
| MCP servers | `crates/services/src/mcps/test_mcp_repository_isolation.rs` | Yes | -- |
| MCP auth | `crates/services/src/mcps/test_mcp_auth_repository_isolation.rs` | Yes | -- |
| Toolsets | `crates/services/src/toolsets/test_toolset_repository_isolation.rs` | Yes | Yes |
| User aliases | `crates/services/src/models/test_user_alias_repository_isolation.rs` | Yes | Yes |
| API model aliases | `crates/services/src/models/test_api_alias_repository_isolation.rs` | Yes | Yes |
| Downloads | `crates/services/src/models/test_download_repository_isolation.rs` | Yes | -- |
| User access reqs | `crates/services/src/users/test_access_repository_isolation.rs` | Yes | -- |
| App access reqs | `crates/services/src/app_access_requests/test_access_request_repository_isolation.rs` | Yes | -- |
| Tenants_users | `crates/services/src/tenants/test_tenant_repository_isolation.rs` | Yes (special) | -- |

The `tenants_users` isolation test verifies the intentional cross-tenant read behavior: `list_user_tenants` returns memberships from both tenants, while `upsert_tenant_user`/`delete_tenant_user` operate within a single tenant's transaction context.

**Routes_app coverage (handler-level isolation):**

| Domain | Test File | Cross-Tenant | Intra-User |
|--------|-----------|:------------:|:----------:|
| Tokens | `crates/routes_app/src/tokens/test_tokens_isolation.rs` | Yes | Yes |
| MCPs (instances) | `crates/routes_app/src/mcps/test_mcps_isolation.rs` | Yes | Yes |
| MCP servers | `crates/routes_app/src/mcps/test_mcp_servers_isolation.rs` | Yes | -- |
| Toolsets | `crates/routes_app/src/toolsets/test_toolsets_isolation.rs` | Yes | Yes |
| API models | `crates/routes_app/src/api_models/test_api_models_isolation.rs` | Yes | Yes |
| Downloads | `crates/routes_app/src/models/test_downloads_isolation.rs` | Yes | -- |
| Middleware | `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` | Yes | -- |

Tests use direct `AuthContext` injection (`.with_auth_context()`) to bypass middleware, focusing on handler-level isolation. Routers are constructed without middleware, mounting handlers directly. Two tenants created via `create_tenant_test()` with deterministic IDs.

### Rust Integration Test Infrastructure

**routes_app oneshot tests** (`crates/routes_app/tests/test_live_multi_tenant.rs`):
- Uses `SettingServiceStub` with programmatic config (no process-global env vars)
- Real `KeycloakAuthService` for token validation
- Session injection via `session_store.save()` with dashboard + resource token keys
- Requests include `Sec-Fetch-Site: same-origin` and `Host: localhost:1135` for middleware
- 7 test scenarios covering `/info`, dashboard auth initiate rejection, tenant activate, `/user/info` dashboard session detection

**server_app live TCP tests** (`crates/server_app/tests/test_live_multi_tenant.rs`):
- Real TCP server on port 51135 with `#[serial_test::serial(live)]`
- Helper functions: `start_multitenant_live_server()`, `create_dashboard_session()`, `get_dashboard_token_via_password_grant()`, `add_resource_token_to_session()`
- `setup_multitenant_app_service()` sets `BODHI_DEPLOYMENT=multi_tenant`, `BODHI_MULTITENANT_CLIENT_ID/SECRET` from `INTEG_TEST_MULTI_TENANT_*` env vars
- 3 scenarios: full end-to-end flow, info state progression, standalone rejection

**Dev endpoints for test support** (routes_app, dev-only):
- `POST /dev/clients/{client_id}/dag` -- Enable Direct Access Grants on a Keycloak client via SPI, return local tenant credentials
- `DELETE /dev/tenants/cleanup` -- Gets `user_id` from auth context, calls `list_tenants_by_creator(user_id)` to discover tenants locally, filters out names starting with `[do-not-delete]`, sends explicit `{ "client_ids": [...] }` body to SPI DELETE endpoint, then optimistically deletes all sent client_ids locally. The SPI `DELETE /bodhi/test/tenants/cleanup` endpoint requires explicit `client_ids` in the request body (no auto-discovery).
- Both gated behind `!is_production()` and require multi-tenant mode + dashboard token

**Supporting infrastructure added in services crate**:
- `DbCore::reset_tenants()` -- truncates tenants table (PostgreSQL: CASCADE, SQLite: DELETE)
- `TenantRepository::list_tenants_by_creator(user_id)` -- discovers tenants created by a user (used by cleanup handler)
- `DefaultDbService.env_type` field with `.with_env_type()` builder -- production guard on `reset_all_tables()` and `reset_tenants()`
- `AuthService::forward_spi_request()` -- generic SPI proxy (owned `String` params for mockall compatibility, D103)
- `create_tenant_test()` -- idempotent upsert (check `get_tenant_by_client_id()` first, return existing if found)

### Playwright E2E Infrastructure

**Dual-project Playwright config** (`crates/lib_bodhiserver_napi/playwright.config.mjs`):

| Project | Port | DB | Deployment | Server Command |
|---------|------|----|-----------|----------------|
| `standalone` | 51135 | SQLite | `standalone` | `npm run e2e:server:standalone` |
| `multi_tenant` | 41135 | PostgreSQL | `multi_tenant` | `npm run e2e:server:multi_tenant` |

**Server startup** (`tests-js/scripts/start-shared-server.mjs`):
- Accepts `--port`, `--db-type`, `--deployment` CLI flags
- For `multi_tenant`: sets `BODHI_DEPLOYMENT`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET` as env vars; uses pre-registered tenant credentials for seeding via `ensure_tenant()`
- PostgreSQL connectivity check (TCP probes to `localhost:64320` and `localhost:54320`) before server start
- DB config from `tests-js/utils/db-config.mjs` (port + PostgreSQL URLs)

**Test filtering for multi_tenant project** (`testIgnore` in playwright.config.mjs):
- `**/setup/**` -- setup flow is standalone-only
- `**/models/**` -- local model alias + metadata require GGUF files
- `**/chat/local-models.spec.mjs` -- standalone-only GGUF testing

**API model helper** (`tests-js/utils/api-model-helpers.mjs`):
- `registerApiModelViaUI(modelsPage, formPage, apiKey)` -- registers an API model (OpenAI `gpt-4.1-nano`) via the UI, reusable across tests
- Uses `ApiModelFixtures.createModelData()`, `ApiModelFixtures.OPENAI_MODEL`, `ApiModelFixtures.getRequiredEnvVars()` from the shared fixture library

**E2E test adaptation for API models**:

Tests that interact with LLM inference use API models (OpenAI `gpt-4.1-nano`) instead of local GGUF models. This enables them to run on both `standalone` and `multi_tenant` projects.

| Test File | Pattern |
|-----------|---------|
| `chat.spec.mjs` | Registers API model after login, uses `selectModel(ApiModelFixtures.OPENAI_MODEL)` for chat |
| `chat-agentic.spec.mjs` | Same pattern -- registers API model after login, selects it for agentic chat (GPT-4.1-nano supports function calling) |
| `api-tokens.spec.mjs` | Restructured into 2 tests: (1) Token Lifecycle, Scopes, and Chat -- token CRUD + scopes + chat integration with API model + deactivate/reactivate cycle; (2) Multi-User Isolation and Error Recovery -- multi-user model registration, token isolation, cross-user token behavior, error handling |
| `oauth-chat-streaming.spec.mjs` | Adds API model registration step + model selection before chat |
| `local-models.spec.mjs` | Standalone-only GGUF smoke test (login + select Qwen + Q&A + create local model alias + verify) |
| `multi-tenant-lifecycle.spec.mjs` | Full multi-tenant lifecycle using invite link flow (see below) |

**Multi-tenant lifecycle E2E test** (`crates/lib_bodhiserver_napi/tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs`):

Exercises the complete multi-user invite link flow end-to-end:

| Step | Actor | Action |
|------|-------|--------|
| 1-4 | User (admin) | Register tenant, create API model, basic operations |
| 5 | Manager | Navigate to invite URL: `${baseUrl}/ui/login/?invite=${userClientId}` |
| 6 | Manager | Dashboard OAuth -> tenant OAuth -> lands on `/ui/request-access/` |
| 7 | Manager | Submit access request |
| 8 | User (new BrowserContext) | Login -> navigate to access request management -> approve Manager's request |
| 9 | Manager | Session cleared on approval -> re-auth triggers |
| 10 | Manager | Now has 2 tenants -> switch to User's tenant |
| 11 | Manager | Verify role assignment |
| 12 | Manager | Data isolation verification (Manager's API model NOT visible in User's tenant) |
| 13 | Both | Logout |

Uses separate Playwright `browser.newContext()` for User approval (isolated cookies/session).

**Key constraints for API-model E2E tests**:
- `autoResetDb` truncates `api_model_aliases` before each `test()` -- models must be registered inside each test body, never in `beforeAll`
- API models are scoped by `user_id` + `tenant_id` -- in multi-user tests, each user needs separate model registration
- Model selection BEFORE API token -- known UI quirk (model selection re-renders and clears token input)

**Auth server client** (`tests-js/utils/auth-server-client.mjs`):
- `getMultiTenantConfig()` -- reads `INTEG_TEST_MT_*` env vars (dashboard client ID/secret, tenant ID/secret)

**Keycloak E2E clients** (separate from Rust integration test clients):

| Client | Type | Purpose |
|--------|------|---------|
| `test-client-bodhi-multi-tenant-e2e` | Confidential, `bodhi.client_type=multi-tenant` | Dashboard client for E2E multi-tenant server (redirect URI: `http://localhost:41135/ui/auth/dashboard/callback`) |
| `bodhi-tenant-ad53d7e6-...` | Confidential | Pre-registered tenant for `user@email.com` under E2E dashboard client |

**Idempotent tenant bootstrap** (`crates/lib_bodhiserver_napi/src/server.rs`):
- `ensure_tenant()` calls `db_service().create_tenant_test()` with upsert semantics
- Prevents UNIQUE constraint violation on `tenants.client_id` across PostgreSQL test runs
- `lib_bodhiserver_napi` depends on `lib_bodhiserver` with `features = ["test-utils"]` to access `create_tenant_test`

### Component Test Infrastructure (Vitest)

**MSW handler extensions** (in `crates/bodhi/src/test-utils/msw-v2/handlers/`):
- `info.ts` -- `mockAppInfo()` accepts `deployment` and `client_id` params for multi-tenant scenarios
- `user.ts` -- `mockUserLoggedIn()` accepts `has_dashboard_session` via spread params

**Missing MSW handlers** (not yet created):
- No handlers for `GET /tenants`, `POST /tenants`, `POST /tenants/{client_id}/activate`
- No handlers for `POST /auth/dashboard/initiate`, `POST /auth/dashboard/callback`

### RLS Verification Tests

See [05-data-isolation-rls.md](05-data-isolation-rls.md#rls-verification-test) for the three RLS verification tests in `crates/services/src/db/test_rls.rs`.

### Frontend Bug Fixes Discovered During E2E Adaptation

1. **`isDashboardLoggedIn` logic** (`login/page.tsx`): Checked `auth_status === 'logged_in'` (always false for dashboard-only sessions). Fixed to use `has_dashboard_session` field from `UserInfoEnvelope`.
2. **`useUser()` return type** (`hooks/useUsers.ts`): Changed from `UserResponse` to `UserInfoEnvelope` to expose `has_dashboard_session`.
3. **`test.skip` syntax** in settings specs: Used incorrect `test.skip` at describe level. Moved into `test.beforeEach` for proper `testInfo` access.

### How Standalone Tests Were Adapted

Several categories of changes ensured existing standalone tests continue to work:

1. **`client_id` in OAuth initiate**: `POST /auth/initiate` now requires `{ client_id }` in the request body (D68). All existing auth tests updated to include `client_id` from `/info` response.
2. **`/info` response shape**: Now returns `deployment` and `client_id` fields. Test assertions updated.
3. **Session key namespacing**: Tests using session injection updated from flat keys (`access_token`) to namespaced keys (`{client_id}:access_token`, `active_client_id`).
4. **Middleware isolation**: Existing middleware tests extended with multi-tenant variant coverage via `test_auth_middleware_isolation.rs`.
5. **NAPI `ensure_tenant()`**: Replaced `update_with_option()` (production code in `lib_bodhiserver`) with test-utils-gated `create_tenant_test()` in NAPI server bootstrap. `update_with_option` removed from `lib_bodhiserver`.
6. **Playwright project rename**: `sqlite` -> `standalone`, `postgres` -> `multi_tenant`. npm scripts and Makefile targets updated accordingly.

## Technical Implementation

### Key Files

| File | Purpose |
|------|---------|
| `crates/services/src/test_utils/auth_context.rs` | `AuthContext::test_multi_tenant_session*` factories, builder methods |
| `crates/services/src/test_utils/db.rs` | `TEST_TENANT_ID`, `TEST_TENANT_B_ID`, `TEST_USER_ID`, `TEST_TENANT_A_USER_B_ID` constants; `TestDbService`, `FrozenTimeService` |
| `crates/services/src/test_utils/sea.rs` | `sea_context("sqlite"/"postgres")` dual-backend test fixture |
| `crates/services/src/test_utils/fixtures.rs` | `Tenant::test_default()`, `Tenant::test_tenant_b()` |
| `crates/services/src/tenants/tenant_repository.rs` | `create_tenant_test()` with upsert semantics (`#[cfg(test)]`-gated) |
| `crates/services/src/db/db_core.rs` | `DbCore::reset_tenants()` trait method |
| `crates/services/src/db/default_service.rs` | `env_type: EnvType` field, production guard, `reset_tenants()` impl |
| `crates/services/src/db/test_rls.rs` | RLS policy verification + app-layer isolation + SQL injection tests |
| `crates/services/src/tenants/test_tenant_repository_isolation.rs` | Cross-tenant `tenants_users` isolation test |
| `crates/routes_app/src/test_utils/auth_context.rs` | `make_auth_with_role()`, `make_auth_no_role()`, `RequestAuthContextExt` for `.with_auth_context()` |
| `crates/routes_app/src/middleware/auth/test_auth_middleware_isolation.rs` | Middleware-level multi-tenant isolation test |
| `crates/routes_app/tests/test_live_multi_tenant.rs` | 7 oneshot integration tests with real Keycloak |
| `crates/routes_app/src/tenants/test_dashboard_auth.rs` | Dashboard auth route unit tests |
| `crates/routes_app/src/tenants/test_tenants.rs` | Tenant CRUD route unit tests |
| `crates/routes_app/src/setup/test_setup.rs` | `/info` endpoint tests for all AuthContext variants |
| `crates/server_app/tests/test_live_multi_tenant.rs` | 3 live TCP integration tests |
| `crates/server_app/tests/utils/live_server_utils.rs` | `setup_multitenant_app_service()`, `create_dashboard_session()`, session helpers |
| `crates/routes_app/src/routes_dev.rs` | `dev_clients_dag_handler`, `dev_tenants_cleanup_handler` |
| `crates/lib_bodhiserver_napi/playwright.config.mjs` | Dual-project config (`standalone` + `multi_tenant`), `testIgnore` for standalone-only specs |
| `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` | Multi-tenant server startup with deployment CLI flag, PostgreSQL connectivity check |
| `crates/lib_bodhiserver_napi/tests-js/utils/db-config.mjs` | Port + DB URL mapping per project |
| `crates/lib_bodhiserver_napi/tests-js/utils/api-model-helpers.mjs` | `registerApiModelViaUI()` shared helper for API model registration in E2E tests |
| `crates/lib_bodhiserver_napi/tests-js/specs/chat/local-models.spec.mjs` | Standalone-only GGUF model smoke test (login + Qwen + Q&A + local alias) |
| `crates/lib_bodhiserver_napi/tests-js/utils/auth-server-client.mjs` | `getMultiTenantConfig()` for E2E Keycloak credentials |
| `crates/lib_bodhiserver_napi/tests-js/pages/RequestAccessPage.mjs` | Navigate to `/ui/request-access/`, submit access request, verify pending state |
| `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestsPage.mjs` | Navigate to access request management, find pending request by username, approve/reject |
| `crates/lib_bodhiserver_napi/tests-js/specs/multi-tenant/multi-tenant-lifecycle.spec.mjs` | Full multi-tenant lifecycle E2E test using invite link flow |
| `crates/lib_bodhiserver_napi/src/server.rs` | `ensure_tenant()` with idempotent upsert |
| `crates/lib_bodhiserver_napi/src/config.rs` | `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET` constants |
| `crates/bodhi/src/test-utils/msw-v2/handlers/info.ts` | `mockAppInfo()` with `deployment` + `client_id` |
| `crates/bodhi/src/test-utils/msw-v2/handlers/user.ts` | `mockUserLoggedIn()` with `has_dashboard_session` |
| `crates/bodhi/src/app/ui/login/page.test.tsx` | Standalone `LoginContent` tests (updated for `client_id`) |
| `crates/bodhi/src/components/AppInitializer.test.tsx` | Standalone status routing tests (not yet updated for multi-tenant) |

## Decisions

Decisions are referenced by ID. See [08-decisions-index.md](08-decisions-index.md) for the canonical decision table with full descriptions.

| ID | Title | Status |
|----|-------|--------|
| D99 | `ensure_valid_dashboard_token` uses TimeService | Implemented (superseded by D105) |
| D103 | `forward_spi_request` uses owned String params | Implemented |
| D104 | DefaultDbService uses builder pattern for EnvType | Implemented |

**Unnumbered decisions (testing-specific)**:

| Decision | Status |
|----------|--------|
| Dual-variant test parameterization (`make_auth_with_role`) | Implemented |
| Playwright project naming: `standalone` / `multi_tenant` | Implemented |
| E2E tests use API models (OpenAI `gpt-4.1-nano`) for multi-tenant compatibility; GGUF testing in standalone-only `local-models.spec.mjs` | Implemented |
| E2E env vars vs system settings for deployment config | Implemented |
| Separate Keycloak clients for E2E vs integration tests | Implemented |
| Pre-seeded tenant for E2E shared server | Implemented |
| Integration tests require real Keycloak | Accepted |
| `create_tenant_test` idempotent upsert | Implemented |
| `update_with_option` removed from lib_bodhiserver | Implemented |
| Repository isolation tests dual-db parameterized | Implemented |

## Known Gaps & TECHDEBT

### Component Tests (Frontend)

1. **No multi-tenant tests for AppInitializer**: `AppInitializer.test.tsx` does not test `tenant_selection` status routing or `setup` + `deployment=multi_tenant` -> `/ui/setup/tenants` routing.

2. **No tests for `MultiTenantLoginContent`**: The 4-state multi-tenant component has zero unit test coverage.

3. **No tests for `DashboardCallbackPage`**: No test file exists.

4. **No tests for `TenantRegistrationPage`**: No test file exists.

5. **No tests for tenant hooks**: `useTenants.ts` hooks have no test file. No MSW handler file for tenant endpoints.

6. **No tests for dashboard auth hooks**: `useDashboardOAuthInitiate` and `useDashboardOAuthCallback` have no test coverage.

### E2E Tests (Playwright)

7. **No component tests for invite link UI on users page or `?invite=` handling on login page**: The invite link copy button on the users page and the `?invite=` query parameter handling on the login page have no Vitest coverage.

8. **Pre-existing standalone E2E failures**: 16 tests using `createServerManager()` with dedicated servers fail on both projects. Root cause: `performOAuthLogin()` clicks "Login" but OAuth redirect does not happen -- likely `/info` returns `client_id: null` for unauthenticated users on dedicated servers.

9. **No E2E coverage for invite link error states**: The lifecycle test covers the happy path. Edge cases (expired invite, invalid client_id in `?invite=`, already-a-member invite) are untested. Note: `tests-js/pages/AddUserPage.mjs` and `tests-js/specs/users/add-user.spec.mjs` were removed when add-user-by-email was replaced by invite links.

### Repository Tests

10. **Untested `TenantRepository` methods**: `upsert_tenant_user`, `delete_tenant_user`, `list_user_tenants`, `has_tenant_memberships` need dedicated dual-db method-level tests.

11. **Untested `api_alias_repository` methods**: `update_api_model_alias` and `update_api_model_cache` lack dual-db tests.

12. **Untested `toolset_repository` methods**: `get_toolset_by_slug` and `list_toolsets_by_toolset_type` lack dual-db tests.

13. **No isolation test for `app_toolset_configs`**: Covered implicitly through toolset tests only.

### Route-Handler Isolation Tests

14. **No routes_app isolation tests for user aliases or user access requests**: Repository-level isolation tests only.

### RLS Verification

15. **`tenants_users` not in `test_rls.rs` policy audit**: `tenants_users_read` and `tenants_users_mutation` policies not verified by the RLS audit test.

16. **No direct RLS blocking evidence test**: Defense-in-depth claim relies on policy presence verification + functional isolation tests, not direct blocking evidence.

### Integration Tests

17. **Integration tests not in CI**: Require a real Keycloak instance. Compile-verified only.

18. **Test infra duplication in `live_server_utils.rs`**: ~200 lines duplicated across `setup_minimal_app_service`, `setup_multitenant_app_service`, and `setup_test_app_service`.

### General

19. **`get_standalone_app()` in test helpers**: `server_app/tests/utils/live_server_utils.rs` uses `get_standalone_app()` which fails if >1 tenant exists. Must be replaced with `get_tenant_by_client_id()`.

20. **PostgreSQL E2E requires Docker containers**: Multi-tenant E2E tests require two PostgreSQL containers (app DB on port 64320, session DB on port 54320). No automated container management in test runner.
