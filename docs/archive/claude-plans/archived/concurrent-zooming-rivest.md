# Plan: Settings-to-DB Review Fixes

## Context

After migrating settings persistence from YAML file to SQLite DB (commit 26feff08), a code review identified 15 findings across 4 crates. This plan addresses all 13 actionable findings (C1, I1-I7, N1-N5). The critical finding (C1) drives the largest architectural change: simplifying `BootstrapService` from a precomputing resolver to a narrow-scope data carrier that only resolves logging configuration.

**Key design decisions (from user):**
- BootstrapService scope narrowed to **only** resolving `logs_dir`, `log_level`, `log_stdout` from: real env vars + settings.yaml + convention
- No `.env` loading, no `defaults.yaml` usage in BootstrapService
- `AppCommand` enum moves to `objs` crate; flows via `setup_app_dirs()` constructor to `DefaultSettingService`
- `AppCommand` → seeds `cmd_lines` (CommandLine source); replaces `set_cli_host()`/`set_cli_port()`
- NAPI `app_settings` → merged with `settings.yaml` in memory at SettingsFile priority (not CommandLine)
- `settings.yaml` is **readonly** at runtime — loaded once into memory, never written
- `DefaultSettingService::new_with_defaults()` + `init_defaults()` → **removed entirely**
- `lib_bodhiserver_napi` is a test utility, not production — keep its `host()`/`port()` methods reading from `env_vars` as-is

---

## Phase 1: `objs` crate — Move AppCommand

**Files:**
- Create: `crates/objs/src/app_command.rs`
- Edit: `crates/objs/src/lib.rs`

**Changes:**
1. Define `AppCommand` enum in `objs`:
   ```rust
   #[derive(Debug, Clone)]
   pub enum AppCommand {
     Serve { host: Option<String>, port: Option<u16> },
     Default,
   }
   ```
   Note: rename `Server` → `Serve` for consistency with the CLI subcommand name. No clap dependency — callers construct it.

2. Re-export from `objs/src/lib.rs`

3. Remove `AppCommand` from `crates/bodhi/src-tauri/src/app.rs`, import from `objs` instead

**Verify:** `cargo test -p objs`

---

## Phase 2: `services` crate — BootstrapService + DefaultSettingService refactor

This is the core of C1/I3/I6. Multiple sub-steps.

### 2a. Simplify BootstrapService (C1, I6)

**File:** `crates/services/src/setting_service/bootstrap_service.rs`

**Current struct fields:**
```
env_wrapper, settings_file, system_settings, defaults, app_settings,
bodhi_home, logs_dir, log_level, log_stdout  (precomputed)
```

**New struct fields:**
```
// Passthrough (flows to DefaultSettingService via into_parts)
env_wrapper, settings_file, system_settings, file_defaults, app_settings, app_command, bodhi_home

// Resolved by BootstrapService (logs only)
logs_dir, log_level, log_stdout
```

**Constructor change:** `new(env_wrapper, bodhi_home, system_settings, file_defaults, settings_file, app_settings, app_command)`
- Does NOT load `.env`
- Does NOT build `defaults` HashMap
- Does NOT use `file_defaults` for its own resolution
- Resolves logs_* from this priority chain:
  1. Real env vars (`env_wrapper.var()`)
  2. `settings.yaml` file (single read)
  3. Convention: `logs_dir = bodhi_home/logs`, `log_level = "warn"`, `log_stdout = false`
- Stores `file_defaults` as passthrough data

**Remove:**
- `set_cli_host()`, `set_cli_port()` methods
- `load_default_env()` method (sync version)
- The `set_default!` macro and all defaults-building logic
- The full priority-chain `resolve` closure (overkill for 3 settings)

### 2b. Revise BootstrapParts

**File:** `crates/services/src/setting_service/bootstrap_service.rs`

**New BootstrapParts:**
```rust
pub struct BootstrapParts {
  pub env_wrapper: Arc<dyn EnvWrapper>,
  pub settings_file: PathBuf,
  pub system_settings: Vec<Setting>,
  pub file_defaults: HashMap<String, Value>,    // raw from defaults.yaml
  pub app_settings: HashMap<String, String>,    // NAPI programmatic overrides
  pub app_command: AppCommand,                  // CLI typed overrides
  pub bodhi_home: PathBuf,                      // for .env path
}
```

