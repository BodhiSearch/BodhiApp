# Issue 02: sqlx Connection Pool Shutdown Delay

**Category**: sqlx `SqlitePoolOptions` default `idle_timeout` blocks Tokio runtime shutdown
**Severity**: 🔴 Critical
**Crates affected**: `services`, `auth_middleware`, `routes_app`, `server_core`
**Tests affected**: ~400 tests, 42–900s each (expected: <1s)
**Current total cost**: ~400 × 90s avg ≈ **~36,000 seconds** (10 hours) if run serially; ~940s worst-case parallel

---

## Problem Summary

`AppServiceStubBuilder::default().build().await` creates 3 real SQLite connection pools. Each pool spawns background Tokio tasks (idle connection cleanup, WAL checkpointing). These background tasks have a default `idle_timeout` of 600 seconds in sqlx. When the `#[tokio::test]` runtime shuts down after a test, it waits for these background tasks to complete.

Under nextest parallel execution, many test processes compete for filesystem I/O and SQLite WAL checkpointing, causing extreme cumulative delays (up to 900s+ per test).

---

## Root Cause

### The 3 SQLite Pools Created Per Test

**Pool 1: TestDbService** (from `AppServiceStubBuilder::with_db_service()`)

`crates/services/src/test_utils/app.rs` — `with_db_service()` creates a `TestDbService` which opens a real SQLite file via `SqlitePoolOptions::new().connect(...)`. Default sqlx `idle_timeout` = 600 seconds.

**Pool 2: SqlitePool in DefaultSessionService** (from `AppServiceStubBuilder::with_session_service()`)

`crates/services/src/session_service/session_service.rs` — `connect_sqlite()` (line ~54):
```rust
pub async fn connect_sqlite(url: &str) -> Result<Self, SessionServiceError> {
    sqlx::any::install_default_drivers();
    let sqlite_pool = SqlitePoolOptions::new()           // Pool 2: NO idle_timeout set → 600s default
        .connect_with(opts).await.map_err(map_sqlx_err)?;
    let any_pool = AnyPool::connect(&any_url).await      // Pool 3: AnyPool also no idle_timeout
        .map_err(map_sqlx_err)?;
    let store = create_sqlite_store(sqlite_pool);
    store.migrate().await?;                              // tower_sessions migrations
    let backend = SessionStoreBackend::new_sqlite(store, any_pool);
    let mut service = Self::new(backend, url.to_string());
    service.run_custom_migration().await?;               // custom user_id column migration
    Ok(service)
}
```

**Pool 3: AnyPool in DefaultSessionService** — same file, same `connect_sqlite()`, second pool opened.

### The AppServiceStubBuilder::build() Flow

`crates/services/src/test_utils/app.rs` — `build()` method (line ~91):
```rust
pub async fn build(&mut self) -> Result<AppServiceStub, AppServiceStubBuilderError> {
    if !matches!(&self.db_service, Some(Some(_))) {
        self.with_db_service().await;     // ← creates Pool 1
    }
    if !matches!(&self.session_service, Some(Some(_))) {
        self.with_session_service().await; // ← creates Pools 2 + 3
    }
    if !matches!(&self.app_instance_service, Some(Some(_))) {
        self.with_app_instance_service().await;
    }
    self.fallback_build()
}
```

**If the caller doesn't provide explicit mock services, ALL 3 real pools are created.**

### The sqlx Background Task Problem

sqlx's `SqlitePoolOptions` (and `AnyPoolOptions`) spawn async background tasks on the Tokio runtime:
- Idle connection cleanup (checks every `idle_timeout / 2`)
- Connection health monitoring
- WAL checkpoint coordination (for SQLite WAL mode)

These tasks keep running after the pool is dropped until their timer fires. Default `idle_timeout` = 600 seconds in sqlx 0.8.x.

When `#[tokio::test]`'s runtime shuts down:
1. Runtime calls `shutdown_timeout(Duration::from_secs(0))` or similar
2. sqlx pool background tasks are on the runtime
3. Tasks hold `JoinHandle`s or are detached with `spawn()`
4. For detached tasks: runtime's `shutdown_background()` sends abort signal
5. But sqlx task may be in a `sleep(idle_timeout)` call — the sleep future is aborted, but cleanup code runs
6. With 3 pools × 2+ background tasks each = 6+ tasks competing for abort acknowledgment

### Parallel Contention Makes It Catastrophic

