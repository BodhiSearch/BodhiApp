# Implementation Guide: Token Refresh Concurrency Control

**Version:** 1.0
**Target Completion:** Phase 1 (Local) - 3-5 days

This guide provides step-by-step instructions for implementing the `ConcurrencyControlService` abstraction and integrating it with BodhiApp's token refresh mechanism.

## Phase 1: Local Implementation (Priority: P0)

### Step 1: Create ConcurrencyControlService Module

**File:** `crates/services/src/concurrency_service.rs`

1. Create the new module file
2. Define the core trait and error types
3. Implement `LocalConcurrencyControlService`
4. Add comprehensive unit tests

**Task Checklist:**
- [ ] Create `concurrency_service.rs` in `services/src/`
- [ ] Copy trait definition from `02-architecture-design.md`
- [ ] Implement `LocalConcurrencyControlService` with HashMap + Mutex
- [ ] Implement `LocalLockGuard` with RAII pattern
- [ ] Add cleanup background task
- [ ] Write unit tests for concurrent lock acquisition
- [ ] Write unit tests for timeout behavior
- [ ] Write unit tests for cleanup mechanism

**Estimated Time:** 4-6 hours

### Step 2: Update Services Module

**File:** `crates/services/src/lib.rs`

Add module export:

```rust
mod concurrency_service;
pub use concurrency_service::*;
```

**Task Checklist:**
- [ ] Add `mod concurrency_service;` to `lib.rs`
- [ ] Add `pub use concurrency_service::*;` to exports
- [ ] Verify compilation with `cargo build -p services`

**Estimated Time:** 5 minutes

### Step 3: Integrate with AppService Registry

**File:** `crates/services/src/app_service.rs`

1. Add `concurrency_service()` method to `AppService` trait
2. Add field to `DefaultAppService`
3. Update constructor and implementation

**Task Checklist:**
- [ ] Add `fn concurrency_service(&self) -> Arc<dyn ConcurrencyControlService>;` to trait
- [ ] Add `concurrency_service: Arc<dyn ConcurrencyControlService>` field
- [ ] Update `DefaultAppService::new()` constructor
- [ ] Implement getter method
- [ ] Update all call sites creating `DefaultAppService`

**Files to Update:**
- `crates/server_app/src/main.rs`
- `crates/bodhi/src-tauri/src/main.rs`
- `crates/services/src/test_utils.rs`

**Estimated Time:** 1-2 hours

### Step 4: Update DefaultTokenService

**File:** `crates/auth_middleware/src/token_service.rs`

Integrate concurrency control into token refresh logic:

```rust
// Add to struct
pub struct DefaultTokenService {
    // ... existing fields
    concurrency_service: Arc<dyn ConcurrencyControlService>,
}

// Update constructor
impl DefaultTokenService {
    pub fn new(
        // ... existing parameters
        concurrency_service: Arc<dyn ConcurrencyControlService>,
    ) -> Self {
        Self {
            // ... existing fields
            concurrency_service,
        }
    }
}

// Modify get_valid_session_token
pub async fn get_valid_session_token(&self, ...) -> Result<...> {
    // ... existing fast-path check

    // Acquire lock before refresh
    let user_id = claims.sub.clone();
    let lock_key = format!("user:{}:token_refresh", user_id);

    let _guard = self.concurrency_service
        .acquire_lock_with_timeout(&lock_key, Duration::from_secs(10))
        .await
        .map_err(|e| AuthError::Internal(format!("Lock acquisition failed: {}", e)))?;

    // Double-check pattern: re-validate after lock acquisition
    let current_access_token = session.get::<String>(SESSION_KEY_ACCESS_TOKEN).await?
        .ok_or(AuthError::RefreshTokenNotFound)?;

    let current_claims = extract_claims::<Claims>(&current_access_token)?;
    if current_claims.exp as i64 > Utc::now().timestamp() {
        // Already refreshed by concurrent request
        tracing::info!("Token already refreshed by concurrent request for user: {}", user_id);
        let client_id = self.secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?.client_id;
        let role = current_claims.resource_access.get(&client_id)
            .map(|roles| ResourceRole::from_resource_role(&roles.roles))
            .transpose()?;
        return Ok((current_access_token, role));
    }

    // ... existing refresh logic
}
```

**Task Checklist:**
- [ ] Add `concurrency_service` field to `DefaultTokenService`
- [ ] Update constructor signature
- [ ] Add lock acquisition before token refresh
- [ ] Implement double-check pattern after lock acquisition
- [ ] Add logging for lock acquisition and double-check hits
- [ ] Update all call sites creating `DefaultTokenService`
- [ ] Update unit tests with mocked `ConcurrencyControlService`

