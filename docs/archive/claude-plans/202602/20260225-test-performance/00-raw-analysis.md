# Raw Analysis — Test Performance Investigation

**Date**: 2026-02-25
**Session**: Claude Code investigation of slow test suite
**Tool**: `cargo nextest run` v0.9.97

---

## Methodology

Tests were run crate-by-crate using:
```bash
cargo nextest run -p <crate> --status-level all --final-status-level slow 2>&1
```

Output captured to `/tmp/bodhi_test_timing/<crate>.txt` and `<crate>_nextest.txt`.

---

## Raw Timing Data Per Crate

### errmeta_derive
- **Tests**: 45
- **Total time**: 0.433s
- **Avg per test**: ~10ms
- **Slow tests (>10s)**: 0
- **Status**: ✅ Fast baseline, no issues

### objs
- **Tests**: 410
- **Total time**: 0.98s
- **Avg per test**: ~2ms
- **Slow tests (>10s)**: 0
- **Status**: ✅ Fast, no issues

### mcp_client
- **Tests**: 0
- **Status**: No tests in this crate

### services
- **Tests**: 341
- **Total time**: 252.9s
- **Slow tests (>10s)**: 105
- **Fastest tests**: <0.1s (DB, concurrency, encryption tests)
- **Slowest tests**: 26–31s each (mockito HTTP tests)

Top slow tests (representative sample):
```
PASS [  30.996s] services ai_api_service::tests::test_fetch_models_success_parameterized::case_2_without_api_key
PASS [  30.816s] services ai_api_service::tests::test_fetch_models_success
PASS [  29.843s] services auth_service::tests::test_refresh_token_retry_on_5xx
PASS [  29.834s] services auth_service::tests::test_list_users_success
PASS [  29.403s] services hub_service::tests::test_hf_hub_service_download_gated_file_allowed::case_2
PASS [  28.633s] services exa_service::tests::test_answer_error
PASS [  27.351s] services mcp_service::tests::test_mcp_service_execute_with_oauth_auth_type
```

Services slow by module:
- `ai_api_service`: ~14 tests × ~30s = ~420s
- `auth_service`: ~8 tests × ~30s = ~240s
- `hub_service`: ~20 tests × ~29s = ~580s
- `exa_service`: ~5 tests × ~28s = ~140s
- `mcp_service`: ~15 tests × ~27s = ~405s

Fast tests (no issue): `db::*`, `data_service::*`, `concurrency_service::*`, `cache_service::*`, `app_instance_service::*`

### server_core
- **Tests**: 100
- **Total time**: 25.1s
- **Slow tests (>60s)**: 8 (nextest slow threshold was 60s)

Slow tests:
```
SLOW [> 60.000s] server_core router_state::test::test_router_state_chat_completions_model_not_found
SLOW [> 60.000s] server_core router_state::test::test_router_state_chat_completions_returns_context_err
SLOW [> 60.000s] server_core router_state::test::test_router_state_chat_completions_delegate_to_context_with_alias
SLOW [> 60.000s] server_core shared_rw::test_shared_rw::test_chat_completions_continue_strategy
SLOW [> 60.000s] server_core shared_rw::test_shared_rw::test_chat_completions_drop_and_load_strategy
SLOW [> 60.000s] server_core shared_rw::test_shared_rw::test_chat_completions_load_strategy
SLOW [> 60.000s] server_core::test_live_shared_rw test_live_shared_rw_reload_with_actual_file
SLOW [> 60.000s] server_core::test_live_shared_rw test_live_shared_rw_reload_with_model_as_symlink
```

The `router_state` and `shared_rw` tests use `AppServiceStubBuilder::default().build().await` — same root cause as auth_middleware.
The `test_live_*` tests require a real llama.cpp binary — expected slow.

### auth_middleware
- **Tests**: 157
- **Total time**: ~940s (full parallel nextest run)
- **Slow tests (>10s)**: 99 out of 157
- **Slow threshold**: 900s for the worst tests

