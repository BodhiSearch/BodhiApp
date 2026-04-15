# lib_bodhiserver_napi -- CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, NAPI interface, TypeScript types, NapiAppOptions fields, Bootstrap Flow

## Purpose

NAPI bindings exposing BodhiApp server to Node.js. Leaf crate -- no downstream Rust consumers. Published as `@bodhiapp/app-bindings` (requires Node.js >= 22) for external consumers.

**Note**: BodhiApp's own E2E suite no longer uses these bindings. The Playwright tests now spawn the `bodhiserver_dev` binary out of `lib_bodhiserver`. See `crates/lib_bodhiserver/tests-js/CLAUDE.md`.

## Architecture Position

**Upstream**: `lib_bodhiserver` (sole Rust dependency for server functionality)
**Downstream**: None (consumed by external Node.js npm package consumers)

## Key Components

### BodhiServer (`src/server.rs`)
- NAPI class wrapping server lifecycle
- `new(config: NapiAppOptions)` -- constructor
- `start()` -- full bootstrap; see `PACKAGE.md` for Bootstrap Flow
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

# Build NAPI bindings (debug)
cd crates/lib_bodhiserver_napi && npm run build

# Build NAPI bindings (release/published)
cd crates/lib_bodhiserver_napi && npm run build:release
```