**Estimated Time:** 2-3 hours

### Step 5: Add Logging and Metrics

Add comprehensive observability:

```rust
// Lock acquisition
tracing::debug!(
    user_id = %user_id,
    lock_key = %lock_key,
    "Attempting to acquire token refresh lock"
);

metrics::histogram!("token_refresh.lock.acquisition.duration_ms")
    .record(lock_duration.as_millis() as f64);

// Double-check pattern success
tracing::info!(
    user_id = %user_id,
    "Token already refreshed by concurrent request (double-check succeeded)"
);

metrics::counter!("token_refresh.double_check.hits", "user_id" => user_id)
    .increment(1);

// Lock timeout
tracing::warn!(
    user_id = %user_id,
    timeout_ms = %timeout.as_millis(),
    "Token refresh lock acquisition timed out"
);

metrics::counter!("token_refresh.lock.timeouts")
    .increment(1);
```

**Task Checklist:**
- [ ] Add lock acquisition duration histogram
- [ ] Add double-check hit counter
- [ ] Add lock timeout counter
- [ ] Add debug/info logging at key points
- [ ] Test metrics emission in integration tests

**Estimated Time:** 1 hour

### Step 6: Update Unit Tests

**File:** `crates/auth_middleware/src/token_service.rs` (tests module)

Add tests for concurrent refresh scenarios:

```rust
#[tokio::test]
async fn test_concurrent_token_refresh_serialization() {
    // Setup: Mock auth service that succeeds only once
    let mut mock_auth = MockAuthService::new();
    mock_auth
        .expect_refresh_token()
        .times(1)  // Only ONE refresh should occur
        .returning(|_, _, _| Ok(("new_access".to_string(), Some("new_refresh".to_string()))));

    // Setup: Real LocalConcurrencyControlService
    let concurrency_service = Arc::new(LocalConcurrencyControlService::new(
        Duration::from_secs(30),
        Arc::new(TestTimeService::frozen()),
    ));

    let token_service = Arc::new(DefaultTokenService::new(
        Arc::new(mock_auth),
        // ... other mocks
        concurrency_service,
    ));

    let session = create_test_session_with_expired_token();

    // Spawn 10 concurrent refresh attempts
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let service = token_service.clone();
            let session = session.clone();
            tokio::spawn(async move {
                service.get_valid_session_token(session, expired_token).await
            })
        })
        .collect();

    // All should succeed (one performs refresh, others use result)
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Verify only ONE refresh call was made (double-check pattern working)
}
```

**Task Checklist:**
- [ ] Add test for concurrent refresh with single auth service call
- [ ] Add test for lock timeout behavior
- [ ] Add test for double-check pattern effectiveness
- [ ] Add test for lock cleanup
- [ ] Verify all existing tests still pass

**Estimated Time:** 2-3 hours

### Step 7: Integration Testing

**File:** `crates/integration-tests/tests/auth_concurrent_refresh.rs` (new file)

Create end-to-end integration test:

```rust
#[tokio::test]
async fn test_concurrent_token_refresh_with_real_keycloak() {
    // Setup: Real auth service pointing to test Keycloak
    let app_service = build_test_app_service().await;

    // Create session with expired token
    let session = create_authenticated_session(&app_service).await;

    // Wait for token expiration
    tokio::time::sleep(Duration::from_secs(token_ttl + 1)).await;

    // Spawn 20 concurrent API requests that will trigger refresh
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let app_service = app_service.clone();
            let session = session.clone();
            tokio::spawn(async move {
                // Make authenticated API call
                make_authenticated_request(&app_service, &session).await
            })
        })
        .collect();

    // All requests should succeed
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Verify: Only ONE refresh call to Keycloak (check audit logs)
    let refresh_count = count_keycloak_refresh_events().await;
    assert_eq!(1, refresh_count, "Should only refresh once despite 20 concurrent requests");
}
```

**Task Checklist:**
- [ ] Create integration test file
- [ ] Test with real Keycloak instance
- [ ] Verify single refresh token call under concurrent load
- [ ] Test with 100+ concurrent requests
- [ ] Measure performance impact (latency, throughput)

**Estimated Time:** 3-4 hours

### Step 8: Documentation Updates

Update documentation to reflect new concurrency control:

