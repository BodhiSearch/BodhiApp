# Multi-Tenant Backend Implementation Plan — ✅ COMPLETED

> **Created**: 2026-03-08
> **Completed**: 2026-03-09
> **Kickoff**: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-bodhi-backend.md`
> **Context**: `ai-docs/claude-plans/20260306-multi-tenant-2/multi-tenant-flow-ctx.md`

---

## Context

The auth middleware now resolves tenants from JWT `azp` claims. The Keycloak SPI is deployed. This plan implements the multi-tenant login flow in the BodhiApp backend: dashboard auth, tenant listing/registration via SPI proxy, tenant switching, namespaced session keys, and unified standalone/multi-tenant auth routes. Frontend is out of scope (deferred).

**Deferred from M2, resolved in M3**: D68 (unified auth_initiate with `client_id` in body — now implemented), D77 (dashboard_callback_url — now implemented), `/info` session-aware TenantSelection status (now implemented). **Still deferred**: D80 (shared code exchange utility), multi-tenant-aware selective logout. See `kickoff-bodhi-backend.md` "Implementation Summary" for details.

---

## Key Decisions (from interview)

| # | Decision |
|---|----------|
| I1 | SPI proxy methods (list_tenants, create_tenant) on **AuthService** — reuses reqwest + Keycloak config |
| I2 | Dashboard-specific services (e.g., dashboard token refresh helper) in **services/tenants/** |
| I3 | `/info` behind **optional_auth_middleware** — API key bearers get tenant context, Anonymous ok |
| I4 | Shared code exchange → **extract inner function** parameterized by key prefix + credentials |
| I5 | Migration → **modify existing** m20250101_000013_apps.rs (fresh DBs, D14 convention) |
| I6 | Refresh lock key → **`{client_id}:{session_id}:refresh_token`** (per tenant per session) |
| I7 | Session migration → baseline → analyze → test-util abstractions → middleware change → migrate tests → gate check (split into 2 sub-agents) |
| I8 | Dashboard token refresh → **dedicated helper** `ensure_valid_dashboard_token()` |
| I9 | `created_by` → **both**: `create_tenant(…, created_by: Option<String>)` AND `set_client_ready(client_id, user_id)` (sets status + created_by) |
| I10 | Testing → **follow convention per crate** (MockAuthService in routes_app, mockito in services) |
| I11 | Sub-agents → **one per phase**, sequential |
| I12 | Redirect URIs → **`setting_service.public_server_url()`** (from bodhi_public_scheme/host/port) |
| I13 | All multi-tenant routes in **routes_app/tenants/** module (dashboard auth + tenant CRUD) |
| I14 | `/info` response → **extend AppInfo** with `deployment: String` + `client_id: Option<String>` |
| I15 | Callback tenant lookup → **store client_id in session during auth_initiate** |
| I16 | Standalone flow → **unified** — `client_id` always required in `POST /auth/initiate` body |
| I17 | `BODHI_MULTITENANT_CLIENT_SECRET` → **settings constant** (env var, via SettingService) |
| I18 | Phase 2 → **split**: 2a (analyze + abstractions) + 2b (middleware + test migration) |
| I19 | Frontend → **backend only**, defer frontend pages |

### TECHDEBT items to add
- `crates/services/TECHDEBT.md`: ConcurrencyService should use PostgreSQL advisory locks when DB is Postgres (cluster-wide lock for multi-deployment)
- `crates/services/TECHDEBT.md`: Encapsulate session token lifecycle — auto-refresh on access, retryable error on refresh failure (replace ad-hoc dashboard refresh helper)

---

## Phase Overview

| Phase | Scope | Crate(s) | Sub-agents | Gate |
|-------|-------|----------|------------|------|
| 1 | Foundation: schema, types, service traits | services | 1 | `cargo test -p services --lib` |
| 2a | Session test analysis + abstractions | routes_app | 1 | Baseline still passes |
| 2b | Middleware + session key migration | routes_app | 1 | All existing tests pass |
| 3 | Dashboard auth routes | routes_app | 1 | New + regression |
| 4 | Tenant management routes | routes_app | 1 | New + regression |
| 5 | Unified auth + enhanced /info + /user/info | routes_app | 1 | Full cross-crate regression |
| 6 | TECHDEBT + documentation | services, routes_app | 1 | N/A |

---

## Phase 1: Foundation (services)

**Sub-agent scope**: All changes in `crates/services/`

### 1.1 Migration: add `created_by` to tenants table

- **File**: `crates/services/src/db/sea_migrations/m20250101_000013_apps.rs`
- Add `created_by VARCHAR(255) NULL` column to tenants table definition
- Modify existing migration inline (D14 convention — fresh DBs only)

### 1.2 Tenant entity + model

- **File**: `crates/services/src/tenants/tenant_entity.rs` — add `created_by: Option<String>` to Model
- **File**: `crates/services/src/tenants/tenant_objs.rs`:
  - Add `created_by: Option<String>` to `Tenant` struct and `TenantRow`
  - Add `AppStatus::TenantSelection` variant (serializes as `"tenant_selection"`)
- **File**: `crates/services/src/tenants/tenant_repository.rs`:
  - Update `insert_tenant()` to accept + store `created_by`
  - Update `decrypt_tenant_row()` to map `created_by`
  - Add `update_tenant_created_by(client_id: &str, created_by: &str)` repository method

### 1.3 TenantService trait changes

- **File**: `crates/services/src/tenants/tenant_service.rs`
  - Change `create_tenant(client_id, client_secret, status)` → `create_tenant(client_id, client_secret, status, created_by: Option<String>)`
  - Add `set_client_ready(client_id: &str, user_id: &str) -> Result<()>` — sets status to Ready AND created_by in one call

### 1.4 Settings constants

- **File**: `crates/services/src/settings/constants.rs`
  - Add `BODHI_MULTITENANT_CLIENT_SECRET` constant
- **File**: `crates/services/src/settings/setting_service.rs`
  - Add `multitenant_client_secret() -> Option<String>` method to trait + impl

### 1.5 AuthService SPI proxy methods

- **File**: `crates/services/src/auth/auth_service.rs`
  - Add to trait:
    ```rust
    async fn list_tenants(&self, bearer_token: &str) -> Result<SpiTenantListResponse>;
    async fn create_tenant(&self, bearer_token: &str, name: &str, description: &str, redirect_uris: Vec<String>) -> Result<SpiCreateTenantResponse>;
    ```
  - Implement in `KeycloakAuthService`:
    - `GET {auth_url}/realms/{realm}/bodhi/tenants` with `Authorization: Bearer {token}`
    - `POST {auth_url}/realms/{realm}/bodhi/tenants` with JSON body + bearer token
    - Error mapping: SPI errors → service-level errors (401, 400, 409)

### 1.6 SPI types

- **File**: `crates/services/src/tenants/spi_types.rs` (new)
  - `SpiTenant { client_id: String, name: String, description: Option<String> }`
  - `SpiTenantListResponse { tenants: Vec<SpiTenant> }`
  - `SpiCreateTenantRequest { name: String, description: String, redirect_uris: Vec<String> }`
  - `SpiCreateTenantResponse { client_id: String, client_secret: String }`

### 1.7 Test fixtures

- Update `Tenant::test_default()` and `Tenant::test_tenant_b()` to include `created_by: None`
- Update `create_tenant_test()` in test utils
- Add `AppStatus::TenantSelection` serialization test
- Add mock expectations for new AuthService methods

### Gate check
```bash
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

