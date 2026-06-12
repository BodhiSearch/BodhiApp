# Phase 4: Generic CacheService + Redis

## Goal
Refactor CacheService from simple string key-value to generic typed cache with Redis backend for multi-tenant and in-memory for single-tenant. Enable org config caching and distributed cache invalidation.

## Prerequisites
- Phase 3 complete (org resolution middleware uses cache for org config)
- During Phase 3, org config lookup goes directly to DB. This phase adds caching.

---

## Step 1: Refactor CacheService Trait

### Current Trait
```rust
pub trait CacheService: Send + Sync + Debug {
  fn get(&self, key: &str) -> Option<String>;
  fn set(&self, key: &str, value: &str);
  fn remove(&self, key: &str);
}
```

### New Generic Trait
```rust
#[async_trait]
pub trait CacheService: Send + Sync + Debug {
  /// Get a cached value by key, deserializing from stored format
  async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>, CacheError>;

  /// Set a cached value with optional TTL in seconds
  async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl_secs: Option<u64>) -> Result<(), CacheError>;

  /// Remove a cached value
  async fn invalidate(&self, key: &str) -> Result<(), CacheError>;

  /// Remove all values matching a prefix pattern
  async fn invalidate_prefix(&self, prefix: &str) -> Result<(), CacheError>;
}
```

### CacheError
```rust
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
  #[error("Cache serialization error: {0}")]
  Serialization(String),
  #[error("Cache backend error: {0}")]
  Backend(String),
}
```

---

## Step 2: In-Memory Implementation (Single-Tenant)

### MokaCacheService (Refactored)
```rust
use mini_moka::sync::Cache;

pub struct MokaCacheService {
  cache: Cache<String, String>,  // JSON-serialized values
}

impl Default for MokaCacheService {
  fn default() -> Self {
    Self {
      cache: Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(300))  // 5 min default TTL
        .build(),
    }
  }
}

#[async_trait]
impl CacheService for MokaCacheService {
  async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>, CacheError> {
    match self.cache.get(&key.to_string()) {
      Some(json) => Ok(Some(serde_json::from_str(&json)?)),
      None => Ok(None),
    }
  }

  async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, _ttl_secs: Option<u64>) -> Result<(), CacheError> {
    let json = serde_json::to_string(value)?;
    self.cache.insert(key.to_string(), json);
    Ok(())
  }

  async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
    self.cache.invalidate(&key.to_string());
    Ok(())
  }

  async fn invalidate_prefix(&self, prefix: &str) -> Result<(), CacheError> {
    // Iterate and remove matching keys
    // mini-moka doesn't support prefix scan natively
    // For single-tenant this is rarely needed
    Ok(())
  }
}
```

---

## Step 3: Redis Implementation (Multi-Tenant)

### Dependencies
```toml
# Cargo.toml workspace
[workspace.dependencies]
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
```

### RedisCacheService
```rust
use redis::AsyncCommands;

pub struct RedisCacheService {
  client: redis::aio::ConnectionManager,
  default_ttl: u64,  // Default TTL in seconds
}

impl RedisCacheService {
  pub async fn new(redis_url: &str, default_ttl: u64) -> Result<Self> {
    let client = redis::Client::open(redis_url)?;
    let manager = client.get_connection_manager().await?;
    Ok(Self { client: manager, default_ttl })
  }
}

#[async_trait]
impl CacheService for RedisCacheService {
  async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>, CacheError> {
    let result: Option<String> = self.client.clone().get(key).await?;
    match result {
      Some(json) => Ok(Some(serde_json::from_str(&json)?)),
      None => Ok(None),
    }
  }

  async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl_secs: Option<u64>) -> Result<(), CacheError> {
    let json = serde_json::to_string(value)?;
    let ttl = ttl_secs.unwrap_or(self.default_ttl);
    self.client.clone().set_ex(key, &json, ttl).await?;
    Ok(())
  }

  async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
    self.client.clone().del(key).await?;
    Ok(())
  }

  async fn invalidate_prefix(&self, prefix: &str) -> Result<(), CacheError> {
    // Use SCAN + DEL for prefix-based invalidation
    let pattern = format!("{}*", prefix);
    let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern)
      .query_async(&mut self.client.clone()).await?;
    if !keys.is_empty() {
      self.client.clone().del(keys).await?;
    }
    Ok(())
  }
}
```

---

## Step 4: Org Config Caching

### Cache Key Convention
```
org:<slug>          → OrgContext (full org config)
org:id:<org_id>     → OrgContext (lookup by ID)
token:<digest>      → CachedToken (existing token cache)
```

### Org Resolution with Cache
```rust
// In org_resolution_middleware
async fn resolve_org_context(slug: &str, state: &Arc<dyn RouterState>) -> Result<OrgContext> {
  let cache = state.app_service().cache_service();
  let cache_key = format!("org:{}", slug);

  // Try cache first
  if let Some(ctx) = cache.get::<OrgContext>(&cache_key).await? {
    return Ok(ctx);
  }

  // Cache miss: query DB
  let org = state.app_service().db_service()
    .get_org_by_slug(slug)
    .await?
    .ok_or(ApiError::OrgNotFound(slug.to_string()))?;

  // Build OrgContext (decrypt secrets)
  let ctx = OrgContext::from_org(&org, &master_encryption_key)?;

  // Cache with TTL
  cache.set(&cache_key, &ctx, Some(300)).await?;  // 5 min TTL

  Ok(ctx)
}
```

