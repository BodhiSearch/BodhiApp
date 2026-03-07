# Multi-Tenant Integration Tests — Implementation Record

> **Created**: 2026-03-08
> **Completed**: 2026-03-08
> **Status**: COMPLETED (all 4 phases)
> **Scope**: Infrastructure changes + critical-path integration tests at routes_app (oneshot) and server_app (multi-turn) levels
> **Kickoff doc**: `ai-docs/claude-plans/20260306-multi-tenant-2/kickoff-integ-test-multi-tenant.md`

---

## Implementation Summary

All 4 phases completed successfully. Infrastructure (services + routes_app dev endpoints) and integration tests at two levels (routes_app oneshot, server_app live TCP).

**Final test counts**:
- services: 832 passed, 0 failed
- routes_app: 656 passed (647 unit + 2 live auth + 7 live multi-tenant), 0 failed
- server_app: 8 unit passed, 3 integration tests compile-verified (need real Keycloak)
- All downstream crates compile: lib_bodhiserver, lib_bodhiserver_napi

**Key deviations from plan**:
- `forward_spi_request` uses owned `String` params (not `&str`) due to mockall lifetime incompatibility
- `DefaultDbService` uses builder pattern `.with_env_type()` instead of constructor param change (avoids breaking existing callers)
- D99 superseded: `ensure_valid_dashboard_token` now takes `time_service: &dyn TimeService` (no longer uses `SystemTime`)

---

## Context

Multi-tenant backend features were implemented in commits `6a7d879..04788eb`. Before these can be considered stable, we need integration tests that exercise the actual HTTP endpoints against a real Keycloak instance. Several infrastructure prerequisites are needed first: DB cleanup methods, a generic KC SPI proxy, TimeService fix, and new dev endpoints for test support.

---

## Execution Strategy

**Main agent** implements Phase 1+2 (infrastructure), then spawns **sequential sub-agents** for Phase 3 and 4. Each phase produces a summary passed as context to the next.

| Step | Executor | Scope | Gate Check | Status |
|------|----------|-------|------------|--------|
| 1 | Main agent | Phase 1 (services) + Phase 2 (dev endpoints) | services: 832 passed, routes_app: 649 passed, lib_bodhiserver + server_app compile OK | COMPLETED |
| 2 | Sub-agent (sequential) | Phase 3 (routes_app integration tests) | routes_app: 656 passed (647 unit + 2 live auth + 7 live multi-tenant) | COMPLETED |
| 3 | Sub-agent (sequential) | Phase 4 (server_app integration tests) | 3 integration tests compile-verified (need real Keycloak for runtime) | COMPLETED |

**Context chain**: Each sub-agent receives the full plan + summary of what prior phases actually implemented (file changes, function signatures, any deviations from plan). This ensures downstream agents have accurate implementation context, not just plan assumptions.

---

## Phase 1: Infrastructure Changes (services crate)

### 1A: DefaultDbService EnvType Guard + reset_tenants

**Goal**: Add production guard to reset methods, add `reset_tenants()` method.

**Files**:
- `crates/services/src/db/db_core.rs` — Add `async fn reset_tenants(&self) -> Result<(), DbError>` to `DbCore` trait
- `crates/services/src/db/default_service.rs` — Add `env_type: EnvType` field (defaults to `Development`) with `.with_env_type()` builder method (D104: avoids breaking existing callers). Add production guard to `reset_all_tables()`. Implement `reset_tenants()` (PostgreSQL: `TRUNCATE TABLE tenants CASCADE`, SQLite: `DELETE FROM tenants`)
- `crates/services/src/db/error.rs` — Add `ProductionGuard(String)` variant to `DbError` with `ErrorType::Forbidden`

**Call-site updates** (add `env_type` param to `DefaultDbService::new()`):
- `crates/services/src/test_utils/db.rs` — `test_db_service_with_temp_dir()`, pass `EnvType::Development`
- `crates/services/src/test_utils/sea.rs` — both sqlite/postgres branches, pass `EnvType::Development`
- `crates/lib_bodhiserver/src/app_service_builder.rs` — `build_db_service()`, derive `EnvType` from existing `is_production` bool
- `crates/server_app/tests/utils/live_server_utils.rs` — `setup_minimal_app_service()` + `setup_multitenant_app_service()`, pass `EnvType::Development`

**Delegate in TestDbService** (`crates/services/src/test_utils/db.rs`):
```rust
async fn reset_tenants(&self) -> Result<(), DbError> {
  self.inner.reset_tenants().await.tap(|_| self.notify("reset_tenants"))
}
```

