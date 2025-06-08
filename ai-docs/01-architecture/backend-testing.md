# Backend Testing

This document provides comprehensive guidance for Rust backend testing patterns, frameworks, and quality assurance approaches in the Bodhi App.

## Required Documentation References

**MUST READ for backend testing:**
- `ai-docs/01-architecture/rust-backend.md` - Backend service patterns and database integration
- `ai-docs/01-architecture/development-conventions.md` - Testing conventions and file organization

**FOR COMPLETE TESTING OVERVIEW:**
- `ai-docs/01-architecture/frontend-testing.md` - Frontend testing approaches
- `ai-docs/01-architecture/TESTING_GUIDE.md` - Complete testing implementation guide

## Testing Philosophy

### Backend Testing Pyramid
1. **Unit Tests** (70%) - Service logic, database operations, and utility functions
2. **Integration Tests** (20%) - API endpoints and service interactions
3. **End-to-End Tests** (10%) - Complete request-response cycles
4. **Performance Tests** - Database query performance and API response times

### Quality Goals
- **Unit Tests**: 80%+ coverage for business logic
- **Integration Tests**: All API endpoints covered
- **Database Tests**: All CRUD operations validated
- **Performance**: Query response times under acceptable thresholds

## Technology Stack

### Core Testing Tools
- **Rust Test Framework** - Built-in Rust testing with `#[test]`
- **rstest** - Parameterized testing and fixtures
- **tokio-test** - Async testing utilities
- **mockall** - Mock object generation
- **TestDbService** - Database testing infrastructure

### Additional Testing Libraries
- **axum-test** - HTTP endpoint testing
- **sqlx-test** - Database testing utilities
- **anyhow** - Error handling in tests
- **serde_json** - JSON serialization for test data
- **uuid** - Test data generation

## Unit Testing Patterns

### Service Testing with rstest
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn test_create_model(
        #[future]
        #[from(test_db_service)]
        service: TestDbService,
    ) -> anyhow::Result<()> {
        let model_data = ModelBuilder::default()
            .name("test-model".to_string())
            .alias("test:latest".to_string())
            .build()?;

        let result = service.create_model(&model_data).await?;
        
        assert_eq!(result.name, "test-model");
        assert_eq!(result.alias, "test:latest");
        assert!(!result.id.is_empty());
        
        Ok(())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn test_get_model_not_found(
        #[future]
        #[from(test_db_service)]
        service: TestDbService,
    ) -> anyhow::Result<()> {
        let result = service.get_model("nonexistent").await;
        
        assert!(matches!(result, Err(DbError::ModelNotFound { .. })));
        
        Ok(())
    }

    #[rstest]
    #[case("valid-alias", true)]
    #[case("invalid alias", false)]
    #[case("", false)]
    fn test_alias_validation(#[case] alias: &str, #[case] expected: bool) {
        assert_eq!(validate_alias(alias), expected);
    }
}
```

### Mock Testing with mockall
```rust
use mockall::predicate::*;

#[tokio::test]
async fn test_service_with_mock() {
    let mut mock_db = MockDbService::new();
    
    mock_db
        .expect_get_model()
        .with(eq("test-alias"))
        .times(1)
        .returning(|_| Ok(create_test_model()));

    mock_db
        .expect_list_models()
        .with(eq(1), eq(10))
        .times(1)
        .returning(|_, _| Ok(vec![create_test_model()]));

    let result = mock_db.get_model("test-alias").await;
    assert!(result.is_ok());
    
    let models = mock_db.list_models(1, 10).await;
    assert_eq!(models.unwrap().len(), 1);
}

#[tokio::test]
async fn test_error_handling_with_mock() {
    let mut mock_db = MockDbService::new();
    
    mock_db
        .expect_get_model()
        .with(eq("error-alias"))
        .times(1)
        .returning(|_| Err(DbError::ConnectionFailed("Database unavailable".to_string())));

    let result = mock_db.get_model("error-alias").await;
    assert!(matches!(result, Err(DbError::ConnectionFailed(_))));
}
```

### Database Testing Patterns

#### Test Database Setup
```rust
// crates/services/src/test_utils/db.rs
use sqlx::SqlitePool;

pub struct TestDbService {
    pool: SqlitePool,
    notifications: Arc<Mutex<Vec<String>>>,
}

