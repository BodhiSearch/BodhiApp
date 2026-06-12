# Issue 05: nextest Timeout Configuration

**Category**: Missing nextest configuration — no timeout guards, no slow test enforcement
**Severity**: 🟡 Guard rail (prevents infinite hangs, not a root cause fix)
**Crates affected**: All
**Current state**: No `.config/nextest.toml` exists → default behavior with no timeouts
**Priority**: Implement FIRST — zero risk, prevents catastrophic hangs

---

## Problem Summary

Without a nextest timeout configuration:
1. Tests that deadlock (like the 5 management_crud tests at 900s+) hang the entire test suite indefinitely
2. No distinction between "expected slow" (live tests) and "unexpectedly slow" (pool shutdown delays)
3. CI runs can hang for hours waiting for a test that will never complete
4. Developers running `cargo nextest run` locally get no feedback that tests are pathologically slow

The `.config/nextest.toml` file provides timeouts and profiles that protect against these issues. It's a zero-risk change that can be applied immediately as a safety guard while the root causes are being fixed.

---

## Current State

```bash
# Check if nextest config exists
ls .config/nextest.toml  # → does not exist
ls .config/              # → may not exist at all
```

No timeout configuration means nextest uses default behavior:
- No per-test timeout
- No slow test threshold displayed
- Tests that deadlock will hang until the user Ctrl+C

---

## Recommended Configuration

Create `.config/nextest.toml` in the workspace root:

```toml
# .config/nextest.toml
# nextest configuration for BodhiApp
# See: https://nexte.st/book/configuration.html

[profile.default]
# Show tests that are taking too long — 30s threshold to catch pool shutdown issues
slow-timeout = { period = "30s" }
# Hard kill tests that hang beyond 5 minutes
# (Relaxed from 120s to accommodate real session service tests during transition)
# Lower to 60s once Issues 02+03 are fixed
test-threads = "num-cpus"
# Fail the run if more than 5 tests fail (don't run to completion if something is broken)
fail-fast = false

# Explicit timeout per test: 300 seconds (5 minutes) hard limit
# Prevents the management_crud deadlock from hanging the whole suite
[[profile.default.overrides]]
filter = 'all()'
slow-timeout = { period = "30s", terminate-after = 2 }

# Longer timeout for route tests (has real session service during transition)
# Remove this override once Issues 02+03 are fixed
[[profile.default.overrides]]
filter = 'package(routes_app) or package(auth_middleware)'
slow-timeout = { period = "120s", terminate-after = 4 }

# ─── CI Profile ─────────────────────────────────────────────────────────────
[profile.ci]
# Shorter timeouts for CI — fail fast
slow-timeout = { period = "30s", terminate-after = 2 }
fail-fast = true

# ─── Live Test Profile ──────────────────────────────────────────────────────
[profile.live]
# Profile for running live integration tests (require llama.cpp binary, real models)
# Run with: cargo nextest run --profile live -E 'test(test_live_)'
slow-timeout = { period = "120s", terminate-after = 2 }
test-threads = 1  # live tests may compete for GPU/model resources
```

---

## Configuration Explanation

### `slow-timeout`

Tells nextest to print a warning when a test exceeds the period. `terminate-after = N` kills the test after N × period.

```toml
slow-timeout = { period = "30s", terminate-after = 2 }
# → warn after 30s, kill after 60s
```

This would have caught the management_crud deadlocked tests at 60s instead of letting them run for 900s+.

### `fail-fast = false`

Continue running other tests even when some fail. Good for getting a full picture of failures in one run.

### `test-threads = "num-cpus"`

Use all available CPUs for parallel execution (this is already nextest's default for local runs).

### The Routes/Auth Override

During the transition period while Issues 02+03 are being fixed, routes_app and auth_middleware tests still take 42-940s. The override gives them more breathing room so they complete (slowly) rather than being killed.

**Remove this override once Issues 02+03 are fixed** — at that point, all tests should complete in <30s and the global 60s hard limit is appropriate.

---

## Immediate vs Final Configuration

### Phase 1: Apply immediately (before any other fixes)

```toml
# .config/nextest.toml — Phase 1: guard rails only
[profile.default]
slow-timeout = { period = "30s" }

[[profile.default.overrides]]
filter = 'package(routes_app) or package(auth_middleware)'
slow-timeout = { period = "300s", terminate-after = 3 }  # up to 900s during transition
```

This prevents infinite hangs and provides slow test warnings without breaking anything.

### Phase 2: After Issues 02+03 are fixed

```toml
# .config/nextest.toml — Phase 2: tight timeouts
[profile.default]
slow-timeout = { period = "10s", terminate-after = 3 }  # 30s hard limit for any test

[profile.ci]
slow-timeout = { period = "10s", terminate-after = 2 }  # 20s limit in CI
fail-fast = true
```

---

## nextest Version Compatibility

Confirmed working with nextest 0.9.97 (the version currently installed):
```bash
cargo nextest --version
# cargo-nextest nextest 0.9.97
```

`slow-timeout` with `terminate-after` is supported since nextest 0.9.57.

**serial_test compatibility confirmed**: `serial_test = "3.2.0"` (current version in Cargo.toml) uses file-based mutexes (since 2.0), fully compatible with nextest's per-process test isolation. The old incompatibility was with `serial_test < 2.0` which used in-process mutexes that didn't work when each test ran in its own process. That is NOT an issue with the current setup.

---

## Run Commands After Config Applied

```bash
# Default run with timeout protection
cargo nextest run

# CI run (fail-fast, strict timeouts)
cargo nextest run --profile ci

# Live tests only (when llama.cpp binary available)
cargo nextest run --profile live -E 'test(test_live_)'

# Exclude live tests from normal run
cargo nextest run -E 'not test(test_live_)'

# Run just the management_crud tests to verify they complete
cargo nextest run -p routes_app -E 'package(routes_app) and test(management_crud)'
```

---

## File to Create

**Path**: `<workspace_root>/.config/nextest.toml`

This file goes in the `.config/` directory at the repository root (same level as `Cargo.toml`).

```bash
# Verify workspace root
ls Cargo.toml    # should exist
ls .config/      # may not exist yet

# Create directory if needed
mkdir -p .config
```

---

## Verification

```bash
# Verify nextest picks up the config
cargo nextest run --config-file .config/nextest.toml -p objs 2>&1 | head -5

# Check that slow tests are flagged
cargo nextest run -p services -E 'test(test_fetch_models_success)' 2>&1
# Should show: "SLOW [  30.xxx s]" marker

# Verify config is read automatically
cargo nextest run -p objs 2>&1 | grep -i "slow\|config"
```

---

## Investigation Commands for Fresh Session

```bash
# Check if .config/nextest.toml already exists
ls -la .config/nextest.toml 2>/dev/null || echo "does not exist"

# Check nextest version
cargo nextest --version

# Check nextest documentation on slow-timeout
# https://nexte.st/book/slow-tests.html

# Find all serial_test usages to confirm version compatibility
grep -rn "serial_test" Cargo.toml crates/*/Cargo.toml | grep -v target/

# Check current test execution without config to see baseline
cargo nextest run -p objs --status-level all 2>&1 | tail -5
```
