# PACKAGE.md - lib_bodhiserver

See [CLAUDE.md](CLAUDE.md) for architectural guidance and design rationale.

## Module Structure

- `src/lib.rs` - Crate root, pub re-exports of services/objs/server_app symbols
- `src/bootstrap_service.rs` - `BootstrapService` struct and impl; owns pre-computed bootstrap-critical values (bodhi_home, logs_dir, log_level, log_stdout) and converts to `BootstrapParts` via `into_parts()`
- `src/app_service_builder.rs` - `AppServiceBuilder`, `build_app_service()`, `update_with_option()`
- `src/app_dirs_builder.rs` - `setup_app_dirs()`, `create_bodhi_home()`, `setup_bootstrap_service()`
- `src/app_options.rs` - `AppOptions`, `AppOptionsBuilder` (builder pattern with derive-new)
- `src/error.rs` - `AppOptionsError`, `AppDirsBuilderError`, `AppServiceBuilderError`
- `src/ui_assets.rs` - `EMBEDDED_UI_ASSETS` (compile-time Next.js embed via `include_dir!`)
- `src/test_utils/` - `AppOptionsBuilder::development()`, `AppOptionsBuilder::with_bodhi_home()`

## Key Implementation Patterns

### AppServiceBuilder — Two-Phase Build

`AppServiceBuilder::new(bootstrap_service)` is the sole constructor; there are no alternative constructors. All other services (`hub_service`, `db_service`, etc.) are optional `Option<Arc<dyn Trait>>` fields injected for testing via chainable setter methods that return `Err(ServiceAlreadySet)` on double-set.

`build()` runs in two phases (see `src/app_service_builder.rs`):

- **Phase 1** — Destructure `BootstrapService` via `into_parts()`. Synchronously extract `is_production` (by matching `BODHI_ENV_TYPE == "production"` in `system_settings`) and `encryption_key_value` (via `env_wrapper.var(BODHI_ENCRYPTION_KEY)`). Build `encryption_key` (keyring or hash), then `DbService` using `bodhi_home.join(PROD_DB)`. Construct `DefaultSettingService::from_parts(bootstrap_parts, db_service)`.

- **Phase 2** — Build remaining services (`hub`, `data`, `session`, `secret`, `auth`, `ai_api`, `tool`, `mcp`, `access_request`, etc.) using `setting_service` for configuration. Each `get_or_build_*` method returns the injected override if present, otherwise constructs the default. `get_or_build_db_service` takes a `db_path: PathBuf` argument directly (not read from `setting_service`).

The public helper `build_app_service(bootstrap_service)` wraps `AppServiceBuilder::new(bs).build().await`.

### setup_app_dirs — Directory Bootstrap

`setup_app_dirs(options, command)` in `src/app_dirs_builder.rs` returns `BootstrapService` (not `DefaultSettingService`):

1. Load `defaults.yaml` from the executable directory (optional, silently empty if missing).
2. Resolve `BODHI_HOME`: check env var → `defaults.yaml` → `~/.cache/bodhi[-dev]` → error `BodhiHomeNotFound`.
3. Create the `BODHI_HOME` directory if absent.
4. Call `setup_bootstrap_service()` which assembles a `Vec<Setting>` of system settings (BODHI_HOME, BODHI_ENV_TYPE, BODHI_APP_TYPE, BODHI_VERSION, BODHI_COMMIT_SHA, BODHI_AUTH_URL, BODHI_AUTH_REALM) and constructs `BootstrapService::new(...)`. The `BootstrapService` internally loads and merges the YAML settings file; subdirectories (aliases, db, hf_home, logs) are created when `BootstrapService::setup_dirs()` is called.

### AppOptions / AppOptionsBuilder

`AppOptions` is the sealed configuration object consumed by `setup_app_dirs`. `AppOptionsBuilder` (derive-new, builder pattern) exposes:
- `set_env(key, value)` — injects environment variables into the inner `EnvWrapper`
- `set_app_setting(key, value)` — command-line overrides applied at `CommandLine` source priority
- `set_system_setting(key, value)` — validates and sets typed fields (`env_type`, `app_type`, `app_version`, `auth_url`, `auth_realm`); returns `UnknownSystemSetting` for unrecognized keys

Test helpers in `src/test_utils/` provide `AppOptionsBuilder::development()` (development env type, Container app type, test auth URL) and `AppOptionsBuilder::with_bodhi_home(path)`.

### Re-exports

`src/lib.rs` re-exports a curated surface from `services`, `objs`, and `server_app` so downstream crates (`bodhi/src-tauri`, `lib_bodhiserver_napi`) only need to depend on `lib_bodhiserver`. `BootstrapService` is defined in this crate (`src/bootstrap_service.rs`) and re-exported directly. Key re-exported groups: all `BODHI_*` / `DEFAULT_*` / `HF_*` constants, `DefaultSettingService`, `DefaultAppService`, `AppService`, `SettingService`, `ServeCommand`, `ServeError`, `ServerShutdownHandle`, `ApiError`, `ErrorMessage`, `ErrorType`, `AppType`, `EnvType`.

## Error Types

All error enums derive `thiserror::Error` + `errmeta_derive::ErrorMeta` and implement `AppError`. Each also has `impl From<XError> for ErrorMessage` for conversion to the wire format. See `src/error.rs`.

| Enum | Variants | ErrorType |
|------|----------|-----------|
| `AppOptionsError` | `ValidationError(String)`, `Parse(strum::ParseError)`, `UnknownSystemSetting(String)` | BadRequest |
| `AppDirsBuilderError` | `BodhiHomeNotFound`, `DirCreate { source, path }`, `IoFileWrite { source, path }`, `SettingServiceError`, `BootstrapBodhiHomeNotFound` | InternalServer |
| `AppServiceBuilderError` | `ServiceAlreadySet(String)`, `PlaceholderValue(String)` | InternalServer / BadRequest |

## Commands

```bash
cargo test -p lib_bodhiserver
cargo test -p lib_bodhiserver app_service_builder
cargo test -p lib_bodhiserver --features test-utils
```
