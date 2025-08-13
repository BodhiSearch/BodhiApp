# CLAUDE.md - services

This file provides guidance to Claude Code when working with the `services` crate, which contains the business logic layer for BodhiApp.

## Purpose

The `services` crate implements the business logic layer providing:

- **Service Layer Architecture**: Trait-based service abstractions with dependency injection
- **Authentication & Authorization**: OAuth2 flows, JWT tokens, and session management
- **Model Management**: Hugging Face Hub integration, local file management, and alias system
- **Data Persistence**: SQLite database operations with migrations
- **Security Services**: Encryption, secrets management, and keyring integration
- **Configuration Management**: Settings, environment variables, and application state
- **Caching**: In-memory caching for performance optimization

## Key Components

### Service Architecture (`src/app_service.rs`)
- `AppService` trait - Central service registry providing access to all business services
- `DefaultAppService` - Concrete implementation with dependency injection
- Service composition pattern with Arc<dyn Trait> for thread-safe shared ownership

### Authentication Services (`src/auth_service.rs`)
- OAuth2 client registration and authorization code exchange
- JWT token management with refresh token support
- Token exchange for service-to-service communication
- Integration with external OAuth providers

### Model Hub Services (`src/hub_service.rs`)
- Hugging Face Hub API integration for model discovery and download
- Model file management with snapshot versioning
- Repository metadata caching and local storage
- Error handling for gated repositories and network issues

### Data Services (`src/data_service.rs`)
- Model alias management with YAML persistence
- Local model file organization and indexing
- Remote model registry with metadata synchronization
- File system operations with safe filename generation

### Database Services (`src/db/`)
- SQLite connection pooling and migration management
- Schema evolution with versioned migrations
- Transaction support for data consistency
- Time service abstraction for testability

### Security Services
- **Secret Service** (`src/secret_service.rs`): Encryption/decryption with AES-GCM
- **Keyring Service** (`src/keyring_service.rs`): Secure credential storage per platform
- **Session Service** (`src/session_service.rs`): HTTP session management with SQLite backend

### Configuration Services (`src/setting_service.rs`)
- Environment variable management with validation
- Application settings with runtime configuration updates
- File-based configuration with YAML support

### Caching Services (`src/cache_service.rs`)
- In-memory caching with TTL and size limits using mini-moka
- Cache invalidation strategies for data consistency

## Dependencies

### Core Infrastructure
- `objs` - Domain objects, error types, and validation
- `llama_server_proc` - LLM server process management
- `errmeta_derive` - Error metadata generation

### Authentication & Security
- `oauth2` - OAuth2 client implementation
- `jsonwebtoken` - JWT token creation and validation
- `aes-gcm` - AES-GCM encryption for secrets
- `pbkdf2` - Password-based key derivation
- `keyring` - Cross-platform secure credential storage

### Database & Persistence
- `sqlx` - Async SQL toolkit with SQLite support
- `tower-sessions` - HTTP session management
- `serde_yaml` - YAML serialization for configuration

### External Integrations
- `hf-hub` - Hugging Face Hub API client
- `reqwest` - HTTP client for API communications

### Caching & Performance
- `mini-moka` - High-performance in-memory cache
- `tokio` - Async runtime for concurrent operations

### Development & Testing
- `mockall` - Mock object generation for testing
- `rstest` - Parameterized testing framework

## Architecture Position

The `services` crate sits at the business logic layer:
- **Above**: Domain objects (`objs`) and infrastructure (`llama_server_proc`)
- **Below**: HTTP routes, server infrastructure, and application entry points
- **Interfaces**: Provides trait abstractions for dependency injection
- **Integration**: Bridges external APIs with internal domain models

## Usage Patterns

### Service Registration and Dependency Injection
```rust
use services::{
    AppService, DefaultAppService, AuthService, HubService, DataService
};

let app_service = DefaultAppService::new(
    setting_service,
    hub_service, 
    data_service,
    auth_service,
    db_service,
    session_service,
    secret_service,
    cache_service,
    localization_service,
    time_service,
);

// Access services through the registry
let hub_service = app_service.hub_service();
let models = hub_service.list_models().await?;
```

### OAuth2 Authentication Flow
```rust
use services::AuthService;

// Register OAuth client
let app_reg = auth_service.register_client(
    "MyApp".to_string(),
    "Description".to_string(),
    vec!["http://localhost:8080/callback".to_string()],
).await?;

// Exchange authorization code for tokens
let (access_token, refresh_token) = auth_service.exchange_auth_code(
    auth_code,
    client_id,
    client_secret,
    redirect_uri,
    code_verifier,
).await?;
```