**Files to Update:**
- `crates/services/CLAUDE.md` - Add ConcurrencyControlService description
- `crates/auth_middleware/CLAUDE.md` - Document token refresh coordination
- `crates/services/PACKAGE.md` - Add API documentation
- `README.md` - Update architecture overview if needed

**Task Checklist:**
- [ ] Document `ConcurrencyControlService` in services crate CLAUDE.md
- [ ] Update token refresh flow documentation
- [ ] Add usage examples to PACKAGE.md
- [ ] Update architectural decision records if applicable

**Estimated Time:** 2 hours

### Step 9: Performance Testing and Benchmarking

Create benchmarks for lock acquisition performance:

```rust
// benches/concurrency_service.rs
#[bench]
fn bench_local_lock_acquisition_no_contention(b: &mut Bencher) {
    let service = LocalConcurrencyControlService::new(Duration::from_secs(30), ...);
    b.iter(|| {
        let _guard = service.acquire_lock("test_key").await;
    });
}

#[bench]
fn bench_local_lock_acquisition_high_contention(b: &mut Bencher) {
    let service = Arc::new(LocalConcurrencyControlService::new(...));
    b.iter(|| {
        // Simulate 100 concurrent acquirers
        let handles: Vec<_> = (0..100).map(|_| {
            let svc = service.clone();
            tokio::spawn(async move {
                let _guard = svc.acquire_lock("hot_key").await;
            })
        }).collect();
        for h in handles { h.await; }
    });
}
```

**Task Checklist:**
- [ ] Create benchmarks for lock acquisition
- [ ] Benchmark with varying concurrency levels (1, 10, 100, 1000)
- [ ] Measure memory overhead
- [ ] Compare performance vs no-lock baseline
- [ ] Document performance characteristics

**Estimated Time:** 2-3 hours

### Step 10: Deployment and Rollout

Deploy Phase 1 implementation to production:

**Task Checklist:**
- [ ] Create pull request with all changes
- [ ] Code review by team
- [ ] Run full test suite (unit + integration + e2e)
- [ ] Deploy to staging environment
- [ ] Monitor for "Token is not active" errors (should be zero)
- [ ] Monitor lock acquisition metrics
- [ ] Gradual rollout to production (canary → 50% → 100%)
- [ ] Post-deployment monitoring for 1 week

**Estimated Time:** 2-3 days (includes code review and monitoring)

## Phase 2: Distributed Implementation (Future)

### Prerequisites
- [ ] Redis infrastructure available
- [ ] Kubernetes cluster for horizontal scaling
- [ ] Monitoring infrastructure for distributed locks

### Implementation Steps

#### Step 1: Add Redis Dependencies

**File:** `crates/services/Cargo.toml`

```toml
[dependencies]
redis = { version = "0.24", features = ["aio", "tokio-comp", "connection-manager"], optional = true }

[features]
distributed-redis = ["redis"]
```

#### Step 2: Implement RedisConcurrencyControlService

**File:** `crates/services/src/concurrency_service.rs`

```rust
#[cfg(feature = "distributed-redis")]
pub struct RedisConcurrencyControlService {
    connection_manager: redis::aio::ConnectionManager,
    default_ttl: Duration,
    time_service: Arc<dyn TimeService>,
    acquire_script: redis::Script,
    release_script: redis::Script,
}

#[cfg(feature = "distributed-redis")]
impl RedisConcurrencyControlService {
    // Implementation from 02-architecture-design.md
}
```

#### Step 3: Configuration Management

Add environment-based selection:

```rust
let concurrency_service: Arc<dyn ConcurrencyControlService> = if cfg!(feature = "distributed-redis") {
    let redis_url = env::var("REDIS_URL")?;
    Arc::new(RedisConcurrencyControlService::new(&redis_url, ...).await?)
} else {
    Arc::new(LocalConcurrencyControlService::new(...))
};
```

#### Step 4: Distributed Testing

**Task Checklist:**
- [ ] Test with Redis Cluster
- [ ] Test with Redis Sentinel
- [ ] Test failover scenarios
- [ ] Test network partition recovery
- [ ] Load test with 10,000+ concurrent requests

#### Step 5: Documentation and Deployment Guides

**Task Checklist:**
- [ ] Document Redis setup and configuration
- [ ] Create Kubernetes deployment manifests
- [ ] Write runbook for distributed lock operations
- [ ] Document monitoring and alerting
- [ ] Create disaster recovery procedures

**Estimated Time:** 5-7 days

## Verification Checklist

