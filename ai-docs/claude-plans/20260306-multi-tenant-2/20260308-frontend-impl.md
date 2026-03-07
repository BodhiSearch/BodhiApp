# Multi-Tenant Frontend + Backend Prerequisites — Implementation Plan

> **Created**: 2026-03-08
> **Status**: COMPLETE (staged, pending commit)
> **Kickoff**: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-frontend.md`
> **Backend plan**: `ai-docs/claude-plans/20260306-multi-tenant-2/20260307-backend-impl.md`
> **Context**: `ai-docs/claude-plans/20260306-multi-tenant-2/multi-tenant-flow-ctx.md`

---

## Context

The backend multi-tenant implementation (M2) was complete but deferred three items that were prerequisites for the frontend:
- **D68**: `POST /auth/initiate` accepting `client_id` in request body
- **D77**: Dashboard callback URL fix (separate `dashboard_callback_url()` method)
- **Session-aware `/info`**: `/info` checking session state for multi-tenant status resolution

This plan implemented those backend prerequisites, then built the full multi-tenant frontend: dashboard login, tenant selection, tenant registration, and setup flow adaptation.

**Out of scope**: E2E/Playwright tests (`lib_bodhiserver_napi`), service construction changes (`lib_bodhiserver`, `server_app`), navigation item visibility (deferred to TECHDEBT).

---

## Key Decisions

| # | Decision |
|---|----------|
| F1 | `/info` session-aware: 0 tenants -> `setup`, 1+ tenants (no active) -> `tenant_selection`, active client -> `ready` |
| F2 | 1-tenant case: do NOT auto-activate in `/info`. Return `tenant_selection`. Login page auto-initiates OAuth |
| F3 | D68 unified: `client_id` always required in `POST /auth/initiate`. Standalone frontend gets it from `/info` |
| F4 | AppInitializer: `allowedStatus` accepts `AppStatus \| AppStatus[]`. Login page uses `['ready', 'tenant_selection']` |
| F5 | Tenant registration at separate `/ui/setup/tenants/page.tsx`. AppInitializer redirects `setup` + `deployment=multi-tenant` there |
| F6 | No setup wizard for multi-tenant. Tenant created `Ready`. API key config accessible from settings |
| F7 | Nav item visibility deferred to TECHDEBT |
| F8 | Service construction changes (`lib_bodhiserver`, `server_app`) deferred |
| F9 | Single logout (clears all session state — both dashboard and resource tokens) |

---

## `/info` Auto-Resolution Logic (F1, F2) — IMPLEMENTED

Multi-tenant mode (`setup_show` handler with session + SPI):
```
1. Has active_client_id with valid resource token?  -> ready + client_id
2. Has dashboard token?
   a. Call ensure_valid_dashboard_token() + SPI GET /tenants
   b. 0 tenants  -> setup
   c. 1+ tenants -> tenant_selection
   d. SPI error  -> tenant_selection (user can retry)
   e. Token refresh failed -> tenant_selection
