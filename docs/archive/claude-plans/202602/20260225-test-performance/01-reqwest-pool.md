# Issue 01: reqwest Connection Pool Keepalive Cleanup

**Category**: reqwest HTTP client keepalive background task
**Severity**: 🔴 Critical
**Crates affected**: `services`
**Tests affected**: ~105 tests, each taking 28–31s (expected: <0.5s)
**Current total cost**: ~105 × 30s ≈ **3,150 seconds** in this crate alone

---

## Problem Summary

Tests that use `DefaultAiApiService`, `DefaultAuthService`, `DefaultHubService`, `DefaultExaService`, and `DefaultMcpService` all create a `reqwest::Client` with a 30-second timeout. The test logic itself completes in <1ms, but the Tokio runtime is held open for ~30s after the test function returns because reqwest's connection pool keepalive background task is still running.

---

## Root Cause

### The reqwest Client Configuration

`crates/services/src/ai_api_service.rs` (line ~87):
```rust
pub fn with_db_service(db_service: Arc<dyn DbService>) -> Self {
    let client = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))  // 30 seconds
        .build()
        .expect("Failed to create HTTP client");
    Self { client, db_service }
}
```

`const DEFAULT_TIMEOUT_SECS: u64 = 30;` (line ~12 of ai_api_service.rs)

### What Happens at Test Shutdown

1. The `#[tokio::test]` async runtime runs the test function
2. Test function returns (after <1ms of actual work)
3. Tokio runtime begins shutdown
4. reqwest `Client` is dropped (along with the test's local variables)
5. reqwest's connection pool has an internal keepalive background task running on the Tokio runtime
6. This background task maintains idle connection timers
7. Tokio's `shutdown_background()` aborts spawned tasks, but reqwest's keepalive task uses an internal timer that fires every ~30s
8. Runtime waits for the keepalive timer to expire before completing shutdown
9. **Total extra wait: ~30 seconds per test**

### Why mockito makes it obvious

Tests use `mockito::Server::new_async()` for HTTP mocking. mockito servers respond instantly (no real network latency). The 30s delay is ENTIRELY Tokio runtime cleanup — NOT slow mock responses or network delays.

**Proof**: Running `test_fetch_models_success` in isolation:
```
time cargo nextest run -p services -E 'test(test_fetch_models_success)'
PASS [  30.816s] services ai_api_service::tests::test_fetch_models_success
real    0m30.816s
```
The mock responded in microseconds. The extra 30s is pure cleanup.

---

## Affected Test Files

### ai_api_service — ~14 tests × ~30s = ~420s

**File**: `crates/services/src/ai_api_service.rs` (inline tests) or sibling `test_ai_api_service.rs`

Slow tests:
```
PASS [  30.996s] services ai_api_service::tests::test_fetch_models_success_parameterized::case_2_without_api_key
PASS [  30.816s] services ai_api_service::tests::test_fetch_models_success
PASS [  30.112s] services ai_api_service::tests::test_fetch_models_error_response
PASS [  29.945s] services ai_api_service::tests::test_fetch_models_parameterized::case_1_...
... (~10 more similar tests)
```

### auth_service — ~8 tests × ~30s = ~240s

**File**: `crates/services/src/auth_service.rs` or sibling test file

Slow tests:
```
PASS [  29.843s] services auth_service::tests::test_refresh_token_retry_on_5xx
PASS [  29.834s] services auth_service::tests::test_list_users_success
PASS [  29.123s] services auth_service::tests::test_refresh_token_success
... (~5 more)
```

### hub_service — ~20 tests × ~29s = ~580s

**File**: `crates/services/src/hub_service.rs` or sibling test file

Slow tests:
```
PASS [  29.403s] services hub_service::tests::test_hf_hub_service_download_gated_file_allowed::case_2
... (~19 more)
```

### exa_service — ~5 tests × ~28s = ~140s

**File**: `crates/services/src/exa_service.rs` or sibling test file

Slow tests:
```
PASS [  28.633s] services exa_service::tests::test_answer_error
... (~4 more)
```

### mcp_service — ~15 tests × ~27s = ~405s

**File**: `crates/services/src/mcp_service.rs` or sibling test file

Slow tests:
```
PASS [  27.351s] services mcp_service::tests::test_mcp_service_execute_with_oauth_auth_type
... (~14 more)
```

---

## Fix Strategy

### Option A: Disable Connection Pooling for Test Client (Recommended)

Add a test-specific constructor that disables keepalive/pooling:

```rust
// In ai_api_service.rs (or each affected service)
#[cfg(any(test, feature = "test-utils"))]
pub fn with_test_db_service(db_service: Arc<dyn DbService>) -> Self {
    let client = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .pool_max_idle_per_host(0)  // disable connection pooling
        .connection_verbose(false)
        .build()
        .expect("Failed to create HTTP client");
    Self { client, db_service }
}
```

Or alternatively, add `pool_max_idle_per_host(0)` to the existing test constructor if services already have separate test vs production constructors.

`pool_max_idle_per_host(0)` tells reqwest to NOT keep idle connections in the pool, which means there's no keepalive background task to wait for at shutdown.

**Why safe**: Tests use mockito which creates fresh connections per request anyway. No connection reuse needed in tests.

### Option B: Use `connection_verbose` + `tcp_keepalive(None)`

```rust
Client::builder()
    .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    .tcp_keepalive(None)      // disable TCP keepalive
    .pool_idle_timeout(Duration::from_millis(1))  // expire idle connections immediately
    .build()
```

### Option C: Inject reqwest Client via DI (Most Flexible)

Make services accept a `reqwest::Client` parameter. In tests, pass a no-pool client. In production, use the default pooled client. This requires refactoring service constructors but gives maximum control.

---

## Expected Improvement

| Metric | Before | After |
|--------|--------|-------|
| Single test time | ~30s | <0.1s |
| services crate total | ~252s | ~8s |
| Tests affected | 105 | 105 (same tests, fast) |

---

## Verification Steps

After applying the fix:
```bash
# Single test should take <1s
time cargo nextest run -p services -E 'test(test_fetch_models_success)'

# Full services crate should be <30s
time cargo nextest run -p services
```

---

## Key Files to Modify

1. `crates/services/src/ai_api_service.rs` — `with_db_service()` constructor
2. `crates/services/src/auth_service.rs` — reqwest client creation
3. `crates/services/src/hub_service.rs` — reqwest client creation
4. `crates/services/src/exa_service.rs` — reqwest client creation
5. `crates/services/src/mcp_service.rs` — reqwest client creation

Look for `Client::builder()` in each file and identify test constructors. Pattern: constructors in `#[cfg(test)]` blocks or named `with_*` that take mock dependencies.

---

## Investigation Commands for Fresh Session

```bash
# Find all reqwest Client::builder() usages in services
grep -rn "Client::builder" crates/services/src/

# Confirm the timeout constant
grep -n "DEFAULT_TIMEOUT_SECS" crates/services/src/ai_api_service.rs

# Verify slowness in isolation
time cargo nextest run -p services -E 'test(test_fetch_models_success)'

# Check which tests are slow in services
cargo nextest run -p services --status-level all --final-status-level slow 2>&1 | grep -E "PASS \[.*[0-9]{2,}\."
```