### Functional Verification
- [ ] Zero "Token is not active" errors in production logs
- [ ] Single refresh token call per expiration event (Keycloak audit logs)
- [ ] All concurrent requests succeed (no 500 errors)
- [ ] Double-check pattern prevents unnecessary refreshes
- [ ] Lock cleanup removes stale locks (memory doesn't grow)

### Performance Verification
- [ ] Lock acquisition latency < 10ms p99 (local)
- [ ] Token refresh total latency increase < 5ms average
- [ ] No degradation in API response times
- [ ] Memory overhead < 1MB for 1000 concurrent users

### Operational Verification
- [ ] Metrics dashboards show lock acquisition rates
- [ ] Alerts configured for lock timeouts
- [ ] Logging provides clear diagnostic information
- [ ] Service restart recovery works correctly

## Common Issues and Troubleshooting

### Issue: Lock Acquisition Timeout

**Symptoms:** Frequent lock timeout errors in logs

**Possible Causes:**
1. Token refresh taking too long (> 10s)
2. Deadlock or stuck lock holder
3. Insufficient concurrency service resources

**Resolution:**
```bash
# Check auth service performance
grep "Token refresh successful" app.log | awk '{print $NF}' | stats

# Check for stuck locks
redis-cli KEYS "user:*:token_refresh" | xargs redis-cli TTL

# Increase timeout if auth service is slow
# Update DefaultTokenService timeout from 10s to 30s
```

### Issue: Memory Leak in LocalConcurrencyControlService

**Symptoms:** Memory usage grows over time

**Possible Causes:**
1. Cleanup task not running
2. Lock map not clearing unused entries
3. Guards not being dropped properly

**Resolution:**
```rust
// Verify cleanup task is running
tracing::debug!("Cleanup task removed {} unused locks", removed_count);

// Force manual cleanup if needed
concurrency_service.cleanup().await;
```

### Issue: Double-Check Pattern Not Working

**Symptoms:** Multiple refresh calls to Keycloak for same user

**Possible Causes:**
1. Session not saving new token before releasing lock
2. Race condition in session read-write
3. Token validation using cached value

**Resolution:**
```rust
// Ensure session.save() is called before releasing lock
session.insert(SESSION_KEY_ACCESS_TOKEN, &new_access_token).await?;
session.save().await?;  // Must complete before lock release
// Lock released here when _guard drops
```

## Rollback Plan

If issues arise in production:

1. **Immediate Rollback:** Revert to previous version without concurrency control
   ```bash
   git revert <commit-hash>
   cargo build --release
   deploy
   ```

2. **Feature Flag Disable:** Add runtime feature flag to disable lock acquisition
   ```rust
   if !env::var("ENABLE_TOKEN_REFRESH_LOCKING").is_ok() {
       // Skip lock acquisition, use old behavior
       return old_token_refresh_logic().await;
   }
   ```

3. **Gradual Re-enable:** After fix, enable for small percentage of traffic
   ```rust
   let enable_locking = hash(user_id) % 100 < rollout_percentage;
   if enable_locking {
       // Use new locking logic
   } else {
       // Use old logic
   }
   ```

## Success Criteria

Phase 1 implementation is considered successful when:

- ✅ Zero "Token is not active" errors in production for 1 week
- ✅ Keycloak audit logs show single refresh per expiration
- ✅ P99 latency increase < 20ms for token refresh operations
- ✅ No authentication-related user complaints
- ✅ All tests passing (unit, integration, e2e)
- ✅ Metrics dashboards deployed and showing healthy status
- ✅ Documentation complete and reviewed

## Timeline Summary

| Phase | Task | Duration | Dependencies |
|-------|------|----------|--------------|
| Phase 1 | Implementation | 3-5 days | None |
| Phase 1 | Code Review | 1 day | Implementation complete |
| Phase 1 | Testing | 1 day | Code review approved |
| Phase 1 | Deployment | 2 days | Testing complete |
| Phase 1 | Monitoring | 1 week | Deployment complete |
| **Phase 1 Total** | **Local Implementation** | **10-15 days** | |
| Phase 2 | Redis Implementation | 5-7 days | Redis infrastructure |
| Phase 2 | Distributed Testing | 2-3 days | Implementation complete |
| Phase 2 | Documentation | 1-2 days | Testing complete |
| Phase 2 | Deployment | 2-3 days | Documentation complete |
| **Phase 2 Total** | **Distributed Implementation** | **10-15 days** | Phase 1 complete |

**Total Project Duration:** 3-4 weeks (with buffer for reviews and issues)