3. No dashboard token -> tenant_selection
```

Standalone mode: unchanged (`app_status_or_default` from tenant DB).

---

## Implementation Summary

### Phase 1: Backend Prerequisites — COMPLETE

**Files changed:**

| File | Change |
|------|--------|
| `crates/services/src/settings/constants.rs` | Added `LOGIN_DASHBOARD_CALLBACK_PATH = "/ui/auth/dashboard/callback"` |
| `crates/services/src/settings/setting_service.rs` | Added `dashboard_callback_url()` method (D77) |
| `crates/routes_app/src/tenants/routes_dashboard_auth.rs` | Changed to use `settings.dashboard_callback_url().await` |
| `crates/routes_app/src/auth/auth_api_schemas.rs` | Added `AuthInitiateRequest { client_id: String }` with `Validate` derive |
| `crates/routes_app/src/auth/routes_auth.rs` | `auth_initiate`: accepts `Json<AuthInitiateRequest>`, uses `get_tenant_by_client_id(&request.client_id)`, stores `auth_client_id` in session. `auth_callback`: reads `auth_client_id` from session, looks up tenant, cleans up after exchange |
| `crates/routes_app/src/setup/routes_setup.rs` | `setup_show`: added `Session` parameter, `resolve_multi_tenant_status()` helper for multi-tenant session-aware status |
| `crates/routes_app/src/setup/test_setup.rs` | Updated tests for new /info behavior |
| `crates/routes_app/src/auth/test_login_initiate.rs` | Updated tests to send `client_id` in request body |
| `crates/routes_app/src/auth/test_login_callback.rs` | Updated tests for `auth_client_id` session key |
| `crates/server_app/tests/utils/live_server_utils.rs` | Updated session keys to namespaced format |

### Phase 1.5: Regenerate Types — COMPLETE

- OpenAPI spec regenerated (`openapi.json`)
- TypeScript client regenerated (`ts-client/`)
- New types available: `AuthInitiateRequest { client_id: string }`
- Updated types: `AppInfo` with `deployment` and `client_id` fields

### Phase 2: Frontend — Hooks, Constants, AppInitializer — COMPLETE

**Files changed:**

| File | Change |
|------|--------|
| `crates/bodhi/src/lib/constants.ts` | Added `ROUTE_SETUP_TENANTS`, `ROUTE_DASHBOARD_CALLBACK` |
| `crates/bodhi/src/hooks/useAuth.ts` | Added `useDashboardOAuthInitiate()`, `useDashboardOAuthCallback()`. Modified `useOAuthInitiate()` to accept `{ client_id: string }`. Added endpoint constants for dashboard auth and tenants |
| `crates/bodhi/src/hooks/useTenants.ts` | **New file**: `useTenants()`, `useCreateTenant()`, `useTenantActivate()` hooks |
| `crates/bodhi/src/components/AppInitializer.tsx` | `allowedStatus` accepts `AppStatus \| AppStatus[]`. Added `tenant_selection` -> `/ui/login` routing. Added `setup` + `deployment=multi-tenant` -> `/ui/setup/tenants` routing |
| `crates/bodhi/src/hooks/useAuth.test.tsx` | Updated tests for new `client_id` parameter |
| `crates/bodhi/src/hooks/useInfo.test.ts` | Updated tests |
| `crates/bodhi/src/test-utils/msw-v2/handlers/info.ts` | Updated handler factory to accept `deployment` and `client_id` params |

### Phase 3: Frontend — Dashboard Callback + Login Page — COMPLETE

**Files changed:**

| File | Change |
|------|--------|
| `crates/bodhi/src/app/ui/auth/dashboard/callback/page.tsx` | **New file**: Dashboard OAuth callback page. Extracts `code`/`state` from URL params, calls dashboard callback endpoint, redirects to `/ui/login` |
| `crates/bodhi/src/app/ui/login/page.tsx` | Refactored with standalone/multi-tenant branches. Multi-tenant: no dashboard session -> "Login to Bodhi Platform" button; dashboard session with 1 tenant -> auto-initiate OAuth; N tenants -> tenant selector; fully authenticated -> current tenant info + switch |
| `crates/bodhi/src/app/ui/login/page.test.tsx` | Updated test |
| `crates/bodhi/src/components/LoginMenu.tsx` | Updated for new auth flow |
| `crates/bodhi/src/components/LoginMenu.test.tsx` | Updated test |

### Phase 4: Frontend — Tenant Registration + Setup — COMPLETE

**Files changed:**

| File | Change |
|------|--------|
| `crates/bodhi/src/app/ui/setup/tenants/page.tsx` | **New file**: Tenant registration form (name required, description required). On success, auto-initiates resource OAuth with returned `client_id` |
| `crates/bodhi/src/app/ui/setup/resource-admin/page.tsx` | Updated for `client_id` in OAuth initiate |
| `crates/bodhi/src/app/ui/setup/resource-admin/page.test.tsx` | Updated test |

### Phase 5: Regression + TECHDEBT — COMPLETE

- Backend regression verified
- `TECHDEBT.md` updated with deferred items (nav visibility, service construction)
- `lib_bodhiserver_napi/src/config.rs` updated for `created_by` field on `Tenant`

---

## Multi-Tenant Flow Summary

```
First visit (no session):
  /info -> tenant_selection -> /ui/login -> "Login to Bodhi Platform" button
  -> dashboard OAuth -> /ui/auth/dashboard/callback -> /ui/login

After dashboard login (0 tenants):
  /info -> setup (0 SPI tenants) -> /ui/setup/tenants
  -> register tenant -> auto resource OAuth -> /ui/auth/callback -> /ui/chat

After dashboard login (1 tenant):
  /info -> tenant_selection -> /ui/login
  -> auto resource OAuth (1 tenant) -> /ui/auth/callback -> /ui/chat

After dashboard login (N tenants):
  /info -> tenant_selection -> /ui/login -> tenant selector dropdown
  -> user picks tenant -> activate (logged_in) or OAuth (not logged_in) -> /ui/chat

Returning user (active tenant):
  /info -> ready + client_id -> normal app flow

Tenant switching:
  /ui/login -> shows current tenant + dropdown -> switch
```
