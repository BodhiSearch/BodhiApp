# Plan: AppInstanceService Post-Implementation Feedback Fixes

## Status: COMPLETED ✅ (2026-02-24)

All layers implemented and verified. `make test.backend` passed with 0 failures.

---

## Context

The AppRegInfo-to-database migration (`.cursor/plans/appreginfo_to_database_76621237.plan.md`) is complete. This plan addresses code review feedback covering error propagation, naming conventions, test utility simplification, and data integrity validation.

## Changes by Crate (upstream → downstream)

---

### 1. `services` crate ✅

#### 1a. Rename DB module file ✅
- **Renamed** `crates/services/src/db/service_app_instance.rs` → `crates/services/src/db/repository_app_instance.rs`
- **Updated** `crates/services/src/db/mod.rs`: `mod service_app_instance` → `mod repository_app_instance`

#### 1b. Migration: remove DEFAULT '' from scope ✅
- **Edited** `crates/services/migrations/0014_apps.up.sql`: `scope TEXT NOT NULL DEFAULT ''` → `scope TEXT NOT NULL`

#### 1c. Add DbError::MultipleAppInstance variant ✅
- **File**: `crates/services/src/db/error.rs`
- Added variant:
  ```rust
  #[error("Multiple application instances found, expected at most one.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MultipleAppInstance,
  ```

#### 1d. Add count validation in repository ✅
- **File**: `crates/services/src/db/repository_app_instance.rs`
- `get_app_instance` implementation:
  1. Query `SELECT COUNT(*) FROM apps` first
  2. If count > 1, return `Err(DbError::MultipleAppInstance)`
  3. If count == 0, return `Ok(None)`
  4. If count == 1, query and return the row (no `LIMIT 1`)

#### 1e. Add AppInstanceError::MultipleAppInstance variant ✅
- **File**: `crates/services/src/app_instance_service.rs`
- Added `AppInstanceError::MultipleAppInstance` variant with `#[error_meta(error_type = ErrorType::InternalServer)]`
- Updated `get_instance()` to explicitly convert `DbError::MultipleAppInstance`:
  ```rust
  async fn get_instance(&self) -> Result<Option<AppInstance>> {
    let row = self.db_service.get_app_instance().await.map_err(|e| match e {
      DbError::MultipleAppInstance => AppInstanceError::MultipleAppInstance,
      other => AppInstanceError::Db(other),
    })?;
    Ok(row.map(row_to_instance))
  }
  ```

#### 1f. Add transparent AppInstanceError to AccessRequestError ✅
- **File**: `crates/services/src/access_request_service/error.rs`
- Added import: `use crate::app_instance_service::AppInstanceError;`
- Added variant:
  ```rust
  #[error(transparent)]
  AppInstance(#[from] AppInstanceError),
  ```

#### 1g. Refactor access_request_service to use transparent error ✅
- **File**: `crates/services/src/access_request_service/service.rs`
- Added import: `use crate::app_instance_service::AppInstanceError;`
- Replaced verbose `map_err` with:
  ```rust
  let instance = self.app_instance_service.get_instance().await?
    .ok_or(AppInstanceError::NotFound)?;
  ```

#### 1h. Remove `DELETE FROM apps` in reset_all_tables ✅
- **File**: `crates/services/src/db/service.rs`
- Removed `DELETE FROM apps;` from the multi-statement query in `reset_all_tables`

#### 1i. Simplify test builder methods ✅
- **File**: `crates/services/src/test_utils/app.rs`
- Renamed `with_empty_app_instance_service()` → `with_app_instance_service()` (no args, creates service with empty DB)
- Renamed `with_app_instance_service_status(status)` → `with_app_instance(instance: AppInstance)` (takes AppInstance, persists to DB)
- Removed old `with_app_instance_service()` (which called `with_app_instance_service_status(AppStatus::Ready)`)

#### 1j. Add AppInstance factory methods in test_utils ✅
- **File**: `crates/services/src/test_utils/objs.rs`
- Added factory methods on `AppInstance`:
  ```rust
  impl AppInstance {
    pub fn test_default() -> Self { /* client_id: TEST_CLIENT_ID, client_secret: TEST_CLIENT_SECRET, scope: "scope_{TEST_CLIENT_ID}", status: Ready */ }
    pub fn test_with_status(status: AppStatus) -> Self { /* same but with given status */ }
  }
  ```
- Updated `app_instance()` rstest fixture to delegate to `AppInstance::test_default()`

---

### 2. `auth_middleware` crate ✅

#### 2a. Remove ad-hoc test utility functions in token_service/tests.rs ✅
- **File**: `crates/auth_middleware/src/token_service/tests.rs`
- Removed `test_app_instance_service()` function
- Removed `test_app_instance_service_empty()` function
- Replaced all usages with `AppServiceStubBuilder` + `.with_app_instance(AppInstance::test_default()).await` or `.with_app_instance_service().await`
- Updated imports: added `AppInstance, AppServiceStubBuilder, AppService`; removed `DefaultAppInstanceService, AppStatus, build_temp_dir`

#### 2b. Simplify auth_middleware builder patterns ✅
- **File**: `crates/auth_middleware/src/auth_middleware/tests.rs`
- Replaced all `.with_app_instance_service_status(status).await` → `.with_app_instance(AppInstance::test_with_status(status)).await`
- Replaced all `.with_app_instance_service().await` (10 occurrences) → `.with_app_instance(AppInstance::test_default()).await`
- Simplified verbose manual `DefaultAppInstanceService` construction block in `test_auth_middleware_with_expired_session_token_and_failed_refresh`
- **File**: `crates/auth_middleware/src/access_request_auth_middleware/tests.rs`
- Replaced 2 occurrences of `.with_empty_app_instance_service()` → `.with_app_instance_service()`

