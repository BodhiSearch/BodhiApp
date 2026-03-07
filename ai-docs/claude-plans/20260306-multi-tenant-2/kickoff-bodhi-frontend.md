# Multi-Tenant BodhiApp Frontend + Backend Prerequisites — Kickoff

> **Created**: 2026-03-06
> **Updated**: 2026-03-08
> **Status**: COMPLETE (staged, pending commit)
> **Scope**: Backend prerequisites in `crates/routes_app/`, `crates/services/`; Frontend in `crates/bodhi/src/`
> **Prior work**:
> - `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-backend.md` (backend API contract)
> - Keycloak SPI implementation: `keycloak-bodhi-ext/ai-docs/claude-plans/20260307-multi-tenant/20260307-tenants-integration-doc.md`
> - Middleware JWT-based tenant resolution (completed)
> **Context doc**: `ai-docs/claude-plans/20260306-multi-tenant-2/multi-tenant-flow-ctx.md`
> **Implementation plan**: `ai-docs/claude-plans/20260306-multi-tenant-2/20260308-frontend-impl.md`

---

## Context

The backend now supports multi-tenant login with (commits `6a7d879..04788eb`):
- Dashboard auth endpoints (`POST /auth/dashboard/initiate`, `POST /auth/dashboard/callback`)
- Tenant listing (`GET /bodhi/v1/tenants`) — enriched with `is_active`, `logged_in` per client
- Tenant registration (`POST /bodhi/v1/tenants`) — body: `{ name, description }`, returns `{ client_id }`
- Tenant switching (`POST /bodhi/v1/tenants/{client_id}/activate`)
- Namespaced session tokens (`{client_id}:access_token`, `active_client_id`)
- New `AppStatus::TenantSelection` variant
- `/info` returns `deployment: String` and `client_id: Option<String>`
- `/user/info` returns `UserInfoEnvelope` with `has_dashboard_session: bool`

The following backend prerequisites were completed alongside the frontend work:
- **D68**: `POST /auth/initiate` now requires `{ client_id: string }` in request body (was using `get_standalone_app()`)
- **D77**: `dashboard_callback_url()` added to `SettingService`, dashboard auth route updated to use it
- **Session-aware `/info`**: `resolve_multi_tenant_status()` checks session for dashboard token / active_client_id to determine `TenantSelection` vs `Ready` vs `Setup`

---

## What Was Built

### Backend Prerequisites (Phase 1)

1. **`POST /auth/initiate`** unified: `client_id` always required in request body. Standalone frontend gets it from `/info` response. Stores `auth_client_id` in session for callback retrieval.
2. **`POST /auth/callback`** updated: reads `auth_client_id` from session to look up tenant (no more `get_standalone_app()`). Cleans up `auth_client_id` after exchange.
3. **`dashboard_callback_url()`**: Added to `SettingService` trait + `DefaultSettingService` impl, using `LOGIN_DASHBOARD_CALLBACK_PATH = "/ui/auth/dashboard/callback"`.
4. **`/info` session-aware**: `resolve_multi_tenant_status()` helper checks active_client_id + resource token -> `Ready`; dashboard token + SPI tenants -> `Setup` or `TenantSelection`; no dashboard token -> `TenantSelection`.

### Frontend (Phases 2-4)

1. **AppInitializer** (`components/AppInitializer.tsx`):
   - `allowedStatus` accepts `AppStatus | AppStatus[]`
   - `tenant_selection` status routes to `/ui/login`
   - `setup` + `deployment=multi-tenant` routes to `/ui/setup/tenants`
   - `setup` + `deployment=standalone` routes to `/ui/setup` (unchanged)

2. **Dashboard callback page** (`app/ui/auth/dashboard/callback/page.tsx`):
   - Follows existing callback page pattern (Suspense, useRef for duplicate prevention)
   - Extracts `code`/`state` from URL params
   - Calls `useDashboardOAuthCallback()` mutation
   - On success: `handleSmartRedirect` to `/ui/login`

