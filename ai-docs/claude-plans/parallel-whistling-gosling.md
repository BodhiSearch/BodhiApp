# Plan: Consolidated Logging with Dynamic Log Level Reconfiguration

## Context

BodhiApp has 3 separate logging initialization paths (server, NAPI, native) with significant code duplication. Logging is set once at startup with no runtime reconfiguration. The NAPI mode has a bug: on server restart in the same process, `try_init()` silently fails, creating an orphaned `WorkerGuard` while the old subscriber (with a dropped guard) continues receiving logs. This plan consolidates logging into a shared `LoggingService` in `lib_bodhiserver`, adds dynamic log level changes via `tracing_subscriber::reload`, and replaces `tauri_plugin_log` in native mode.

## Architecture

### LoggingService (new, in `lib_bodhiserver`)

```
LoggingService {
    _guard: WorkerGuard,              // keeps NonBlocking writer alive
    reload_filter: Box<dyn Fn(EnvFilter) + Send + Sync>,  // type-erased reload handle
}
```

- `setup(bootstrap: &BootstrapService) -> Result<Self>` creates the tracing subscriber with a `reload::Layer<EnvFilter>` wrapping the filter, composes file + optional stdout layers, installs globally
- `try_setup(bootstrap: &BootstrapService) -> Result<Option<Self>>` same as above but uses `try_init()` - returns `None` if subscriber already set (NAPI re-start case)
- `update_log_level(&self, level: LogLevel)` rebuilds `EnvFilter` with new base level + hardcoded per-crate directives, calls `reload_filter(new_filter)`
- Implements `SettingsChangeListener` - on `BODHI_LOG_LEVEL` change, parses new value and calls `update_log_level()`

### Type Erasure for Reload Handle

The `reload::Handle<EnvFilter, S>` has a subscriber-dependent type param `S`. We erase it with a closure:

```rust
let (filter_layer, handle) = reload::Layer::new(env_filter);
// ... compose subscriber and init ...
let reload_filter: Box<dyn Fn(EnvFilter) + Send + Sync> = Box::new(move |new_filter| {
    let _ = handle.reload(new_filter);
});
```

### Shared Filter Builder

```rust
fn build_env_filter(level: LogLevel) -> EnvFilter {
    let level: LevelFilter = level.into();
    EnvFilter::new(level.to_string())
        .add_directive("hf_hub=error".parse().unwrap())
        .add_directive("tower_sessions=warn".parse().unwrap())
        .add_directive("tower_http=warn".parse().unwrap())
        .add_directive("tower_sessions_core=warn".parse().unwrap())
}
```

### Two-Phase Initialization (all modes)

1. **Bootstrap phase** (sync, before/without tokio): `LoggingService::setup(&bootstrap)` using BootstrapService values (env + yaml only)
2. **Reconcile phase** (async, after AppService built): Check if DB has a different `BODHI_LOG_LEVEL`, update if so, then register as `SettingsChangeListener`

### NAPI Re-start Fix

- `LoggingService` persists across server start/stop cycles in `BodhiServer` struct
- `stop()` does NOT drop `LoggingService` (guard stays alive, subscriber stays installed)
- `start()` reuses existing `LoggingService` if present; only creates new one on first start
- Limitation: changing `BODHI_HOME` between NAPI restarts won't change log file location (test-only scenario, acceptable)

## Files to Modify

### 1. `Cargo.toml` (workspace root)
- No changes needed (tracing deps already in workspace)

### 2. `crates/lib_bodhiserver/Cargo.toml`
- Add dependencies: `tracing-subscriber = { workspace = true, features = ["env-filter"] }`, `tracing-appender = { workspace = true }`

### 3. `crates/lib_bodhiserver/src/logging_service.rs` (NEW)
- `build_env_filter(level: LogLevel) -> EnvFilter`
- `LoggingService` struct with `_guard: WorkerGuard` and `reload_filter: Box<dyn Fn(EnvFilter) + Send + Sync>`
- `LoggingService::setup(bootstrap: &BootstrapService) -> Result<Self, io::Error>` - creates subscriber with reload layer, uses `.init()`
- `LoggingService::try_setup(bootstrap: &BootstrapService) -> Result<Option<Self>, io::Error>` - same but `.try_init()`, returns `None` if already set
- `LoggingService::update_log_level(&self, level: LogLevel)` - rebuilds filter and reloads
- `impl SettingsChangeListener for LoggingService` - reacts to `BODHI_LOG_LEVEL` changes
- Re-export from `lib_bodhiserver/src/lib.rs`

