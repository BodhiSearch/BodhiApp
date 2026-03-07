# Pre-E2E Fixes — Kickoff

> **Created**: 2026-03-09
> **Status**: COMPLETED (Items 1-4 resolved in Batch 0-2; Item 5 resolved by removing dead code; Item 6 deferred)
> **Scope**: Backend + Frontend fixes for gaps discovered during doc audit, blocking E2E test implementation
> **Prior work**:
> - Backend: `kickoff-bodhi-backend.md` (M2), `20260308-rs-integration-test.md` (M4)
> - Frontend: `20260308-frontend-impl.md` (M3)
> - TECHDEBT: `TECHDEBT.md` (gaps identified during doc-vs-code audit)
> **Context doc**: `multi-tenant-flow-ctx.md`
> **Next step**: `kickoff-e2e-multi-tenant.md` (E2E tests, depends on this kickoff completing)

---

## Task

Fix implementation gaps discovered during doc-vs-code audit. These are planned features that were either missed or partially implemented during M2/M3. They must be resolved before E2E tests can exercise the full multi-tenant flow correctly.

Six items, ordered by dependency:

1. **`BODHI_DEPLOYMENT` rename**: `"multi-tenant"` → `"multi_tenant"` (snake_case)
2. **`/info` move to `optional_auth`**: D54 — move from `public_apis` to `optional_auth` router
3. **Standalone `created_by` fix**: Replaced `update_status()` with `set_client_ready()` which handles both status and created_by
4. **Dashboard endpoints in OpenAPI**: Register in `openapi.rs` + regenerate TypeScript client
5. **`id_token` storage**: Dead code (`id_token_key()`, `DASHBOARD_ID_TOKEN_KEY`) removed instead — not needed until OIDC logout
6. **`CreateTenantRequest.description` optional**: Change from required to optional field

---

## Item 1: Rename `BODHI_DEPLOYMENT` Value to Snake Case

### Why

The deployment mode value `"multi-tenant"` uses a hyphen, which is inconsistent with Rust/TypeScript snake_case conventions and creates ambiguity. Rename to `"multi_tenant"` for clear, consistent serialization. This must happen first because all other items and the E2E project rename (`sqlite`→`standalone`, `postgres`→`multi_tenant`) depend on this value.

### Files to change

**Backend (Rust)** — search all `.rs` files for the literal string `"multi-tenant"`:
- `crates/services/src/settings/setting_service.rs` ~line 255 — `is_multi_tenant()` compares against `"multi-tenant"`
- Any test files asserting deployment mode value (grep `"multi-tenant"` in test files)
- `crates/server_app/tests/utils/live_server_utils.rs` — `setup_multitenant_app_service()` sets this value
- `crates/routes_app/tests/test_live_multi_tenant.rs` — assertions checking JSON responses

**Frontend (TypeScript/React)** — search all `.tsx`/`.ts` files for `'multi-tenant'`:
- `crates/bodhi/src/app/ui/login/page.tsx` ~line 348 — `appInfo?.deployment === 'multi-tenant'`
- `crates/bodhi/src/components/AppInitializer.tsx` ~line 58 — `appInfo.deployment === 'multi-tenant'`

**Regenerate after backend changes**:
```bash
cargo run --package xtask openapi && cd ts-client && npm run generate
```

### Gate check
```bash
cargo test -p services --lib -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED"
cd crates/bodhi && npm test
```

---

## Item 2: Move `/info` to `optional_auth` Router

### Why (D54)

`/info` (`setup_show`) is registered in `public_apis` (line 81 of `routes.rs`). The handler uses `AuthScope` which falls back to `Anonymous { client_id: None, tenant_id: None }` without middleware. This means in standalone mode, even when a user is logged in, `/info` returns `client_id: null` because `AuthContext` is never populated.

The E2E flow depends on: `GET /info` → extract `client_id` → `POST /auth/initiate { client_id }`. With `client_id: null`, the standalone login flow breaks.

In multi-tenant mode this works by accident because `resolve_multi_tenant_status()` reads `client_id` from session keys directly, bypassing `AuthContext`.

### What to change

Move `ENDPOINT_APP_INFO` from `public_apis` to `optional_auth` in `crates/routes_app/src/routes.rs`:

