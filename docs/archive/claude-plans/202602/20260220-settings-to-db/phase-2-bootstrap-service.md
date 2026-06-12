# Phase 2: Introduce BootstrapService — Decouple Init from DB

## Goal

The current flow requires `SettingService` (which needs `DbService`) to exist before directories can be set up — a chicken-and-egg problem. Solve it by introducing a sync `BootstrapService` for the pre-DB phase, then constructing `DefaultSettingService` later in `AppServiceBuilder` once `DbService` is available.

## Motivation

`DefaultSettingService::from_parts()` will need a `SettingsRepository` (backed by SQLite). But SQLite can't be connected until `BODHI_HOME` is known — and `BODHI_HOME` is currently resolved by `BootstrapService` acting as a `SettingService`. Breaking the two roles apart eliminates the circular dependency.

---

## Application Lifecycle After This Phase

```
Phase 1 (sync):  setup_app_dirs()          → BootstrapService
Phase 2 (sync):  setup_logs(bootstrap)     → reads logs_dir, log_level, log_stdout
Phase 3 (async): AppServiceBuilder::build() → connects SQLite, creates DefaultSettingService
Phase 4 (async): route handlers            → use SettingService normally
```

---

## Step 2.1: Define `BootstrapService` and `BootstrapParts` — `services` crate

**New file**: `crates/services/src/setting_service/bootstrap_service.rs`

```rust
pub struct BootstrapParts {
  pub env_wrapper: Arc<dyn EnvWrapper>,
  pub settings_file: PathBuf,
  pub system_settings: Vec<Setting>,
  pub file_defaults: HashMap<String, Value>,
  pub app_settings: HashMap<String, String>,
  pub app_command: AppCommand,
  pub bodhi_home: PathBuf,
}

#[derive(Debug)]
pub struct BootstrapService {
  // Passthrough state (flows to SettingService via into_parts)
  env_wrapper: Arc<dyn EnvWrapper>,
  settings_file: PathBuf,
  system_settings: Vec<Setting>,
  file_defaults: HashMap<String, Value>,
  app_settings: HashMap<String, String>,
  app_command: AppCommand,

  // Pre-computed bootstrap-critical values (eager construction)
  bodhi_home: PathBuf,
  logs_dir: PathBuf,
  log_level: LogLevel,
  log_stdout: bool,
}
```

**Constructor** `BootstrapService::new(...)`:
- Receives `system_settings` (already contains BODHI_HOME as a `Setting` with `System` source)
- Reads `settings_file` once for log-related resolution
- Priority chain for each value: System > Env > settings.yaml > hardcoded default
- `bodhi_home`: extracted from `system_settings` (always present)
- `logs_dir`: `env(BODHI_LOGS)` > `settings.yaml[BODHI_LOGS]` > `bodhi_home/logs`
- `log_level`: `env(BODHI_LOG_LEVEL)` > `settings.yaml[BODHI_LOG_LEVEL]` > `"warn"`
- `log_stdout`: `env(BODHI_LOG_STDOUT)` > `settings.yaml[BODHI_LOG_STDOUT]` > `false`

**Methods**:
- `bodhi_home() -> PathBuf`
- `logs_dir() -> PathBuf`
- `log_level() -> LogLevel`
- `log_stdout() -> bool`
- `into_parts(self) -> BootstrapParts`

**Error**: Add `BootstrapServiceError::BodhiHomeNotFound` (when the system setting is absent/invalid).

**Export**: `pub use bootstrap_service::{BootstrapService, BootstrapParts}` from `setting_service/mod.rs`.

**Gate**: `cargo test -p services`

---

## Step 2.2: Add `DefaultSettingService::from_parts()` — `services` crate

**File**: `crates/services/src/setting_service/default_service.rs`

New constructor alongside the existing `new_with_defaults`:

```rust
pub fn from_parts(parts: BootstrapParts, db_service: Arc<dyn SettingsRepository>) -> Self {
  // 1. Load .env from bodhi_home/.env
  let env_file = parts.bodhi_home.join(".env");
  if env_file.exists() {
    parts.env_wrapper.load(&env_file);
  }

  // 2. Load settings.yaml into memory
  let mut settings_file_values = load_settings_yaml(&parts.settings_file);

  // 3. Overlay app_settings (NAPI/CLI config) onto settings_file_values
  for (key, value_str) in &parts.app_settings {
    let metadata = Self::setting_metadata_static(key);
    let parsed = metadata.parse(Value::String(value_str.clone()));
    settings_file_values.insert(key.clone(), parsed);
  }

  // 4. Extract cmd_lines from AppCommand (Serve host/port)
  let mut cmd_lines = HashMap::new();
  if let AppCommand::Serve { ref host, ref port } = parts.app_command {
    if let Some(h) = host {
      cmd_lines.insert(BODHI_HOST.to_string(), Value::String(h.clone()));
    }
    if let Some(p) = port {
      cmd_lines.insert(BODHI_PORT.to_string(), Value::Number((*p).into()));
    }
  }

  // 5. Build all defaults from file_defaults + hardcoded runtime defaults
  let defaults = build_all_defaults(parts.env_wrapper.as_ref(), &parts.file_defaults);

  Self {
    env_wrapper: parts.env_wrapper,
    system_settings: parts.system_settings,
    cmd_lines: RwLock::new(cmd_lines),
    settings_file_values: RwLock::new(settings_file_values),
    defaults: RwLock::new(defaults),
    listeners: RwLock::new(Vec::new()),
    db_service,
  }
}
```

