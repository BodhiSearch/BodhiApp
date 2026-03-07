# Multi-Tenant TECHDEBT

Items deferred from the multi-tenant backend (M2) and frontend (M3) implementation.

## Navigation Item Visibility (F7)

Hide LLM-specific navigation items in multi-tenant mode based on `deployment` from `/info`:
- Model Files, Model Downloads, local model features should be hidden when `deployment === 'multi_tenant'`
- Implement in `use-navigation.tsx` by filtering `defaultNavigationItems` based on deployment mode
- `AppInfo.deployment` is already available via `useAppInfo()` hook

## Service Construction Changes (F8)

Conditional route registration and listener skipping for multi-tenant mode:
- `lib_bodhiserver`: Skip registering LLM-specific routes (model pull, model files) in multi-tenant mode
- `server_app`: Skip llama_server_proc listener startup in multi-tenant mode
- Requires changes to `ServeCommand` and `AppServiceBuilder` to check deployment mode

## Shared Code Exchange Utility (D80)

Code exchange logic is duplicated between `routes_auth.rs` (resource callback) and `routes_dashboard_auth.rs` (dashboard callback). Should be extracted into a shared parameterized function. Low priority — no functional impact.

## Multi-Tenant-Aware Logout (D63)

Currently `session.delete()` clears ALL tokens (dashboard + all resource-client tokens). Future work:
- Selective resource-client logout (clear `{client_id}:access_token` + `active_client_id`, keep dashboard token)
- Full logout (clear everything including dashboard tokens)

## Integration Test CI Pipeline

M4 integration tests (server_app Phase 4) are compile-verified but not run in CI yet. The 3 tests in `crates/server_app/tests/test_live_multi_tenant.rs` require a real Keycloak instance and `.env.test` credentials (`INTEG_TEST_MULTI_TENANT_CLIENT_ID`, `INTEG_TEST_MULTI_TENANT_CLIENT_SECRET`). CI pipeline needs to be configured with network access to the dev Keycloak and these credentials.

## E2E/Playwright Tests for Multi-Tenant Flows

Backend integration tests exist (M4: 7 routes_app oneshot + 3 server_app live TCP), but no Playwright E2E tests for multi-tenant flows yet. Needed:
- Dashboard login flow (dashboard OAuth -> tenant selection -> resource OAuth)
- Tenant registration flow (0 clients -> /ui/setup/tenants/ -> register -> auto-login)
- Tenant switching flow (N clients -> switch -> instant or re-login)
- Standalone regression (existing flows still work)

Dashboard test client available: `INTEG_TEST_MULTI_TENANT_CLIENT_ID=test-client-bodhi-multi-tenant` in `server_app/tests/resources/.env.test`

## `get_standalone_app()` Usages (Architecturally Wrong for Multi-Tenant)

Production code that calls `get_standalone_app()` will error with `DbError::MultipleTenant` if >1 tenant exists. These must be replaced with tenant-aware lookups.

| File | Line | Risk | Issue |
|------|------|------|-------|
| `routes_app/src/apps/routes_apps.rs` | 95 | HIGH | `apps_create_access_request` breaks if >1 tenant. Use `get_tenant_by_id()` |
| `routes_app/src/middleware/utils.rs` | 24 | MEDIUM | `.ok().flatten()` silently returns wrong status. Propagate error or add mode check |
| `routes_app/src/routes_dev.rs` | 42 | MEDIUM | Dev endpoint silently fails. Add deployment mode check |
| `server_app/tests/utils/live_server_utils.rs` | 342,398,720 | HIGH | Test helpers fail with >1 tenant. Use `get_tenant_by_client_id()` |

## Tenant Description Field Optionality Mismatch

`POST /bodhi/v1/tenants` accepts `description` as `Option<String>` (optional). However, the frontend registration form at `/ui/setup/tenants/` requires it with `min: 1, max: 1000` validation. The frontend should be updated to make description optional, matching the API contract. Low priority — cosmetic UX discrepancy.

## Frontend Unit Tests

Some new frontend components lack comprehensive unit tests:
- Dashboard callback page (`/ui/auth/dashboard/callback`)
- Tenant registration page (`/ui/setup/tenants`)
- Login page multi-tenant sub-states
- New hooks: `useTenants`, `useCreateTenant`, `useTenantActivate`, `useDashboardOAuthInitiate`, `useDashboardOAuthCallback`