```rust
// Before (line 81):
let public_apis = Router::new()
    .route(ENDPOINT_PING, get(ping_handler))
    .route(ENDPOINT_HEALTH, get(health_handler))
    .route(ENDPOINT_APP_INFO, get(setup_show))       // <-- remove from here
    .route(ENDPOINT_APP_SETUP, post(setup_create))
    ...

// After:
let mut optional_auth = Router::new()
    .route(ENDPOINT_APP_INFO, get(setup_show))        // <-- add here
    .route(ENDPOINT_USER_INFO, get(users_info))
    ...
```

### Impact analysis

Explore whether any code depends on `/info` being unauthenticated. The `optional_auth_middleware` does NOT block unauthenticated requests — it just populates `AuthContext` when auth is present and falls back to `Anonymous` otherwise. So `/info` will still work without auth, but now `client_id` will be populated for authenticated users.

Check:
- `crates/routes_app/src/setup/routes_setup.rs` — `setup_show` handler. Does it use `auth_scope.auth_context().client_id()`? In standalone mode it does (the `else` branch). Verify this will now return `Some(client_id)` when session auth is present.
- Frontend: `crates/bodhi/src/hooks/useInfo.test.ts` — any test expectations about `client_id` being null?
- E2E tests: `crates/lib_bodhiserver_napi/tests-js/` — any tests that call `/info` and check the response?

### Gate check
```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED"
```

---

## Item 3: Standalone `created_by` Fix

### Why

`auth_callback` in `routes_auth.rs` transitions standalone tenants from `ResourceAdmin` → `Ready` (lines 269-279) but does NOT call `tenant_svc.update_created_by(client_id, user_id)`. The method exists on `TenantService` (line 28 in `tenant_service.rs`) and works. Multi-tenant correctly sets `created_by` during `POST /bodhi/v1/tenants`.

### What to change

In `crates/routes_app/src/auth/routes_auth.rs`, after the `tenant_svc.update_status(&AppStatus::Ready)` call (line 273), add:

```rust
tenant_svc.update_created_by(&instance.client_id, &user_id).await?;
```

The `user_id` is already extracted from JWT claims on line 267. The `instance.client_id` is already available.

### Explore first

- Read `crates/services/src/tenants/tenant_service.rs` — `update_created_by()` signature and implementation
- Check if `update_created_by` needs a transaction or tenant-scoped txn
- Verify error handling — the method returns `Result<()>`, so `?` propagation should work via `AuthRouteError`

### Gate check
```bash
cargo test -p routes_app -- auth 2>&1 | grep -E "test result|FAILED"
```

Also add a unit test verifying `created_by` is set after the ResourceAdmin→Ready transition.

---

## Item 4: Dashboard Endpoints in OpenAPI

### Why

`dashboard_auth_initiate` and `dashboard_auth_callback` have `#[utoipa::path]` annotations in `routes_dashboard_auth.rs` but are NOT registered in `BodhiOpenAPIDoc` in `openapi.rs`. The `__path_*` symbols are never imported, so these endpoints are absent from `openapi.json` and the generated TypeScript client.

The frontend currently uses hand-rolled API calls for these endpoints (in `useAuth.ts` hooks). E2E tests need proper TypeScript types.

### What to change

In `crates/routes_app/src/shared/openapi.rs`:
1. Add `__path_dashboard_auth_initiate` and `__path_dashboard_auth_callback` to the imports
2. Add them to the `paths(...)` section of `BodhiOpenAPIDoc`
3. Add any missing request/response schemas to the `components(schemas(...))` section

### Explore first

- Read `crates/routes_app/src/tenants/routes_dashboard_auth.rs` — the `#[utoipa::path]` annotations for request/response types
- Check what schemas are referenced (`DashboardAuthInitiateRequest`? The callback reuses `AuthCallbackRequest`)
- Look at how existing auth endpoints are registered in `openapi.rs` for the pattern

### After changes
```bash
cargo run --package xtask openapi
cd ts-client && npm run generate
cd crates/bodhi && npm test
```

### Follow-up

Update frontend hooks in `crates/bodhi/src/hooks/useAuth.ts` to use the generated TypeScript client types instead of hand-rolled API calls, if the types are now available. This is optional — the hooks work either way.

---

## Item 5: Store `id_token` in Session

### Why

