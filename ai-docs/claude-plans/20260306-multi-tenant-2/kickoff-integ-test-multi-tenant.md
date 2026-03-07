# Multi-Tenant Integration Tests -- Kickoff

> **Created**: 2026-03-08
> **Updated**: 2026-03-08
> **Status**: COMPLETED
> **Plan file**: `ai-docs/claude-plans/fluttering-napping-duckling.md`
> **Scope**: Backend integration tests in `crates/routes_app/tests/` (oneshot) and `crates/server_app/tests/` (multi-turn live server)
> **Prior work**:
> - Backend implementation: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-backend.md` (commits `6a7d879..04788eb`)
> - Frontend + backend prerequisites: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-frontend.md`
> - Keycloak SPI: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-keycloak-spi.md`
> **Context doc**: `ai-docs/claude-plans/20260306-multi-tenant-2/multi-tenant-flow-ctx.md`

---

## Implementation Summary

All 4 phases completed (Phase 0 handled during the planning conversation). Infrastructure changes (services + routes_app dev endpoints) plus integration tests at two levels.

**Phase 0** (analysis/Keycloak prep): Handled during planning conversation — analyzed existing test patterns, confirmed Keycloak client configs, verified `.env.test` files.

**Phase 1** (infrastructure): `DbService` production guard + `reset_tenants()`, `AuthService.forward_spi_request()`, `ensure_valid_dashboard_token` TimeService fix.

**Phase 2** (dev endpoints): `POST /dev/clients/{client_id}/dag` and `DELETE /dev/tenants/cleanup` — dev-only test support endpoints.

**Phase 3** (routes_app integration tests): 7 oneshot tests in `crates/routes_app/tests/test_live_multi_tenant.rs` using tower::oneshot() with real Keycloak tokens.

**Phase 4** (server_app integration tests): 3 live TCP tests in `crates/server_app/tests/test_live_multi_tenant.rs` — compile-verified (need real Keycloak to run).

**Final test counts**:
- services: 832 passed
- routes_app: 656 passed (647 unit + 2 live auth + 7 live multi-tenant)
- server_app: 8 unit passed, 3 integration tests compile-verified

---

## Task

Add integration tests for the multi-tenant features implemented since commit `6a7d879`. These tests run against a real Keycloak instance (dev server) and exercise the actual HTTP endpoints -- no mocking.

Two test levels:

1. **`routes_app` integration tests** (`crates/routes_app/tests/`): Single-turn tests using `tower::oneshot()`. Each test constructs a router, sends one request, and asserts the response. No TCP listener.

2. **`server_app` integration tests** (`crates/server_app/tests/`): Multi-turn flows using a real TCP server on port 51135. Tests can make sequential HTTP requests, carry cookies between them, and verify session state. Serial execution via `#[serial_test::serial(live)]`.

Both levels should test `bodhi_deployment=multi-tenant` scenarios. Also verify that `bodhi_deployment=standalone` still works correctly with the new changes (e.g., `/auth/initiate` now requires `client_id` in both modes).

---

## Testing Infrastructure

### routes_app integration tests

- Live in `crates/routes_app/tests/`
- Use `tower::oneshot()` to send requests through the Axum router
- Test utilities: `crates/routes_app/src/test_utils/` -- especially `AuthServerTestClient`, `AuthServerConfig`, `TestUser`
- Environment config: `crates/routes_app/tests/resources/.env.test` (loaded via `dotenv`)
- Existing examples: `crates/routes_app/tests/test_live_auth_middleware.rs`

### server_app integration tests

- Live in `crates/server_app/tests/`
- Real TCP server via `ServeCommand::ByParams` on port 51135
- Fixtures: `live_server` (real Keycloak), `start_test_live_server()` (no Keycloak)
- Session manipulation: `create_authenticated_session()`, `create_test_session_for_live_server()`
- Environment config: `crates/server_app/tests/resources/.env.test`
- Existing examples: `crates/server_app/tests/test_live_mcp.rs`, `crates/server_app/tests/test_oauth_external_token.rs`
- Key utility: `crates/server_app/tests/utils/live_server_utils.rs` -- `setup_minimal_app_service()`, `get_oauth_tokens()`, session helpers

### Environment variables

Both `.env.test` files already have multi-tenant client config:
```
INTEG_TEST_MULTI_TENANT_CLIENT_ID=test-client-bodhi-multi-tenant
INTEG_TEST_MULTI_TENANT_CLIENT_SECRET=<secret>
```

Standard variables: `INTEG_TEST_AUTH_URL`, `INTEG_TEST_AUTH_REALM`, `INTEG_TEST_USERNAME`, `INTEG_TEST_PASSWORD`, `INTEG_TEST_RESOURCE_CLIENT_ID`, `INTEG_TEST_RESOURCE_CLIENT_SECRET`.

---

## Phase 0: Analysis and Keycloak Preparation

Before writing any tests, analyze the codebase and prepare a plan. Then **pause and provide**:

1. A list of Keycloak client configurations needed for multi-tenant integration testing
2. For each client: client type, required settings (confidential/public, direct access grants, consent, redirect URIs, `bodhi.client_type` attribute)
3. Whether existing clients can be reused

### Currently available Keycloak clients

- **Resource client** (confidential, direct access grants enabled, `bodhi.client_type=resource`)
- **App client** (public, direct access grants enabled, no user consent)
- **Dashboard client** (`test-client-bodhi-multi-tenant`, confidential, `bodhi.client_type=multi-tenant`) -- already configured in `.env.test`

Reuse existing clients where possible. Only request new ones if a test scenario genuinely requires a different client configuration.

