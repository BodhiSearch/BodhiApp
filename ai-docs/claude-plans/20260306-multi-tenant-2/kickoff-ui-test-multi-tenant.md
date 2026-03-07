# Multi-Tenant Frontend Unit Tests -- Kickoff

> **Created**: 2026-03-08
> **Status**: TODO
> **Scope**: Frontend unit tests in `crates/bodhi/src/` (Vitest + MSW v2)
> **Prior work**:
> - Frontend implementation: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-frontend.md`
> - Implementation plan: `ai-docs/claude-plans/20260306-multi-tenant-2/20260308-frontend-impl.md`
> - Backend API contract: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-backend.md`
> **Known gaps**: `ai-docs/claude-plans/20260306-multi-tenant-2/TECHDEBT.md` (section "Frontend Unit Tests")

---

## Task

Add comprehensive unit tests for the multi-tenant frontend features implemented since commit `6a7d879`. These features span new components, new hooks, and modifications to existing components. The goal is to cover the `deployment=multi-tenant` code paths that were added alongside the existing `standalone` paths.

Run tests with: `cd crates/bodhi && npm test`

---

## Testing Infrastructure

### Vitest + MSW v2

- **Test runner**: Vitest with jsdom environment
- **API mocking**: MSW v2 with `openapi-msw` for type-safe handlers
- **Type-safe HTTP**: `typedHttp` from `src/test-utils/msw-v2/setup.ts` wraps MSW with OpenAPI schema enforcement
- **Test wrapper**: `src/tests/wrapper.tsx` -- `createWrapper()` provides `QueryClientProvider`
- **Test setup**: `src/tests/setup.ts` -- configures `baseURL`, mocks `matchMedia`, `ResizeObserver`, pointer events
- **Generated types**: All API types from `@bodhiapp/ts-client` (file:../../ts-client)

### MSW handler pattern

Handlers live in `src/test-utils/msw-v2/handlers/`. Each domain has its own file with factory functions like `mockAppInfo()`, `mockUserLoggedIn()`, etc. Handlers use a `hasBeenCalled` guard for one-shot behavior, with an opt-in `stub` flag for persistent handlers.

Study these existing handlers to understand the pattern:
- `src/test-utils/msw-v2/handlers/info.ts` -- `mockAppInfo()` already accepts `deployment` and `client_id` params
- `src/test-utils/msw-v2/handlers/user.ts` -- `mockUserLoggedIn()`, `mockUserLoggedOut()`
- `src/test-utils/msw-v2/handlers/auth.ts` -- OAuth mock handlers
- `src/test-utils/msw-v2/handlers/setup.ts` -- setup endpoint handlers

### Window location mocking

`src/tests/wrapper.tsx` exports `mockWindowLocation()` for tests that check URL-based behavior (callback pages, redirects).

---

## Before Writing Tests

Read the actual implementation to understand what was built:

```bash
git diff 6a7d879..HEAD -- crates/bodhi/src/
```

Also read:
- `ai-docs/claude-plans/20260306-multi-tenant-2/20260308-frontend-impl.md` -- full implementation summary
- `ai-docs/claude-plans/20260306-multi-tenant-2/TECHDEBT.md` -- known test gaps
- Existing test files for components that changed (to understand what is already covered and what is not)

---

## Files to Explore

### Changed components

- `src/components/AppInitializer.tsx` + `src/components/AppInitializer.test.tsx` -- `allowedStatus` now accepts arrays, new routing for `tenant_selection` and `setup` + multi-tenant
- `src/app/ui/login/page.tsx` + `src/app/ui/login/page.test.tsx` -- standalone vs multi-tenant branching, sub-states (no dashboard session, 1 tenant auto-login, N tenants selector)
- `src/components/LoginMenu.test.tsx` -- may need `client_id` guard updates

### New components (need tests)

- `src/app/ui/auth/dashboard/callback/page.tsx` -- dashboard OAuth callback (Suspense, `useDashboardOAuthCallback`)
- `src/app/ui/setup/tenants/page.tsx` -- tenant registration form (name/description, `useCreateTenant`, auto-initiate OAuth on success)

### New hooks (need tests)

- `src/hooks/useTenants.ts` -- `useTenants()`, `useCreateTenant()`, `useTenantActivate()`
- `src/hooks/useAuth.ts` -- `useDashboardOAuthInitiate()`, `useDashboardOAuthCallback()`, updated `useOAuthInitiate()` (now requires `{ client_id }`)

