# Rust Backend Development

This document provides focused guidance for Rust backend development in the Bodhi App, including service patterns, database integration, and development best practices.

## Required Documentation References

**MUST READ before any Rust changes:**
- `ai-docs/01-architecture/system-overview.md` - System architecture and crate organization
- `ai-docs/01-architecture/development-conventions.md` - Rust-specific patterns and database conventions

**FOR SPECIFIC CRATES:**
- Reference `ai-docs/03-crates/[specific-crate].md` for detailed crate documentation

## Technology Stack

### Core Technologies
- **Rust** - Systems programming language for performance and safety
- **Axum** - Web framework for HTTP APIs with excellent performance
- **Tokio** - Async runtime for concurrent operations
- **SQLx** - Async SQL toolkit with compile-time checked queries
- **SQLite** - Embedded database for local data storage

### Additional Libraries
- **Serde** - Serialization/deserialization framework
- **UUID** - Unique identifier generation
- **Chrono** - Date and time handling
- **Anyhow/Thiserror** - Error handling
- **Tracing** - Structured logging and diagnostics

## Crate Organization

### Foundation Crates
- **objs** - Domain objects, types, errors, validation
- **services** - Business logic, external integrations
- **server_core** - HTTP server infrastructure
- **auth_middleware** - Authentication and authorization

### API Crates
- **routes_oai** - OpenAI-compatible API endpoints
- **routes_app** - Application-specific API endpoints
- **routes_all** - Unified route composition

### Utility Crates
- **llama_server_proc** - LLM process management
- **errmeta_derive** - Error metadata macros
- **integration-tests** - End-to-end testing
- **xtask** - Build automation

## Database Layer Patterns

### Migration Files
- **Location**: `crates/services/migrations/`
- **Naming**: `NNNN_descriptive_name.{up,down}.sql`
- **Format**: Plain SQL with descriptive comments
- Always include both up and down migrations

### Database Models
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, derive_builder::Builder)]
pub struct ModelName {
    pub id: String,                    // UUID as string
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // ... other fields
}
```

### Enums with Serialization
```rust
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum StatusType {
    Active,
    Inactive,
}
```

## Service Layer Patterns

### Trait Definitions
```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait DbService: std::fmt::Debug + Send + Sync {
    async fn method_name(&self, param: Type) -> Result<ReturnType, DbError>;
}
```

### Service Implementation
```rust
impl DbService for DbServiceImpl {
    async fn get_items(&self, status: StatusType, limit: i64, offset: i64) -> Result<Vec<Item>, DbError> {
        let items = query_as::<_, Item>(
            "SELECT id, name, status, created_at, updated_at 
             FROM items 
             WHERE status = ? 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?"
        )
        .bind(status.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(items)
    }
}
```

### Database Query Patterns
- **Use SQLx**: Prefer `query_as` over raw `query!` macro
- **Error handling**: Implement proper error handling with `DbError`
- **Bind parameters**: Use bind parameters for all values
- **Type safety**: Leverage Rust's type system for query safety

```rust
// Good: Type-safe query with bind parameters
query_as::<_, (String, String, DateTime<Utc>)>(
    "SELECT id, name, created_at FROM table WHERE status = ? LIMIT ? OFFSET ?"
)
.bind(status.to_string())
.bind(limit)
.bind(offset)
.fetch_all(&pool)
.await?
```

## API Route Patterns

### Route Handler Structure
```rust
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreateResponse {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_item(
    State(services): State<AppServices>,
    Json(request): Json<CreateRequest>,
) -> Result<Json<CreateResponse>, AppError> {
    let item = services.db_service
        .create_item(&request.name, request.description.as_deref())
        .await?;
    
    Ok(Json(CreateResponse {
        id: item.id,
        name: item.name,
        created_at: item.created_at,
    }))
}
```

### Route Registration
```rust
use axum::{routing::post, Router};

pub fn create_routes() -> Router<AppServices> {
    Router::new()
        .route("/items", post(create_item))
        .route("/items/:id", get(get_item))
}
```

## Error Handling Patterns

### Error Types with Metadata
```rust
use errmeta_derive::ErrorMeta;

#[derive(Debug, ErrorMeta)]
pub enum DbError {
    #[error_meta(
        code = "DB_CONNECTION_FAILED",
        message = "Failed to connect to database",
        status = 500
    )]
    ConnectionFailed(#[source] sqlx::Error),
    
    #[error_meta(
        code = "ITEM_NOT_FOUND",
        message = "Item not found",
        status = 404
    )]
    ItemNotFound { id: String },
}
```

### Error Conversion
```rust
impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::ItemNotFound { id } => AppError::NotFound {
                resource: "item".to_string(),
                id,
            },
            _ => AppError::Internal(err.into()),
        }
    }
}
```

## Testing Standards

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    
    #[rstest]
    #[awt]
    #[tokio::test]
    async fn test_create_item() {
        let service = create_test_service().await;
        let result = service.create_item("test", None).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests with Database
```rust
use crate::test_utils::TestDbService;