### 4. `crates/lib_bodhiserver/src/lib.rs`
- Add `mod logging_service;` and re-export `LoggingService`

### 5. `crates/bodhi/src-tauri/src/server_init.rs`
- Remove inline `setup_logs()` function (lines 84-138)
- Replace with `LoggingService::setup(&bootstrap)` at line 62
- Store `LoggingService` (not just `_guard`) so it survives the runtime block
- After `build_app_service`, register logging_service as SettingsChangeListener:
  ```rust
  app_service.setting_service().add_listener(Arc::new(logging_service)).await;
  ```
- Reconcile: check DB log_level vs bootstrap log_level, update if different

### 6. `crates/lib_bodhiserver_napi/src/server.rs`
- Remove inline `setup_logs()` function (lines 266-329)
- Remove `tracing-subscriber` and `tracing-appender` imports (they come via LoggingService)
- Change `BodhiServer` field from `log_guard: Option<WorkerGuard>` to `logging_service: Option<Arc<LoggingService>>`
- In `start()`: if `self.logging_service.is_none()`, call `LoggingService::setup()` or `try_setup()`. If already set, reuse.
- After `build_app_service`, register as SettingsChangeListener
- In `stop()`: do NOT take/drop logging_service. Only drop server handle.
- In `Drop`: logging_service drops naturally with the struct

### 7. `crates/lib_bodhiserver_napi/Cargo.toml`
- Remove direct `tracing-subscriber` and `tracing-appender` dependencies (now transitive via lib_bodhiserver)

### 8. `crates/bodhi/src-tauri/src/native_init.rs`
- Remove `tauri_plugin_log` usage (lines 57-77 in `aexecute`)
- In `initialize_and_execute()`: call `LoggingService::setup(&bootstrap)` BEFORE `bootstrap.into_parts()` (move it between lines 197-198)
- Pass `LoggingService` into the tokio runtime block
- After `build_app_service`, reconcile DB log_level and register as SettingsChangeListener
- Remove tauri `.plugin(log_plugin)` from the builder

### 9. `crates/bodhi/src-tauri/Cargo.toml`
- Remove `tauri-plugin-log` from dependencies and from `native` feature list
- Remove direct `tracing-subscriber` and `tracing-appender` dependencies if now transitive

### 10. `crates/bodhi/src-tauri/src/native_init.rs` imports
- Remove `BODHI_LOGS`, `BODHI_LOG_STDOUT` imports (no longer needed for logging setup)
- Remove `LogLevel` import if only used for tauri_plugin_log

## Implementation Order (Layered Development)

1. **lib_bodhiserver** (upstream): Add deps, create `logging_service.rs`, re-export. Run `cargo test -p lib_bodhiserver`
2. **bodhi/src-tauri server_init** (downstream): Replace `setup_logs` with `LoggingService::setup`. Run `cargo test -p bodhi --features production`
3. **lib_bodhiserver_napi** (downstream): Replace `setup_logs`, fix NAPI re-start. Run `cargo test -p lib_bodhiserver_napi`
4. **bodhi/src-tauri native_init** (downstream): Replace `tauri_plugin_log`. Run `cargo test -p bodhi --features native`
5. **Full validation**: `make test.backend`

## Scope Boundaries

- **In scope**: Dynamic `BODHI_LOG_LEVEL` changes at runtime, NAPI re-start fix, logging consolidation, tauri_plugin_log removal
- **Out of scope**: Dynamic `BODHI_LOG_STDOUT` toggle (requires restart), per-crate directive customization, log file rotation changes
- **Limitation**: NAPI re-start reuses original log file path even if BODHI_HOME changes between restarts

## Verification

1. `cargo test -p lib_bodhiserver` - new LoggingService unit tests
2. `cargo test -p bodhi` - server_init still works
3. `cargo test -p lib_bodhiserver_napi` - NAPI lifecycle tests pass without orphaned guards
4. `make test.backend` - full regression
5. Manual test: start server, change BODHI_LOG_LEVEL via API/DB, verify new log level takes effect without restart
6. Manual test: verify log file rotation still works (daily rolling)
