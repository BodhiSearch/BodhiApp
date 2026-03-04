# PACKAGE.md - lib_bodhiserver_napi

See [CLAUDE.md](CLAUDE.md) for architectural overview.

## Source File Index

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module declarations, feature-gated test_utils |
| `src/config.rs` | `NapiAppOptions`, config free functions, `try_build_app_options_internal()`, constant exports |
| `src/server.rs` | `BodhiServer` NAPI class, `setup_logs()`, `Drop` impl |
| `src/test_utils/mod.rs` | Test utilities module |
| `src/test_utils/config.rs` | `test_config` rstest fixture returning `(NapiAppOptions, TempDir)` |
| `build.rs` | NAPI build configuration |

## NAPI Interface

### BodhiServer Class

| Method | Signature | Notes |
|--------|-----------|-------|
| `constructor` | `new(config: NapiAppOptions)` | |
| `config` (getter) | `-> NapiAppOptions` | Returns clone |
| `server_url` | `-> String` | `{scheme}://{host}:{port}` (omits port for 80/443) |
| `host` | `-> String` | From `BODHI_HOST` env var or `DEFAULT_HOST` |
| `port` | `-> u16` | From `BODHI_PORT` env var or `DEFAULT_PORT` |
| `public_host` | `-> String` | From `BODHI_PUBLIC_HOST` or falls back to `host()` |
| `public_port` | `-> u16` | From `BODHI_PUBLIC_PORT` or falls back to `port()` |
| `public_scheme` | `-> String` | From `BODHI_PUBLIC_SCHEME` > `BODHI_SCHEME` > `DEFAULT_SCHEME` |
| `start` | `async -> ()` | Full bootstrap + server start. Errors if already running |
| `stop` | `async -> ()` | Graceful shutdown |
| `is_running` | `async -> bool` | Checks shutdown handle presence |
| `ping` | `async -> bool` | HTTP GET to `/ping` |

### Free Functions

| Function | Signature |
|----------|-----------|
| `create_napi_app_options` | `-> NapiAppOptions` (empty config) |
| `set_env_var` | `(config, key, value) -> NapiAppOptions` |
| `set_app_setting` | `(config, key, value) -> NapiAppOptions` |
| `set_system_setting` | `(config, key, value) -> NapiAppOptions` |
| `set_client_credentials` | `(config, client_id, client_secret) -> NapiAppOptions` |
| `set_app_status` | `(config, status) -> Result<NapiAppOptions>` (validates status) |

### Exported Constants

`BODHI_HOME`, `BODHI_HOST`, `BODHI_PORT`, `BODHI_SCHEME`, `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`, `BODHI_LOGS`, `BODHI_ENV_TYPE`, `BODHI_APP_TYPE`, `BODHI_VERSION`, `BODHI_COMMIT_SHA`, `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`, `BODHI_ENCRYPTION_KEY`, `BODHI_EXEC_LOOKUP_PATH`, `BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`, `BODHI_PUBLIC_SCHEME`, `BODHI_PUBLIC_HOST`, `BODHI_PUBLIC_PORT`, `BODHI_SESSION_DB_URL`, `BODHI_APP_DB_URL`, `BODHI_DEPLOYMENT`, `HF_HOME`, `DEFAULT_HOST` (`"localhost"`), `DEFAULT_PORT` (`1135`)

## Build & Test

```bash
# Dev build
cd crates/lib_bodhiserver_napi && npm run build

# Release build
cd crates/lib_bodhiserver_napi && npm run build:release

# Rust tests
cargo test -p lib_bodhiserver_napi

# JS unit tests (vitest)
cd crates/lib_bodhiserver_napi && npm run test:run

# Playwright E2E tests
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

## Cross-Platform Targets

`aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`
