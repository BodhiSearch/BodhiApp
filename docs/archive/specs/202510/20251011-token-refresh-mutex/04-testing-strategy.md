# Testing Strategy: Concurrency Control Implementation

## Test Coverage Goals

- **Unit Tests:** 90%+ code coverage
- **Integration Tests:** All critical paths covered
- **Load Tests:** Verified under 1000+ concurrent requests
- **Failure Scenarios:** All error paths tested

## Unit Tests

### ConcurrencyControlService Tests

```rust
#[tokio::test]
async fn test_acquire_lock_single_thread() {
    let service = LocalConcurrencyControlService::new(Duration::from_secs(30), time_service);
    let _guard = service.acquire_lock("test_key").await.unwrap();
    assert_eq!("test_key", _guard.key());
}

#[tokio::test]
async fn test_concurrent_lock_acquisition_serializes() {
    let service = Arc::new(LocalConcurrencyControlService::new(...));
    let counter = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..10).map(|_| {
        let svc = service.clone();
        let cnt = counter.clone();
        tokio::spawn(async move {
            let _guard = svc.acquire_lock("test_key").await.unwrap();
            let prev = cnt.fetch_add(1, Ordering::SeqCst);
            assert_eq!(prev, 0, "Multiple threads in critical section");
            tokio::time::sleep(Duration::from_millis(10)).await;
            cnt.fetch_sub(1, Ordering::SeqCst);
        })
    }).collect();

    for h in handles { h.await.unwrap(); }
    assert_eq!(0, counter.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_try_acquire_lock_returns_none_when_held() {
    let service = Arc::new(LocalConcurrencyControlService::new(...));
    let _guard1 = service.acquire_lock("test_key").await.unwrap();
    let guard2 = service.try_acquire_lock("test_key").await.unwrap();
    assert!(guard2.is_none(), "Should not acquire held lock");
}

#[tokio::test]
async fn test_acquire_lock_with_timeout() {
    let service = Arc::new(LocalConcurrencyControlService::new(...));
    let _guard1 = service.acquire_lock("test_key").await.unwrap();

    let start = Instant::now();
    let result = service.acquire_lock_with_timeout("test_key", Duration::from_millis(100)).await;
    assert!(result.is_err());
    assert!(start.elapsed() >= Duration::from_millis(100));
}

#[tokio::test]
async fn test_lock_cleanup_removes_unused() {
    let service = LocalConcurrencyControlService::new(...);
    {
        let _guard = service.acquire_lock("temp_key").await.unwrap();
        // Guard dropped here
    }
    // Trigger cleanup
    tokio::time::sleep(Duration::from_secs(35)).await;

    // Verify lock removed from internal map (via metrics or inspection)
    assert_eq!(0, service.active_lock_count());
}
```

### DefaultTokenService Tests

```rust
#[tokio::test]
async fn test_token_refresh_with_single_call_under_concurrency() {
    let mut mock_auth = MockAuthService::new();
    mock_auth
        .expect_refresh_token()
        .times(1)  // CRITICAL: Only ONE refresh
        .returning(|_, _, _| Ok(("new_access".to_string(), Some("new_refresh".to_string()))));

    let concurrency_service = Arc::new(LocalConcurrencyControlService::new(...));
    let token_service = Arc::new(DefaultTokenService::new(..., concurrency_service));
    let session = create_expired_token_session();

    let handles: Vec<_> = (0..20).map(|_| {
        let svc = token_service.clone();
        let sess = session.clone();
        tokio::spawn(async move {
            svc.get_valid_session_token(sess, expired_token).await
        })
    }).collect();

    for h in handles {
        assert!(h.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_double_check_pattern_prevents_unnecessary_refresh() {
    let mut mock_auth = MockAuthService::new();
    mock_auth.expect_refresh_token().times(0);  // Should NOT be called

    let token_service = DefaultTokenService::new(...);
    let session = create_valid_token_session();  // Token NOT expired

    let result = token_service.get_valid_session_token(session, valid_token).await;
    assert!(result.is_ok());  // Returns immediately without refresh
}
```

