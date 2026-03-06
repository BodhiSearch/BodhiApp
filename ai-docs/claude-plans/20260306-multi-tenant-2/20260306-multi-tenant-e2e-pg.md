# Fix PostgreSQL E2E Test Startup Constraint Error

## Context

PostgreSQL E2E tests fail on server startup with a UNIQUE constraint violation on `tenants.client_id`. The `update_with_option()` function (in `lib_bodhiserver`) calls `create_tenant()` which always does a plain INSERT. PostgreSQL retains data across test runs (unlike SQLite which uses temp dirs that get wiped), so the same `client_id` triggers the constraint error on subsequent runs.

Additionally, `update_with_option()` is only used by `lib_bodhiserver_napi` (E2E test infra), not by the production desktop app (`bodhi/src-tauri`). It should be moved out of the production library.

## Changes

### Step 1: Make `create_tenant_test` idempotent (services crate)

**File:** `crates/services/src/tenants/tenant_repository.rs` (line 182)

Add upsert semantics: check `get_tenant_by_client_id()` first, return existing row if found, otherwise INSERT.

```rust
#[cfg(any(test, feature = "test-utils"))]
async fn create_tenant_test(&self, tenant: &Tenant) -> Result<TenantRow, DbError> {
    // Upsert: return existing tenant if client_id already exists
    if let Some(existing) = self.get_tenant_by_client_id(&tenant.client_id).await? {
        return Ok(existing);
    }
    // ... existing INSERT logic unchanged ...
}
```

**Verify:** `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"`

### Step 2: Remove `update_with_option` from lib_bodhiserver

**File:** `crates/lib_bodhiserver/src/app_service_builder.rs`

Delete lines 374-389 (the `update_with_option` function). The `build_app_service` function (lines 368-372) stays.

**Verify:** `cargo check -p lib_bodhiserver`

### Step 3: Enable `test-utils` on regular dependency in lib_bodhiserver_napi

**File:** `crates/lib_bodhiserver_napi/Cargo.toml` (line 10)

Change:
```toml
lib_bodhiserver = { workspace = true }
```
To:
```toml
lib_bodhiserver = { workspace = true, features = ["test-utils"] }
```

This is appropriate because lib_bodhiserver_napi is test infrastructure (E2E/npm), not a production server. Enables `create_tenant_test` in `server.rs`.

### Step 4: Move and fix tenant logic in lib_bodhiserver_napi

**File:** `crates/lib_bodhiserver_napi/src/server.rs`

1. Remove `update_with_option` from imports
2. Add local `ensure_tenant` function that calls `db_service().create_tenant_test()`:

```rust
async fn ensure_tenant(
    service: &Arc<dyn AppService>,
    instance: Option<&Tenant>,
) -> std::result::Result<(), BootstrapError> {
    if let Some(instance) = instance {
        service.db_service().create_tenant_test(instance).await?;
    }
    Ok(())
}
```

3. Replace `update_with_option(...)` call (line 179) with `ensure_tenant(...)`

**Note:** `create_tenant_test` takes `&Tenant` directly (with caller-specified ID), and now has upsert semantics from Step 1. `DbService` extends `TenantRepository` (line 22 of `db/service.rs`). `BootstrapError` has `From<DbError>` conversion.

**Verify:** `cargo check -p lib_bodhiserver_napi`

### Step 5: Add postgres webServer to playwright config

**File:** `crates/lib_bodhiserver_napi/playwright.config.mjs`

Add to the `webServer` array (after the sqlite entry at line 128):

```javascript
{
    command: 'node tests-js/scripts/start-shared-server.mjs --port 41135 --db-type postgres',
    url: 'http://localhost:41135/ping',
    reuseExistingServer: false,
    timeout: 60000,
},
```

Port 41135 matches `db-config.mjs` postgres config.

### Step 6: Add PostgreSQL connectivity check

**File:** `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs`

Before server creation, when `dbType === 'postgres'`, verify both PostgreSQL containers are reachable via TCP. Fail fast with clear error message pointing to `docker compose -f docker/docker-compose.test.yml up -d`.

Use Node.js `net.createConnection` to check:
- App DB: `localhost:64320`
- Session DB: `localhost:54320`

## Key Files

| File | Change |
|------|--------|
| `crates/services/src/tenants/tenant_repository.rs` | Add upsert to `create_tenant_test` |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | Remove `update_with_option` |
| `crates/lib_bodhiserver_napi/Cargo.toml` | Enable `test-utils` on regular dep |
| `crates/lib_bodhiserver_napi/src/server.rs` | Add local `ensure_tenant`, replace import |
| `crates/lib_bodhiserver_napi/playwright.config.mjs` | Add postgres webServer entry |
| `crates/lib_bodhiserver_napi/tests-js/scripts/start-shared-server.mjs` | Add postgres connectivity check |

## Reused Functions

- `TenantRepository::get_tenant_by_client_id()` — `crates/services/src/tenants/tenant_repository.rs:77`
- `TenantRepository::create_tenant_test()` — `crates/services/src/tenants/tenant_repository.rs:182`
- `DbService` extends `TenantRepository` — `crates/services/src/db/service.rs:22`
- `AppService::db_service()` — returns `Arc<dyn DbService>`
- `getDbConfig('postgres')` — `crates/lib_bodhiserver_napi/tests-js/utils/db-config.mjs`

## Verification

1. `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"` — services tests pass
2. `cargo check -p lib_bodhiserver` — production crate compiles without `update_with_option`
3. `cargo check -p lib_bodhiserver_napi` — NAPI crate compiles with new `ensure_tenant`
4. `docker compose -f docker/docker-compose.test.yml up -d` — start postgres containers
5. `cd crates/lib_bodhiserver_napi && npx playwright test --project postgres` — postgres E2E tests pass
6. `cd crates/lib_bodhiserver_napi && npx playwright test --project sqlite` — sqlite E2E tests still pass