---

## Phase 2a: Session Test Analysis & Abstractions (routes_app)

**Sub-agent scope**: Read-only analysis + test utility creation only

### 2a.1 Run baseline
```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
```
Record passing count.

### 2a.2 Catalog session key usage

Search all test files for:
- `SESSION_KEY_ACCESS_TOKEN` / `"access_token"` in session setup
- `SESSION_KEY_REFRESH_TOKEN` / `"refresh_token"` in session setup
- `SESSION_KEY_USER_ID` / `"user_id"` in session setup
- `create_authenticated_session()` helper usage
- `set_token_in_session()` helper usage
- Direct `session.insert()` calls with token keys

### 2a.3 Create session helper abstractions

- **File**: `crates/routes_app/src/test_utils/session_helpers.rs` (new)
  - `set_resource_session(session, client_id, access_token, refresh_token, user_id)` — sets namespaced keys + `active_client_id`
  - `set_dashboard_session(session, access_token, refresh_token)` — sets `dashboard:*` keys
  - Update `create_authenticated_session()` to use namespaced keys internally
  - Constants for test client IDs

### Gate check
```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
```
Must match baseline count.

---

## Phase 2b: Middleware & Session Key Migration (routes_app)

**Sub-agent scope**: Production code changes + test migration

### 2b.1 Session key constants

