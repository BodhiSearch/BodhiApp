# PACKAGE.md - crates/bodhi/src-tauri

See [CLAUDE.md](CLAUDE.md) for architectural guidance and design rationale.

## Module Structure

- `src/main.rs` - Entry point; delegates to `initialize_and_execute(command)`
- `src/lib.rs` - Feature-conditional module inclusion (`native_init` vs `server_init`)
- `src/app.rs` - `AppCommand` enum, clap CLI definition, feature-conditional subcommands
- `src/native_init.rs` - Native desktop init: Tauri setup, system tray, `NativeCommand`, `NativeError`
- `src/server_init.rs` - Container/server init: logging setup, `set_feature_settings`, `ServeCommand`
- `src/common.rs` - `build_app_options(app_type)` shared by both modes
- `src/env.rs` - `ENV_TYPE`, `AUTH_URL`, `AUTH_REALM` constants switched by `production` feature
- `src/ui.rs` - Re-exports `lib_bodhiserver::EMBEDDED_UI_ASSETS as ASSETS`
- `src/error.rs` - `AppSetupError` (async runtime), `impl From<AppSetupError> for ErrorMessage`
- `src/error_test.rs` - Unit tests for error conversions

## Key Implementation Patterns

### Dual-Mode via Feature Flag

The `native` feature flag selects the initialization module at compile time (`src/lib.rs`). Both modes expose `initialize_and_execute(command: AppCommand) -> Result<(), ErrorMessage>` with the same signature. The `AppCommand` enum has two variants: `Serve { host: Option<String>, port: Option<u16> }` and `Default`. See `src/app.rs`.

### CLI Interface

Defined in `src/app.rs` using clap with derive macros. The `serve` subcommand accepts `-H`/`--host` and `-p`/`--port`. In native mode, the `serve` subcommand is hidden/absent and `Default` is used instead. Tests use rstest `#[case]` to parameterize valid and invalid CLI argument combinations.

### Native Desktop Mode (`src/native_init.rs`)

`initialize_and_execute` is called inside `tokio::task::block_in_place` because Tauri requires a multi-threaded runtime already running. The flow: build `AppOptions` → `setup_app_dirs` → `build_app_service` → launch Tauri with `NativeCommand`. `NativeCommand` holds `Arc<dyn AppService>` and a `ui: bool` flag. The Tauri setup callback creates a system tray with two menu items ("Open Homepage", "Quit"), wires `on_menu_event`, starts the embedded HTTP server via `ServeCommand`, and optionally opens the browser. Log level and stdout flag are read from `SettingService` (can access DB in native mode — intentional, since logging is configured inside the async context). See N14 comment in source.

### Container/Server Mode (`src/server_init.rs`)

`initialize_and_execute` spins up a `tokio` runtime via `tokio::runtime::Builder`. The flow: build `AppOptions` → `setup_app_dirs` → configure logging → apply command-line host/port overrides via `SettingService::set_setting_with_source(..., CommandLine)` → `build_app_service` → `set_feature_settings` → run `ServeCommand`. Logging uses `tracing_appender` with daily-rotating file output; log level is read from `BootstrapService` (env + YAML only, no DB access at this stage). `set_feature_settings` uses `ErrorType::InternalServer.to_string()` for the error type field in `ErrorMessage`.

### Configuration

`build_app_options(app_type)` in `src/common.rs` constructs `AppOptions` using `AppOptionsBuilder` with values from `src/env.rs`. The `production` feature flag switches `ENV_TYPE` between `Production`/`Development` and selects the appropriate `AUTH_URL` (`id.getbodhi.app` vs `main-id.getbodhi.app`).

## Error Types

| Enum            | Variants                                   | Notes                                                    |
| --------------- | ------------------------------------------ | -------------------------------------------------------- |
| `NativeError`   | `Tauri(tauri::Error)`, `Serve(ServeError)` | `src/native_init.rs`; code = `"tauri"` for Tauri variant |
| `AppSetupError` | `AsyncRuntime(io::Error)`                  | `src/error.rs`; async runtime spawn failure              |

Both implement `AppError` via `errmeta_derive::ErrorMeta`. `AppSetupError` also has `impl From<AppSetupError> for ErrorMessage`.

## Feature Flags

- `native` — Tauri desktop app (tauri, tauri-plugin-log, webbrowser); selects `native_init`
- `production` — switches auth endpoints and `ENV_TYPE` to production values
- `test-utils` — minimal test utilities foundation

## Commands

```bash
cargo tauri build --features native      # native desktop
cargo build -p bodhi                         # server/container
cargo test -p bodhi --features native
cargo test -p bodhi
```
