# API Reference: ConcurrencyControlService

## Module: `services::concurrency_service`

### Trait: `ConcurrencyControlService`

Provides distributed locking capabilities for coordinating concurrent operations.

```rust
#[async_trait]
pub trait ConcurrencyControlService: Send + Sync + Debug {
    async fn acquire_lock(&self, key: &str) -> Result<Box<dyn LockGuard>, ConcurrencyError>;
    async fn try_acquire_lock(&self, key: &str) -> Result<Option<Box<dyn LockGuard>>, ConcurrencyError>;
    async fn acquire_lock_with_timeout(&self, key: &str, timeout: Duration) -> Result<Box<dyn LockGuard>, ConcurrencyError>;
}
```

#### Methods

##### `acquire_lock`

```rust
async fn acquire_lock(&self, key: &str) -> Result<Box<dyn LockGuard>, ConcurrencyError>
```

Acquires exclusive lock for the specified key, blocking until available.

**Parameters:**
- `key`: Unique identifier for the resource to lock

**Returns:**
- `Ok(LockGuard)`: Lock acquired successfully
- `Err(ConcurrencyError)`: Lock acquisition failed

**Example:**
```rust
let guard = service.acquire_lock("user:123:token_refresh").await?;
// Critical section
perform_operation().await?;
// Lock released when guard dropped
```

##### `try_acquire_lock`

```rust
async fn try_acquire_lock(&self, key: &str) -> Result<Option<Box<dyn LockGuard>>, ConcurrencyError>
```

Attempts to acquire lock without blocking.

**Parameters:**
- `key`: Unique identifier for the resource to lock

**Returns:**
- `Ok(Some(LockGuard))`: Lock acquired
- `Ok(None)`: Lock currently held by another operation
- `Err(ConcurrencyError)`: Service error

**Example:**
```rust
if let Some(guard) = service.try_acquire_lock("user:123:token_refresh").await? {
    perform_operation().await?;
} else {
    return use_cached_result().await;
}
```

##### `acquire_lock_with_timeout`

```rust
async fn acquire_lock_with_timeout(&self, key: &str, timeout: Duration) -> Result<Box<dyn LockGuard>, ConcurrencyError>
```

Acquires lock with maximum wait time.

**Parameters:**
- `key`: Unique identifier for the resource to lock
- `timeout`: Maximum duration to wait for lock

**Returns:**
- `Ok(LockGuard)`: Lock acquired within timeout
- `Err(ConcurrencyError::Timeout)`: Timeout elapsed
- `Err(ConcurrencyError)`: Other service error

**Example:**
```rust
let guard = service
    .acquire_lock_with_timeout("user:123:token_refresh", Duration::from_secs(10))
    .await?;
```

---

### Trait: `LockGuard`

RAII guard that automatically releases lock when dropped.

```rust
pub trait LockGuard: Send + Sync + Debug {
    fn key(&self) -> &str;
    fn acquired_at(&self) -> DateTime<Utc>;
    fn ttl(&self) -> Duration;
}
```

#### Methods

##### `key`

```rust
fn key(&self) -> &str
```

Returns the key this lock guards.

##### `acquired_at`

```rust
fn acquired_at(&self) -> DateTime<Utc>
```

Returns timestamp when lock was acquired.

##### `ttl`

```rust
fn ttl(&self) -> Duration
```

Returns lock TTL (time until automatic expiration).

---

### Enum: `ConcurrencyError`

