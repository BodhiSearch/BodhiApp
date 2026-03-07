# Plan: Multi-Tenant E2E — Shared Tests on `multi_tenant` Project

> **Status**: COMPLETED (Phase 1 + Phase 2)
> **Completed**: 2026-03-09

## What Was Done

### Phase 1: Infrastructure (Steps 1-4)

Renamed Playwright projects, configured multi-tenant server, fixed NAPI bootstrap, filtered tests.

#### Files Modified

| File | Change |
|------|--------|
| `playwright.config.mjs` | Renamed `sqlite`→`standalone`, `postgres`→`multi_tenant`; added `testIgnore` for GGUF-dependent specs + setup |
| `tests-js/utils/db-config.mjs` | Renamed config keys `standalone`/`multi_tenant` |
| `tests-js/scripts/start-shared-server.mjs` | Added `--deployment` CLI flag; multi-tenant sets env vars for `BODHI_DEPLOYMENT`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET`; uses tenant credentials for seeding |
| `tests-js/utils/auth-server-client.mjs` | Added `getMultiTenantConfig()` reading `INTEG_TEST_MT_*` env vars |
| `tests-js/test-helpers.mjs` | Added `systemSettings` option to `createFullTestConfig()` |
| `src/server.rs` (NAPI) | Removed `get_standalone_app()` post-seed verification block |
| `src/config.rs` (NAPI) | Added `BODHI_MULTITENANT_CLIENT_ID` and `BODHI_MULTITENANT_CLIENT_SECRET` constants |
| `package.json` | Renamed npm scripts: `test:playwright:sqlite`→`standalone`, `test:playwright:postgres`→`multi_tenant` |
| `Makefile` | Renamed target: `test.napi.sqlite`→`test.napi.standalone` |
| `specs/settings/network-ip-setup-flow.spec.mjs` | Fixed `test.skip` syntax (moved into `test.beforeEach` for proper `testInfo` access) |
| `specs/settings/public-host-auth.spec.mjs` | Same fix |

#### Key decision: env vars vs system settings

`BODHI_DEPLOYMENT`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET` are set as **env vars** (not system settings via `setSystemSetting()`), because `AppOptionsBuilder::set_system_setting()` only allows 6 specific keys. `SettingService.get_setting()` and `get_env()` both read from env vars, so this works correctly.

### Phase 2: Frontend Bug Fix + Test Filtering

#### Root cause: multi-tenant login flow broken

The `MultiTenantLoginContent` auto-login `useEffect` never fired because `isDashboardLoggedIn` was always `false` for dashboard-only sessions.

**Bug**: `crates/bodhi/src/app/ui/login/page.tsx` line 80:
```javascript
// BEFORE (broken): auth_status is "logged_out" for dashboard-only sessions
const isDashboardLoggedIn = userInfo?.auth_status === 'logged_in' && !appInfo?.client_id;

// AFTER (fixed): has_dashboard_session is true when dashboard token exists
const isDashboardLoggedIn = !!userInfo?.has_dashboard_session && !appInfo?.client_id;
```

**Type fix**: `crates/bodhi/src/hooks/useUsers.ts` — Changed `useUser()` return type from `UserResponse` to `UserInfoEnvelope` (which includes `has_dashboard_session`).

#### Test filtering: GGUF-dependent specs excluded from multi_tenant

Multi-tenant mode uses `MultiTenantDataService` which only returns API model aliases from DB — no file-based GGUF models. Tests that depend on local GGUF files are excluded:

```javascript
testIgnore: [
  '**/setup/**',                   // Setup flow is standalone-only
  '**/chat/chat.spec.mjs',         // Requires local GGUF model (selectModelQwen)
  '**/chat/chat-agentic.spec.mjs', // Requires local GGUF model (selectModelQwen)
  '**/models/**',                  // Local model alias + metadata require GGUF files
  '**/tokens/**',                  // api-tokens uses selectModelQwen for chat integration
]
```

### Test Results

**Multi-tenant project**: 30 passed, 3 skipped, 16 failed (all pre-existing — verified same failures on standalone)

| Category | Tests | Result |
|----------|-------|--------|
| api-models (4 specs) | 9 | All pass |
| mcps-crud | 5 | All pass |
| mcps-oauth (3 specs) | 7 pass, 3 fail | Failures are pre-existing |
| toolsets-config | 6 | All pass |
| toolsets-auth-restrictions | 0 pass, 6 fail | Pre-existing |
| chat-toolsets | 1 | Pass |
| oauth (2 specs) | 1 pass, 3 fail | Pre-existing |
| users/list-users | 0 pass, 1 fail | Pre-existing |
| request-access | 0 pass, 1 fail | Pre-existing |
| settings (2 specs) | 3 skipped | Correctly skipped |

### Documentation

- `TECHDEBT.md` — Updated with `get_standalone_app()` usages (4 production locations)

---

## What's Next

### Standalone test failures (separate task)

Several tests using `createServerManager()` with dedicated servers fail on BOTH projects — `performOAuthLogin()` clicks "Login" but the OAuth redirect doesn't happen. See `kickoff-e2e-standalone-fixes.md`.

### Multi-tenant coverage tests (separate task)

New E2E tests for multi-tenant-specific flows (dashboard login, tenant registration, tenant switching). See `kickoff-e2e-multi-tenant-coverage.md`.
