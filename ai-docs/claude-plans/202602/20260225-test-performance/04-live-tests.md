# Issue 04: Live / External Dependency Tests

**Category**: Tests requiring real llama.cpp binary, real network, or external services
**Severity**: 🟡 Expected (known slow, needs proper gating)
**Crates affected**: `server_core`, `auth_middleware` (implicit), `routes_app` (implicit)
**Tests affected**: ~20 tests
**Current behavior**: Run with normal test suite, can cause >60s delays and CI flakiness

---

## Problem Summary

A small number of tests genuinely require external dependencies: a real llama.cpp binary, running processes, or live network connections. These tests SHOULD be slow — that's expected. The problem is they run mixed in with fast unit tests, causing:
1. False slowness signals (hard to distinguish slow live tests from slow unit tests)
2. CI failures when binary not present
3. No way to skip them for fast local iteration

These tests should be gated behind an environment variable or a dedicated nextest profile so they only run in environments where dependencies are available.

---

## Identified Live Tests

### server_core — 2 live tests (require llama.cpp binary)

**File**: `crates/server_core/src/test_live_shared_rw.rs` (or inline `#[cfg(test)]`)

```
SLOW [> 60.000s] server_core::test_live_shared_rw test_live_shared_rw_reload_with_actual_file
SLOW [> 60.000s] server_core::test_live_shared_rw test_live_shared_rw_reload_with_actual_file
SLOW [> 60.000s] server_core::test_live_shared_rw test_live_shared_rw_reload_with_model_as_symlink
```

These tests:
- Load and interact with a real llama.cpp shared object / binary
- Require a real GGUF model file present on the filesystem
- Test actual model loading and reloading behavior
- Are genuinely integration tests, not unit tests
- ~60-120s each depending on hardware

**Current state**: Run unconditionally with `cargo nextest run -p server_core`. Will fail if llama.cpp binary not present.

### auth_middleware — potential live tests

Based on timing distribution, the `test_api_auth_user_scope_*` tests taking 938-940s appear to be affected by the sqlx pool issue (Issue 02/03), NOT live dependencies. No confirmed live tests found in auth_middleware.

### routes_app — potential live tests

Some route handler tests may use real HTTP clients to test OAuth2 flows. Needs investigation.

---

## Fix Strategy

### Option A: Environment Variable Gate (Recommended)

Add a check at the top of each live test:

```rust
#[tokio::test]
async fn test_live_shared_rw_reload_with_actual_file() {
    // Skip if not in live test environment
    if std::env::var("BODHI_LIVE_TESTS").is_err() {
        println!("Skipping live test — set BODHI_LIVE_TESTS=1 to enable");
        return;
    }
    // ... actual test code ...
}
```

Or use a custom attribute macro that checks the env var.

### Option B: nextest Profile for Live Tests

Add to `.config/nextest.toml` (see `05-nextest-config.md`):

```toml
[[profile.live]]
slow-timeout = { period = "120s", terminate-after = 2 }

[profile.live.junit]
# ...
```

Then run live tests explicitly:
```bash
cargo nextest run --profile live -E 'test(test_live_)'
```

And exclude them from default runs:
```bash
# In .config/nextest.toml default profile
[profile.default]
filter = "not test(test_live_)"
```

### Option C: Feature Flag Gate

```rust
#[cfg(feature = "live-tests")]
mod test_live_shared_rw {
    // ... live tests here ...
}
```

Enable only when needed:
```bash
cargo nextest run -p server_core --features live-tests
```

### Recommended: Combine A + B

1. Gate live tests with `BODHI_LIVE_TESTS` env var (Option A) — immediate fix
2. Also add nextest profile for explicit live test runs (Option B) — better UX
3. CI: Set `BODHI_LIVE_TESTS=1` in integration test environments where binary is available

---

## Test Categories Summary

| Test Type | Example | Expected Duration | Fix |
|-----------|---------|-------------------|-----|
| Live llama.cpp | `test_live_shared_rw_*` | 60-120s | Gate with env var |
| Real model file | Any test loading `.gguf` | 10-60s | Gate with env var |
| Real OAuth2 flow | Server-level auth tests | 5-30s | Gate or separate suite |
| Mockito HTTP | `test_fetch_models_*` | <0.5s after fix | Issue 01 |
| SQLite pool | All AppServiceStub tests | <2s after fix | Issues 02+03 |

---

## Current Timing

```
server_core live tests (measured):
- test_live_shared_rw_reload_with_actual_file: >60s (exact time cut off by nextest)
- test_live_shared_rw_reload_with_model_as_symlink: >60s
```

These are expected to remain slow (60-120s) after other fixes — they need real model files. The goal is to exclude them from normal runs, not to make them faster.

---

## Key Files to Examine

1. `crates/server_core/src/test_live_shared_rw.rs` (or wherever `test_live_shared_rw_*` tests are defined)
2. `crates/server_core/src/shared_rw.rs` — contains the non-live shared_rw tests that ARE slow due to sqlx (Issue 02)

---

## Investigation Commands

```bash
# Find all live test files/functions
grep -rn "test_live\|live_test\|LIVE_TESTS" crates/ --include="*.rs" | grep -v target/

# Find tests requiring binary paths
grep -rn "llama\|llamafile\|binary\|exec_path" crates/ --include="*.rs" | grep -E "#\[test\]|tokio::test" -A 5 | grep -v target/

# Check which tests require env vars
grep -rn "std::env::var\|env!" crates/ --include="*.rs" | grep -v target/ | grep test

# Run server_core in isolation to see live test timing
cargo nextest run -p server_core --status-level all --final-status-level slow 2>&1 | grep -E "SLOW|test_live"
```