---

### 3. `routes_app` crate ✅

#### 3a. Simplify test_access_request.rs ✅
- **File**: `crates/routes_app/src/routes_apps/test_access_request.rs`
- In `build_test_harness`, replaced 10-line inline `DefaultAppInstanceService` construction with:
  ```rust
  builder.with_app_instance(services::AppInstance::test_default()).await;
  ```

#### 3b. Simplify login test files ✅
- `crates/routes_app/src/routes_auth/test_login_callback.rs` — replaced 4 inline constructions
- `crates/routes_app/src/routes_auth/test_login_initiate.rs` — replaced 3 inline constructions
- `crates/routes_app/src/routes_auth/test_login_resource_admin.rs` — replaced 1 inline construction
- `crates/routes_app/src/routes_users/test_access_request_admin.rs` — replaced 1 inline construction

#### 3c. Update other test files using old method names ✅
- `crates/routes_app/src/routes_setup/test_setup.rs` — updated 6 occurrences of `with_empty_app_instance_service()` → `with_app_instance_service()`; updated `with_app_instance_service_status(...)` → `with_app_instance(AppInstance::test_with_status(...))`

#### 3d. Fix build_test_router (critical regression fix) ✅
- **File**: `crates/routes_app/src/test_utils/router.rs`
- Added `services::AppInstance` import
- Added `.with_app_instance(AppInstance::test_default()).await` explicitly to both `build_test_router()` and `build_live_test_router()` builder chains
- **Root cause**: The old default builder behavior implicitly created a Ready app instance. After renaming `with_app_instance_service()` to mean "empty DB", tests expecting auth errors (401) got 500 instead (no app instance → not Ready). 184 test failures fixed by making the Ready instance explicit.

---

### 4. `server_app` + leaf crates ✅

#### 4a. live_server_utils.rs — no change needed ✅
- **File**: `crates/server_app/tests/utils/live_server_utils.rs`
- Left unchanged: uses manual service assembly with environment-variable Keycloak credentials and hardcoded `"test-resource-client"` for `ExternalTokenSimulator`. These are live integration tests that require specific external OAuth setup, not suitable for the simplified builder pattern.

#### 4b. `bodhi/src-tauri` and `lib_bodhiserver` — no change needed ✅
- Verified: no usages of old method names in either crate
- `lib_bodhiserver/app_service_builder.rs` correctly uses empty service (production code — app instance created during setup flow, not bootstrap)

---

## Layered Execution Plan

Each layer is implemented by a dedicated sub-agent, with tests run per layer before moving downstream.

### Layer 1: `services` crate ✅
**Result**: 322 tests pass

### Layer 2: `auth_middleware` crate ✅
**Result**: 154 tests pass

### Layer 3: `routes_app` crate ✅
**Result**: 470 tests pass (after fixing build_test_router regression — step 3d)

### Layer 4: `server_app` + leaf crates ✅
**Result**: Compile clean, no source changes needed

### Layer 5: Full backend validation ✅
**Result**: `make test.backend` — all tests pass (exit code 0)

### Layer 6: Frontend rebuild & validation
**Status**: Not executed (not requested)
**Commands** (if needed):
1. `make build.ui-rebuild` -- Rebuild embedded UI + NAPI bindings
2. `make build.ts-client` -- Regenerate TypeScript types
3. `make test.ui` -- Run frontend tests

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/services/src/db/mod.rs` | Rename module declaration |
| `crates/services/src/db/service_app_instance.rs` → `repository_app_instance.rs` | Rename + add count validation |
| `crates/services/src/db/error.rs` | Add `MultipleAppInstance` variant |
| `crates/services/src/db/service.rs` | Remove `DELETE FROM apps` from reset |
| `crates/services/migrations/0014_apps.up.sql` | Remove `DEFAULT ''` from scope |
| `crates/services/src/app_instance_service.rs` | Add `MultipleAppInstance` variant, convert from DbError |
| `crates/services/src/access_request_service/error.rs` | Add `AppInstance(#[from] AppInstanceError)` |
| `crates/services/src/access_request_service/service.rs` | Use `?` operator instead of `map_err` |
| `crates/services/src/test_utils/app.rs` | Rename/simplify builder methods |
| `crates/services/src/test_utils/objs.rs` | Add `AppInstance::test_default()`, `test_with_status()` |
| `crates/auth_middleware/src/token_service/tests.rs` | Remove ad-hoc functions, use builder |
| `crates/auth_middleware/src/auth_middleware/tests.rs` | Simplify builder patterns (10 occurrences) |
| `crates/auth_middleware/src/access_request_auth_middleware/tests.rs` | Replace `with_empty_app_instance_service()` |
| `crates/routes_app/src/routes_apps/test_access_request.rs` | Use builder |
| `crates/routes_app/src/routes_auth/test_login_callback.rs` | Use builder (4 occurrences) |
| `crates/routes_app/src/routes_auth/test_login_initiate.rs` | Use builder (3 occurrences) |
| `crates/routes_app/src/routes_auth/test_login_resource_admin.rs` | Use builder |
| `crates/routes_app/src/routes_users/test_access_request_admin.rs` | Use builder |
| `crates/routes_app/src/routes_setup/test_setup.rs` | Update old method names |
| `crates/routes_app/src/test_utils/router.rs` | Add `with_app_instance(AppInstance::test_default())` to fix regression |
