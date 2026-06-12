# Plan: Simplify AppServiceBuilder and pass BootstrapParts directly

## Context

`AppServiceBuilder` has accumulated complexity:
- Takes `BootstrapService` but only needs `BootstrapParts` (the data, not the logging config)
- 13 Option fields, but only 3 service setters are ever used externally (in tests only): `secret_service`, `time_service`, `cache_service`
- `encryption_key` field is purely internal, never exposed
- Second `get_or_build_db_service` call is redundant (always returns cached value)
- `setup_app_dirs` combines directory creation with bootstrap service construction, making tests harder

This plan simplifies the builder from 13 fields to 4, removes 6 unused setter methods, and splits `setup_app_dirs` for better testability.

## Changes

### 1. Split `setup_app_dirs` (`crates/lib_bodhiserver/src/app_dirs_builder.rs`)

- **`setup_app_dirs(&AppOptions)`** → returns `Result<(PathBuf, SettingSource, HashMap<String, Value>), AppDirsBuilderError>`. Drop `command` param. Body: `load_defaults_yaml()` + `create_bodhi_home()`.

- **`setup_bootstrap_service`** → change from `fn` to `pub fn`. Existing signature (minus the reordering already present).

### 2. Simplify `AppServiceBuilder` (`crates/lib_bodhiserver/src/app_service_builder.rs`)

**Struct: 13 fields → 4 fields**
```rust
pub struct AppServiceBuilder {
  bootstrap_parts: Option<BootstrapParts>,
  // Externally injectable services (for testing).
  // To add injection for a new service, follow the cache_service pattern.
  time_service: Option<Arc<dyn TimeService>>,
  secret_service: Option<Arc<dyn SecretService>>,
  cache_service: Option<Arc<dyn CacheService>>,
}
```

**Remove these setters** (never called externally): `hub_service()`, `data_service()`, `db_service()`, `session_service()`, `auth_service()`, `ai_api_service()`

**Remove these Option fields**: `hub_service`, `data_service`, `db_service`, `session_service`, `auth_service`, `ai_api_service`, `access_request_service`, `network_service`, `encryption_key`

**Keep these setters** (used in tests): `time_service()`, `secret_service()`, `cache_service()`

**`new()`**: Takes `BootstrapParts`, wraps in `Some()`.

**Rename `get_or_build_*` → `build_*`** for services without setters: `build_hub_service`, `build_data_service`, `build_db_service`, `build_session_service`, `build_auth_service`, `build_ai_api_service`, `build_concurrency_service`, `build_tool_service`, `build_access_request_service`, `build_network_service`, `build_mcp_service`. These no longer check an Option field.

**Keep `get_or_build_*` for injected services**: `get_or_build_time_service`, `get_or_build_secret_service`, `get_or_build_cache_service`.

**`encryption_key`**: Remove field. Extract `get_or_build_encryption_key` to standalone fn `build_encryption_key(is_production, env_value) -> Result<Vec<u8>, ApiError>`. No `&mut self` needed.

**Remove redundant second `get_or_build_db_service` call** in `build()` (lines 224-230). Reuse the `db_service` local from the first call.

**`build()` flow** (simplified):
```rust
pub async fn build(mut self) -> Result<DefaultAppService, ErrorMessage> {
  let time_service = self.get_or_build_time_service();
  let parts = self.bootstrap_parts.take().expect("build() requires BootstrapParts");

  // Extract pre-from_parts values (borrows, no clone needed)
  let is_production = parts.system_settings.iter().any(|s| ...);
  let encryption_key_value = parts.env_wrapper.var(BODHI_ENCRYPTION_KEY).ok();
  let encryption_key = build_encryption_key(is_production, encryption_key_value).await?;
  let db_service = Self::build_db_service(parts.bodhi_home.join(PROD_DB), time_service.clone(), encryption_key.clone()).await?;

  let setting_service = Arc::new(DefaultSettingService::from_parts(parts, db_service.clone()));

  // Build remaining services
  let hub_service = Self::build_hub_service(&setting_service).await?;
  let secret_service = self.get_or_build_secret_service(&setting_service, encryption_key.clone()).await?;
  let data_service = Self::build_data_service(hub_service.clone(), db_service.clone());
  // ... etc
}
```

**Add comment** on the struct: `// To add injection for a new service, follow the cache_service pattern.`

### 3. Update `build_app_service` convenience fn

```rust
pub async fn build_app_service(bootstrap_parts: BootstrapParts) -> Result<DefaultAppService, ErrorMessage> {
  AppServiceBuilder::new(bootstrap_parts).build().await
}
```

### 4. Update `lib.rs` exports (`crates/lib_bodhiserver/src/lib.rs`)

Add `BootstrapParts` to the services re-exports. Keep `BootstrapService` export (still needed for logging).

### 5. Update callers

All 3 callers follow the same pattern change:
```rust
// Before:
let bootstrap = setup_app_dirs(&options, command)?;
setup_logs(&bootstrap);  // (native_init skips this)
build_app_service(bootstrap).await

// After:
let (bodhi_home, source, file_defaults) = setup_app_dirs(&options)?;
let bootstrap = setup_bootstrap_service(&options, bodhi_home, source, file_defaults, command)?;
setup_logs(&bootstrap);  // (native_init skips this)
let parts = bootstrap.into_parts();
build_app_service(parts).await
```

Files: `crates/bodhi/src-tauri/src/server_init.rs`, `crates/bodhi/src-tauri/src/native_init.rs`, `crates/lib_bodhiserver_napi/src/server.rs`

### 6. Update tests

**test_app_service_builder.rs**:
- Tests construct `BootstrapParts` directly (no BootstrapService)
- `test_service_already_set_errors` → update for remaining 3 services
- Remove imports of `setup_app_dirs`, `AppCommand`

**test_app_dirs_builder.rs**:
- `test_setup_app_dirs_integration` → assert against returned tuple `(bodhi_home, source, file_defaults)`
- `test_setup_app_dirs_with_app_settings` → call `setup_app_dirs` then `setup_bootstrap_service` separately

## Files Modified

1. `crates/lib_bodhiserver/src/app_dirs_builder.rs` - split setup_app_dirs, pub setup_bootstrap_service
2. `crates/lib_bodhiserver/src/app_service_builder.rs` - major simplification (13→4 fields, remove 6 setters, rename methods)
3. `crates/lib_bodhiserver/src/lib.rs` - add BootstrapParts re-export
4. `crates/lib_bodhiserver/src/test_app_service_builder.rs` - update tests
5. `crates/lib_bodhiserver/src/test_app_dirs_builder.rs` - update tests
6. `crates/bodhi/src-tauri/src/server_init.rs` - caller update
7. `crates/bodhi/src-tauri/src/native_init.rs` - caller update
8. `crates/lib_bodhiserver_napi/src/server.rs` - caller update

## No Changes

- `BootstrapService` struct (`bootstrap_service.rs`) - remains as-is
- `BootstrapParts` struct (`services/src/setting_service/bootstrap_parts.rs`) - no changes
- `DefaultSettingService::from_parts()` - keeps taking `BootstrapParts` by value

## Verification

1. `cargo check -p lib_bodhiserver` - verify compilation
2. `cargo check -p bodhi --features native` - verify Tauri caller compiles
3. `cargo check -p lib_bodhiserver_napi` - verify NAPI caller compiles
4. `cargo test -p lib_bodhiserver` - run lib_bodhiserver tests
5. `make test.backend` - full backend test suite
