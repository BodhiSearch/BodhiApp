# Phase 6: lib_bodhiserver_napi crate -- NAPI Config Export

## Functional Goal

Export the `BODHI_SESSION_DB_URL` constant through the NAPI bindings so that the Node.js test infrastructure and Playwright E2E tests can configure the session database URL when starting test servers.

## Prerequisites

- Phases 1-5 completed (all Rust crate changes done, `make test.backend` passes)
- Discover main's current NAPI config structure (`crates/lib_bodhiserver_napi/src/config.rs`)

## Changes

### 1. Export BODHI_SESSION_DB_URL via NAPI

In `crates/lib_bodhiserver_napi/src/config.rs`, add the new constant alongside the other `#[napi]` exports:

```rust
#[napi]
pub const BODHI_SESSION_DB_URL: &str = "BODHI_SESSION_DB_URL";
```

This follows the same pattern as `BODHI_PUBLIC_HOST`, `BODHI_PUBLIC_PORT`, and other configuration constants already exported via NAPI.

### 2. Export BODHI_DEPLOYMENT via NAPI

Similarly, export the deployment mode constant:

```rust
#[napi]
pub const BODHI_DEPLOYMENT: &str = "BODHI_DEPLOYMENT";
```

### 3. Rebuild NAPI Bindings

After adding the constants, rebuild the NAPI bindings to generate the TypeScript definitions:

```bash
cd crates/lib_bodhiserver_napi && npm run build:release
```

This generates the `.node` binary and TypeScript type declarations that make the constants available to JavaScript/TypeScript code.

## Verification

1. `cargo check -p lib_bodhiserver_napi` -- compiles
2. NAPI bindings build successfully
3. `make test.backend` -- full backend still passes
4. `cargo fmt` -- clean formatting

## Future: Playwright E2E Integration (Not in Scope)

The worktree also contained Playwright infrastructure for running E2E tests against both SQLite and PostgreSQL session backends (two servers on different ports, two Playwright projects). This is explicitly excluded from this plan per scoping decisions. When E2E PG testing is needed:

- A `start-pg-server.mjs` script would start a second server on port 61135 with `BODHI_SESSION_DB_URL` pointing to docker-compose PostgreSQL
- The Playwright config would add a `pg-chromium` project with `baseURL: 'http://localhost:61135'`
- Fixtures would resolve the base URL from the Playwright project config
- The `webServer` array in `playwright.config.mjs` would include both servers

These details are preserved here for reference but should be planned separately when the E2E test infrastructure work is undertaken.