#[tokio::test]
async fn test_full_workflow() {
    let db_service = TestDbService::new().await;
    
    // Test database operations
    let item = db_service.create_item("test", None).await.unwrap();
    assert_eq!(item.name, "test");
    
    let retrieved = db_service.get_item(&item.id).await.unwrap();
    assert_eq!(retrieved.id, item.id);
}
```

### Mock Services for Testing
```rust
use mockall::predicate::*;

#[tokio::test]
async fn test_with_mock() {
    let mut mock_service = MockDbService::new();
    
    mock_service
        .expect_get_item()
        .with(eq("test-id"))
        .times(1)
        .returning(|_| Ok(create_test_item()));
    
    let result = mock_service.get_item("test-id").await;
    assert!(result.is_ok());
}
```

## Configuration Management

### Environment Configuration
```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
    pub auth_enabled: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        envy::from_env().map_err(ConfigError::from)
    }
}
```

### Runtime Configuration Updates
```rust
pub struct ConfigService {
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigService {
    pub async fn update_config(&self, new_config: AppConfig) -> Result<(), ConfigError> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }
    
    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }
}
```

## Async Patterns

### Concurrent Operations
```rust
use tokio::try_join;

pub async fn process_multiple_items(items: Vec<String>) -> Result<Vec<ProcessedItem>, ProcessError> {
    let futures = items.into_iter().map(|item| process_item(item));
    let results = try_join_all(futures).await?;
    Ok(results)
}
```

### Background Tasks
```rust
use tokio::spawn;

pub async fn start_background_task() -> JoinHandle<Result<(), TaskError>> {
    spawn(async move {
        loop {
            // Background work
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    })
}
```

## Security Best Practices

### Input Validation
```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
}

pub async fn create_user(request: CreateUserRequest) -> Result<User, ValidationError> {
    request.validate()?;
    // Process validated request
}
```

### SQL Injection Prevention
```rust
// Always use bind parameters
let user = query_as::<_, User>(
    "SELECT * FROM users WHERE email = ? AND status = ?"
)
.bind(&email)  // Bind parameter prevents SQL injection
.bind("active")
.fetch_one(&pool)
.await?;
```

## Performance Considerations

### Connection Pooling
```rust
use sqlx::sqlite::SqlitePoolOptions;