- **File**: `crates/routes_app/src/middleware/auth/auth_middleware.rs`
  - Replace flat constants:
    - ~~`SESSION_KEY_ACCESS_TOKEN = "access_token"`~~
    - ~~`SESSION_KEY_REFRESH_TOKEN = "refresh_token"`~~
  - Add:
    - `SESSION_KEY_ACTIVE_CLIENT_ID = "active_client_id"`
  - Add helper functions:
    - `fn access_token_key(client_id: &str) -> String` → `format!("{client_id}:access_token")`
    - `fn refresh_token_key(client_id: &str) -> String` → `format!("{client_id}:refresh_token")`
    - These are now defined in `services::session_keys` module, re-exported from the `services` crate

### 2b.2 Middleware two-step lookup

- **File**: `crates/routes_app/src/middleware/auth/auth_middleware.rs`
  - `auth_middleware` and `optional_auth_middleware`:
    1. Read `session["active_client_id"]`
    2. If present: read `session[access_token_key(active_client_id)]`
    3. Extract JWT azp, resolve tenant via `get_tenant_by_client_id(azp)`
    4. Call `get_valid_session_token(session, access_token, &tenant)` for refresh
    5. If no `active_client_id`: Anonymous (optional) or error (strict)

### 2b.3 Token service refresh lock

- **File**: `crates/routes_app/src/middleware/token_service/token_service.rs`
  - Change lock key from `refresh_token:{session_id}` → `refresh_token:{session_id}:{client_id}`
  - `get_valid_session_token()` already takes `&Tenant` — use `tenant.client_id` for lock key
  - Session token read/write: use `access_token_key(tenant.client_id)` and `refresh_token_key(tenant.client_id)`

### 2b.4 Auth callback: write namespaced keys

- **File**: `crates/routes_app/src/auth/routes_auth.rs`
  - `auth_callback`: after code exchange:
    - `session.insert("active_client_id", tenant.client_id)`
    - `session.insert(access_token_key(&tenant.client_id), access_token)`
    - `session.insert(refresh_token_key(&tenant.client_id), refresh_token)`
    - Keep `SESSION_KEY_USER_ID` as-is (same user across all tenants)
  - Still uses `get_standalone_app()` for tenant lookup in this phase (unified in Phase 5)

### 2b.5 Migrate all failing tests

- Update test-utils: `create_authenticated_session()` writes namespaced keys
- Migrate all tests that directly set session keys to use helper abstractions from Phase 2a
- Update middleware isolation tests for two-step lookup

### Gate check
```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
```
Must match or exceed baseline count.

---

## Phase 3: Dashboard Auth Routes (routes_app)

**Sub-agent scope**: New module + routes, no changes to existing routes

### 3.1 Module structure

