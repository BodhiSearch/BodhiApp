# bodhi/src-tauri — CLAUDE.md

**Companion docs** (load as needed):

- `PACKAGE.md` — Module structure, error types, feature flags, commands

## Purpose

Unified application entry point for BodhiApp with dual-mode deployment via `native` feature flag: Tauri desktop app or headless container server.

## Architecture Position

Leaf crate — consumes foundation crates, not consumed by any other Rust crate.

**Upstream dependencies**:

- `lib_bodhiserver` — `build_app_service()`, `setup_app_dirs()`, `setup_bootstrap_service()`, `ServeCommand`, `AppCommand`, `EMBEDDED_UI_ASSETS`
- `services` (via lib_bodhiserver re-exports) — domain types, `AppType`, `EnvType`, `SettingService`
- `errmeta_derive` — `#[derive(ErrorMeta)]` for error types

## Dual-Mode Architecture

### Feature Flag: `native`

`src/lib.rs` conditionally includes `native_init` or `server_init`. Both expose `initialize_and_execute(command: AppCommand) -> Result<(), AppSetupError>`. `AppCommand` (defined in `services`, re-exported via `lib_bodhiserver`) has variants: `Serve { host, port }` and `Default`.

### Native Desktop Mode (`src/native_init.rs`)

Flow: `build_app_options` → `setup_app_dirs` → `setup_bootstrap_service` → create multi-threaded tokio runtime → `build_app_service` → `NativeCommand::aexecute`.

`NativeCommand` holds `Arc<dyn AppService>` and `ui: bool`. The Tauri setup callback:

- Sets `ActivationPolicy::Accessory` on macOS
- Configures `BODHI_EXEC_LOOKUP_PATH` from Tauri resource dir
- Starts embedded HTTP server via `ServeCommand::get_server_handle`
- Creates system tray with "Open Homepage" and "Quit" menu items
- Opens browser if `ui: true`
- Window close hides instead of quitting (`on_window_event`)

Log level and stdout flag read from `SettingService` (can access DB in native mode — intentional, since logging is configured inside the async context).

### Container/Server Mode (`src/server_init.rs`)

Flow: `build_app_options` → `setup_app_dirs` → `setup_bootstrap_service` → `setup_logs` (tracing with daily-rotating file appender) → create tokio runtime → `build_app_service` → `set_feature_settings` → `ServeCommand::aexecute`.

Log level read from `BootstrapService` (env + YAML only, no DB access at this stage). `set_feature_settings` in dev mode sets `BODHI_EXEC_LOOKUP_PATH` to `CARGO_MANIFEST_DIR/bin` if not already set.

### CLI (`src/app.rs`)

Clap-derived CLI. `serve` subcommand (with `-H`/`--host`, `-p`/`--port`) only available when `native` feature is NOT enabled. In native mode, only `Default` command is used. `pub fn main(args)` is the CLI-first entry point.

### Configuration (`src/common.rs`, `src/env.rs`)

`build_app_options(app_type)` constructs `AppOptions` via `AppOptionsBuilder`. The `production` feature flag switches:

- `ENV_TYPE`: `Production` vs `Development`
- `AUTH_URL`: `id.getbodhi.app` vs `main-id.getbodhi.app`
- `AUTH_REALM`: `bodhi` (both)

### Embedded UI (`src/ui.rs`)

Re-exports `lib_bodhiserver::EMBEDDED_UI_ASSETS as ASSETS`.

## Error Types

| Enum            | Variants                                                                                                           | Location             |
| --------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------- |
| `AppSetupError` | `Bootstrap(BootstrapError)`, `AsyncRuntime(io::Error)`, `Serve(ServeError)`, `SettingService(SettingServiceError)` | `src/error.rs`       |
| `NativeError`   | `Tauri(tauri::Error)`, `Serve(ServeError)`                                                                         | `src/native_init.rs` |

Both implement `AppError` via `errmeta_derive::ErrorMeta`.

## Commands

```bash
cargo tauri build --features native      # native desktop
cargo build -p bodhi                     # server/container
cargo test -p bodhi --features native    # test native mode
cargo test -p bodhi                      # test server mode
```