The `build_all_defaults()` helper seeds runtime defaults (`BODHI_SCHEME`, `BODHI_HOST`, `BODHI_PORT`, `HF_HOME`, etc.) using `entry().or_insert()` so `file_defaults` values are preserved.

**Gate**: `cargo test -p services`

---

## Step 2.3: Refactor `setup_app_dirs()` — `lib_bodhiserver` crate

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs`

Split into two public functions:

```rust
/// Step A: Create BODHI_HOME directory, load file defaults.
/// Returns (bodhi_home, source, file_defaults) for use with setup_bootstrap_service().
pub fn setup_app_dirs(
  options: &AppOptions,
) -> Result<(PathBuf, SettingSource, HashMap<String, Value>), BootstrapError> {
  let file_defaults = load_defaults_yaml();
  let (bodhi_home, source) = create_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  Ok((bodhi_home, source, file_defaults))
}

/// Step B: Build BootstrapService from the parts returned by setup_app_dirs().
pub fn setup_bootstrap_service(
  options: &AppOptions,
  bodhi_home: PathBuf,
  source: SettingSource,
  file_defaults: HashMap<String, Value>,
  command: AppCommand,
) -> Result<BootstrapService, BootstrapError> {
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let system_settings = build_system_settings(options, &bodhi_home, &source, &file_defaults);
  Ok(BootstrapService::new(
    options.env_wrapper.clone(),
    system_settings,
    file_defaults,
    settings_file,
    options.app_settings.clone(),
    command,
  )?)
}
```

Remove the old monolithic `setup_app_dirs()` that returned `BootstrapService` directly.

**All three callers** update to two-step pattern:
- `crates/bodhi/src-tauri/src/server_init.rs`
- `crates/bodhi/src-tauri/src/native_init.rs`
- `crates/lib_bodhiserver_napi/src/server.rs`

**Gate**: `cargo test -p lib_bodhiserver`

---

## Step 2.4: Refactor `AppServiceBuilder` — `lib_bodhiserver` crate

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

`AppServiceBuilder::new()` takes `BootstrapParts` directly (not `BootstrapService`):

```rust
pub struct AppServiceBuilder {
  bootstrap_parts: Option<BootstrapParts>,
  time_service: Option<Arc<dyn TimeService>>,
  secret_service: Option<Arc<dyn SecretService>>,
  cache_service: Option<Arc<dyn CacheService>>,
}

impl AppServiceBuilder {
  pub fn new(bootstrap_parts: BootstrapParts) -> Self { ... }
}
```

In `build()`:
1. Take `bootstrap_parts` from `self.bootstrap_parts`
2. Build `DbService` from `bodhi_home.join(PROD_DB)` (after `DbPool::connect`)
3. Call `DefaultSettingService::from_parts(parts, db_service.clone())`
4. Use `setting_service` for the rest of service construction

Convenience function at module level:

```rust
pub async fn build_app_service(
  bootstrap_parts: BootstrapParts,
) -> Result<DefaultAppService, ErrorMessage> {
  AppServiceBuilder::new(bootstrap_parts).build().await
}
```

Update `lib_bodhiserver/src/lib.rs` to re-export `BootstrapParts`.

**Gate**: `cargo test -p lib_bodhiserver`

---

## Step 2.5: Update callers — Tauri + NAPI

**File**: `crates/bodhi/src-tauri/src/server_init.rs`

```rust
pub fn run_server(options: AppOptions, command: ServerCommand) -> Result<(), AppSetupError> {
  let (bodhi_home, source, file_defaults) = setup_app_dirs(&options)?;
  let mut bootstrap = setup_bootstrap_service(&options, bodhi_home, source, file_defaults, command.into())?;
  let _guard = setup_logs(&bootstrap)?;

  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;

  runtime.block_on(async {
    let app_service = build_app_service(bootstrap.into_parts()).await?;
    // host/port resolved from SettingService, not BootstrapService
    let host = app_service.setting_service().host().await;
    let port = app_service.setting_service().port().await;
    set_feature_settings(app_service.setting_service().as_ref()).await;
    ServeCommand::ByParams { host, port }.aexecute(app_service, ...).await
  })
}
```

`setup_logs()` reads `bootstrap.logs_dir()`, `bootstrap.log_level()`, `bootstrap.log_stdout()` — all sync typed accessors.

**File**: `crates/lib_bodhiserver_napi/src/server.rs`

Same split pattern. `setup_logs()` uses typed accessors on `BootstrapService`.

**Gate**: `cargo test -p bodhi && cargo test -p lib_bodhiserver_napi`

---

## Full Validation

```
make test.backend
```

---

## Key Files Changed

| File | Change |
|------|--------|
| `services/src/setting_service/bootstrap_service.rs` | **New file** — `BootstrapService` + `BootstrapParts` |
| `services/src/setting_service/default_service.rs` | Add `from_parts()` constructor |
| `services/src/setting_service/mod.rs` | Export `BootstrapService`, `BootstrapParts` |
| `lib_bodhiserver/src/app_dirs_builder.rs` | Split into `setup_app_dirs()` + `setup_bootstrap_service()` |
| `lib_bodhiserver/src/app_service_builder.rs` | Accept `BootstrapParts`, call `from_parts()` |
| `lib_bodhiserver/src/lib.rs` | Re-export `BootstrapParts` |
| `bodhi/src-tauri/src/server_init.rs` | Two-step setup, async block for host/port |
| `bodhi/src-tauri/src/native_init.rs` | Two-step setup |
| `lib_bodhiserver_napi/src/server.rs` | Two-step setup, typed log accessors |
