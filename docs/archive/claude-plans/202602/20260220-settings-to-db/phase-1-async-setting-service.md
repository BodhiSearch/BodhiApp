# Phase 1: Convert SettingService Trait to Async

## Goal

Make all `SettingService` trait methods `async` so they can eventually call the SQLite database. This is a purely mechanical transformation — no logic changes, just adding `async`/`.await` throughout the dependency chain.

## Motivation

Settings are currently read from a YAML file synchronously. Migrating to SQLite requires async DB calls. The trait must be async before the persistence layer can be added.

---

## Step 1.1: Make trait methods async — `services` crate

**File**: `crates/services/src/setting_service/service.rs`

Add `#[async_trait::async_trait]` to the `SettingService` trait and prefix every method with `async`:

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SettingService: std::fmt::Debug + Send + Sync {
  async fn load(&self, path: &Path);
  async fn home_dir(&self) -> Option<PathBuf>;
  async fn list(&self) -> Vec<SettingInfo>;
  async fn get_default_value(&self, key: &str) -> Option<Value>;
  async fn get_setting_metadata(&self, key: &str) -> SettingMetadata;
  async fn get_env(&self, key: &str) -> Option<String>;
  async fn get_setting(&self, key: &str) -> Option<String> { ... }   // default impl
  async fn get_setting_value(&self, key: &str) -> Option<Value> { ... }
  async fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource);
  async fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource) -> Result<()>;
  async fn set_setting(&self, key: &str, value: &str) -> Result<()> { ... }
  async fn set_setting_value(&self, key: &str, value: &Value) -> Result<()> { ... }
  async fn set_default(&self, key: &str, value: &Value) -> Result<()> { ... }
  async fn delete_setting(&self, key: &str) -> Result<()>;
  // ... all typed accessors: host(), port(), hf_cache(), is_production(), etc.
}
```

**File**: `crates/services/src/setting_service/default_service.rs`

- Add `#[async_trait::async_trait]` to the `DefaultSettingService` impl block
- All methods gain `async` keyword; internal calls gain `.await`
- `RwLock` operations remain sync (they don't need async)

**File**: `crates/services/src/test_utils/envs.rs`

- `SettingServiceStub` also gains `#[async_trait::async_trait]` and `async` on all methods

**Gate**: `cargo test -p services`

---

## Step 1.2: Update `server_core` — async SharedContext

**File**: `crates/server_core/src/shared_rw.rs`

Methods that call `setting_service` must be `async`:
- `canonical_url()`, `find_model()`, and others that read settings become `async`
- Update callers inside the crate (test files, integration tests)

**Gate**: `cargo test -p server_core`

---

## Step 1.3: Update `auth_middleware` — async auth helpers

**File**: `crates/auth_middleware/src/` (relevant files)

- Methods reading `canonical_url` from SettingService become `async`
- `DefaultTokenService` methods that call setting_service become `async`
- Update all usages throughout auth middleware

**Gate**: `cargo test -p auth_middleware`

---

## Step 1.4: Update `routes_app` — async route handlers

All route handlers that call `SettingService` methods gain `.await`:

- `routes_settings/route_settings.rs` — `get_setting`, `set_setting`, `delete_setting`, `list`, `get_setting_metadata`
- `routes_setup/route_setup.rs` — `host()`, `port()`, etc.
- `routes_auth/login.rs` — `auth_url()`, `auth_realm()`, `is_production()`, `canonical_url()`
- `shared/openapi.rs` — `apply_security_schemes()`, `get_setting_metadata()`
- `routes_oai/models.rs` — any setting reads
- `routes_ollama/handlers.rs` — any setting reads

**Gate**: `cargo test -p routes_app`

---

## Step 1.5: Update `server_app`

**File**: `crates/server_app/src/serve.rs`

- `serve()` function and `listener_variant.rs`, `listener_keep_alive.rs` gain `.await` where SettingService is called

**Gate**: `cargo test -p server_app`

---

## Step 1.6: Update `lib_bodhiserver`

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs`

- All calls to SettingService methods gain `.await`
- Functions that call these become `async`

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

- `build()` and helper methods gain `.await`

**Gate**: `cargo test -p lib_bodhiserver`

---

## Step 1.7: Update `bodhi/src-tauri` + `lib_bodhiserver_napi`

**File**: `crates/bodhi/src-tauri/src/server_init.rs`

- Wrap async calls in `tokio::runtime::Builder::new_current_thread().block_on(async { ... })`
- `set_feature_settings` becomes async, called inside the block

**File**: `crates/lib_bodhiserver_napi/src/server.rs`

- `setup_logs()` and `start()` gain `.await` on setting reads

**Gate**: `cargo test -p lib_bodhiserver_napi`

---

## Full Validation

```
make test.backend
```

All Rust tests pass. No logic changed — only async/await added.

---

## Key Files Changed

| File | Change |
|------|--------|
| `services/src/setting_service/service.rs` | Add `async_trait`, all methods async |
| `services/src/setting_service/default_service.rs` | Add `async_trait` impl, `.await` on internals |
| `services/src/test_utils/envs.rs` | `SettingServiceStub` gains `async_trait` |
| `server_core/src/shared_rw.rs` | `canonical_url()` and others become async |
| `auth_middleware/src/` | Auth helpers gain `.await` |
| `routes_app/src/routes_settings/` | Handlers gain `.await` |
| `routes_app/src/routes_auth/` | Login handlers gain `.await` |
| `routes_app/src/shared/openapi.rs` | `apply_security_schemes()` gains `.await` |
| `server_app/src/serve.rs` | Serve function gains `.await` |
| `lib_bodhiserver/src/app_service_builder.rs` | Builder methods gain `.await` |
| `bodhi/src-tauri/src/server_init.rs` | Wrap in `block_on` |
| `lib_bodhiserver_napi/src/server.rs` | `start()` gains `.await` |