Removed fields: `defaults` (was computed), `cmd_lines` (was always empty)

### 2c. Refactor DefaultSettingService (C1, I3)

**File:** `crates/services/src/setting_service/default_service.rs`

**Remove entirely:**
- `new_with_defaults()` constructor
- `init_defaults()` method
- `with_settings_read_lock()` method (file reads on every call)
- `settings_lock: RwLock<()>` field (vestige, write side never used)

**New struct:**
```rust
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  system_settings: Vec<Setting>,
  cmd_lines: RwLock<HashMap<String, Value>>,              // from AppCommand
  settings_file_values: RwLock<HashMap<String, Value>>,   // loaded settings.yaml + app_settings overlay
  defaults: RwLock<HashMap<String, Value>>,                // all defaults
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  db_service: Arc<dyn SettingsRepository>,
}
```

Key change: `settings_file_values` replaces on-demand file reads. Loaded once at construction.

**Revised `from_parts()`:**
```rust
pub fn from_parts(parts: BootstrapParts, db_service: Arc<dyn SettingsRepository>) -> Self {
  // 1. Load .env from bodhi_home/.env (mutates process env)
  let env_file = parts.bodhi_home.join(".env");
  if env_file.exists() {
    parts.env_wrapper.load(&env_file);
  }

  // 2. Load settings.yaml once into memory
  let mut settings_file_values = load_settings_yaml(&parts.settings_file);

  // 3. Overlay NAPI app_settings onto settings_file_values (app_settings wins)
  for (key, value_str) in &parts.app_settings {
    let metadata = setting_metadata(key);
    let parsed = metadata.parse(Value::String(value_str.clone()));
    settings_file_values.insert(key.clone(), parsed);
  }

  // 4. Extract cmd_lines from AppCommand
  let mut cmd_lines = HashMap::new();
  if let AppCommand::Serve { host, port } = &parts.app_command {
    if let Some(h) = host {
      cmd_lines.insert(BODHI_HOST.to_string(), Value::String(h.clone()));
    }
    if let Some(p) = port {
      cmd_lines.insert(BODHI_PORT.to_string(), Value::Number((*p).into()));
    }
  }

  // 5. Build all defaults from file_defaults + hardcoded
  let defaults = build_all_defaults(parts.env_wrapper.as_ref(), &parts.file_defaults);

  Self { env_wrapper: parts.env_wrapper, system_settings: parts.system_settings,
    cmd_lines: RwLock::new(cmd_lines), settings_file_values: RwLock::new(settings_file_values),
    defaults: RwLock::new(defaults), listeners: RwLock::new(Vec::new()), db_service }
}
```

**New `build_all_defaults()` function** (in `mod.rs` or `defaults.rs`):
- Single source of truth for all defaults
- Combines what was spread across `BootstrapService::new_with_defaults()`, `init_defaults()`, and `ensure_runtime_defaults()`
- Uses `file_defaults` as base, overlays hardcoded defaults for missing keys
- Keys: BODHI_HOME, BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT, BODHI_SCHEME, BODHI_HOST, BODHI_PORT, BODHI_EXEC_*, BODHI_LLAMACPP_ARGS, BODHI_KEEP_ALIVE_SECS, BODHI_CANONICAL_REDIRECT, HF_HOME

**Revised `get_setting_value_with_source()`** — SettingsFile step changes from file read to in-memory lookup:
```rust
// 4. SettingsFile (in-memory, loaded once)
let result = self.settings_file_values.read().unwrap().get(key).cloned();
if let Some(value) = result {
  return (Some(metadata.parse(value)), SettingSource::SettingsFile);
}
```

**Remove `load_default_env()` from SettingService trait** — `.env` loading now happens in `from_parts()`. Update trait definition in `service.rs`. Update `SettingServiceStub` (remove no-op impl). Update `MockSettingService`. Update test callers in `server_app/tests/utils/live_server_utils.rs` (remove the `setting_service.load_default_env().await` calls — .env is already loaded).

### 2d. Change `set_setting_with_source` to return `Result<()>` (I1)

**Files:**
- `crates/services/src/setting_service/service.rs` — trait definition
- `crates/services/src/setting_service/default_service.rs` — implementation
- `crates/services/src/test_utils/envs.rs` — SettingServiceStub