Also add `reset_tenants` to the `MockDbService` expectations block in the same file.

**Verify**: `cargo check -p services && cargo test -p services --lib`

### 1B: AuthService Generic SPI Proxy Method

**Goal**: Add a generic method to proxy arbitrary HTTP requests to the KC Bodhi SPI.

**File**: Find `AuthService` trait definition (likely `crates/services/src/auth/auth_service.rs` or similar)

Add to trait (D103: uses owned `String` params, not `&str`, due to mockall lifetime incompatibility):
```rust
async fn forward_spi_request(
  &self,
  method: String,
  endpoint: String,
  authorization: Option<String>,
  body: Option<serde_json::Value>,
) -> Result<(u16, serde_json::Value), AuthServiceError>;
```

Implementation on `KeycloakAuthService` (or `DefaultAuthService`):
- Build URL: `{auth_url}/realms/{realm}/bodhi/{endpoint}` using stored `auth_url` and `realm`
- Dispatch using reqwest based on `method` (GET/POST/PUT/DELETE/PATCH)
- Attach `Authorization: Bearer {token}` if `authorization` is Some
- Attach JSON body if provided
- Return `(status_code, response_json)` — parse body as JSON, fallback to `{"raw": body_text}` if not JSON
- On HTTP >= 400, log error but still return (let caller decide how to handle)

MockAuthService auto-generates the mock via `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]`.

**Verify**: `cargo check -p services && cargo test -p services --lib`

### 1C: Fix ensure_valid_dashboard_token to use TimeService

**File**: `crates/routes_app/src/tenants/dashboard_helpers.rs`

Change function signature to accept `time_service: &dyn TimeService`:
```rust
pub async fn ensure_valid_dashboard_token(
  session: &Session,
  auth_service: &dyn AuthService,
  setting_service: &dyn SettingService,
  time_service: &dyn TimeService,    // NEW
) -> Result<String, DashboardAuthRouteError> {
```

Replace lines 25-28 (`SystemTime::now()`) with:
```rust
let now = time_service.utc_now().timestamp() as u64;
```

Remove `use std::time::{SystemTime, UNIX_EPOCH};`, add `use services::TimeService;`.

**Update all callers** to pass `time_service`:
- `crates/routes_app/src/tenants/routes_tenants.rs` — `tenants_index()` and `tenants_create()` pass `auth_scope.time_service().as_ref()`
- `crates/routes_app/src/setup/routes_setup.rs` — `resolve_multi_tenant_status()` if it calls this helper
- New dev handlers (Phase 2)

Also update any unit tests in `crates/routes_app/src/tenants/test_dashboard_auth.rs` that call this function.

**Verify**: `cargo check -p routes_app && cargo test -p routes_app`

---

## Phase 2: New Dev Endpoints (routes_app crate)

### 2A: POST /dev/clients/{client_id}/dag

**Purpose**: Enable Direct Access Grants on a KC client (via SPI) and return client credentials from local DB.

**Files**:
- `crates/routes_app/src/shared/openapi.rs` — Add `pub const ENDPOINT_DEV_CLIENTS_DAG: &str = "/dev/clients/{client_id}/dag";`
- `crates/routes_app/src/routes_dev.rs` — New handler + error variants

**Handler** `dev_clients_dag_handler(auth_scope: AuthScope, session: Session, Path(client_id): Path<String>)`:
1. Check `auth_scope.settings().is_multi_tenant().await` → error if false
2. Read dashboard token: `ensure_valid_dashboard_token(&session, auth_scope.auth_service().as_ref(), auth_scope.settings().as_ref(), auth_scope.time_service().as_ref())`
3. Call KC SPI: `auth_scope.auth_service().forward_spi_request("POST", &format!("test/clients/{client_id}/dag"), Some(&dashboard_token), None)`
4. Check status >= 400 → return SPI error
5. Look up local tenant: `auth_scope.tenant().get_tenant_by_client_id(&client_id)` → returns `Tenant` with decrypted `client_secret`
6. Return `200 { "client_id": "...", "client_secret": "..." }`

**Error variants** to add to `DevError`:
```rust
#[error(transparent)]
AuthServiceError(#[from] AuthServiceError),
#[error(transparent)]
DashboardAuthRouteError(#[from] DashboardAuthRouteError),
#[error("Not in multi-tenant mode")]
#[error_meta(error_type = ErrorType::InvalidAppState)]
NotMultiTenant,
#[error("Tenant '{0}' not found in local database")]
#[error_meta(error_type = ErrorType::NotFound)]
TenantNotFoundLocal(String),
#[error("KC SPI request failed (status {status})")]
#[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
SpiRequestFailed { status: u16, body: String },
```