3. **Login page** (`app/ui/login/page.tsx`):
   - `AppInitializer allowedStatus={['ready', 'tenant_selection']}`
   - **Standalone mode**: Not logged in -> "Login" button with `client_id` from `/info`; logged in -> user info + home/logout
   - **Multi-tenant, no dashboard session**: "Login to Bodhi Platform" button -> dashboard OAuth
   - **Multi-tenant, dashboard session, 1 tenant**: Auto-initiate resource OAuth
   - **Multi-tenant, dashboard session, N tenants**: Tenant selector with connect/switch buttons
   - **Multi-tenant, fully authenticated**: Current tenant info + switch dropdown

4. **Tenant registration page** (`app/ui/setup/tenants/page.tsx`):
   - Form with `name` (required) and `description` (required)
   - `useCreateTenant()` mutation
   - On success: auto-initiate resource OAuth with returned `client_id`

5. **New hooks** (`hooks/useAuth.ts`, `hooks/useTenants.ts`):
   - `useDashboardOAuthInitiate()` — POST `/bodhi/v1/auth/dashboard/initiate`
   - `useDashboardOAuthCallback()` — POST `/bodhi/v1/auth/dashboard/callback`
   - `useTenants()` — GET `/bodhi/v1/tenants`
   - `useCreateTenant()` — POST `/bodhi/v1/tenants`
   - `useTenantActivate()` — POST `/bodhi/v1/tenants/{client_id}/activate`
   - `useOAuthInitiate()` — updated to accept `{ client_id: string }` variable

6. **Constants** (`lib/constants.ts`):
   - `ROUTE_SETUP_TENANTS = '/ui/setup/tenants'`
   - `ROUTE_DASHBOARD_CALLBACK = '/ui/auth/dashboard/callback'`

### Other Changes
- `lib_bodhiserver_napi/src/config.rs`: Updated for `created_by` field on `Tenant` struct
- `server_app/tests/utils/live_server_utils.rs`: Updated session keys to namespaced format
- OpenAPI spec + TypeScript client regenerated

---

## TypeScript Types (from `@bodhiapp/ts-client`)

- `AppStatus` enum — includes `tenant_selection`
- `AppInfo` — `deployment: string`, `client_id?: string | null`
- `AuthInitiateRequest` — `{ client_id: string }`
- `TenantListResponse` — `{ tenants: TenantListItem[] }`
- `TenantListItem` — `{ client_id, name, description?, is_active, logged_in }`
- `CreateTenantRequest` — `{ name: string, description: string }` (both required, with validation)
- `CreateTenantResponse` — `{ client_id }`
- NOTE: No `DashboardAuthCallbackRequest` type — dashboard callback reuses `AuthCallbackRequest` (`{ code, state, error, error_description }`)
- `UserInfoEnvelope` — extends UserResponse with `has_dashboard_session?: boolean`

---

## Decision Summary

| # | Decision |
|---|----------|
| D30 | Auto-redirect for single client, dropdown for multiple |
| D38 | Multi-tenant: no mandatory wizard — tenant Ready immediately |
| D39 | /ui/login reused for tenant selection and switching |
| D47 | Dashboard callback redirects to /ui/login |
| D48 | Registration UI at /ui/setup/tenants/ |
| D49 | /user/info extended for dashboard session detection |
| D50 | GET /tenants enriched with `is_active`, `logged_in` per client |
| D55 | `POST /bodhi/v1/tenants/{client_id}/activate` for instant tenant switching |
| D60 | Tenant registration API: user sends name + description only |
| D67 | `/info` returns `deployment: "standalone" \| "multi_tenant"`. Frontend uses for feature visibility |
| D68 | `client_id` always required in `POST /auth/initiate` — standalone frontend gets it from `/info` |
| D70 | `/info` includes `client_id` |
| D77 | Separate frontend callback routes: `/ui/auth/dashboard/callback` vs `/ui/auth/callback` |
| D79 | Tenant created Ready immediately. API key config accessible from settings |
| D84 | No role in tenant dropdown — role visible after login via JWT claims |

---

## Outstanding Work

### Not yet done (deferred):
- E2E/Playwright tests for multi-tenant flows
- Service construction changes (skip LLM routes in multi-tenant mode)
- Navigation item visibility (hide LLM-specific items in multi-tenant)
- Frontend unit tests for new components (some tests updated, not all new components have tests)

### Gate Checks

```bash
# Backend
cargo test -p routes_app
cargo test -p services --lib
make test.backend

# Frontend
cd crates/bodhi && npm test

# E2E (after UI rebuild)
make build.ui-rebuild
make test.napi
```