**Changes:**
- `set_setting_with_source(&self, key, value, source) -> Result<(), SettingServiceError>`
- `set_setting_value(&self, key, value) -> Result<(), SettingServiceError>` (convenience)
- `set_setting(&self, key, value: &str) -> Result<(), SettingServiceError>` (convenience)
- `set_default(&self, key, value) -> Result<(), SettingServiceError>` (convenience)
- CommandLine/Default arms return `Ok(())`
- Database arm propagates `db_service.upsert_setting()` errors
- SettingsFile/Environment/System arms return `Err(SettingServiceError::InvalidSource)` instead of logging

### 2e. Eliminate upsert read-back (I7)

**File:** `crates/services/src/db/service_settings.rs`

**Change `upsert_setting()`:**
- Remove the `self.get_setting(&setting.key).await?` read-back after INSERT
- Construct return value directly from input + computed `now` timestamp:
  ```rust
  Ok(DbSetting { key: setting.key.clone(), value: setting.value.clone(),
    value_type: setting.value_type.clone(), created_at: now, updated_at: now })
  ```

### 2f. Add Database precedence tests (I2)

**File:** Create `crates/services/src/setting_service/test_service_db.rs`
**Declare:** `#[cfg(test)] #[path = "test_service_db.rs"] mod test_service_db;` in `mod.rs`

**Tests:**
- `test_database_over_default` — DB value present, no CLI/Env/File → returns DB value with `SettingSource::Database`
- `test_env_over_database` — Both Env and DB values → returns Env with `SettingSource::Environment`
- `test_file_over_database` — Both SettingsFile and DB values → returns SettingsFile

### 2g. Doc comments (N3, N5)

- **N3:** Add doc comment on `add_listener` method in `default_service.rs`:
  `/// Deduplication is based on Arc pointer equality. Separately allocated Arc instances wrapping equivalent implementations will not be deduplicated.`
- **N5:** Add comment on `SettingServiceStub::get_setting_value_with_source` in `test_utils/envs.rs`:
  `// Returns Database source for all found settings; stub does not distinguish source layers`

### 2h. Update SettingServiceStub

**File:** `crates/services/src/test_utils/envs.rs`

- Update `set_setting_with_source` to return `Result<(), SettingServiceError>`
- Update `set_setting_value`, `set_setting`, `set_default` signatures
- Remove `load_default_env` no-op

**Verify Phase 2:** `cargo test -p services`

---

## Phase 3: `lib_bodhiserver` crate

### 3a. Update `setup_app_dirs` to take AppCommand (I6)

**File:** `crates/lib_bodhiserver/src/app_dirs_builder.rs`

- Change signature: `pub fn setup_app_dirs(options: &AppOptions, command: AppCommand) -> Result<BootstrapService, ...>`
- Pass `command` through `setup_bootstrap_service()` to `BootstrapService::new()`
- Remove `set_cli_host`/`set_cli_port` calls from callers (handled by AppCommand now)

### 3b. Update AppServiceBuilder (I5, C1)

**File:** `crates/lib_bodhiserver/src/app_service_builder.rs`

- **I5:** Gate `with_setting_service()` with `#[cfg(any(test, feature = "test-utils"))]`
- Update `build()` to pass revised `BootstrapParts` (with `file_defaults`, `app_command`, `bodhi_home`) to `DefaultSettingService::from_parts()`
- Remove `.env` loading from builder (now in `from_parts()`)

### 3c. Update `build_app_service` signature

**File:** `crates/lib_bodhiserver/src/app_service_builder.rs`

- `build_app_service()` already takes `BootstrapService` which now carries `AppCommand` internally. No signature change needed for this function.

**Verify Phase 3:** `cargo test -p lib_bodhiserver`

---

## Phase 4: `routes_app` crate

### 4a. Refactor `update_setting_handler` — eliminate double `list()` (I4)

**File:** `crates/routes_app/src/routes_settings/route_settings.rs`

**Current flow:** `list()` → validate → write → `list()` → return
**New flow:**
1. Check `key == BODHI_HOME` → error
2. `setting_service.get_setting_value_with_source(&key).await` → validate key exists
3. Check `EDIT_SETTINGS_ALLOWED` → Unsupported error
4. Get metadata via `setting_metadata(&key)`, validate value with `metadata.convert()`
5. `setting_service.set_setting_value(&key, &value).await?` → propagate Result (I1)
6. `setting_service.get_setting_value_with_source(&key).await` → construct SettingInfo response directly
7. Return single SettingInfo (not from list)