## Integration Tests

### End-to-End Concurrent Refresh Test

```rust
#[tokio::test]
async fn test_concurrent_requests_with_real_keycloak() {
    let app_service = build_test_app_service_with_real_keycloak().await;
    let session = authenticate_test_user(&app_service).await;

    // Wait for token expiration
    tokio::time::sleep(Duration::from_secs(ACCESS_TOKEN_TTL + 1)).await;

    // Start Keycloak audit log monitoring
    let audit_monitor = start_keycloak_audit_monitor().await;

    // Spawn 50 concurrent authenticated requests
    let handles: Vec<_> = (0..50).map(|_| {
        let app = app_service.clone();
        let sess = session.clone();
        tokio::spawn(async move {
            make_authenticated_api_call(&app, &sess).await
        })
    }).collect();

    // All should succeed
    for h in handles {
        assert!(h.await.unwrap().is_ok());
    }

    // Verify: Only ONE refresh call to Keycloak
    let refresh_count = audit_monitor.count_refresh_events().await;
    assert_eq!(1, refresh_count, "Expected single refresh despite 50 concurrent requests");
}
```

## Load Tests

### High Concurrency Stress Test

```rust
#[tokio::test]
async fn stress_test_1000_concurrent_refreshes() {
    let service = Arc::new(LocalConcurrencyControlService::new(...));
    let start = Instant::now();

    let handles: Vec<_> = (0..1000).map(|i| {
        let svc = service.clone();
        let key = format!("user:{}:token_refresh", i % 100);  // 100 unique users
        tokio::spawn(async move {
            let _guard = svc.acquire_lock(&key).await.unwrap();
            tokio::time::sleep(Duration::from_millis(5)).await;
        })
    }).collect();

    for h in handles { h.await.unwrap(); }

    let elapsed = start.elapsed();
    println!("1000 concurrent locks completed in {:?}", elapsed);
    assert!(elapsed < Duration::from_secs(30), "Performance degradation detected");
}
```

## Failure Scenario Tests

### Lock Timeout Handling

```rust
#[tokio::test]
async fn test_lock_timeout_returns_error() {
    let service = Arc::new(LocalConcurrencyControlService::new(...));
    let _long_holder = service.acquire_lock("key").await.unwrap();

    let result = service.acquire_lock_with_timeout("key", Duration::from_millis(100)).await;
    assert!(matches!(result, Err(ConcurrencyError::Timeout(_))));
}
```

### Redis Failure Handling (Phase 2)

```rust
#[tokio::test]
async fn test_redis_unavailable_returns_service_error() {
    let service = RedisConcurrencyControlService::new("redis://invalid:9999", ...).await;
    assert!(matches!(service, Err(ConcurrencyError::ServiceUnavailable(_))));
}
```

## Performance Benchmarks

```rust
criterion_group!(benches,
    bench_local_no_contention,
    bench_local_10_concurrent,
    bench_local_100_concurrent
);

fn bench_local_no_contention(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = Arc::new(LocalConcurrencyControlService::new(...));

    c.bench_function("local_lock_no_contention", |b| {
        b.to_async(&rt).iter(|| async {
            let _guard = service.acquire_lock("bench_key").await.unwrap();
        });
    });
}
```

## Test Execution

```bash
# Run all unit tests
cargo test -p services concurrency
cargo test -p auth_middleware token_service

# Run integration tests
cargo test -p integration-tests --test auth_concurrent_refresh

# Run load tests (requires --release for accurate performance)
cargo test --release -p integration-tests --test load_concurrent_refresh -- --ignored

# Run benchmarks
cargo bench -p services --bench concurrency_service
```

## Success Criteria

- ✅ All unit tests pass
- ✅ Integration tests show single refresh under concurrency
- ✅ Load tests complete without errors at 1000+ concurrent requests
- ✅ Performance benchmarks show <10ms p99 lock acquisition (local)
- ✅ Zero race condition failures after 10 test runs
