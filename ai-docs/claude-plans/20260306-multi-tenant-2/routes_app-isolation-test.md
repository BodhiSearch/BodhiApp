# Multi-Tenant Isolation Tests — routes_app

## Context

Services-layer repository isolation tests already verify cross-tenant and intra-tenant data isolation at the DB level. This plan adds the HTTP handler layer companion — verifying that when multiple tenants exist, HTTP requests authenticated as tenant A only see tenant A's resources, even though tenant B's data exists in the same database. This also introduces PostgreSQL support into the route-level test infrastructure (currently SQLite-only).

---

## Phase 1: Infrastructure — Postgres Support + Additive `with_tenant()` + `create_tenant_test()`

**Goal**: Make `with_tenant()` additive/idempotent, add `create_tenant_test()` to `TenantRepository` for test-only tenant creation with specified IDs, add Postgres-backed DB support for route tests, migrate `test_tokens_crud.rs` as validation.

### 1A. Make `with_tenant()` additive

**File**: `crates/services/src/test_utils/app.rs` (lines 398-412)

Current behavior: each `with_tenant()` call creates a NEW `DefaultTenantService` and overwrites `self.tenant_service`. Fix: check if `tenant_service` is already initialized — if so, reuse that instance; if not, create one. Then insert the tenant.

```rust
pub async fn with_tenant(&mut self, instance: Tenant) -> &mut Self {
    let db_service = self.get_db_service().await;
    // Reuse existing tenant_service or create a new one
    let svc = if let Some(Some(existing)) = &self.tenant_service {
        existing.clone()
    } else {
        let new_svc = Arc::new(DefaultTenantService::new(db_service));
        self.tenant_service = Some(Some(new_svc.clone()));
        new_svc
    };
    svc.create_tenant(&instance.client_id, &instance.client_secret, instance.status)
        .await
        .unwrap();
    self
}
```

All existing callers use `with_tenant()` exactly once — fully backward-compatible.

### 1B. Add `create_tenant_test()` to `TenantRepository` trait

**File**: `crates/services/src/tenants/tenant_repository.rs`

