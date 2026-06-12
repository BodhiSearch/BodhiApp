# Phase 4: BootstrapService Cleanup + AppServiceBuilder Simplification

## Goal

Clean up the code left over from phases 1–3:
1. Remove remaining generic methods from `BootstrapService` (it currently still mirrors `SettingService` in some areas)
2. Unify duplicated `AppServiceBuilder` methods
3. Delegate service-owned setup concerns (SQLite file creation, HF cache dir, logs dir) to their owning services
4. Move runtime defaults from `BootstrapService` to `DefaultSettingService`
5. Various code quality improvements from review findings

Each step produces a compilable state. Run `cargo test -p services && cargo test -p lib_bodhiserver` after each.

---

## Step 4.1: `DbPool::connect` — auto-create SQLite files

**File**: `crates/services/src/db/sqlite_pool.rs`

Change `connect()` to use `create_if_missing(true)` so SQLite files are created on first connect instead of requiring pre-existing files:

```rust
pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
  let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
  Ok(SqlitePool::connect_with(options).await?)
}
```

Update test: rename `test_db_pool_raises_error` → `test_db_pool_creates_file_if_missing`. The test should verify the file is created, not that connecting fails.

**Effect**: `setup_bodhi_subdirs()` (which created empty `.sqlite` files via `File::create`) is no longer needed.

---

## Step 4.2: Remove `setup_bodhi_subdirs` from `app_dirs_builder`

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs`

- Remove `setup_bodhi_subdirs()` function
- Remove call from `setup_app_dirs()` or `setup_bootstrap_service()`
- Remove `app_db_path()` and `session_db_path()` from `BootstrapService` (no longer needed — DB files are auto-created by `DbPool::connect`)

**Tests**: Remove `test_setup_bodhi_subdirs_success`. Update `test_setup_app_dirs_integration` to not assert that `.sqlite` files exist (they're created later by `AppServiceBuilder`).

---

## Step 4.3: Unify `AppServiceBuilder` methods — eliminate duplication

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

The builder previously had `_early`/`_from_parts` variants for db_service and encryption_key due to the chicken-and-egg problem (now solved by Phase 2). Consolidate:

**Before**:
- `get_or_build_db_service_early(parts, time, key)` and `get_or_build_db_service(setting_service, time, key)`
- `get_or_build_encryption_key_from_parts(parts)` and `get_or_build_encryption_key(setting_service)`

**After**: Plain associated functions (no `Option` field — no injection needed for these):
```rust
async fn build_db_service(
  db_path: PathBuf,
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,
) -> Result<Arc<dyn DbService>, ApiError> { ... }

async fn build_encryption_key(
  is_production: bool,
  encryption_key_value: Option<String>,
) -> Result<Vec<u8>, ApiError> { ... }
```

Remove `get_app_name()` helper — inline the app name logic into `build_encryption_key`.

Reduce `AppServiceBuilder` struct from 13 fields down to 4:
```rust
pub struct AppServiceBuilder {
  bootstrap_parts: Option<BootstrapParts>,
  time_service: Option<Arc<dyn TimeService>>,
  secret_service: Option<Arc<dyn SecretService>>,
  cache_service: Option<Arc<dyn CacheService>>,
}
```

Remove the 6 unused setter methods: `hub_service`, `data_service`, `db_service`, `session_service`, `auth_service`, `ai_api_service` — these were never used in production and the test utilities now inject via `SecretService`/`CacheService` instead.

Rename remaining `get_or_build_*` → `build_*` for services without external injection.

---

## Step 4.4: Move HF cache dir creation into `HfHubService`

**File**: `crates/services/src/hub_service/service.rs`

Change `new_from_hf_cache()` to return `Result` and create the directory itself:

```rust
pub fn new_from_hf_cache(
  hf_cache: PathBuf,
  hf_env_token: Option<String>,
  progress_bar: bool,
) -> Result<Self, std::io::Error> {
  fs::create_dir_all(&hf_cache)?;
  let cache = Cache::new(hf_cache);
  let token = hf_env_token.or_else(|| cache.token());
  Ok(Self { cache, progress_bar, token })
}
```

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

Update `build_hub_service()` to handle the `Result`:
```rust
let hub_service = HfHubService::new_from_hf_cache(hf_cache, hf_token, true)
  .map_err(|err| ApiError::from(IoError::from(err)))?;
```

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs`

Remove `setup_hf_home()` function — HF cache dir is now created by `HfHubService`.

Remove `hf_home` field from `BootstrapService` and its accessor. The `HF_HOME` default remains in the defaults `HashMap` (flows through `BootstrapParts` to `DefaultSettingService`).

