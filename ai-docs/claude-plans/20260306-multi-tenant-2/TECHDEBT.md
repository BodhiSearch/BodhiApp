# Multi-Tenant TECHDEBT

Items deferred from multi-tenant stage 2 implementation (M1-M6).

---

## `get_standalone_app()` Usages (Architecturally Wrong for Multi-Tenant)

Production code calling `get_standalone_app()` will error with `DbError::MultipleTenant` if >1 tenant exists. Must be replaced with tenant-aware lookups.

| File | Line | Risk | Issue |
|------|------|------|-------|
| `routes_app/src/setup/routes_setup.rs` | 66 | HIGH | `setup_show` uses `get_standalone_app().ok().flatten()` — returns wrong status if >1 tenant |
| `routes_app/src/routes_dev.rs` | 41 | MEDIUM | `dev_secrets_handler` — dev-only but silently fails |
| `routes_app/src/middleware/utils.rs` | 28 | MEDIUM | `standalone_app_status_or_default()` — `.ok().flatten()` silently returns wrong status |
| `services/src/tenants/auth_scoped.rs` | 74 | LOW | Auth-scoped passthrough — only called from above routes |

**Test code**: 6+ usages in `routes_app/src/setup/test_setup.rs` and `server_app/tests/utils/live_server_utils.rs` — will fail with >1 tenant in test context.

---

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

Code exchange logic is duplicated between `routes_auth.rs` (resource callback, line ~243) and `routes_dashboard_auth.rs` (dashboard callback, line ~191). Both call `auth_flow.exchange_auth_code()` with different client credentials. Should be extracted into a shared parameterized function. Low priority — no functional impact.

## Multi-Tenant-Aware Logout (D63)

Currently `session.delete()` clears ALL tokens (dashboard + all resource-client tokens). Future work:
- Selective resource-client logout (clear `{client_id}:access_token` + `active_client_id`, keep dashboard token)
- Full logout (clear everything including dashboard tokens)

## Integration Test CI Pipeline

M4 integration tests (server_app) are compile-verified but not run in CI yet. The 3 tests in `crates/server_app/tests/test_live_multi_tenant.rs` require:
- Network access to `main-id.getbodhi.app` Keycloak
- `.env.test` credentials (`INTEG_TEST_MT_*` vars)

## E2E/Playwright Tests for Multi-Tenant Flows

**Status**: Dedicated task — see `kickoff-e2e-multi-tenant-coverage.md` for comprehensive plan.

Backend integration tests exist (M4: 7 routes_app oneshot + 3 server_app live TCP), 30 shared Playwright tests pass on multi_tenant project, but no multi-tenant-specific E2E scenarios yet.

## Tenant Name Length Mismatch

Backend `CreateTenantRequest` validates name with `min = 1, max = 255`. Frontend registration form (`/ui/setup/tenants/page.tsx`) requires `minLength={3}`. The frontend is stricter than the API — cosmetic inconsistency.

## Frontend Unit Tests

New multi-tenant frontend components lack unit tests:
- Dashboard callback page (`/ui/auth/dashboard/callback`)
- Tenant registration page (`/ui/setup/tenants`)
- `MultiTenantLoginContent` sub-states (A, B1, B2, C)
- New hooks: `useTenants`, `useCreateTenant`, `useTenantActivate`, `useDashboardOAuthInitiate`, `useDashboardOAuthCallback`

## Access-Request E2E Under Multi-Tenant Project

`multi-user-request-approval-flow.spec.mjs` uses `createServerManager()` (standalone-specific dedicated server). Not yet excluded from multi_tenant testIgnore — may fail. Should be added to testIgnore as part of E2E Phase A.

---

## Summary Table

| Item | Severity | Status |
|------|----------|--------|
| `get_standalone_app()` production usages | HIGH | 4 files identified |
| Navigation visibility (F7) | Medium | Deferred |
| Service construction (F8) | Medium | Deferred |
| Code exchange duplication (D80) | Low | Deferred |
| MT-aware logout (D63) | Medium | Deferred |
| Integration test CI | Medium | Needs Keycloak in CI |
| E2E multi-tenant scenarios | Medium | Kickoff written |
| Tenant name length mismatch | Low | Cosmetic |
| Frontend unit tests | Medium | Missing for MT components |
| Access-request E2E testIgnore | Medium | Add to testIgnore |
| SPI orphan tenant (D52) | Low | Documented — KC orphan if local DB insert fails |
| Test infra duplication in live_server_utils.rs | Low | ~200 lines duplicated across setup functions |