```
crates/routes_app/src/tenants/
  mod.rs
  routes_dashboard_auth.rs      — dashboard initiate/callback
  routes_tenants.rs             — (Phase 4) GET/POST /tenants, activate
  dashboard_helpers.rs          — ensure_valid_dashboard_token()
  test_dashboard_auth.rs        — tests for dashboard auth
```

### 3.2 Dashboard auth initiate

- **Endpoint**: `POST /bodhi/v1/auth/dashboard/initiate`
- **Guard**: `setting_service.is_multi_tenant()` must be true; else return error
- Read `BODHI_MULTITENANT_CLIENT_ID` + `BODHI_MULTITENANT_CLIENT_SECRET` from settings
- Generate PKCE challenge + random state
- Store `oauth_state`, `pkce_verifier`, `callback_url` in session (same as resource initiate)
- Construct Keycloak authorization URL with dashboard client
- Return redirect URL (HTTP 201)

### 3.3 Dashboard auth callback

- **Endpoint**: `POST /bodhi/v1/auth/dashboard/callback`
- Validate state from session
- Exchange code using dashboard client credentials
- Store tokens: `dashboard:access_token`, `dashboard:refresh_token` (NOTE: `dashboard:id_token` constant exists but is NOT stored — deferred for OIDC logout)
- Redirect to `/ui/login`

### 3.4 Shared code exchange utility

- **File**: `crates/routes_app/src/auth/routes_auth.rs` (or shared module)
- Extract:
  ```rust
  async fn exchange_and_store_tokens(
    session: &Session,
    auth_service: &dyn AuthService,
    code: AuthorizationCode,
    client_id: &str,
    client_secret: &str,
    redirect_uri: RedirectUrl,
    pkce_verifier: PkceCodeVerifier,
    key_prefix: &str,  // "dashboard:" or "{azp}:"
  ) -> Result<(String, String), ...>
  ```
- Dashboard callback calls with `("dashboard:", dashboard_creds)`
- Resource callback calls with `("{azp}:", tenant_creds)` (updated in Phase 5)

### 3.5 Dashboard token refresh helper

- **File**: `crates/routes_app/src/tenants/dashboard_helpers.rs`
  ```rust
  pub async fn ensure_valid_dashboard_token(
    session: &Session,
    auth_service: &dyn AuthService,
    settings: &dyn SettingService,
  ) -> Result<String, ApiError>
  ```
- Read `dashboard:access_token`, check JWT exp
- If expired: refresh using `dashboard:refresh_token` + dashboard client creds
- Update session with new tokens
- Return valid access token
- On refresh failure: return error (frontend redirects to dashboard re-login)

### 3.6 Route registration

- **File**: `crates/routes_app/src/routes.rs`
- Add dashboard auth routes to public/optional-auth group (no resource auth needed)
- Dashboard endpoints are session-based, not token-based

### 3.7 Session key constants

- **File**: `crates/routes_app/src/tenants/mod.rs` (or constants)
  - `DASHBOARD_ACCESS_TOKEN = "dashboard:access_token"`
  - `DASHBOARD_REFRESH_TOKEN = "dashboard:refresh_token"`
  - `DASHBOARD_ID_TOKEN = "dashboard:id_token"`

### Gate check
```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
```

---

## Phase 4: Tenant Management Routes (routes_app)

**Sub-agent scope**: New routes for tenant CRUD + switching

### 4.1 GET /bodhi/v1/tenants

- **File**: `crates/routes_app/src/tenants/routes_tenants.rs`
- Requires dashboard token in session (call `ensure_valid_dashboard_token()`)
- Proxy to SPI: `auth_service.list_tenants(dashboard_token)`
- Enrich response per client:
  - `is_active: bool` — matches `session["active_client_id"]`
  - `logged_in: bool` — `{client_id}:access_token` exists in session and not expired
- Return enriched list
- SPI errors → OpenAI-compatible error format (use existing `ApiError` / `OpenAIApiError`)