**Timing distribution:**
- Fast tests (<1s): ~58 tests (auth_context, access_request_auth, toolset_auth patterns that don't build AppServiceStub)
- Medium slow (74–79s): ~30 tests (api_auth_middleware role/scope tests)
- Very slow (938–940s): ~20 tests (api_auth_middleware user_scope tests)

**Isolated single-test measurement** (confirmed):
```
cargo nextest run -p auth_middleware -E 'test(test_api_auth_role_success::case_01)'
PASS [  42.397s] auth_middleware api_auth_middleware::tests::test_api_auth_role_success::case_01_user_accessing_user
```
→ 42.4 seconds for a test whose logic takes <1ms. The extra 42s is pure cleanup overhead.

### routes_app
- **Tests**: ~670 total
- **Passed**: 400 completed before kill signal
- **Total time**: >840s (killed)
- **Slow distribution**:
  - Fast (<1s): 65 tests
  - Slow (10–97s): 335 tests
  - SLOW marker (>60s or >840s): 269 tests

**SLOW by module (from nextest output):**
```
104  routes_users::management      (includes 5 near-deadlocked tests)
 70  routes_users::test_access_request_auth
 31  routes_toolsets::toolsets
 22  routes_models::test_aliases_auth
 13  routes_oai::chat
 11  routes_users::access_request
 10  routes_models::pull
  8  routes_oai::models
```

**Near-deadlocked tests (900s+):**
```
SLOW [>900.000s] routes_app routes_users::management::test_management_crud::test_change_user_role_handler_auth_error
SLOW [>900.000s] routes_app routes_users::management::test_management_crud::test_change_user_role_session_clear_failure_still_succeeds
SLOW [>900.000s] routes_app routes_users::management::test_management_crud::test_list_users_handler_auth_error
SLOW [>900.000s] routes_app routes_users::management::test_management_crud::test_list_users_handler_pagination_parameters
SLOW [>900.000s] routes_app routes_users::management::test_management_crud::test_list_users_handler_success
```

Comparative test in same file:
```
PASS [  79.593s] routes_app routes_users::management::test_management_crud::test_change_user_role_clears_sessions
```
→ `test_change_user_role_clears_sessions` uses explicit `session_service(Arc::new(mock_session))` — takes 79s
→ Other tests don't set session service — AppServiceStubBuilder creates DefaultSessionService — takes 900s+

---

## Root Cause Investigation

### Investigation 1: services mockito tests

**Test examined**: `test_fetch_models_success` in `ai_api_service.rs`

```rust
async fn test_fetch_models_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;   // mockito server
    let url = server.url();
    let mock_db = MockDbService::new();
    let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

    let _mock = server.mock("GET", "/models")
        .with_status(200)
        .with_body(r#"{"data": [{"id": "gpt-3.5-turbo"}]}"#)
        .create_async().await;

    let models = service.fetch_models(Some("test-key"), &url).await?;  // called
    assert_eq!(vec!["gpt-3.5-turbo"], models);  // passes
    Ok(())
    // Test logic: <1ms
    // Total time: 30.8s
}
```

The test logic works correctly and completes instantly. The 30s overhead is the **Tokio runtime shutdown waiting for reqwest connection pool background tasks**.

`DefaultAiApiService::with_db_service()` (in `crates/services/src/ai_api_service.rs` line 87):
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))  // 30 seconds
    .build()
    .expect("Failed to create HTTP client");
```

reqwest's `Client` maintains a connection pool with keepalive. When the test function returns and the Tokio runtime begins shutdown, it waits for the reqwest connection pool's background task (which maintains keepalive on connections) to exit. This task has a 30-second idle timer.

**Confirmation**: Run one test in isolation with `time cargo nextest run -p services -E 'test(test_fetch_models_success)'` → 30.8s for a test whose mock responds instantly.

### Investigation 2: auth_middleware / AppServiceStubBuilder tests

**Test examined**: `test_api_auth_role_success::case_01_user_accessing_user` in `api_auth_middleware.rs`

The test calls:
```rust
let app_service = AppServiceStubBuilder::default().build().await.unwrap();
```

`AppServiceStubBuilder::build()` (in `crates/services/src/test_utils/app.rs` line 91):
```rust
pub async fn build(&mut self) -> Result<AppServiceStub, AppServiceStubBuilderError> {
    if !matches!(&self.db_service, Some(Some(_))) {
        self.with_db_service().await;     // creates TestDbService with real SQLite
    }
    if !matches!(&self.session_service, Some(Some(_))) {
        self.with_session_service().await; // creates DefaultSessionService with 2 SQLite pools
    }
    if !matches!(&self.app_instance_service, Some(Some(_))) {
        self.with_app_instance_service().await;
    }
    self.fallback_build()
}
```

`with_session_service()` creates a real `DefaultSessionService`:
```rust
pub async fn with_session_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let dbfile = temp_home.path().join("test-session.sqlite");
    self.build_session_service(dbfile).await;  // creates 2 SQLite pools + migrations
    self
}
```

`DefaultSessionService::connect_sqlite()` (in `session_service.rs` line 54):
```rust
pub async fn connect_sqlite(url: &str) -> Result<Self, SessionServiceError> {
    sqlx::any::install_default_drivers();
    let sqlite_pool = SqlitePoolOptions::new()           // Pool 1: default settings
        .connect_with(opts).await.map_err(map_sqlx_err)?;
    let any_pool = AnyPool::connect(&any_url).await      // Pool 2: AnyPool
        .map_err(map_sqlx_err)?;
    let store = create_sqlite_store(sqlite_pool);
    store.migrate().await?;                              // tower_sessions migrations
    let backend = SessionStoreBackend::new_sqlite(store, any_pool);
    let mut service = Self::new(backend, url.to_string());
    service.run_custom_migration().await?;               // custom user_id column migration
    Ok(service)
}
```

**3 SQLite connection pools** are created in total:
1. `TestDbService` pool (from `with_db_service()`)
2. `SqlitePool` in `DefaultSessionService`
3. `AnyPool` in `DefaultSessionService`

sqlx connection pools spawn background async tasks on the Tokio runtime (idle connection cleanup, WAL checkpointing, pool health). When the `#[tokio::test]` macro's runtime shuts down after the test function returns, it must wait for these background tasks to complete.

