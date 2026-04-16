# lib_bodhiserver -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, error types, file index
- `tests-js/CLAUDE.md` -- E2E Playwright suite (runs against `bodhiserver_dev`)
- `tests-js/E2E.md` -- E2E test writing conventions

## Purpose

Embeddable server library: service composition (`AppServiceBuilder`), application directory setup (`setup_app_dirs`), configuration management (`AppOptions`/`AppOptionsBuilder`), and re-exports for downstream crates.

Also hosts:
- `src/bin/bodhiserver_dev.rs` — env-var-driven binary used by the Playwright suite. Forces `BODHI_DEV_PROXY_UI=true` and runs without embedded UI assets so iteration on Rust + Vite no longer requires rebuilding NAPI bindings.
- `tests-js/` — Playwright E2E tests + supporting MCP/OAuth fixture servers, migrated from `lib_bodhiserver_napi`.

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
- `BodhiErrorResponse` from `routes_app` (canonical Bodhi error envelope; OAI wire-format `OaiApiError` lives in `routes_app::oai` and is not re-exported here)
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

## Feature Flags

- `embed-ui` (default on) — gates `ui_assets::EMBEDDED_UI_ASSETS` and the frontend build step in `build.rs`. Disable with `--no-default-features` to skip the npm/Vite pipeline (used when building only `bodhiserver_dev`).
- `test-utils` — required by the `bodhiserver_dev` bin (`required-features = ["test-utils"]`) for `create_tenant_test`.

## Commands

```bash
cargo test -p lib_bodhiserver
cargo test -p lib_bodhiserver --features test-utils

# Build the dev binary without invoking npm/Vite
cargo build --no-default-features --features test-utils -p lib_bodhiserver --bin bodhiserver_dev
# Or via Make:
make build.dev-server

# Run the full Playwright matrix against the dev binary + live Vite
make test.e2e
```
