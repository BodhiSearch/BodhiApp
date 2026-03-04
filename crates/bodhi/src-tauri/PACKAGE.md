# bodhi/src-tauri — PACKAGE.md

See [CLAUDE.md](CLAUDE.md) for architectural guidance.

## Module Structure

- `src/main.rs` — Entry point; calls `bodhi::app::main(&args)`
- `src/lib.rs` — Feature-conditional module inclusion (`native_init` vs `server_init`)
- `src/app.rs` — `Cli` struct (clap), `Commands` enum, `pub fn main(args)` entry, re-exports `lib_bodhiserver::AppCommand`
- `src/native_init.rs` — Native desktop init: Tauri setup, system tray, `NativeCommand`, `NativeError`
- `src/server_init.rs` — Container/server init: logging setup, `set_feature_settings`, `ServeCommand`
- `src/common.rs` — `build_app_options(app_type)` shared by both modes
- `src/env.rs` — `ENV_TYPE`, `AUTH_URL`, `AUTH_REALM` constants switched by `production` feature
- `src/ui.rs` — Re-exports `lib_bodhiserver::EMBEDDED_UI_ASSETS as ASSETS`
- `src/error.rs` — `AppSetupError` enum

## Feature Flags

- `native` — Tauri desktop app (tauri, tauri-plugin-log, webbrowser); selects `native_init`
- `production` — switches auth endpoints and `ENV_TYPE` to production values
- `test-utils` — minimal test utilities foundation

## Commands

```bash
cargo tauri build --features native      # native desktop
cargo build -p bodhi                     # server/container
cargo test -p bodhi --features native
cargo test -p bodhi
```