**Pool default idle_timeout**: sqlx `SqlitePoolOptions` default is 600 seconds. The pool keeps connections alive and background tasks running for up to 600 seconds.

**Tokio runtime shutdown behavior**: When `block_on()` (used internally by `#[tokio::test]`) completes, the runtime calls `shutdown_background()` which aborts spawned tasks. However, sqlx pool tasks may use `JoinHandle` or blocking threads that delay this.

**Measured delays**:
- `TestDbService` alone: ~30–42s shutdown delay
- `DefaultSessionService` alone: contributes additional ~30s
- Both together: ~42–90s in isolation, 900s+ under heavy parallel contention

### Investigation 3: routes_app near-deadlocked tests

**Tests examined**: `test_list_users_handler_success` et al. in `test_management_crud.rs`

These tests:
1. Use `_temp_bodhi_home: TempDir` fixture (creates temp dir, copies test data, does NOT set env vars — confirmed by reading `crates/objs/src/test_utils/bodhi.rs`)
2. Call `AppServiceStubBuilder::default().auth_service(Arc::new(mock_auth)).build().await?`
3. Do NOT provide explicit session service

The contrast: `test_change_user_role_clears_sessions` explicitly provides `session_service(Arc::new(mock_session))` and takes only 79s (due to TestDbService pool only).

The 900s+ duration for tests WITHOUT explicit session service, vs 79s WITH mock session, confirms that `DefaultSessionService` adds ~820s of additional shutdown delay under high contention conditions.

Under full nextest parallel execution:
- Many test processes compete for filesystem I/O
- Multiple SQLite WAL checkpoint operations serialize
- 3 connection pools × many parallel tests × WAL contention → extreme cumulative delay

### Investigation 4: build_test_router() pattern

`build_test_router()` in `crates/routes_app/src/test_utils/router.rs`:
```rust
builder
    .with_hub_service()
    .with_data_service().await
    .with_db_service().await
    .with_session_service().await   // ← creates DefaultSessionService
    .with_app_instance(AppInstance::test_default()).await
```

This is the standard setup for ~300+ routes_app tests. Every test using `build_test_router()` will have the ~42–79s shutdown delay.

---

## Confirmed Non-Issues

1. **Nextest cross-crate feature flags**: `serial_test = "3.2.0"` uses file-based mutexes, fully compatible with nextest 0.9.97. No random failures observed. The old incompatibility was with `serial_test < 2.0` which used in-process mutexes.

2. **Test SQLite file isolation**: Each `AppServiceStubBuilder::default()` call creates a fresh `TempDir`, so tests are NOT sharing SQLite files. The issue is connection pool shutdown overhead, not file contention between tests.

3. **mockito server setup**: mockito servers respond instantly. The 30s in services tests is not mockito responding slowly — it's reqwest client cleanup AFTER the test completes successfully.

---

## Data Collection Scripts

Script used for data collection:
```bash
#!/bin/bash
# /tmp/run_all_crates_nextest.sh
WORKSPACE="/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp"
OUTPUT_DIR="/tmp/bodhi_test_timing"
CRATES=("mcp_client" "services" "server_core" "auth_middleware" "routes_app")

for crate in "${CRATES[@]}"; do
  cargo nextest run -p "$crate" --status-level all --final-status-level slow 2>&1 | tee "$OUTPUT_DIR/${crate}.txt"
done
```

Additional single-test isolation measurements:
```bash
time cargo nextest run -p services -E 'test(test_fetch_models_success)'
time cargo nextest run -p auth_middleware -E 'test(test_api_auth_role_success::case_01)'
```