### 2B: DELETE /dev/tenants/cleanup

**Purpose**: Clean up KC tenants for the authenticated user + truncate local tenants table.

**Handler** `dev_tenants_cleanup_handler(auth_scope: AuthScope, session: Session)`:
1. Check multi-tenant mode
2. Read dashboard token from session via `ensure_valid_dashboard_token()`
3. Call KC SPI: `auth_scope.auth_service().forward_spi_request("DELETE", "test/tenants/cleanup", Some(&dashboard_token), None)`
4. Check status >= 400 → return SPI error
5. Truncate local tenants: `auth_scope.db().reset_tenants().await`
6. Return `200` with KC cleanup response JSON

### 2C: Route Registration

**File**: `crates/routes_app/src/routes.rs`

In the `!is_production()` block (lines 114-121), add:
```rust
.route(ENDPOINT_DEV_CLIENTS_DAG, post(dev_clients_dag_handler))
.route(ENDPOINT_DEV_TENANTS_CLEANUP, delete(dev_tenants_cleanup_handler))
```

Add imports for `delete` from `axum::routing`, new handlers, and new endpoint constants.

**Verify**: `cargo check -p routes_app && cargo test -p routes_app`

---

## Phase 3: Integration Tests — routes_app (Oneshot)

### File: `crates/routes_app/tests/test_live_multi_tenant.rs`

Uses real Keycloak tokens via `AuthServerTestClient` but exercises routes via `tower::oneshot()`.

**Setup infrastructure**:
- Fixture loading `INTEG_TEST_MULTI_TENANT_CLIENT_ID` / `SECRET` from `.env.test`
- `create_multi_tenant_router()` helper using real `KeycloakAuthService` + `SettingServiceStub` with `BODHI_DEPLOYMENT=multi-tenant`
- Session injection via `session_service.get_session_store().save(&record)` with dashboard token keys

**Test scenarios** (critical paths):

| # | Test | What it verifies |
|---|------|-----------------|
| 1 | `test_info_multi_tenant_no_session` | GET `/info` → `tenant_selection`, `deployment: "multi_tenant"` |
| 2 | `test_info_multi_tenant_with_dashboard_and_active_tenant` | Inject dashboard + resource tokens → GET `/info` → `ready` with `client_id` |
| 3 | `test_dashboard_auth_initiate_standalone_rejected` | POST `/auth/dashboard/initiate` in standalone mode → `NotMultiTenant` error |
| 4 | `test_tenants_activate_success` | Inject resource token → POST `/tenants/{client_id}/activate` → 200 |
| 5 | `test_tenants_activate_not_logged_in` | No resource token → POST `/tenants/{client_id}/activate` → `TenantNotLoggedIn` |
| 6 | `test_user_info_has_dashboard_session` | Inject dashboard token → GET `/user/info` → `has_dashboard_session: true` |
| 7 | `test_user_info_no_dashboard_session` | No dashboard token → GET `/user/info` → `has_dashboard_session: false` |

**Session injection pattern** (from existing `test_dashboard_auth.rs`):
```rust
let mut record = Record {
  id: Id::default(),
  data: hashmap! {
    "dashboard:access_token".to_string() => Value::String(token.clone()),
    "active_client_id".to_string() => Value::String(client_id.clone()),
    "{client_id}:access_token".to_string() => Value::String(resource_token.clone()),
  },
  expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
};
session_store.save(&mut record).await?;
```

Requests through full router must include `Sec-Fetch-Site: same-origin` and `Host: localhost:1135` for auth middleware.

---

## Phase 4: Integration Tests — server_app (Multi-Turn Live)

### File: `crates/server_app/tests/test_live_multi_tenant.rs`

Real TCP server on port 51135, real Keycloak, `#[serial_test::serial(live)]`.

**Setup infrastructure** in `crates/server_app/tests/utils/live_server_utils.rs`:

`setup_multi_tenant_app_service(temp_dir)`:
- Mirror `setup_minimal_app_service()` but set `BODHI_DEPLOYMENT=multi-tenant`
- Set `BODHI_MULTITENANT_CLIENT_ID` / `BODHI_MULTITENANT_CLIENT_SECRET` from `INTEG_TEST_MULTI_TENANT_*` env vars
- Do NOT register a standalone tenant (multi-tenant starts clean)

`create_dashboard_session(app_service)`:
- Get dashboard token via `AuthServerTestClient::get_user_token(multitenant_client_id, secret, username, password, None)`
- Inject into session store under `dashboard:access_token`, `dashboard:refresh_token`
- Return session cookie string