pub async fn create_db_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}
```

### Efficient Queries
```rust
// Use appropriate indexes and limit results
let recent_items = query_as::<_, Item>(
    "SELECT * FROM items 
     WHERE created_at > ? 
     ORDER BY created_at DESC 
     LIMIT ?"
)
.bind(since_date)
.bind(limit)
.fetch_all(&pool)
.await?;
```

## HTTP Status Code Mapping

### Standard Response Codes
- **200**: Success - Request completed successfully
- **201**: Created - Resource created successfully
- **400**: Bad Request - Validation errors or malformed request
- **401**: Unauthorized - Authentication required
- **403**: Forbidden - Insufficient permissions for the requested operation
- **404**: Not Found - Requested resource does not exist
- **409**: Conflict - Resource already exists or state conflict
- **422**: Unprocessable Entity - Business logic errors
- **500**: Internal Server Error - Unexpected server error

### Error Response Format
```json
{
  "error": {
    "message": "User-friendly error message",
    "type": "validation_error",
    "code": "INVALID_INPUT",
    "param": "field_name"
  }
}
```

## API Token Management

### Token Types and Scopes
- **Session Tokens**: Short-lived, cookie-based authentication for web clients
- **API Tokens**: Long-lived tokens for programmatic access
- **Refresh Tokens**: For automatic token renewal

### Token Scopes
- **`read`**: Read-only access to resources
- **`write`**: Full CRUD operations on user-owned resources
- **`admin`**: Administrative functions and system-wide operations

### Token Validation
```rust
// Token validation in auth middleware
pub async fn validate_token(token: &str) -> Result<TokenClaims, AuthError> {
    // JWT validation logic
    // Scope verification
    // Expiration checking
}
```

## Settings Service Architecture

### Configuration Loading Priority
The settings service loads configuration from multiple sources in this priority order:

1. **Command line arguments** - Highest priority, overrides all other sources
2. **Environment variables** - Second priority, useful for deployment configuration
3. **Configuration files** - Third priority, for persistent local settings
4. **Database settings** - Fourth priority, for user-specific preferences
5. **Default values** - Lowest priority, fallback values

### Settings Categories
- **System Settings**: Core application configuration (server port, database URL)
- **User Preferences**: Per-user customizations (theme, language)
- **Model Defaults**: Default parameters for LLM models
- **Feature Flags**: Experimental feature toggles

## Real-time Communication

### Server-Sent Events (SSE)

**Chat Streaming Implementation**:
- Chunked response processing
- Real-time content delivery
- Metadata extraction
- Error handling during streaming
- Connection management

**SSE Message Format**:
```
data: {"choices":[{"delta":{"content":"Hello"}}]}

data: {"choices":[{"delta":{"content":" world"}}]}

data: {"choices":[{"finish_reason":"stop","usage":{"total_tokens":10}}]}

data: [DONE]
```

### WebSocket Considerations

While currently using SSE for chat streaming, the architecture supports WebSocket integration for:
- Bidirectional communication
- Real-time notifications
- Collaborative features
- System status updates

## Logging & Observability

**Structured Logging**:
- Request/response logging
- Performance metrics
- Error tracking
- User action auditing

**Log Levels**:
- `ERROR`: System errors requiring attention
- `WARN`: Recoverable issues
- `INFO`: General application flow
- `DEBUG`: Detailed debugging information
- `TRACE`: Fine-grained execution details

## Future Improvements

### Service Architecture Roadmap
1. **Microservice Decomposition**
   - Split monolithic services into focused microservices
   - Implement service mesh for inter-service communication
   - Establish clear service boundaries and contracts

2. **Performance Enhancements**
   - Horizontal scaling capabilities with load balancing
   - Database sharding for improved query performance
   - CDN integration for static asset delivery
   - Response compression and caching strategies

3. **Monitoring & Observability**
   - Real-time metrics dashboard with key performance indicators
   - Comprehensive alerting system for proactive issue detection
   - Performance analytics and bottleneck identification
   - Health check endpoints for service monitoring

4. **Security Improvements**
   - Advanced threat detection and prevention
   - Rate limiting implementation to prevent abuse
   - Automated API security scanning
   - Compliance automation for security standards

## Related Documentation

- **[System Overview](system-overview.md)** - High-level system architecture
- **[Authentication](authentication.md)** - Security implementation details
- **[API Integration](api-integration.md)** - Frontend-backend integration patterns
- **[Backend Testing](backend-testing.md)** - Backend testing approaches
- **[Development Conventions](development-conventions.md)** - Coding standards and best practices

---

*For detailed crate-specific implementation, see the [Crates](../03-crates/) documentation. For API patterns, see [API Integration](api-integration.md).*