Errors that can occur during lock operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConcurrencyError {
    #[error("lock acquisition timed out after {0:?}")]
    Timeout(Duration),

    #[error("concurrency service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("invalid lock key: {0}")]
    InvalidKey(String),

    #[error("lock already held: {0}")]
    AlreadyHeld(String),

    #[error("internal error: {0}")]
    Internal(String),
}
```

---

## Implementations

### `LocalConcurrencyControlService`

In-memory implementation for single-instance deployments.

```rust
pub struct LocalConcurrencyControlService {
    locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    default_ttl: Duration,
    time_service: Arc<dyn TimeService>,
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl LocalConcurrencyControlService {
    pub fn new(default_ttl: Duration, time_service: Arc<dyn TimeService>) -> Self;
}
```

**Characteristics:**
- Zero external dependencies
- Sub-millisecond lock acquisition (no contention)
- Automatic cleanup of unused locks
- Suitable for: Desktop app, standalone server, single Docker container

**Example:**
```rust
let concurrency_service = LocalConcurrencyControlService::new(
    Duration::from_secs(30),
    Arc::new(DefaultTimeService::new()),
);
```

---

### `RedisConcurrencyControlService` (Phase 2)

Redis-based implementation for distributed deployments.

```rust
#[cfg(feature = "distributed-redis")]
pub struct RedisConcurrencyControlService {
    connection_manager: redis::aio::ConnectionManager,
    default_ttl: Duration,
    time_service: Arc<dyn TimeService>,
    acquire_script: redis::Script,
    release_script: redis::Script,
}

impl RedisConcurrencyControlService {
    pub async fn new(
        redis_url: &str,
        default_ttl: Duration,
        time_service: Arc<dyn TimeService>,
    ) -> Result<Self, ConcurrencyError>;
}
```

**Characteristics:**
- Distributed coordination across multiple instances
- 1-5ms lock acquisition (local network)
- Automatic expiration via Redis TTL
- Suitable for: Kubernetes clusters, horizontal scaling

**Example:**
```rust
let concurrency_service = RedisConcurrencyControlService::new(
    "redis://127.0.0.1:6379",
    Duration::from_secs(30),
    Arc::new(DefaultTimeService::new()),
).await?;
```

---

## Integration with AppService

### Adding to Service Registry

```rust
pub trait AppService: Send + Sync + Debug {
    fn concurrency_service(&self) -> Arc<dyn ConcurrencyControlService>;
}
```

### Usage in Token Service

```rust
impl DefaultTokenService {
    pub async fn get_valid_session_token(...) -> Result<...> {
        let user_id = claims.sub.clone();
        let lock_key = format!("user:{}:token_refresh", user_id);

        let _guard = self.concurrency_service
            .acquire_lock_with_timeout(&lock_key, Duration::from_secs(10))
            .await?;

        // Critical section: token refresh
        let (new_access_token, new_refresh_token) = self.auth_service
            .refresh_token(...)
            .await?;

        // Lock automatically released when _guard dropped
        Ok((new_access_token, role))
    }
}
```

---

## Lock Key Naming Conventions

Follow consistent naming pattern for lock keys:

```
<resource_type>:<resource_id>:<operation>
```

**Examples:**
- `user:123:token_refresh` - User token refresh operation
- `model:llama2:download` - Model download operation
- `session:abc-def:update` - Session update operation

**Best Practices:**
- Use lowercase with underscore separators
- Include enough context to avoid collisions
- Keep keys short for performance (< 64 chars)
- Use stable IDs (user_id, not session_id)

---

## Configuration

### Environment Variables

**Local Mode (default):**
```bash
# No configuration required
```

**Distributed Redis Mode:**
```bash
export REDIS_URL="redis://127.0.0.1:6379"
export LOCK_TTL_SECONDS="30"
```

### Cargo Features

```toml
# Default: Local in-memory
cargo build --release

# Redis distributed locks
cargo build --release --features distributed-redis
```

---

## Performance Characteristics

| Metric | Local | Redis (Distributed) |
|--------|-------|---------------------|
| Acquisition Latency (no contention) | < 10 μs | 1-5 ms |
| Acquisition Latency (high contention) | < 1 ms | 10-50 ms |
| Memory per Lock | ~200 bytes | ~100 bytes (Redis) |
| Max Concurrent Locks | 100,000+ | 1,000,000+ |
| Network Dependency | None | Redis availability |

---

## Error Handling

```rust
match concurrency_service.acquire_lock_with_timeout(key, timeout).await {
    Ok(guard) => {
        // Perform critical operation
        operation().await?;
        // Lock released automatically
    },
    Err(ConcurrencyError::Timeout(duration)) => {
        tracing::warn!("Lock acquisition timed out after {:?}", duration);
        return Err(AuthError::Internal("Lock timeout".to_string()));
    },
    Err(ConcurrencyError::ServiceUnavailable(msg)) => {
        tracing::error!("Concurrency service unavailable: {}", msg);
        // Fallback or retry logic
    },
    Err(e) => {
        tracing::error!("Unexpected concurrency error: {}", e);
        return Err(AuthError::Internal(e.to_string()));
    },
}
```

---

## Testing

### Mocking for Unit Tests

```rust
use services::MockConcurrencyControlService;

let mut mock_concurrency = MockConcurrencyControlService::new();
mock_concurrency
    .expect_acquire_lock()
    .with(eq("user:123:token_refresh"))
    .times(1)
    .returning(|_| Ok(Box::new(MockLockGuard::new(...))));
```

---

## Migration from Existing Code

Before (without concurrency control):
```rust
pub async fn get_valid_session_token(...) -> Result<...> {
    if token_expired {
        // ❌ Race condition: multiple requests can refresh simultaneously
        refresh_token().await?;
    }
}
```

After (with concurrency control):
```rust
pub async fn get_valid_session_token(...) -> Result<...> {
    if token_expired {
        let _guard = concurrency_service.acquire_lock(key).await?;

        // ✅ Double-check after acquiring lock
        if !token_expired {
            return current_token;  // Already refreshed
        }

        refresh_token().await?;
    }
}
```

---

## See Also

- [Problem Analysis](./01-problem-analysis.md) - Root cause details
- [Architecture Design](./02-architecture-design.md) - Design rationale
- [Implementation Guide](./03-implementation-guide.md) - Step-by-step instructions
- [Testing Strategy](./04-testing-strategy.md) - Test coverage
- [Migration Plan](./05-migration-plan.md) - Rollout strategy
