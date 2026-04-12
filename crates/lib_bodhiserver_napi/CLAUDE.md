# lib_bodhiserver_napi -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, NAPI interface, TypeScript types, NapiAppOptions fields, Bootstrap Flow
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
- `start()` -- full bootstrap; see `PACKAGE.md` for 9-step Bootstrap Flow
- `stop()` -- calls `ServerShutdownHandle::shutdown()`
- `is_running()` -- checks if shutdown handle exists
- `ping()` -- HTTP GET to `/ping` endpoint
- `server_url()`, `host()`, `port()`, `public_host()`, `public_port()`, `public_scheme()` -- config accessors
- `Drop` impl cleans up temp dir and log guard

### NapiAppOptions (`src/config.rs`)
Struct + free-function API for building server configuration. See `PACKAGE.md` for full field list, free functions, and exported constants.

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