`id_token_key()` in `auth_middleware.rs` and `DASHBOARD_ID_TOKEN_KEY` in `tenants/mod.rs` are defined but never used. Neither `auth_callback` nor `dashboard_auth_callback` stores the `id_token` from the token exchange response.

While not immediately blocking, OIDC logout requires `id_token_hint`, and E2E tests for logout flows will need this.

### What to change

Explore the token exchange response type to find where `id_token` is available:
- `crates/services/src/auth/auth_service.rs` — `exchange_auth_code()` return type
- The Keycloak token response includes `id_token` — check if it's in the response tuple

If available, store it in both callbacks:

**`auth_callback`** (in `routes_auth.rs`):
```rust
// After storing access_token and refresh_token:
if let Some(id_token) = token_response.id_token {
    session.insert(&id_token_key(&instance.client_id), id_token).await?;
}
```

**`dashboard_auth_callback`** (in `routes_dashboard_auth.rs`):
```rust
session.insert(DASHBOARD_ID_TOKEN_KEY, id_token).await?;
```

### Priority

Low. If `exchange_auth_code()` doesn't return `id_token` today, this requires service-layer changes. In that case, defer and add a note to TECHDEBT.md. The E2E tests can proceed without logout flow testing initially.

---

## Item 6: `CreateTenantRequest.description` — Make Optional

### Why

The `CreateTenantRequest` in `crates/routes_app/src/tenants/` has `description: String` (required). The frontend registration page and the design docs say description is optional. The SPI also accepts `description` as optional (`null` → empty string).

### What to change

- Change `description: String` to `description: Option<String>` in the request struct
- Update the handler to pass `description.unwrap_or_default()` or similar when proxying to SPI
- Update validation if any (`#[validate]` constraints)

### Explore first

- Read the `CreateTenantRequest` struct definition
- Read the handler that uses it — `tenants_create` in `routes_tenants.rs`
- Check what the SPI expects (`POST /realms/{realm}/bodhi/tenants` body)

### Gate check
```bash
cargo test -p routes_app -- tenants 2>&1 | grep -E "test result|FAILED"
cargo run --package xtask openapi && cd ts-client && npm run generate
```

---

## Execution Order

Items have dependencies — follow this order:

```
1. BODHI_DEPLOYMENT rename ("multi-tenant" → "multi_tenant")
   ↓
2. /info move to optional_auth (D54)
   ↓
3. Standalone created_by fix
   |
4. Dashboard endpoints in OpenAPI    ← these three are independent
   |
5. id_token storage
   |
6. CreateTenantRequest.description optional
   ↓
Final: cargo run --package xtask openapi && cd ts-client && npm run generate
Final: make test.backend
Final: cd crates/bodhi && npm test
```

Items 3-6 are independent of each other and can be done in any order after items 1-2.

---

## Gate Checks

### Per-item (run after each item)
```bash
cargo test -p services --lib -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED"
```

### Final validation
```bash
make test.backend
cd crates/bodhi && npm test
cargo run --package xtask openapi && cd ts-client && npm run generate
make ci.ts-client-check
```

---

## Files to Explore

### Core files being changed
- `crates/routes_app/src/routes.rs` — route registration (items 2)
- `crates/routes_app/src/auth/routes_auth.rs` — auth_callback handler (items 3, 5)
- `crates/routes_app/src/tenants/routes_dashboard_auth.rs` — dashboard callback (items 5)
- `crates/routes_app/src/shared/openapi.rs` — OpenAPI registration (item 4)
- `crates/services/src/settings/setting_service.rs` — deployment mode check (item 1)

### Frontend files
- `crates/bodhi/src/app/ui/login/page.tsx` — deployment check (item 1)
- `crates/bodhi/src/components/AppInitializer.tsx` — deployment check (item 1)

### Test files to update
- `crates/routes_app/src/setup/test_setup.rs` — `/info` response assertions
- `crates/routes_app/src/auth/test_login_callback.rs` — auth_callback tests
- `crates/routes_app/tests/test_live_multi_tenant.rs` — deployment value assertions
- `crates/server_app/tests/utils/live_server_utils.rs` — multi-tenant setup

### Reference
- `crates/services/src/tenants/tenant_service.rs` — `update_created_by()` method
- `crates/services/src/auth/auth_service.rs` — `exchange_auth_code()` return type
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — `id_token_key()`, session key helpers