Under nextest with default parallelism (= CPU count):
- All test processes run simultaneously
- Each process has its own SQLite files in TempDir
- But SQLite WAL mode still uses OS-level file locks
- Multiple WAL checkpoint operations from different processes can't run concurrently on the same filesystem
- Checkpoint I/O serializes across processes even though files are separate
- This cascading I/O contention amplifies 42s isolation delays to 900s+ in parallel

---

## Measured Timing Evidence

**Isolated single-test measurement** (pool shutdown delay only, no contention):
```
cargo nextest run -p auth_middleware -E 'test(test_api_auth_role_success::case_01)'
PASS [  42.397s] auth_middleware api_auth_middleware::tests::test_api_auth_role_success::case_01_user_accessing_user
```
→ 42s for a test whose logic takes <1ms. The 42s is Tokio runtime cleanup only.

**With explicit MockSessionService** (removes 2 of the 3 pools):
```
PASS [  79.593s] routes_app routes_users::management::test_management_crud::test_change_user_role_clears_sessions
```
→ 79s with only TestDbService pool (1 pool instead of 3).

**Without explicit MockSessionService** (all 3 pools):
```
SLOW [>900.000s] routes_app routes_users::management::test_management_crud::test_list_users_handler_success
```
→ 900s+ with all 3 pools under parallel contention.

---

## Affected Crates and Patterns

### services — ~30 tests × ~42s = ~1,260s

**Pattern**: Tests that call `AppServiceStubBuilder::default().build()` in services own test utilities.

Fast tests in services (`db::*`, `data_service::*`) use `TestDbService` directly without the full AppServiceStub — only 1 pool, still slow but less so.

### auth_middleware — 99 slow tests

**File**: `crates/auth_middleware/src/api_auth_middleware.rs`

Two test helpers at lines ~148 and ~175:
```rust
// Pattern A: role tests (74-79s each)
async fn test_router_with_auth_context(...) {
    let app_service = AppServiceStubBuilder::default().build().await.unwrap();
    // ... builds router with app_service ...
}

// Pattern B: user_scope tests (938-940s each)
async fn test_router_user_scope_with_auth_context(...) {
    let app_service = AppServiceStubBuilder::default().build().await.unwrap();
    // ... builds router with app_service ...
}
```

Note: The session layer is NOT attached to the test router in auth_middleware tests — `DefaultSessionService` is created but never used. Its pools are pure waste.

**Timing distribution**:
- Role tests (74-79s): ~30 tests
- User scope tests (938-940s): ~20 tests (worse parallel contention)

### routes_app — ~269 tests with SLOW marker

**File**: `crates/routes_app/src/test_utils/router.rs` — `build_test_router()`:
```rust
pub async fn build_test_router(/* ... */) -> Router {
    builder
        .with_hub_service()
        .with_data_service().await
        .with_db_service().await
        .with_session_service().await   // ← creates DefaultSessionService (Pools 2 + 3)
        .with_app_instance(AppInstance::test_default()).await
        // ...
}
```

This is the standard test helper used by ~300+ tests in routes_app. Every test using `build_test_router()` gets all 3 SQLite pools.

Near-deadlocked tests in `test_management_crud.rs` (900s+):
- `test_list_users_handler_success`
- `test_list_users_handler_auth_error`
- `test_list_users_handler_pagination_parameters`
- `test_change_user_role_handler_auth_error`
- `test_change_user_role_session_clear_failure_still_succeeds`

### server_core — 6 slow tests (non-live)

**File**: `crates/server_core/src/router_state.rs` and `crates/server_core/src/shared_rw.rs`

Tests use `AppServiceStubBuilder::default().build().await` for router state tests:
```
SLOW [> 60.000s] server_core router_state::test::test_router_state_chat_completions_model_not_found
SLOW [> 60.000s] server_core router_state::test::test_router_state_chat_completions_returns_context_err
SLOW [> 60.000s] server_core router_state::test::test_router_state_chat_completions_delegate_to_context_with_alias
SLOW [> 60.000s] server_core shared_rw::test_shared_rw::test_chat_completions_continue_strategy
SLOW [> 60.000s] server_core shared_rw::test_shared_rw::test_chat_completions_drop_and_load_strategy
SLOW [> 60.000s] server_core shared_rw::test_shared_rw::test_chat_completions_load_strategy
```

---

## Fix Strategy