### 4.2 POST /bodhi/v1/tenants

- Requires dashboard token in session
- Request body: `{ name: String, description: String }`
- Backend constructs: `redirect_uris = ["{public_server_url}/ui/auth/callback"]`
- Proxy to SPI: `auth_service.create_tenant(dashboard_token, name, description, redirect_uris)`
- On SPI success (201):
  - Extract `user_id` from dashboard JWT `sub` claim
  - Create tenant row: `tenant_service.create_tenant(client_id, client_secret, AppStatus::Ready, Some(user_id))`
  - If local creation fails: log critical, return success anyway (D52: accept orphans)
- Return `{ client_id }` — frontend auto-initiates resource login

### 4.3 POST /bodhi/v1/tenants/{client_id}/activate

- Validate `{client_id}:access_token` exists in session and not expired
- Set `session["active_client_id"] = client_id`
- Return 200

### 4.4 Response types

- **File**: `crates/routes_app/src/tenants/tenant_types.rs` (new)
  - `TenantListItem { client_id, name, description, is_active, logged_in }`
  - `TenantListResponse { tenants: Vec<TenantListItem> }`
  - `CreateTenantRequest { name: String, description: String }`
  - `CreateTenantResponse { client_id: String }`

### Gate check
```bash
cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"
```

---

## Phase 5: Unified Auth & Enhanced Endpoints (routes_app)

**Sub-agent scope**: Modify existing routes for unified standalone/multi-tenant flow

### 5.1 Enhanced auth_initiate

- **File**: `crates/routes_app/src/auth/routes_auth.rs`
- `client_id` always required in request body (both modes)
- Store `client_id` in session as `auth_client_id` (for callback to retrieve)
- Lookup tenant via `get_tenant_by_client_id(client_id)` — remove `get_standalone_app()` usage
- Standalone frontend sends `client_id` obtained from `/info` response

### 5.2 Enhanced auth_callback

- Use shared `exchange_and_store_tokens()` utility (from Phase 3.4)
- Read `auth_client_id` from session to lookup tenant (instead of `get_standalone_app()`)
- After exchange: extract `azp` from JWT, use as session key namespace
- Set `active_client_id` in session
- ResourceAdmin → Ready transition: call `tenant_service.set_client_ready(client_id, user_id)` (sets both status=Ready and created_by in one call)
- Redirect to `/ui/chat`

### 5.3 Enhanced /info

- **File**: `crates/routes_app/src/setup/routes_setup.rs`
- Move `/bodhi/v1/info` from public routes to optional_auth group in `routes.rs`
- Extend `AppInfo` struct:
  - `deployment: String` — `"standalone"` or `"multi_tenant"` (from `setting_service.deployment_mode()`)
  - `client_id: Option<String>` — active tenant's client_id
- Multi-tenant status logic (handler needs session access via `optional_auth_middleware`):
  - No dashboard token → `TenantSelection`
  - Dashboard token but no `active_client_id` → `TenantSelection`
  - `active_client_id` with valid resource token → `Ready`
- Standalone: existing logic (Setup → ResourceAdmin → Ready)

### 5.4 Enhanced /user/info

- **File**: `crates/routes_app/src/users/routes_users_info.rs`
- Extend `UserResponse` with dashboard session info:
  - Read `dashboard:access_token` directly from session (independent of AuthContext)
  - If dashboard token exists: include `has_dashboard_session: true` in response
  - Frontend uses this to distinguish "show login button" vs "show tenant selector"
- Exact response shape per D81 (decide during implementation)

### 5.5 auth_logout

- **File**: `crates/routes_app/src/auth/routes_auth.rs`
- `session.delete()` already clears everything — no change needed
- Verify it works with namespaced keys (it deletes entire session)

