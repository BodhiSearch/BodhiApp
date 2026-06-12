# Testing Context

## Current Test Infrastructure

### Backend Tests
- **Command**: `make test.backend` → `cargo test` + `cargo test -p bodhi --features native`
- **DB tests**: In-memory SQLite via `TestDbService` with `TempDir`
- **Mock services**: `mockall` generates `MockDbService`, `MockAuthService`, etc.
- **Event tracking**: `TestDbService` broadcasts operation names via `Sender<String>`
- **Time control**: `FrozenTimeService` for deterministic timestamps

### TestDbService (crates/services/src/test_utils/db.rs)
```rust
pub struct TestDbService {
  _temp_dir: Arc<TempDir>,
  inner: SqliteDbService,
  event_sender: Sender<String>,
  now: DateTime<Utc>,
  encryption_key: Vec<u8>,
}
```
- Wraps real SqliteDbService with in-memory SQLite
- Runs actual migrations
- Broadcasts method calls for test assertions
- Deterministic time via FrozenTimeService

### MockDbService
- Generated via `mockall::mock!` macro
- All repository trait methods mockable
- Used for unit tests where DB behavior needs fine control

---

## Multi-Tenant Test Strategy

### Decision: SQLite for all local testing
- All unit and integration tests continue using in-memory SQLite
- sqlx::Any with `sqlite::memory:` URLs
- No PostgreSQL needed for local development/CI
- PostgreSQL testing only in dedicated multi-tenant E2E pipeline

### Changes to TestDbService
```rust
pub struct TestDbService {
  _temp_dir: Arc<TempDir>,
  inner: DbServiceImpl,          // Renamed from SqliteDbService, uses AnyPool
  event_sender: Sender<String>,
  now: DateTime<Utc>,
  encryption_key: Vec<u8>,
  default_org_id: String,        // NEW: default test org_id
}
```

### Test Org Fixtures
```rust
// Test helpers for org-scoped tests
pub fn test_org() -> Organization {
  Organization {
    id: "test-org-id".to_string(),
    slug: "test-org".to_string(),
    display_name: "Test Organization".to_string(),
    kc_client_id: "test_client_id".to_string(),
    client_secret: "test_client_secret".to_string(),
    encryption_key: "test_encryption_key".to_string(),
    status: OrgStatus::Active,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  }
}

// Multi-org test helper
pub fn test_org_beta() -> Organization {
  Organization {
    id: "beta-org-id".to_string(),
    slug: "beta-org".to_string(),
    // ... different org for isolation testing
  }
}
```

### Test Patterns for Org Isolation

#### Repository Tests
```rust
#[tokio::test]
async fn test_toolsets_isolated_by_org() {
  let db = TestDbService::new().await;

  // Create org-alpha toolset
  db.create_toolset("org-alpha", &toolset_alpha).await.unwrap();

  // Create org-beta toolset with same slug
  db.create_toolset("org-beta", &toolset_beta).await.unwrap();

  // Query org-alpha: should only see its toolset
  let result = db.list_toolsets("org-alpha", "user-1").await.unwrap();
  assert_eq!(1, result.len());
  assert_eq!("org-alpha", result[0].org_id);

  // Query org-beta: should only see its toolset
  let result = db.list_toolsets("org-beta", "user-1").await.unwrap();
  assert_eq!(1, result.len());
  assert_eq!("org-beta", result[0].org_id);
}
```

#### Middleware Tests
```rust
#[tokio::test]
async fn test_org_resolution_middleware_injects_context() {
  let app = test_app_with_org("test-org").await;

  let response = app
    .oneshot(
      Request::get("/api/toolsets")
        .header("X-BodhiApp-Org", "test-org")
        .body(Body::empty())
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(200, response.status());
}

#[tokio::test]
async fn test_org_resolution_rejects_unknown_org() {
  let app = test_app_with_org("test-org").await;

  let response = app
    .oneshot(
      Request::get("/api/toolsets")
        .header("X-BodhiApp-Org", "nonexistent-org")
        .body(Body::empty())
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(404, response.status());
}
```

### Existing Test Migration

All existing tests that call repository methods need `org_id` parameter added:
```rust
// Before
db.create_api_token(&token).await.unwrap();

// After
db.create_api_token("test-org-id", &token).await.unwrap();
```

This is a mechanical change across all test files. Strategy:
1. Update repository trait signatures
2. Update DbServiceImpl implementation
3. Update TestDbService delegation
4. Update MockDbService mock definition
5. Update all test call sites (search and replace with org_id parameter)

### CI Multi-Tenant E2E (Future)
- Dedicated CI pipeline spins up:
  - PostgreSQL container
  - Redis container
  - Keycloak container with test realm + organizations
  - BodhiApp with BODHI_MULTI_TENANT=true
- Black-box E2E tests against the full stack
- Tests org isolation, cross-org security, session scoping
- Not part of regular `make test` - separate CI job