Also apply same pattern to `delete_setting_handler`.

### 4b. Update OpenAPI docs (N1)

**File:** `crates/routes_app/src/routes_settings/route_settings.rs`

- Line ~100-103: Change "persisted to the settings file" → "persisted to the application database"
- Line ~119: Change `"source": "settings_file"` → `"source": "database"` in response example

### 4c. Add Unsupported error test (N2)

**File:** `crates/routes_app/src/routes_settings/test_settings.rs`

Add `test_routes_setting_update_unsupported_key`:
- PUT `/api/settings/BODHI_PORT` with valid value
- Assert HTTP 400, error code `settings_error-unsupported`

### 4d. Update test infrastructure

- `test_setting_service()` in `test_settings.rs` currently uses `DefaultSettingService::new_with_defaults()` (being removed)
- Refactor to use `DefaultSettingService::from_parts()` with mock `SettingsRepository` (reuse existing `noop_settings_repo()`)
- Construct `BootstrapParts` directly with test data

**Verify Phase 4:** `cargo test -p routes_app`

---

## Phase 5: Downstream callers

### 5a. `crates/bodhi/src-tauri/src/app.rs`

- Import `AppCommand` from `objs` instead of local definition
- Map clap `Commands::Serve { host, port }` → `AppCommand::Serve { host, port }`

### 5b. `crates/bodhi/src-tauri/src/server_init.rs`

- Remove `set_cli_host()`/`set_cli_port()` calls
- Pass `command` to `setup_app_dirs(&app_options, command)`
- Keep existing `setting_service.host()` / `setting_service.port()` reads for ServeCommand (already correct)

### 5c. `crates/bodhi/src-tauri/src/native_init.rs`

- Pass `AppCommand::Default` to `setup_app_dirs(&app_options, AppCommand::Default)`

### 5d. `crates/lib_bodhiserver_napi/src/server.rs`

- Pass `AppCommand::Default` to `setup_app_dirs(&app_options, AppCommand::Default)`
- Keep `host()`, `port()`, `server_url()` reading from `self.config.env_vars` (test utility, not production)
- Keep `ServeCommand::ByParams { host: self.host(), port: self.port() }` pattern (test-controlled env)

### 5e. `crates/server_app/tests/utils/live_server_utils.rs`

- Remove `setting_service.load_default_env().await` calls (`.env` now loaded in `from_parts()`)
- Pass `AppCommand::Default` where `setup_app_dirs()` is called

### 5f. Update `set_setting_value` / `set_setting` call sites

All callers of `set_setting_value()` / `set_setting()` must now handle `Result<()>`. Search for all call sites and add `.await?` or appropriate error handling.

**Verify Phase 5:** `make test.backend`

---

## Phase 6: OpenAPI + TypeScript

1. Regenerate OpenAPI spec: `cargo run --package xtask openapi`
2. Regenerate TypeScript client: `make build.ts-client`
3. Verify no breaking changes in `types.gen.ts`

---

## Summary: Finding → Fix Mapping

| # | Finding | Fix Location |
|---|---------|-------------|
| C1 | BootstrapService precomputes + loads .env | Phase 2a-2c |
| I1 | Silent DB write failures | Phase 2d |
| I2 | Missing DB precedence tests | Phase 2f |
| I3 | Duplicate defaults logic | Phase 2c (single `build_all_defaults()`) |
| I4 | Double `list()` in update handler | Phase 4a |
| I5 | `with_setting_service` not test-gated | Phase 3b |
| I6 | BootstrapService two-stage resolution | Phase 2a + 3a |
| I7 | Upsert read-back | Phase 2e |
| N1 | Stale OpenAPI "settings_file" refs | Phase 4b |
| N2 | Missing Unsupported error test | Phase 4c |
| N3 | Listener dedup doc comment | Phase 2g |
| N4 | NAPI independent config reading | Keep as-is (test utility) |
| N5 | Stub source comment | Phase 2g |

---

## Verification

1. After each phase: `cargo test -p <crate>` for the affected crate
2. After Phase 5: `make test.backend` (full backend)
3. After Phase 6: `make build.ts-client` (TypeScript types)
4. Manual smoke test: `cargo run --bin bodhi -- serve --port 1135` — verify server starts, settings API works