### Gate check
```bash
cargo test -p services -p server_core -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

---

## Phase 6: TECHDEBT & Documentation

**Sub-agent scope**: Non-code updates

### 6.1 TECHDEBT.md

- **File**: `crates/services/TECHDEBT.md` (create or append)
  - ConcurrencyService: use PostgreSQL advisory locks when DB backend is Postgres for cluster-wide distributed locking in multi-deployment
  - Session token lifecycle: encapsulate token refresh into session management layer — auto-refresh on access, retryable error on refresh failure. Replace ad-hoc `ensure_valid_dashboard_token()` with unified approach

### 6.2 Documentation updates

- Update `crates/routes_app/CLAUDE.md` if needed (new tenants/ module, session key format)
- Update `crates/services/CLAUDE.md` if needed (new AuthService methods, TenantService changes)

### Gate check
```bash
cargo test -p services -p server_core -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

---

## Critical Files Reference

### Services crate (Phase 1)
- `crates/services/src/db/sea_migrations/m20250101_000013_apps.rs` — tenants table migration
- `crates/services/src/tenants/tenant_entity.rs` — SeaORM entity
- `crates/services/src/tenants/tenant_objs.rs` — Tenant, AppStatus, TenantRow
- `crates/services/src/tenants/tenant_service.rs` — TenantService trait + impl
- `crates/services/src/tenants/tenant_repository.rs` — DB operations
- `crates/services/src/auth/auth_service.rs` — AuthService trait + KeycloakAuthService
- `crates/services/src/settings/constants.rs` — env var constants
- `crates/services/src/settings/setting_service.rs` — SettingService trait
- `crates/services/src/test_utils/fixtures.rs` — Tenant::test_default(), test_tenant_b()

### Routes_app crate (Phases 2-5)
- `crates/routes_app/src/middleware/auth/auth_middleware.rs` — session key constants, auth/optional middleware
- `crates/routes_app/src/middleware/token_service/token_service.rs` — token refresh with lock
- `crates/routes_app/src/auth/routes_auth.rs` — auth_initiate, auth_callback, auth_logout
- `crates/routes_app/src/setup/routes_setup.rs` — setup_show (/info), setup_create
- `crates/routes_app/src/users/routes_users_info.rs` — /user/info
- `crates/routes_app/src/routes.rs` — route composition + middleware layers
- `crates/routes_app/src/test_utils/` — test helpers, session setup, router builder

### Reusable utilities
- `setting_service.public_server_url()` — for redirect_uris construction
- `setting_service.is_multi_tenant()` / `deployment_mode()` — mode detection
- `setting_service.multitenant_client_id()` — dashboard client ID
- `extract_claims::<Claims>(token)` — JWT claim extraction
- `AppServiceStubBuilder` — test fixture composition
- `AuthContext::test_session()` / `.with_tenant_id()` — test auth context factories

---

## Testing Approach

**No real Keycloak required for this plan.** All testing uses mocks per existing crate conventions:

| Crate | Mock strategy | Tool |
|-------|--------------|------|
| services | `MockAuthService` (mockall) for trait tests; `mockito` for `KeycloakAuthService` HTTP tests (SPI proxy methods) | mockall, mockito |
| routes_app | `MockAuthService` injected via `AppServiceStubBuilder`; test JWTs created locally | mockall |
| middleware | Local test JWTs + `create_authenticated_session()` helpers; no Keycloak calls | local JWT |

**Real Keycloak with multi-tenant SPI** is deferred to:
- Frontend E2E tests (Playwright) — separate plan
- Manual integration testing — after frontend integration

---

## Verification Strategy

### Per-phase gate checks (see each phase above)

### Final end-to-end verification
```bash
# Full backend test suite
make test.backend

# Cross-crate regression
cargo test -p services -p server_core -p routes_app -p server_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```

### Manual verification (deferred to frontend integration)
- Dashboard login flow with test Keycloak
- Tenant creation + resource login
- Tenant switching
- /info response in standalone vs multi-tenant mode
