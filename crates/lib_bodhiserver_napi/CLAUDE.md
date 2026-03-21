# lib_bodhiserver_napi -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, NAPI interface, TypeScript types
- `tests-js/CLAUDE.md` -- E2E Playwright test suite
- `tests-js/E2E.md` -- E2E test writing conventions

## Purpose

NAPI bindings exposing BodhiApp server to Node.js. Leaf crate -- no downstream Rust consumers. Published as `@bodhiapp/app-bindings` (requires Node.js >= 22).

## Architecture Position

**Upstream**: `lib_bodhiserver` (sole Rust dependency for server functionality)
**Downstream**: None (consumed by Node.js/Playwright tests and npm package consumers)

## Key Components

### BodhiServer (`src/server.rs`)
- NAPI class wrapping server lifecycle
- `new(config: NapiAppOptions)` -- constructor
- `start()` -- builds `AppOptions`, calls `setup_app_dirs()` + `setup_bootstrap_service()` + `build_app_service()` + `ServeCommand::get_server_handle()`
- `stop()` -- calls `ServerShutdownHandle::shutdown()`
- `is_running()` -- checks if shutdown handle exists
- `ping()` -- HTTP GET to `/ping` endpoint
- `server_url()`, `host()`, `port()`, `public_host()`, `public_port()`, `public_scheme()` -- config accessors
- `Drop` impl cleans up temp dir and log guard

### NapiAppOptions (`src/config.rs`)
- `#[napi(object)]` struct: `env_vars`, `app_settings`, `system_settings`, `client_id`, `client_secret`, `app_status`, `created_by`, `tenant_name`
- Free functions: `create_napi_app_options()`, `set_env_var()`, `set_app_setting()`, `set_system_setting()`, `set_client_credentials()`, `set_app_status()`, `set_created_by()`, `set_tenant_name()`
- `try_build_app_options_internal()` -- converts `NapiAppOptions` to `AppOptionsBuilder`, builds `Tenant` if client credentials provided
- Exports 25 `BODHI_*` / `HF_*` / `DEFAULT_*` constants for JS usage (includes `BODHI_DEPLOYMENT`, `BODHI_SESSION_DB_URL`, `BODHI_APP_DB_URL`, `BODHI_MULTITENANT_CLIENT_ID`, `BODHI_MULTITENANT_CLIENT_SECRET`)

### Bootstrap Flow (in `start()`)
1. `try_build_app_options_internal(config)` -> `AppOptionsBuilder`
2. `builder.build()` -> `AppOptions`
3. `setup_app_dirs(&app_options)` -> `(bodhi_home, source, file_defaults)`
4. `setup_bootstrap_service(...)` -> `BootstrapService`
5. `setup_logs(&bootstrap)` -> `WorkerGuard`
6. `bootstrap.into_parts()` -> `BootstrapParts`
7. `build_app_service(parts)` -> `DefaultAppService`
8. `ensure_tenant(&app_service, tenant)` -- persist tenant if provided via `db_service().create_tenant_test()`
9. `ServeCommand::get_server_handle(app_service, EMBEDDED_UI_ASSETS)`

## Commands

```bash
# Rust tests
cargo test -p lib_bodhiserver_napi

# Build NAPI bindings
cd crates/lib_bodhiserver_napi && npm run build

# Run all JS tests
cd crates/lib_bodhiserver_napi && npm test

# Playwright E2E tests only
cd crates/lib_bodhiserver_napi && npm run test:playwright
```