### Existing tests to check for coverage

- `src/hooks/useAuth.test.tsx` -- already updated for `client_id`, but check if dashboard hooks are tested
- `src/hooks/useInfo.test.ts` -- may need multi-tenant status tests
- `src/components/AppInitializer.test.tsx` -- check if array `allowedStatus` and multi-tenant routing are covered

### MSW handlers (may need new ones)

- `src/test-utils/msw-v2/handlers/info.ts` -- already supports `deployment` and `client_id` params
- `src/test-utils/msw-v2/handlers/user.ts` -- may need `has_dashboard_session` support in `mockUserLoggedIn()`
- New handlers needed for: `GET /tenants`, `POST /tenants`, `POST /tenants/{client_id}/activate`, `POST /auth/dashboard/initiate`, `POST /auth/dashboard/callback`

### TypeScript types

- Check `@bodhiapp/ts-client` for: `AppStatus` (includes `tenant_selection`), `AppInfo` (has `deployment`, `client_id`), `TenantListResponse`, `TenantListItem`, `CreateTenantRequest`, `CreateTenantResponse`, `AuthCallbackRequest` (reused for dashboard callback — no separate `DashboardAuthCallbackRequest`), `UserInfoEnvelope`

---

## Key Areas to Cover

These are directional. Discover the actual coverage gaps by reading existing tests.

### AppInitializer

- `allowedStatus` as an array (e.g., `['ready', 'tenant_selection']`)
- `tenant_selection` status routes to `/ui/login`
- `setup` status + `deployment=multi-tenant` routes to `/ui/setup/tenants` (not `/ui/setup`)
- `setup` status + `deployment=standalone` routes to `/ui/setup` (unchanged)
- All existing standalone-mode tests still pass

### Login page

**Standalone mode**:
- Not logged in: shows login button with `client_id` from `/info`
- Logged in: shows user info and navigation

**Multi-tenant mode, no dashboard session** (`has_dashboard_session: false`):
- Shows "Login to Bodhi Platform" button
- Clicking initiates dashboard OAuth

**Multi-tenant mode, dashboard session, 1 tenant**:
- Auto-initiates resource OAuth for the single tenant (no user interaction)

**Multi-tenant mode, dashboard session, N tenants**:
- Shows tenant selector/list
- Connect button for tenants not yet logged in
- Switch/activate button for tenants already logged in

**Multi-tenant mode, fully authenticated** (resource token active):
- Shows current tenant info
- Tenant switching UI

### Dashboard callback page

- Extracts `code` and `state` from URL search params
- Calls `useDashboardOAuthCallback` mutation
- On success: redirects to `/ui/login`
- Error handling: shows error state
- Duplicate prevention (useRef pattern -- follows existing callback page)

### Tenant registration page

- Form with `name` (required) and `description` (optional)
- Validation: name is required
- Calls `useCreateTenant` mutation on submit
- On success: auto-initiates resource OAuth with returned `client_id`
- Error handling: shows error from API

### New hooks

- `useTenants()` -- fetches `GET /tenants`, returns tenant list
- `useCreateTenant()` -- `POST /tenants` with name/description
- `useTenantActivate()` -- `POST /tenants/{client_id}/activate`
- `useDashboardOAuthInitiate()` -- `POST /auth/dashboard/initiate`, handles redirect
- `useDashboardOAuthCallback()` -- `POST /auth/dashboard/callback` with code/state
- `useOAuthInitiate()` -- now requires `{ client_id }` variable (verify existing tests updated)

### Standalone regression

Verify that existing tests for standalone mode still pass. The `client_id` requirement in `useOAuthInitiate` and the `deployment` field in `AppInfo` should not break standalone flows as long as `/info` returns `client_id` and `deployment: 'standalone'`.

---

## MSW Handler Notes

New MSW handlers will likely be needed for the tenant and dashboard auth endpoints. Follow the pattern in existing handler files:
- Factory function with configurable response fields
- `hasBeenCalled` guard with `stub` opt-in
- Export from a dedicated file (e.g., `handlers/tenants.ts`, or extend `handlers/auth.ts` for dashboard auth)
- Use `typedHttp` from `../setup` for OpenAPI type safety

Check the hook files for the endpoint constants (e.g., `ENDPOINT_TENANTS`, `ENDPOINT_DASHBOARD_AUTH_INITIATE`) to use in handlers.

---

## Gate Checks

```bash
cd crates/bodhi && npm test
```