**Test scenarios** (critical paths):

### test_multi_tenant_full_flow
The complete end-to-end scenario:
```
1. GET /info (no cookie) → tenant_selection
2. Inject dashboard token into session
3. DELETE /dev/tenants/cleanup (clean slate)
4. POST /tenants {name, description} → 201 {client_id}
5. POST /dev/clients/{client_id}/dag → 200 {client_id, client_secret}
6. Get resource token via direct grant (client_id + secret + username + password)
7. Inject resource token into session under access_token:{client_id}
8. POST /tenants/{client_id}/activate → 200
9. GET /info → ready, client_id matches
10. GET /user/info → has_dashboard_session: true
11. Cleanup: DELETE /dev/tenants/cleanup
```

### test_info_state_progression
```
1. No session → tenant_selection
2. Dashboard session only, no tenants → setup or tenant_selection
3. Dashboard + activated tenant → ready
```

### test_standalone_rejects_dashboard_auth
Start standalone server → POST `/auth/dashboard/initiate` → `NotMultiTenant` error.

---

## Gate Checks & Results (per-phase)

### After Phase 1+2 (main agent) — PASSED
- services: 832 passed, 0 failed
- routes_app: 649 passed, 0 failed
- lib_bodhiserver: compiles OK
- server_app: compiles OK

### After Phase 3 (sub-agent) — PASSED
- routes_app: 656 passed (647 unit + 2 live auth + 7 live multi-tenant), 0 failed

### After Phase 4 (sub-agent) — COMPILE-VERIFIED
- server_app: 8 unit tests passed, 3 integration tests compile-verified
- Integration tests require real Keycloak with `.env.test` credentials to run

### Final validation
- All downstream crates compile: lib_bodhiserver, lib_bodhiserver_napi

---

## Key Decisions (from interview)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| SPI availability | SPI deployed on dev KC | Can test tenant CRUD end-to-end |
| Dashboard client DAG | Enabled | Password grant works for dashboard tokens |
| KC cleanup approach | `user_email` for setup, `client_ids` for teardown | Clean slate + surgical cleanup |
| Env isolation (routes_app) | `SettingServiceStub` programmatic config | No process-global env var conflicts |
| Resource login method | Direct grant + session injection | No browser flows in integration tests |
| Token refresh testing | Skip (fix TimeService, trust unit tests) | `ensure_valid_dashboard_token` uses TimeService after fix |
| EnvType guard layer | Service level (DefaultDbService) | Fail-safe regardless of caller |
| AuthService proxy | On AuthService trait | Clean single interface, MockAuthService auto-generated |
| KC SPI DAG endpoint | User implements separately in keycloak-bodhi-ext | `POST /test/clients/{client_id}/dag` → 200 empty body |
| KC SPI cleanup endpoint | `DELETE /test/tenants/cleanup` with Bearer token | Deletes user's tenants for their dashboard client |
| Local DB cleanup | `reset_tenants()` truncates entire tenants table | Dev endpoint, simpler than per-user delete |

---

## Critical Files Reference

| File | Role |
|------|------|
| `crates/services/src/db/default_service.rs` | EnvType guard + reset_tenants impl |
| `crates/services/src/db/db_core.rs` | DbCore trait (add reset_tenants) |
| `crates/services/src/auth/auth_service.rs` | AuthService trait (add forward_spi_request) |
| `crates/routes_app/src/tenants/dashboard_helpers.rs` | TimeService fix |
| `crates/routes_app/src/routes_dev.rs` | New dev endpoint handlers |
| `crates/routes_app/src/routes.rs` | Route registration (lines 114-121) |
| `crates/routes_app/src/shared/openapi.rs` | Endpoint constants |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | DefaultDbService::new() call site |
| `crates/server_app/tests/utils/live_server_utils.rs` | Multi-tenant app service + session helpers |
| `crates/routes_app/tests/test_live_multi_tenant.rs` | NEW: oneshot integration tests |
| `crates/server_app/tests/test_live_multi_tenant.rs` | NEW: multi-turn live tests |
| `crates/routes_app/src/tenants/test_dashboard_auth.rs` | Reference: build_multitenant_app_service pattern |
| `crates/routes_app/tests/test_live_auth_middleware.rs` | Reference: AuthServerTestClient + fixture pattern |
| `crates/services/src/app_service/auth_scoped.rs` | AuthScopedAppService accessors: `.tenant()`, `.auth_service()`, `.settings()`, `.db()`, `.sessions()`, `.time_service()` |