impl TestDbService {
    pub async fn new() -> Self {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Run migrations
        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .unwrap();

        Self {
            pool,
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_notifications(&self) -> Vec<String> {
        self.notifications.lock().await.clone()
    }

    pub async fn clear_notifications(&self) {
        self.notifications.lock().await.clear();
    }
}

#[fixture]
pub async fn test_db_service() -> TestDbService {
    TestDbService::new().await
}
```

#### Database Query Testing
```rust
#[rstest]
#[awt]
#[tokio::test]
async fn test_model_crud_operations(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
) -> anyhow::Result<()> {
    // Create
    let model = service.create_model("test-model", "test:latest").await?;
    assert_eq!(model.name, "test-model");
    
    // Read
    let retrieved = service.get_model(&model.alias).await?;
    assert_eq!(retrieved.id, model.id);
    
    // Update
    let updated = service.update_model(&model.alias, "updated-model", None).await?;
    assert_eq!(updated.name, "updated-model");
    
    // Delete
    service.delete_model(&model.alias).await?;
    let result = service.get_model(&model.alias).await;
    assert!(matches!(result, Err(DbError::ModelNotFound { .. })));
    
    Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_pagination(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
) -> anyhow::Result<()> {
    // Create test data
    for i in 1..=15 {
        service.create_model(&format!("model-{}", i), &format!("model{}:latest", i)).await?;
    }
    
    // Test first page
    let page1 = service.list_models(1, 10).await?;
    assert_eq!(page1.len(), 10);
    
    // Test second page
    let page2 = service.list_models(2, 10).await?;
    assert_eq!(page2.len(), 5);
    
    // Test page size
    let small_page = service.list_models(1, 5).await?;
    assert_eq!(small_page.len(), 5);
    
    Ok(())
}
```

#### Test Data Builders
```rust
// crates/services/src/test_utils/builders.rs
use derive_builder::Builder;

#[derive(Builder)]
pub struct ModelBuilder {
    #[builder(default = "uuid::Uuid::new_v4().to_string()")]
    pub id: String,
    pub name: String,
    pub alias: String,
    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}

impl ModelBuilder {
    pub fn test_model() -> Self {
        Self::default()
            .name("test-model".to_string())
            .alias("test:latest".to_string())
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

pub fn create_test_model() -> Model {
    ModelBuilder::test_model().build().unwrap()
}

pub fn create_test_models(count: usize) -> Vec<Model> {
    (0..count)
        .map(|i| {
            ModelBuilder::default()
                .name(format!("model-{}", i))
                .alias(format!("model{}:latest", i))
                .build()
                .unwrap()
        })
        .collect()
}
```

## Integration Testing

### API Endpoint Testing
```rust
// crates/integration-tests/src/api_tests.rs
use axum_test::TestServer;

#[tokio::test]
async fn test_create_model_endpoint() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    let response = server
        .post("/bodhi/v1/models")
        .json(&json!({
            "name": "test-model",
            "alias": "test:latest"
        }))
        .await;

    response.assert_status_created();
    
    let model: Model = response.json();
    assert_eq!(model.name, "test-model");
    assert_eq!(model.alias, "test:latest");
}

#[tokio::test]
async fn test_get_models_with_pagination() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    // Create test data
    for i in 1..=15 {
        server
            .post("/bodhi/v1/models")
            .json(&json!({
                "name": format!("model-{}", i),
                "alias": format!("model{}:latest", i)
            }))
            .await;
    }

    let response = server
        .get("/bodhi/v1/models")
        .add_query_param("page", "1")
        .add_query_param("page_size", "10")
        .await;

    response.assert_status_ok();
    
    let page: PagedResponse<Vec<Model>> = response.json();
    assert_eq!(page.data.len(), 10);
    assert_eq!(page.total, Some(15));
    assert_eq!(page.page, Some(1));
    assert_eq!(page.page_size, Some(10));
}

#[tokio::test]
async fn test_error_handling() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    // Test 404 for non-existent model
    let response = server
        .get("/bodhi/v1/models/nonexistent")
        .await;

    response.assert_status_not_found();
    
    let error: ErrorResponse = response.json();
    assert_eq!(error.error.message, "Model not found");
    assert_eq!(error.error.type_, "not_found");

    // Test 400 for invalid input
    let response = server
        .post("/bodhi/v1/models")
        .json(&json!({
            "name": "",  // Invalid empty name
            "alias": "test:latest"
        }))
        .await;

    response.assert_status_bad_request();
}
```

### Authentication Testing
```rust
#[tokio::test]
async fn test_protected_endpoint_requires_auth() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    let response = server
        .get("/bodhi/v1/models")
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    let token = create_test_token("user123", &["read"]);

    let response = server
        .get("/bodhi/v1/models")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_insufficient_permissions() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    let token = create_test_token("user123", &["read"]);  // Only read permission

    let response = server
        .post("/bodhi/v1/models")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "test-model",
            "alias": "test:latest"
        }))
        .await;

    response.assert_status_forbidden();
}
```

## Performance Testing

### Database Performance Tests
```rust
#[tokio::test]
async fn test_query_performance() {
    let service = TestDbService::new().await;
    
    // Create test data
    for i in 0..1000 {
        service.create_model(&format!("model-{}", i), &format!("alias-{}", i)).await.unwrap();
    }
    
    let start = std::time::Instant::now();
    let results = service.list_models(1, 100).await.unwrap();
    let duration = start.elapsed();
    
    assert!(duration < std::time::Duration::from_millis(50));
    assert_eq!(results.len(), 100);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let service = Arc::new(TestDbService::new().await);
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let service = service.clone();
            tokio::spawn(async move {
                service.create_model(&format!("model-{}", i), &format!("alias-{}", i)).await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // All operations should succeed
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}
```

### API Performance Tests
```rust
#[tokio::test]
async fn test_api_response_time() {
    let app = create_test_app().await;
    let server = TestServer::new(app)?;

    let start = std::time::Instant::now();
    let response = server
        .get("/bodhi/v1/models")
        .await;
    let duration = start.elapsed();

    response.assert_status_ok();
    assert!(duration < std::time::Duration::from_millis(100));
}
```

## Error Testing Patterns

### Database Error Scenarios
```rust
#[tokio::test]
async fn test_duplicate_alias_error() {
    let service = TestDbService::new().await;
    
    // Create first model
    service.create_model("model1", "duplicate:latest").await.unwrap();
    
    // Try to create second model with same alias
    let result = service.create_model("model2", "duplicate:latest").await;
    
    assert!(matches!(result, Err(DbError::DuplicateAlias { .. })));
}

#[tokio::test]
async fn test_foreign_key_constraint() {
    let service = TestDbService::new().await;
    
    // Try to create a record with invalid foreign key
    let result = service.create_model_with_invalid_user("model", "alias", "nonexistent-user").await;
    
    assert!(matches!(result, Err(DbError::ForeignKeyViolation { .. })));
}
```

## Test Commands and Configuration

### Backend Test Commands
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p services

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test -p integration-tests

# Run tests with coverage
cargo tarpaulin --out html
```

### Test Configuration

#### Cargo.toml Test Dependencies
```toml
[dev-dependencies]
rstest = "0.18"
tokio-test = "0.4"
mockall = "0.11"
anyhow = "1.0"
serde_json = "1.0"
axum-test = "14.0"

[features]
test-utils = ["mockall"]
```

#### Test Environment Setup
```rust
// tests/common/mod.rs
use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        env_logger::init();
        // Other one-time setup
    });
}
```

## Common Testing Patterns

### Testing Async Functions
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Testing Error Propagation
```rust
#[tokio::test]
async fn test_error_propagation() {
    let service = create_failing_service();
    let result = service.operation_that_should_fail().await;
    
    match result {
        Err(ServiceError::DatabaseError(db_err)) => {
            assert_eq!(db_err.to_string(), "Connection failed");
        }
        _ => panic!("Expected DatabaseError"),
    }
}
```

### Testing with Timeouts
```rust
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        long_running_operation()
    ).await;
    
    assert!(result.is_ok());
}
```

## Related Documentation

- **[Frontend Testing](frontend-testing.md)** - Frontend testing approaches and patterns
- **[Rust Backend](rust-backend.md)** - Backend service patterns and database integration
- **[Development Conventions](development-conventions.md)** - Testing conventions and file organization
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Complete testing implementation guide

---

*For frontend testing patterns, see [Frontend Testing](frontend-testing.md). For complete testing implementation examples, see [TESTING_GUIDE.md](TESTING_GUIDE.md).*