Add a test-only method gated behind `#[cfg(any(test, feature = "test-utils"))]` that accepts a `Tenant` struct and inserts it with the **specified ID** (unlike `create_tenant()` which auto-generates a ULID):

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TenantRepository: Send + Sync {
    // ... existing methods ...

    #[cfg(any(test, feature = "test-utils"))]
    async fn create_tenant_test(&self, tenant: &Tenant) -> Result<TenantRow, DbError>;
}
```

**Implementation on `DefaultDbService`**:

```rust
#[cfg(any(test, feature = "test-utils"))]
async fn create_tenant_test(&self, tenant: &Tenant) -> Result<TenantRow, DbError> {
    let now = self.time_service.utc_now();
    let (encrypted_secret, salt_secret, nonce_secret) =
        encrypt_api_key(&self.encryption_key, &tenant.client_secret)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let active = tenant_entity::ActiveModel {
        id: Set(tenant.id.clone()),  // USE THE PROVIDED ID
        client_id: Set(tenant.client_id.clone()),
        encrypted_client_secret: Set(Some(encrypted_secret)),
        salt_client_secret: Set(Some(salt_secret)),
        nonce_client_secret: Set(Some(nonce_secret)),
        app_status: Set(tenant.status.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    tenant_entity::Entity::insert(active).exec(&self.db).await.map_err(DbError::from)?;

    Ok(TenantRow {
        id: tenant.id.clone(),
        client_id: tenant.client_id.clone(),
        client_secret: tenant.client_secret.clone(),
        app_status: tenant.status.clone(),
        created_at: now,
        updated_at: now,
    })
}
```

**Why this approach**: `create_tenant()` auto-generates `ulid::Ulid::new()` and ignores the `Tenant.id` field. Tests need deterministic IDs (e.g., `TEST_TENANT_B_ID`) to match `AuthContext::with_tenant_id()`. Adding `create_tenant_test()` lets tests create tenants with specific IDs after the app service is built. It's gated behind `cfg(test)` / `feature = "test-utils"` so production code never sees it. Since `DbService: TenantRepository`, it's callable on `Arc<dyn DbService>`.

### 1C. Add `Tenant::test_tenant_b()` factory

**File**: `crates/services/src/test_utils/fixtures.rs`

Add alongside `Tenant::test_default()`:

```rust
pub fn test_tenant_b() -> Self {
    Tenant {
        id: TEST_TENANT_B_ID.to_string(),
        client_id: "test-client-b".to_string(),
        client_secret: "test-client-secret-b".to_string(),
        status: AppStatus::Ready,
    }
}
```

### 1D. Add `build_test_router_with_db()` to route test utils

**File**: `crates/routes_app/src/test_utils/router.rs`

New function that accepts `db_type` parameter. Uses `sea_context()` for Postgres, existing path for SQLite:

```rust
pub async fn build_test_router_with_db(
    db_type: &str,
) -> anyhow::Result<(Router, Arc<dyn AppService>, Arc<TempDir>)> {
    // If "postgres": create DbService via sea_context("postgres")
    // If "sqlite": use existing build_test_router() path
    // Pass db_service into AppServiceStubBuilder
    // Rest of setup identical to build_test_router()
}
```

**Key**: `sea_context()` is in `services::test_utils::sea` — already available via dev-dependency. Needs `INTEG_TEST_APP_DB_PG_URL` env var (set in CI, loaded from `.env.test` locally).

### 1E. Migrate `test_tokens_crud.rs` to dual DB

**File**: `crates/routes_app/src/tokens/test_tokens_crud.rs`

- Add `#[values("sqlite", "postgres")]` parameterization to existing tests
- Add `#[serial(pg_app)]` attribute
- Add `_setup_env: ()` fixture
- Replace `test_db_service` fixture with `sea_context(db_type).await` for DB creation
- Pass the `DbService` to `AppServiceStubBuilder::default().db_service(...)`

### Phase 1 Verification

```bash
cargo check -p services
cargo test -p services -- test_utils  # ensure with_tenant changes don't break
cargo check -p routes_app
cargo test -p routes_app -- test_tokens_crud  # both sqlite + postgres
```

---

## Phase 2: Tokens Isolation Tests (Template)

**Goal**: Establish the canonical isolation test pattern using tokens domain.

### 2A. Create test file

**File to create**: `crates/routes_app/src/tokens/test_tokens_isolation.rs`
**Register in**: `crates/routes_app/src/tokens/mod.rs`

```rust
#[cfg(test)]
#[path = "test_tokens_isolation.rs"]
mod test_tokens_isolation;
```

### 2B. Router pattern — custom, no middleware

Follow `test_tokens_crud.rs` pattern. Mount handlers directly on `Router::new()` without auth middleware. Use `.with_auth_context()` for injection.

**Multi-tenant setup**: Build app service with one tenant via `with_tenant()`, then create tenant B post-build via `create_tenant_test()`:

```rust
async fn isolation_router(db_type: &str) -> anyhow::Result<(Router, Arc<dyn AppService>)> {
    let ctx = sea_context(db_type).await;
    let db_svc: Arc<dyn DbService> = Arc::new(ctx.service);
    let mut builder = AppServiceStubBuilder::default();
    builder
        .db_service(db_svc)
        .with_tenant(Tenant::test_default()).await;
    let app_service: Arc<dyn AppService> = Arc::new(builder.build().await?);

    // Create tenant B with deterministic ID after app service is built
    app_service.db_service().create_tenant_test(&Tenant::test_tenant_b()).await?;

    let router = Router::new()
        .route("/api/tokens", get(tokens_index).post(tokens_create))
        .route("/api/tokens/{token_id}", put(tokens_update))
        .with_state(app_service.clone());

    Ok((router, app_service))
}
```

### 2C. Test functions

| Test | What it verifies |
|------|-----------------|
| `test_cross_tenant_token_list_isolation` | POST token as tenant A, POST as tenant B, GET list each — only own tokens |
| `test_cross_tenant_token_get_isolation` | GET token by ID from wrong tenant — 404 |
| `test_cross_tenant_token_update_isolation` | PUT token from wrong tenant — 404 |
| `test_intra_tenant_user_token_list_isolation` | Same tenant, different users — each sees only own tokens |
| `test_intra_tenant_user_token_update_isolation` | Same tenant, user B updates user A's token — 404 |

### 2D. Auth context construction

Uses hardcoded `TEST_TENANT_ID` and `TEST_TENANT_B_ID` constants (matching the IDs created via `Tenant::test_default()` and `create_tenant_test(&Tenant::test_tenant_b())`):

```rust
// Tenant A, User A (uses TEST_TENANT_ID from Tenant::test_default())
let auth_a = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin);
// test_session defaults to TEST_TENANT_ID, no .with_tenant_id() needed

// Tenant B, same user (cross-tenant)
let auth_b = AuthContext::test_session("user-a", "a@test.com", ResourceRole::Admin)
    .with_tenant_id(TEST_TENANT_B_ID);

// Tenant A, different user (intra-tenant)
let auth_a_user_b = AuthContext::test_session("user-b", "b@test.com", ResourceRole::Admin);
// Same tenant (TEST_TENANT_ID default), different user_id
```

### 2E. Assertion pattern

- List: assert `response.status() == 200`, deserialize `PaginatedTokenResponse`, assert `data.len()` and content
- Get/Update/Delete cross-boundary: assert `response.status() == 404` (NOT 403 — prevents ID enumeration)
- HTTP response only — no DB verification (services-layer tests cover that)

### Phase 2 Verification

```bash
cargo test -p routes_app -- test_tokens_isolation
```

---

## Phase 3: Handpicked Middleware Integration Tests

**Goal**: A small set of tests using `build_test_router_with_db()` (full middleware) to validate that auth middleware correctly resolves tenant from session cookies, API tokens, and external app tokens — and that isolation holds end-to-end.

### 3A. Create test file

**File to create**: `crates/routes_app/src/tokens/test_tokens_middleware_isolation.rs`
**Register in**: `crates/routes_app/src/tokens/mod.rs`

### 3B. Tests (handpicked, not exhaustive)

| Test | Auth method | What it validates |
|------|------------|-------------------|
| `test_session_auth_tenant_isolation` | Session cookie | Middleware resolves tenant from session -> handler sees correct tenant |
| `test_api_token_auth_tenant_isolation` | Bearer `bodhiapp_<random>.<client_id>` | Token's `client_id` suffix resolves to correct tenant |

**Note**: API token auth already supports multi-tenant (resolves tenant from `client_id` in token). Session auth currently calls `get_standalone_app()` which calls `get_tenant()` — this returns `Err(MultipleTenant)` when >1 tenant exists (`tenant_repository.rs:57`).

**Risk**: Session-path middleware integration tests may need `get_tenant()` / `get_standalone_app()` fixed first, or we scope them to API token auth only (which uses `get_tenant_by_client_id()`).

### Phase 3 Verification

```bash
cargo test -p routes_app -- test_tokens_middleware_isolation
```

---

## Phase 4: Tenant+User Scoped Domains

**Goal**: Add isolation tests for all tenant+user scoped domains, following the tokens template.

### 4A. Domains and files

| Domain | New test file | Register in |
|--------|--------------|-------------|
| MCPs | `crates/routes_app/src/mcps/test_mcps_isolation.rs` | `mcps/mod.rs` |
| Toolsets | `crates/routes_app/src/toolsets/test_toolsets_isolation.rs` | `toolsets/mod.rs` |
| API Models | `crates/routes_app/src/api_models/test_api_models_isolation.rs` | `api_models/mod.rs` |
| User Aliases | `crates/routes_app/src/models/test_user_aliases_isolation.rs` | `models/mod.rs` |

### 4B. Domain-specific notes

**MCPs**: FK dependency — MCP instances require `mcp_server_id`. Setup sequence:
1. POST MCP server (tenant-scoped, no user_id)
2. POST MCP instance referencing the server
Use helpers from `crates/routes_app/src/test_utils/mcp.rs` (`setup_mcp_server_in_db()`)

**Toolsets**: Straightforward CRUD, no FK dependencies.

**API Models**: Straightforward CRUD, no FK dependencies.

**User Aliases**: Uses `GET /bodhi/v1/models` (list), `POST` (create), `DELETE` (destroy). No update endpoint. May need `HubService` setup for model file resolution.

### 4C. Each domain gets

- `test_cross_tenant_<domain>_list_isolation`
- `test_cross_tenant_<domain>_get_isolation` (where GET by ID exists)
- `test_cross_tenant_<domain>_update_isolation` (where PUT exists)
- `test_cross_tenant_<domain>_delete_isolation` (where DELETE exists)
- `test_intra_tenant_user_<domain>_list_isolation`

### Phase 4 Verification

```bash
cargo test -p routes_app -- test_mcps_isolation test_toolsets_isolation test_api_models_isolation test_user_aliases_isolation
```

---

## Phase 5: Tenant-Only Scoped Domains

**Goal**: Cross-tenant isolation tests for domains without user-scoping.

### 5A. Domains and files

| Domain | New test file | Register in |
|--------|--------------|-------------|
| MCP Servers | `crates/routes_app/src/mcps/test_mcp_servers_isolation.rs` | `mcps/mod.rs` |
| Downloads | `crates/routes_app/src/models/test_downloads_isolation.rs` | `models/mod.rs` |
| User Access Requests | `crates/routes_app/src/users/test_access_requests_isolation.rs` | `users/mod.rs` |

### 5B. Domain-specific notes

**MCP Servers**: No FK deps (root entity). Tenant-scoped only. CRUD: GET list, GET by ID, POST create, PUT update, DELETE.

**Downloads**: POST creates a download and spawns async `tokio::spawn` task. For isolation tests, focus on GET list and GET by ID isolation. Seed data via `DbService::create_download_request()` directly (avoid async spawn complexity).

**User Access Requests**: Mixed auth — some endpoints session-only, some manager/admin. GET list is the primary isolation target. Seed via POST or direct DB insert.

### 5C. Each domain gets

- `test_cross_tenant_<domain>_list_isolation`
- `test_cross_tenant_<domain>_get_isolation` (where applicable)
- `test_cross_tenant_<domain>_write_isolation` (where applicable)

No intra-tenant user tests (these are tenant-scoped only).

### Phase 5 Verification

```bash
cargo test -p routes_app -- test_mcp_servers_isolation test_downloads_isolation test_access_requests_isolation
```

---

## Execution Protocol Per Phase

Each phase follows this sequence (executed by sub-agent):

1. `cargo check -p services -p routes_app` — compile check
2. `cargo test -p routes_app --no-run` — test compile
3. `cargo test -p routes_app -- <new_test_names>` — run new tests
4. Fix any failures, iterate
5. `cargo test -p routes_app` — full regression
6. Local commit

---

## Key Files Reference

### To Modify
- `crates/services/src/tenants/tenant_repository.rs` — add `create_tenant_test()` to `TenantRepository` trait (cfg-gated) + impl on `DefaultDbService`
- `crates/services/src/test_utils/app.rs` — `with_tenant()` additive (reuse existing tenant_service)
- `crates/services/src/test_utils/fixtures.rs` — `Tenant::test_tenant_b()`
- `crates/routes_app/src/test_utils/router.rs` — `build_test_router_with_db(db_type)`
- `crates/routes_app/src/tokens/test_tokens_crud.rs` — migrate to dual SQLite/Postgres
- `crates/routes_app/src/tokens/mod.rs` — register new test modules
- Domain `mod.rs` files — register isolation test modules

### To Create
- `crates/routes_app/src/tokens/test_tokens_isolation.rs`
- `crates/routes_app/src/tokens/test_tokens_middleware_isolation.rs`
- `crates/routes_app/src/mcps/test_mcps_isolation.rs`
- `crates/routes_app/src/mcps/test_mcp_servers_isolation.rs`
- `crates/routes_app/src/toolsets/test_toolsets_isolation.rs`
- `crates/routes_app/src/api_models/test_api_models_isolation.rs`
- `crates/routes_app/src/models/test_user_aliases_isolation.rs`
- `crates/routes_app/src/models/test_downloads_isolation.rs`
- `crates/routes_app/src/users/test_access_requests_isolation.rs`

### To Reuse (no changes)
- `sea_context()` — `crates/services/src/test_utils/sea.rs`
- `AuthContext::test_session()` + `.with_tenant_id()` — `crates/services/src/test_utils/auth_context.rs`
- `.with_auth_context()` — `crates/routes_app/src/test_utils/auth_context.rs`
- `ResponseTestExt::json()` — `crates/server_core/src/test_utils/http.rs`
- MCP test helpers — `crates/routes_app/src/test_utils/mcp.rs`
- Test constants (`TEST_TENANT_ID`, `TEST_TENANT_B_ID`, `TEST_USER_ID`, `TEST_TENANT_A_USER_B_ID`) — `crates/services/src/test_utils/db.rs`

### Critical Constraints
- `create_tenant()` auto-generates ULID — use `create_tenant_test()` post-build for deterministic IDs
- `get_tenant()` errors with `MultipleTenant` when >1 tenant row — isolation tests must NOT trigger this (use custom router without middleware)
- `#[serial(pg_app)]` required on all Postgres-parameterized tests
- Real DB only — no mock DBs in route-level tests
- `create_tenant_test()` is `#[cfg(any(test, feature = "test-utils"))]` — never available in production