---

## Step 5: Cache Invalidation (Event-Driven)

### For Multi-Tenant (Redis Pub/Sub)
```rust
// Publisher: called when org config changes (by provisioning service)
async fn invalidate_org_cache(cache: &dyn CacheService, slug: &str) {
  cache.invalidate(&format!("org:{}", slug)).await?;
  cache.invalidate(&format!("org:id:{}", org_id)).await?;
}

// Subscriber: background task listening for invalidation events
async fn cache_invalidation_listener(redis_url: &str, cache: Arc<dyn CacheService>) {
  let client = redis::Client::open(redis_url).unwrap();
  let mut pubsub = client.get_async_pubsub().await.unwrap();
  pubsub.subscribe("cache:invalidate").await.unwrap();

  while let Some(msg) = pubsub.on_message().next().await {
    let key: String = msg.get_payload().unwrap();
    cache.invalidate(&key).await.ok();
  }
}
```

### For Single-Tenant (In-Memory, No Pub/Sub)
- Cache invalidation is local (same process)
- Direct `cache.invalidate(key)` call
- No external message bus needed

---

## Step 6: ConcurrencyService Update

### Current: LocalConcurrencyService (In-Process Mutex)
Used for token refresh locking to prevent concurrent refresh races.

### Multi-Tenant: Distributed Lock via Redis
```rust
pub struct RedisConcurrencyService {
  client: redis::aio::ConnectionManager,
}

#[async_trait]
impl ConcurrencyService for RedisConcurrencyService {
  async fn with_lock<F, T>(&self, key: &str, f: F) -> Result<T>
  where F: Future<Output = Result<T>> + Send {
    // Redis SETNX-based distributed lock
    let lock_key = format!("lock:{}", key);
    let lock_value = uuid::Uuid::new_v4().to_string();

    // Acquire lock with TTL (prevent deadlock)
    loop {
      let acquired: bool = redis::cmd("SET")
        .arg(&lock_key).arg(&lock_value)
        .arg("NX").arg("EX").arg(30)  // 30 sec TTL
        .query_async(&mut self.client.clone()).await?;

      if acquired { break; }
      tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let result = f.await;

    // Release lock (only if we still own it)
    let script = redis::Script::new(
      "if redis.call('get', KEYS[1]) == ARGV[1] then return redis.call('del', KEYS[1]) else return 0 end"
    );
    script.key(&lock_key).arg(&lock_value)
      .invoke_async(&mut self.client.clone()).await?;

    result
  }
}
```

---

## Step 7: AppServiceBuilder Cache Selection

```rust
// In AppServiceBuilder
async fn build_cache_service(&self) -> Arc<dyn CacheService> {
  if let Some(redis_url) = self.setting_service.get_setting("REDIS_URL") {
    Arc::new(RedisCacheService::new(&redis_url, 300).await.unwrap())
  } else {
    Arc::new(MokaCacheService::default())
  }
}

async fn build_concurrency_service(&self) -> Arc<dyn ConcurrencyService> {
  if let Some(redis_url) = self.setting_service.get_setting("REDIS_URL") {
    Arc::new(RedisConcurrencyService::new(&redis_url).await.unwrap())
  } else {
    Arc::new(LocalConcurrencyService::default())
  }
}
```

---

## Step 8: Migrate Existing Token Cache

### Current: Token validation cache in TokenService
Currently uses CacheService with string values for caching validated tokens.

### Update to new generic interface:
```rust
// Before
cache.set(&cache_key, &token_json);
let cached = cache.get(&cache_key);

// After
cache.set(&cache_key, &cached_token, Some(3600)).await?;
let cached: Option<CachedToken> = cache.get(&cache_key).await?;
```

---

## Step 9: Update Tests

### MockCacheService
```rust
mockall::mock! {
  pub CacheService {}

  #[async_trait]
  impl CacheService for CacheService {
    async fn get<T: DeserializeOwned + Send + 'static>(&self, key: &str) -> Result<Option<T>, CacheError>;
    async fn set<T: Serialize + Send + Sync + 'static>(&self, key: &str, value: &T, ttl_secs: Option<u64>) -> Result<(), CacheError>;
    async fn invalidate(&self, key: &str) -> Result<(), CacheError>;
    async fn invalidate_prefix(&self, prefix: &str) -> Result<(), CacheError>;
  }
}
```

### Test Cache Service
```rust
// For integration tests, use MokaCacheService (in-memory)
// No Redis needed for local testing
```

---

## Deliverable
- Generic CacheService trait with typed get/set/invalidate
- MokaCacheService (in-memory) for single-tenant
- RedisCacheService for multi-tenant
- Org config caching in org resolution middleware
- Distributed locking via RedisConcurrencyService
- Cache invalidation via Redis Pub/Sub
- Existing token cache migrated to new interface
- All tests pass (using MokaCacheService)

## Testing Checklist
- [ ] MokaCacheService works with typed serialization
- [ ] Org config cache hit/miss works correctly
- [ ] Token cache migration works
- [ ] ConcurrencyService lock prevents concurrent access
- [ ] All existing tests pass
- [ ] Cache invalidation clears correct keys