### Fix A: Set Short `idle_timeout` on Test Pools (Structural Fix — Recommended)

Modify `DefaultSessionService::connect_sqlite()` to detect test context OR add a separate test constructor with short pool idle timeout:

```rust
// In session_service.rs — test-specific constructor
#[cfg(any(test, feature = "test-utils"))]
pub async fn connect_sqlite_for_test(url: &str) -> Result<Self, SessionServiceError> {
    sqlx::any::install_default_drivers();
    let opts = SqliteConnectOptions::from_str(url)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let sqlite_pool = SqlitePoolOptions::new()
        .idle_timeout(Duration::from_millis(100))  // ← 100ms instead of 600s
        .max_lifetime(Duration::from_secs(1))       // ← also limit lifetime
        .connect_with(opts).await.map_err(map_sqlx_err)?;

    let any_pool = AnyPoolOptions::new()
        .idle_timeout(Duration::from_millis(100))  // ← same for AnyPool
        .connect(&any_url).await
        .map_err(map_sqlx_err)?;

    // ... rest of constructor
}
```

Similarly for `TestDbService` pool creation.

### Fix B: Explicit Pool Close Before Test Ends

Add `pool.close().await` in teardown. Since `AppServiceStub` owns the pools indirectly through `DefaultSessionService`, this requires adding a `close()` method to the service traits.

```rust
// This is the nuclear option — guaranteed fast shutdown
impl AppServiceStub {
    pub async fn close_all_pools(&self) {
        if let Some(session_service) = &self.session_service {
            session_service.close().await;
        }
        if let Some(db_service) = &self.db_service {
            db_service.close().await;
        }
    }
}
```

Tests would need to call `app_service.close_all_pools().await` before returning. This is reliable but invasive.

### Fix C: Use Mock Services Instead of Real Pools (Best Long-Term)

The cleanest fix — addressed in detail in `03-mock-session.md`. Tests that don't exercise session storage behavior should use `MockSessionService` instead of `DefaultSessionService`.

```rust
// In AppServiceStubBuilder::build() — when session service not explicitly set,
// use MockSessionService (no-op) instead of DefaultSessionService (3 real pools)
if !matches!(&self.session_service, Some(Some(_))) {
    self.session_service = Some(Some(Arc::new(MockSessionService::new())));
}
```

This is the root fix for auth_middleware and routes_app tests.

---

## Expected Improvement

| Crate | Before (isolated) | After Fix A | After Fix C |
|-------|------------------|-------------|-------------|
| services (pool delay) | ~42s/test | ~1s/test | ~1s/test |
| auth_middleware | 42–940s/test | ~2s/test | ~0.5s/test |
| routes_app | 77–900s/test | ~3s/test | ~1s/test |
| server_core (pool tests) | >60s/test | ~2s/test | ~2s/test |

---

## Verification

```bash
# services pool tests (after fix A)
time cargo nextest run -p services -E 'test(test_db)'

# auth_middleware isolated (target: <5s)
time cargo nextest run -p auth_middleware -E 'test(test_api_auth_role_success::case_01)'

# routes_app full suite (target: <30s)
time cargo nextest run -p routes_app
```

---

## Key Files to Modify

1. `crates/services/src/session_service/session_service.rs` — `connect_sqlite()` — add `idle_timeout`
2. `crates/services/src/test_utils/app.rs` — `with_session_service()` / `with_db_service()` — use shorter timeouts
3. `crates/services/src/db/service.rs` (or `test_db_service.rs`) — `TestDbService` pool options

---

## Investigation Commands for Fresh Session

```bash
# Find all SqlitePoolOptions usages
grep -rn "SqlitePoolOptions" crates/services/src/

# Find all AnyPoolOptions usages
grep -rn "AnyPool\|AnyPoolOptions" crates/services/src/

# Verify the idle_timeout issue (check sqlx source if needed)
grep -rn "idle_timeout" crates/services/src/

# Find the session service constructor
grep -n "connect_sqlite\|SqlitePoolOptions::new" crates/services/src/session_service/session_service.rs

# Measure auth_middleware in isolation (confirm 42s)
time cargo nextest run -p auth_middleware -E 'test(test_api_auth_role_success::case_01)'

# Check slow tests in auth_middleware
cargo nextest run -p auth_middleware --status-level all --final-status-level slow 2>&1 | grep -E "SLOW|PASS \[.*[0-9]{2,}\."
```