Once the user provides the Keycloak config, check `.env.test` files for correctness and write a first validation test (e.g., obtain a dashboard token via direct grant and verify the `azp` claim).

---

## Files to Explore

### Changed backend files (since `6a7d879`)

Read these to understand the implementation being tested:

- `crates/routes_app/src/auth/routes_auth.rs` -- `auth_initiate` now requires `client_id` in body, `auth_callback` reads `auth_client_id` from session
- `crates/routes_app/src/setup/routes_setup.rs` -- `/info` is session-aware, returns `deployment` and `client_id`, `resolve_multi_tenant_status()` helper
- `crates/routes_app/src/tenants/routes_dashboard_auth.rs` -- `POST /auth/dashboard/initiate`, `POST /auth/dashboard/callback`
- `crates/routes_app/src/tenants/routes_tenants.rs` -- `GET /tenants`, `POST /tenants`, `POST /tenants/{client_id}/activate`
- `crates/routes_app/src/tenants/dashboard_helpers.rs` -- `ensure_valid_dashboard_token()`, dashboard session management
- `crates/routes_app/src/tenants/error.rs` -- `DashboardAuthRouteError` variants
- `crates/routes_app/src/users/routes_users.rs` -- `/user/info` returns `has_dashboard_session`

### Session infrastructure

- `crates/routes_app/src/middleware/auth/auth_middleware.rs` -- two-step lookup: `active_client_id` -> `{client_id}:access_token`
- `crates/services/src/session_keys.rs` -- `access_token_key()`, `refresh_token_key()`, `SESSION_KEY_ACTIVE_CLIENT_ID`, `SESSION_KEY_USER_ID` (re-exported from `services` crate)

### Existing test patterns

- `crates/routes_app/tests/test_live_auth_middleware.rs` -- oneshot pattern with real Keycloak tokens
- `crates/server_app/tests/test_oauth_external_token.rs` -- multi-turn OAuth flow without Keycloak
- `crates/server_app/tests/test_live_mcp.rs` -- live server with session-based auth
- `crates/server_app/tests/utils/live_server_utils.rs` -- `setup_minimal_app_service()`, session creation helpers
- `crates/server_app/tests/utils/external_token.rs` -- `ExternalTokenSimulator` pattern

### Services and config

- `crates/services/src/settings/setting_service.rs` -- `deployment()`, `multitenant_client_id()`, `multitenant_client_secret()`
- `crates/services/src/settings/constants.rs` -- `BODHI_DEPLOYMENT`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET`

---

## Key Scenarios to Explore

These are directional, not exhaustive. Discover gaps by reading the implementation.

### `/info` endpoint behavior

- Standalone mode: returns `setup`, `resource_admin`, or `ready` based on tenant DB state
- Multi-tenant mode, no session: returns `tenant_selection` with `deployment: "multi_tenant"`
- Multi-tenant mode, dashboard token in session but no active tenant: returns `tenant_selection` or `setup` depending on SPI tenant count
- Multi-tenant mode, active tenant with valid resource token: returns `ready` with `client_id`

### `/auth/initiate` with `client_id`

- Standalone: must send `client_id` in body (gets it from `/info` response)
- Multi-tenant: sends selected tenant's `client_id`
- Missing or invalid `client_id`: appropriate error

### Dashboard auth flow

- `POST /auth/dashboard/initiate`: returns Keycloak auth URL for dashboard client. Only works in multi-tenant mode.
- `POST /auth/dashboard/callback`: exchanges code, stores dashboard tokens in session under `dashboard:*` keys
- Both endpoints return error in standalone mode (`dashboard_auth_route_error-not_multi_tenant`)

### Tenant CRUD

- `GET /tenants`: requires dashboard token in session, proxies to SPI, enriches with `is_active` and `logged_in`
- `POST /tenants`: requires dashboard token, proxies to SPI, creates local tenant row
- `POST /tenants/{client_id}/activate`: validates resource token exists in session, sets `active_client_id`

### Multi-turn flows (server_app level)

- Full flow: dashboard login -> list tenants -> tenant registration -> resource login -> API access
- Tenant switching: login to tenant A -> switch to tenant B (if logged in) or re-login
- Session key namespacing: tokens stored under `{client_id}:access_token`, multiple resource tokens coexist

### `/user/info`

- Returns `has_dashboard_session: true` when dashboard token exists in session
- Returns `has_dashboard_session: false` or absent when no dashboard session

---

## Setup Notes

### Building the multi-tenant app service

The existing `setup_minimal_app_service()` in `live_server_utils.rs` creates a standalone-mode service. For multi-tenant tests, you will need to set `BODHI_DEPLOYMENT=multi-tenant`, `BODHI_MULTITENANT_CLIENT_ID`, and `BODHI_MULTITENANT_CLIENT_SECRET` in the env wrapper. Study the existing function and create a multi-tenant variant or parameterize it.

### Dashboard token acquisition

Use Keycloak's direct access grant (password grant) against the dashboard client:
```
POST /realms/{realm}/protocol/openid-connect/token
grant_type=password
client_id=test-client-bodhi-multi-tenant
client_secret=<secret>
username=<test_user>
password=<test_password>
```

The existing `AuthServerTestClient::get_user_token()` already supports this pattern.

### Session injection

For oneshot tests, you may need to inject dashboard tokens into the session store before sending requests. Study `create_authenticated_session()` and `create_test_session_for_live_server()` for the pattern. Dashboard tokens use keys like `dashboard:access_token`, `dashboard:refresh_token`.

---

## Gate Checks — PASSED

```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
# Result: 656 passed, 0 failed
cargo test -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
# Result: 8 passed, 0 failed (integration tests compile-verified, need real Keycloak)
```
