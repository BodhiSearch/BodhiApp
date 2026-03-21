# lib_bodhiserver -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, error types, file index

## Purpose

Embeddable server library: service composition (`AppServiceBuilder`), application directory setup (`setup_app_dirs`), configuration management (`AppOptions`/`AppOptionsBuilder`), and re-exports for downstream crates.

## Architecture Position

**Upstream**: `services`, `server_core`, `routes_app`, `server_app`
**Downstream**: `bodhi/src-tauri`, `lib_bodhiserver_napi`

## Key Components

### AppServiceBuilder (`src/app_service_builder.rs`)
- `AppServiceBuilder::new(bootstrap_parts: BootstrapParts)` -- sole constructor (NOT `BootstrapService`)
- Injectable services: `time_service`, `cache_service` (return `Err(ServiceAlreadySet)` on double-set)
- `build()` -- two-phase async build returning `DefaultAppService`
  - **Phase 1**: Extract `is_production`, build encryption key (keyring or hash), build `DbService`, build `DefaultSettingService::from_parts()`
  - **Phase 2**: Build remaining services using `setting_service` for config. Multi-tenant mode: `MultiTenantDataService` + `MultitenantInferenceService`; standalone: `LocalDataService` + `StandaloneInferenceService`
- `build_app_service(bootstrap_parts)` -- convenience wrapper

### setup_app_dirs (`src/app_dirs_builder.rs`)
- `setup_app_dirs(options: &AppOptions)` -- returns `(PathBuf, SettingSource, HashMap<String, Value>)` (bodhi_home, source, file_defaults)
- `setup_bootstrap_service(options, bodhi_home, source, file_defaults, command)` -- returns `BootstrapService`
- `create_bodhi_home()` -- resolves: env var > defaults.yaml > `~/.bodhi[-dev]`

### BootstrapService (`src/bootstrap_service.rs`)
- Holds pre-computed bootstrap-critical values: `bodhi_home`, `logs_dir`, `log_level`, `log_stdout`
- `new(env_wrapper, system_settings, file_defaults, settings_file, app_settings, app_command)` -- reads settings YAML once for log resolution
- `into_parts()` -- converts to `BootstrapParts` for `AppServiceBuilder`
- Accessors: `bodhi_home()`, `logs_dir()`, `log_level()`, `log_stdout()`

### AppOptions / AppOptionsBuilder (`src/app_options.rs`)
- `AppOptions` -- sealed config consumed by `setup_app_dirs`: `env_wrapper`, `env_type`, `app_type`, `app_version`, `app_commit_sha`, `auth_url`, `auth_realm`, `deployment_mode`, `app_settings`, `tenant: Option<Tenant>`
- `AppOptionsBuilder` -- chainable: `set_env()`, `set_app_setting()`, `set_system_setting()` (validates known keys), `set_tenant()`, `build()` validates required fields

### Re-exports (`src/lib.rs`)
Re-exports curated surface from `services`, `routes_app`, `server_app` so downstream crates only depend on `lib_bodhiserver`:
- `ApiError`, `OpenAIApiError` from `routes_app`
- Service traits/impls, setting constants, `AppCommand`, `AppType`, `EnvType`
- `ServeCommand`, `ServeError`, `ServerShutdownHandle` from `server_app`
- `EMBEDDED_UI_ASSETS` -- compile-time Vite frontend embed via `include_dir!`
- `BUILD_COMMIT_SHA` -- captured at build time

### Error Types (`src/error.rs`)
`BootstrapError` -- unified enum with variants:
`BodhiHomeNotResolved`, `DirCreate`, `BodhiHomeNotSet`, `ValidationError`, `Parse`, `UnknownSystemSetting`, `ServiceAlreadySet`, `PlaceholderValue`, `MissingBootstrapParts`, `SettingNotFound`, `Db`, `Tenant`, `SessionService`, `Keyring`, `Io`

## Test Utils (`src/test_utils/`, feature-gated)
- `AppOptionsBuilder::development()` -- dev defaults (Development, Container, test auth URL)
- `AppOptionsBuilder::with_bodhi_home(path)` -- development builder with custom BODHI_HOME

## Commands

```bash
cargo test -p lib_bodhiserver
cargo test -p lib_bodhiserver --features test-utils
```
