# Test Performance Analysis — 20260225

**Date**: 2026-02-25
**Analyst**: Claude Code (claude-sonnet-4-6)
**Scope**: Rust backend test suite — crates run with `cargo nextest run`
**nextest version**: 0.9.97

## Summary

The BodhiApp test suite has critical performance problems. Tests that should complete in <1ms are taking 30–900 seconds due to connection pool background task shutdown delays in the Tokio async runtime.

**Total current test time** (all crates, sequential wall): **>3000 seconds (~50 minutes)**
**Expected after fixes**: **~30–60 seconds total**

## Problem Categories

| File | Category | Crates Affected | Tests Affected | Severity |
|------|----------|-----------------|----------------|----------|
| [01-reqwest-pool.md](./01-reqwest-pool.md) | reqwest keepalive cleanup | `services` | 105 tests, 28–31s each | 🔴 Critical |
| [02-sqlx-pool.md](./02-sqlx-pool.md) | sqlx pool shutdown delay | `services`, `auth_middleware`, `routes_app`, `server_core` | ~400 tests, 42–900s each | 🔴 Critical |
| [03-mock-session.md](./03-mock-session.md) | Missing MockSessionService in tests | `auth_middleware`, `routes_app` | ~500 tests | 🔴 Critical |
| [04-live-tests.md](./04-live-tests.md) | Live/external dependency tests | `auth_middleware`, `server_core`, `routes_app` | ~20 tests | 🟡 Expected |
| [05-nextest-config.md](./05-nextest-config.md) | nextest timeout configuration | All | All | 🟡 Guard rail |

## Raw Data Files

| File | Content |
|------|---------|
| [00-raw-analysis.md](./00-raw-analysis.md) | Complete timing data and root cause investigation notes |

## Crate-by-Crate Baseline

| Crate | Tests | Current Time | Expected After Fix |
|-------|-------|-------------|-------------------|
| `errmeta_derive` | 45 | 0.43s | 0.43s (already fast) |
| `objs` | 410 | 0.98s | 0.98s (already fast) |
| `mcp_client` | 0 | — | — |
| `services` | 341 | 252.9s | ~8s |
| `server_core` | 100 | 25.1s | ~5s |
| `auth_middleware` | 157 | ~940s | ~5s |
| `routes_app` | ~670 | >840s | ~20s |

## Fix Order (Recommended)

1. **[05-nextest-config.md]** — Add `.config/nextest.toml` immediately. Zero risk. Prevents infinite hangs.
2. **[03-mock-session.md]** — Use `MockSessionService` in tests that don't need sessions. Fast wins, large impact.
3. **[02-sqlx-pool.md]** — Fix sqlx pool idle timeout. Structural fix eliminating the root cause.
4. **[01-reqwest-pool.md]** — Fix reqwest pool in services. Eliminates the 30s services tax.
5. **[04-live-tests.md]** — Gate live tests behind env var or nextest profile.

## Key Files to Read Before Starting

- `crates/services/src/test_utils/app.rs` — `AppServiceStubBuilder` — core test builder causing most slowness
- `crates/services/src/session_service/session_service.rs` — `DefaultSessionService::connect_sqlite`
- `crates/services/src/ai_api_service.rs` — reqwest client with 30s timeout
- `crates/auth_middleware/src/api_auth_middleware.rs` — tests using `AppServiceStubBuilder::default().build()`
- `crates/routes_app/src/test_utils/router.rs` — `build_test_router()` creating real session service
- `crates/routes_app/src/routes_users/test_management_crud.rs` — 5 near-deadlocked tests
