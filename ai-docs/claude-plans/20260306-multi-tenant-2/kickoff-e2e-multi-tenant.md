# Multi-Tenant E2E Tests — Kickoff

> **Created**: 2026-03-09
> **Status**: COMPLETED (Phase 1 + Phase 2)
> **Completed**: 2026-03-09
> **Scope**: Playwright E2E infrastructure for multi-tenant + shared test compatibility
> **Implementation**: `20260307-e2e-test-failure.md`
> **Follow-up**: `kickoff-e2e-multi-tenant-coverage.md` (Phase 3 — new multi-tenant-specific tests)
> **Prior work**:
> - Pre-E2E fixes: `kickoff-pre-e2e-fixes.md` (M5 — **COMPLETED**)
> - Backend integration tests: `20260308-rs-integration-test.md` (M4)
> - Backend implementation: `kickoff-bodhi-backend.md` (M2)
> - Frontend implementation: `20260308-frontend-impl.md` (M3)
> **Context doc**: `multi-tenant-flow-ctx.md`

---

## Summary of Completed Work

### Phase 1: Rename Playwright Projects + Multi-Tenant Server

- Renamed `sqlite`→`standalone` (port 51135), `postgres`→`multi_tenant` (port 41135)
- Configured multi-tenant server with `BODHI_DEPLOYMENT=multi_tenant` env var + dashboard credentials
- Pre-seeded tenant for `user@email.com` via `ensure_tenant()` at server startup
- Removed broken `get_standalone_app()` post-seed verification in NAPI `server.rs`
- Added `getMultiTenantConfig()` helper for reading `INTEG_TEST_MT_*` env vars

### Phase 2: Fix Existing Tests on `multi_tenant` Project

- **Frontend bug fix**: `isDashboardLoggedIn` in `login/page.tsx` checked `auth_status === 'logged_in'` (always false for dashboard-only sessions). Fixed to use `has_dashboard_session` field from `UserInfoEnvelope`.
- **Type fix**: `useUser()` hook return type changed from `UserResponse` to `UserInfoEnvelope` to expose `has_dashboard_session`.
- **Test filtering**: Excluded GGUF-dependent specs (`chat/chat.spec.mjs`, `chat-agentic.spec.mjs`, `models/**`, `tokens/**`) from multi_tenant — these require local model files that don't exist in multi-tenant mode.
- **`test.skip` fix**: Settings specs used incorrect `test.skip` syntax at describe level — moved into `test.beforeEach` for proper `testInfo` access.
- **Results**: 30 passed, 3 skipped, 16 failed (all pre-existing failures verified on standalone too).

---

## Keycloak Setup (reference)

The multi-tenant E2E tests use a **separate dashboard client** and a **pre-registered tenant** in the dev Keycloak at `main-id.getbodhi.app`:

**Dashboard client** (for the multi-tenant app):

| Setting | Value |
|---------|-------|
| Client ID | `test-client-bodhi-multi-tenant-e2e` |
| Client Type | Confidential |
| Client Attribute | `bodhi.client_type=multi-tenant` |
| Redirect URIs | `http://localhost:41135/ui/auth/dashboard/callback` |

**Pre-registered tenant** (resource client registered by `user@email.com` under the dashboard client above):

| Setting | Value |
|---------|-------|
| Client ID | `bodhi-tenant-ad53d7e6-6963-47d3-9bb9-82024c86b250` |
| Client Type | Confidential |

These credentials are in `crates/lib_bodhiserver_napi/tests-js/.env.test`:
```
INTEG_TEST_MT_DASHBOARD_CLIENT_ID=test-client-bodhi-multi-tenant-e2e
INTEG_TEST_MT_DASHBOARD_CLIENT_SECRET=<secret>
INTEG_TEST_MT_TENANT_ID=bodhi-tenant-ad53d7e6-6963-47d3-9bb9-82024c86b250
INTEG_TEST_MT_TENANT_SECRET=<secret>
```

---

## Build & Run

### Prerequisites

1. **Rebuild UI**: After any frontend changes, run `make build.ui-rebuild`
2. **Docker containers**: PostgreSQL on `localhost:64320` (app DB) and `localhost:54320` (session DB)
3. **Keycloak**: Dev Keycloak at `main-id.getbodhi.app` reachable with dashboard client + test users
4. **Environment**: `.env.test` must have multi-tenant credentials

### Running tests

```bash
# Build UI first (required after any frontend/backend changes)
make build.ui-rebuild

# Run all E2E tests (both projects)
cd crates/lib_bodhiserver_napi && npm run test:playwright

# Run standalone tests only
cd crates/lib_bodhiserver_napi && npx playwright test --project standalone

# Run multi-tenant tests only
cd crates/lib_bodhiserver_napi && npx playwright test --project multi_tenant
```

---

## Key Architecture Decisions

### Env vars vs system settings

`BODHI_DEPLOYMENT`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET` are set as **env vars** (not system settings), because `AppOptionsBuilder::set_system_setting()` only allows 6 specific keys. `SettingService.get_setting()` reads env vars, so this works.

### GGUF-dependent tests excluded from multi_tenant

Multi-tenant mode uses `MultiTenantDataService` which only returns API model aliases from DB — no file-based GGUF defaults. Tests using `selectModelQwen()` or local model files are excluded via `testIgnore`.

### Login flow

The multi-tenant login page auto-detects single-tenant users and initiates resource OAuth automatically. `has_dashboard_session` (from `UserInfoEnvelope`) drives the auto-login, not `auth_status`.

---

## Files Modified (reference)

### E2E infrastructure
- `crates/lib_bodhiserver_napi/playwright.config.mjs`
- `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs`
- `crates/lib_bodhiserver_napi/tests-js/utils/db-config.mjs`
- `crates/lib_bodhiserver_napi/tests-js/test-helpers.mjs`
- `crates/lib_bodhiserver_napi/tests-js/utils/auth-server-client.mjs`
- `crates/lib_bodhiserver_napi/package.json`
- `crates/lib_bodhiserver_napi/Makefile`

### NAPI server
- `crates/lib_bodhiserver_napi/src/server.rs`
- `crates/lib_bodhiserver_napi/src/config.rs`

### Frontend
- `crates/bodhi/src/app/ui/login/page.tsx`
- `crates/bodhi/src/hooks/useUsers.ts`

### Test specs
- `crates/lib_bodhiserver_napi/tests-js/specs/settings/network-ip-setup-flow.spec.mjs`
- `crates/lib_bodhiserver_napi/tests-js/specs/settings/public-host-auth.spec.mjs`

### Documentation
- `ai-docs/claude-plans/20260306-multi-tenant-2/TECHDEBT.md`