### Model Management
```rust
use services::{HubService, DataService};

// Search and download models from Hugging Face Hub
let models = hub_service.list_models("microsoft/DialoGPT").await?;
let model_files = hub_service.download_model(&repo, &snapshot).await?;

// Manage local aliases
let alias = AliasBuilder::default()
    .name("gpt-3.5-turbo")
    .source(AliasSource::HuggingFace { repo: "microsoft/DialoGPT".into() })
    .build()?;
    
data_service.save_alias(&alias)?;
```

### Database Operations
```rust
use services::db::{DbService, Transaction};

let tx = db_service.begin_transaction().await?;
// Perform multiple database operations
tx.commit().await?;
```

### Secret Management
```rust
use services::{SecretService, KeyringService};

// Store encrypted secrets
let encrypted = secret_service.encrypt(&plaintext_secret).await?;

// Store credentials securely in system keyring
keyring_service.store_credential("api_key", &token).await?;
let stored_token = keyring_service.retrieve_credential("api_key").await?;
```

## Integration Points

### With HTTP Layer (Routes)
- Services injected into route handlers via dependency injection
- Error types converted to HTTP responses with proper status codes
- Authentication middleware uses auth services for token validation

### With External APIs
- Hugging Face Hub for model discovery and download
- OAuth providers for user authentication
- External services via HTTP client with proper error handling

### With Database Layer
- SQLite for persistent storage with connection pooling
- Migration management for schema evolution
- Transaction support for data consistency

### With File System
- Local model storage with organized directory structure
- Configuration file management with atomic updates
- Temporary file handling for downloads and processing

## Error Handling Strategy

### Service-Specific Errors
Each service defines its own error enum implementing `AppError`:
- `AuthServiceError` - Authentication and token exchange errors
- `HubServiceError` - Model hub and download errors  
- `DataServiceError` - File operations and alias management errors
- `DatabaseError` - SQL operations and connection errors

### Error Propagation
- Transparent error wrapping with `#[error(transparent)]`
- Automatic error conversion using `impl_error_from!` macro
- Localized error messages via `errmeta_derive`

### HTTP Status Mapping
Errors map to appropriate HTTP status codes:
- `BadRequest` (400) - Validation failures, malformed input
- `Authentication` (401) - Invalid or missing credentials
- `Forbidden` (403) - Insufficient permissions
- `NotFound` (404) - Resources not found
- `InternalServer` (500) - System errors and external service failures

## Security Considerations

### Credential Management
- Never store plaintext secrets in memory or logs
- Use system keyring for persistent credential storage
- Encrypt sensitive data with AES-GCM before database storage

### Authentication
- JWT tokens with proper expiration and refresh mechanisms
- OAuth2 PKCE for authorization code flow security
- Session management with secure cookie handling

### Data Protection
- Input validation on all service boundaries
- SQL injection prevention via parameterized queries
- File path validation to prevent directory traversal

## Testing Strategy

### Mock Services
- All service traits have `#[mockall::automock]` for unit testing
- Dependency injection enables easy mock substitution
- Isolated testing of business logic without external dependencies

### Integration Testing
- Database tests with temporary SQLite files
- HTTP client mocking for external API testing
- File system operations with temporary directories

## Development Guidelines

### Adding New Services
1. Define service trait with async methods
2. Create error enum implementing `AppError`
3. Implement concrete service with proper error handling
4. Add mock generation with `#[mockall::automock]`
5. Register service in `AppService` trait and implementation

### Database Changes
1. Create new migration file in appropriate module
2. Update database schema definitions
3. Add migration to schema evolution logic
4. Test migration with existing data

### External API Integration
1. Use `reqwest` for HTTP client operations
2. Implement proper timeout and retry strategies
3. Handle API rate limiting and error responses
4. Cache responses when appropriate for performance

## Performance Considerations

### Caching Strategy
- Use `CacheService` for frequently accessed data
- Implement cache invalidation for data consistency
- Consider cache warming for critical data paths

### Database Optimization
- Use connection pooling to prevent connection exhaustion
- Implement proper indexing for query performance
- Use transactions appropriately for data consistency

### Async Operations
- All service methods are async for non-blocking I/O
- Use `tokio` runtime features for concurrent processing
- Avoid blocking operations in async contexts