**Tests**: Remove test assertions for `$HF_HOME/hub` existing after `setup_app_dirs()`.

---

## Step 4.5: Move logs dir creation into `setup_logs()`

**File**: `crates/bodhi/src-tauri/src/server_init.rs`

Add dir creation in `setup_logs()`:

```rust
fn setup_logs(bootstrap_service: &BootstrapService) -> Result<WorkerGuard, std::io::Error> {
  let logs_dir = bootstrap_service.logs_dir();
  fs::create_dir_all(&logs_dir)?;  // NEW — create if missing
  let file_appender = tracing_appender::rolling::daily(&logs_dir, "bodhi.log");
  ...
}
```

**File**: `crates/lib_bodhiserver_napi/src/server.rs`

Same pattern — `setup_logs()` creates the directory before the file appender.

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs`

Remove `setup_logs_dir()` function — logs dir is now created by `setup_logs()`.

After this step, `setup_app_dirs()` is lean:

```rust
pub fn setup_app_dirs(options: &AppOptions) -> Result<(PathBuf, SettingSource, HashMap<String, Value>), BootstrapError> {
  let file_defaults = load_defaults_yaml();
  let (bodhi_home, source) = create_bodhi_home(options.env_wrapper.clone(), &options.env_type, &file_defaults)?;
  Ok((bodhi_home, source, file_defaults))
}
```

---

## Step 4.6: Move runtime defaults from `BootstrapService` to `DefaultSettingService`

**Problem**: `BootstrapService` previously seeded ~15 runtime defaults (`BODHI_SCHEME`, `BODHI_HOST`, `BODHI_PORT`, `BODHI_EXEC_*`, `BODHI_LLAMACPP_ARGS`, `BODHI_KEEP_ALIVE_SECS`, `BODHI_CANONICAL_REDIRECT`, `HF_HOME`) that it never uses. These flowed through `BootstrapParts` to `DefaultSettingService`.

**Solution**: Move runtime default seeding into `DefaultSettingService::build_all_defaults()` (the helper called by `from_parts()`).

**File**: `crates/services/src/setting_service/default_service.rs`

```rust
fn build_all_defaults(env_wrapper: &dyn EnvWrapper, file_defaults: &HashMap<String, Value>) -> HashMap<String, Value> {
  let mut defaults = file_defaults.clone();
  // Use entry().or_insert() to preserve file_defaults values
  if let Some(home_dir) = env_wrapper.home_dir() {
    defaults.entry(HF_HOME.to_string())
      .or_insert(Value::String(home_dir.join(".cache/huggingface").display().to_string()));
  }
  defaults.entry(BODHI_SCHEME.to_string()).or_insert(Value::String(DEFAULT_SCHEME.to_string()));
  defaults.entry(BODHI_HOST.to_string()).or_insert(Value::String(DEFAULT_HOST.to_string()));
  defaults.entry(BODHI_PORT.to_string()).or_insert(Value::Number(DEFAULT_PORT.into()));
  // ... all other runtime defaults
  defaults
}
```

**File**: `crates/services/src/setting_service/bootstrap_service.rs`

`BootstrapService::new()` no longer seeds runtime defaults. It only reads/resolves the 4 bootstrap-critical values from:
- `system_settings` (for `bodhi_home`)
- `env_wrapper` (for log settings)
- `settings_file` (for log settings fallback)

**BootstrapService after cleanup**:
```
Fields: bodhi_home, logs_dir, log_level, log_stdout (+ passthrough state)
Typed accessors: bodhi_home(), logs_dir(), log_level(), log_stdout()
No generic methods. No RwLocks.
```

---

## Step 4.7: Extract `constants.rs` in `services` crate

**File**: `crates/services/src/setting_service/constants.rs` (**new**)

Move all `pub const BODHI_*` and `pub const HF_HOME` string constants out of `mod.rs` into a dedicated `constants.rs`. Keep `SETTING_VARS` slice there too.

Strip `setting_service/mod.rs` down to module declarations and re-exports only.

---

## Step 4.8: Consolidate `setting_metadata_static` as a `DefaultSettingService` static method

**File**: `crates/services/src/setting_service/default_service.rs`

```rust
impl DefaultSettingService {
  pub fn setting_metadata_static(key: &str) -> SettingMetadata {
    match key {
      BODHI_PORT => SettingMetadata::Number { min: Some(0), max: Some(65535) },
      BODHI_LOG_STDOUT | BODHI_CANONICAL_REDIRECT | BODHI_ON_RUNPOD => SettingMetadata::Boolean,
      BODHI_LOG_LEVEL => SettingMetadata::Option {
        options: LogLevel::variants().iter().map(|l| l.to_string()).collect(),
      },
      // etc.
      _ => SettingMetadata::String,
    }
  }
}
```

Remove the previously added module-level `setting_metadata_static()` free function. Update all callers (`routes_app`) to call `DefaultSettingService::setting_metadata_static(key)` or use `setting_service.get_setting_metadata(key).await`.

---

## Step 4.9: `routes_app` — use `setting_service.get_setting_metadata()` in handlers

**File**: `crates/routes_app/src/routes_settings/route_settings.rs`

Replace direct calls to the free function with the async trait method:

```rust
let metadata = setting_service.get_setting_metadata(&key).await;
```

---

## Step 4.10: Minor fixes from review

**`objs/src/envs.rs`**: Remove unused `PartialOrd` from `Setting` and `SettingInfo` derives (not needed for comparison).

**`services/src/setting_service/error.rs`**: Add `InvalidKey` error variant for rejected DB key write attempts.

**`lib_bodhiserver/src/error.rs`**: Remove dead `HfHomeNotFound` variant from `BootstrapError` (no longer needed after HF dir moves to `HfHubService`).

**`lib_bodhiserver/src/app_service_builder.rs`**: Use `PROD_DB`, `BODHI_ENV_TYPE`, `BODHI_ENCRYPTION_KEY` constants instead of inline string literals.

---

## Full Validation

```
make test.backend
```

---

## Summary: Before vs After

### `BootstrapService`

| | Before | After |
|--|--------|-------|
| Fields | bodhi_home, logs_dir, hf_home, log_level, log_stdout + passthrough + RwLocks | bodhi_home, logs_dir, log_level, log_stdout + passthrough (no RwLocks) |
| Typed accessors | bodhi_home(), app_db_path(), session_db_path(), logs_dir(), hf_home(), log_level(), log_stdout() + 50+ generic methods | bodhi_home(), logs_dir(), log_level(), log_stdout() only |
| Generic methods | get_setting(), set_setting_with_source(), etc. | None |
| Defaults seeded | ~18 (all runtime settings) | 0 (none — moved to DefaultSettingService) |

### `AppServiceBuilder`

| | Before | After |
|--|--------|-------|
| Struct fields | 13 (6 injectable services + 7 others) | 4 (bootstrap_parts, time_service, secret_service, cache_service) |
| Duplicate methods | get_or_build_db_service + _early, get_or_build_encryption_key + _from_parts | build_db_service, build_encryption_key (plain fns, no duplication) |
| Unused setters | 6 (hub, data, db, session, auth, ai_api) | 0 (removed) |

### `setup_app_dirs()`

| | Before | After |
|--|--------|-------|
| Steps | 6: load_defaults, create_bodhi_home, setup_bootstrap_service, setup_bodhi_subdirs, setup_hf_home, setup_logs_dir | 2: load_defaults, create_bodhi_home → returns (PathBuf, SettingSource, HashMap) |
| SQLite files | Created by `setup_bodhi_subdirs()` via `File::create` | Created by `DbPool::connect(create_if_missing)` |
| HF cache dir | Created by `setup_hf_home()` | Created by `HfHubService::new_from_hf_cache()` |
| Logs dir | Created by `setup_logs_dir()` | Created by `setup_logs()` before file appender |

### Key Files Changed

| File | Change |
|------|--------|
| `services/src/db/sqlite_pool.rs` | `connect()` uses `create_if_missing(true)` |
| `services/src/setting_service/constants.rs` | **New** — extracted constants |
| `services/src/setting_service/default_service.rs` | `build_all_defaults()` with runtime defaults; `setting_metadata_static()` as associated method |
| `services/src/setting_service/bootstrap_service.rs` | Stripped to 4 typed accessors only |
| `services/src/hub_service/service.rs` | `new_from_hf_cache()` returns `Result`, creates dirs |
| `lib_bodhiserver/src/app_dirs_builder.rs` | Remove setup_bodhi_subdirs, setup_hf_home, setup_logs_dir |
| `lib_bodhiserver/src/app_service_builder.rs` | 13→4 fields; unified build_* fns; use constants |
| `lib_bodhiserver/src/error.rs` | Remove dead `HfHomeNotFound` |
| `bodhi/src-tauri/src/server_init.rs` | `setup_logs()` creates logs dir |
| `lib_bodhiserver_napi/src/server.rs` | `setup_logs()` creates logs dir |
| `routes_app/src/routes_settings/route_settings.rs` | Use `setting_service.get_setting_metadata().await` |
| `objs/src/envs.rs` | Remove unused `PartialOrd` from Setting/SettingInfo |
