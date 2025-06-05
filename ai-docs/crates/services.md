# services - Business Logic Layer

## Overview

The `services` crate implements the core business logic layer of BodhiApp, providing high-level services that orchestrate domain operations, external integrations, and data management. It serves as the bridge between the HTTP routes and the underlying domain objects.

## Purpose

- **Business Logic**: Implements core application workflows and business rules
- **External Integrations**: Manages connections to external services (HuggingFace, OAuth providers)
- **Data Management**: Handles database operations and data persistence
- **Authentication**: Manages user authentication and authorization
- **Configuration**: Application settings and environment management
- **Caching**: Performance optimization through intelligent caching

## Key Services

### Core Application Services

#### App Service (`app_service.rs`)
- Application lifecycle management
- System initialization and shutdown
- Health checks and status monitoring
- Application metadata and version information

#### Data Service (`data_service.rs`)
- High-level data operations
- Business logic for data manipulation
- Transaction management
- Data validation and integrity

#### Cache Service (`cache_service.rs`)
- In-memory caching for performance optimization
- Cache invalidation strategies
- TTL (Time-To-Live) management
- Cache warming and preloading

### Authentication and Security

#### Auth Service (`auth_service.rs`)
- User authentication workflows
- OAuth2 integration
- JWT token management
- Role-based access control
- Session management

#### Session Service (`session_service.rs`)
- User session lifecycle
- Session storage and retrieval
- Session expiration handling
- Cross-request state management

#### Secret Service (`secret_service.rs`)
- Secure secret storage and retrieval
- Encryption/decryption operations
- Key management
- Secure configuration handling

#### Keyring Service (`keyring_service.rs`)
- Operating system keyring integration
- Secure credential storage
- Cross-platform secret management
- API key storage

### External Integrations

#### Hub Service (`hub_service.rs`)
- HuggingFace Hub integration
- Model repository browsing
- File download management
- Repository metadata retrieval
- Model search and discovery

### Configuration and Environment

#### Setting Service (`setting_service.rs`)
- Application configuration management
- User preferences storage
- Environment-specific settings
- Configuration validation and defaults

#### Init Service (`init_service.rs`)
- Application initialization workflows
- Database setup and migrations
- Service dependency resolution
- Startup configuration validation

#### Environment Wrapper (`env_wrapper.rs`)
- Environment variable management
- Configuration loading
- Environment-specific behavior
- Development vs production settings

### Database Layer

#### Database Service (`db/`)
- **`service.rs`**: Core database service implementation
- **`sqlite_pool.rs`**: SQLite connection pool management
- **`objs.rs`**: Database object mappings
- **`error.rs`**: Database-specific error handling

The database layer provides:
- Connection pool management
- Query execution and transaction handling
- Database migrations
- Error handling and recovery

### Token Management

#### Token Service (`token.rs`)
- API token generation and validation
- Token scope management
- Token expiration and renewal
- Token-based authentication

## Directory Structure

```
src/
├── lib.rs                    # Main module exports and constants
├── app_service.rs            # Application lifecycle management
├── auth_service.rs           # Authentication workflows
├── cache_service.rs          # Caching layer
├── data_service.rs           # High-level data operations
├── db/                       # Database layer
│   ├── mod.rs
│   ├── service.rs           # Core database service
│   ├── sqlite_pool.rs       # Connection pool
│   ├── objs.rs              # Database mappings
│   └── error.rs             # Database errors
├── env_wrapper.rs            # Environment management
├── hub_service.rs            # HuggingFace integration
├── init_service.rs           # Initialization workflows
├── keyring_service.rs        # OS keyring integration
├── macros.rs                 # Service macros
├── models.yaml               # Model configuration
├── obj_exts/                 # Object extensions
│   ├── mod.rs
│   └── chat_template_type.rs
├── objs.rs                   # Service object definitions
├── resources/                # Localization resources
│   └── en-US/
├── secret_service.rs         # Secret management
├── service_ext.rs            # Service extensions
├── session_service.rs        # Session management
├── setting_service.rs        # Configuration management
├── test_utils/               # Testing utilities
│   ├── mod.rs
│   ├── app.rs
│   ├── auth.rs
│   ├── data.rs
│   ├── db.rs
│   ├── envs.rs
│   ├── hf.rs
│   ├── objs.rs
│   ├── secret.rs
│   ├── session.rs
│   └── settings.rs
└── token.rs                  # Token management
```

## Key Features

### Dependency Injection
- Services are designed for dependency injection
- Mock implementations for testing
- Interface-based design for flexibility

### Async/Await Support
- Full async/await support throughout
- Non-blocking I/O operations
- Concurrent request handling

### Error Handling
- Comprehensive error propagation
- Service-specific error types
- Graceful error recovery

### Testing Support
- Extensive test utilities in `test_utils/`
- Mock service implementations
- Integration test helpers

### Configuration Management
- Environment-based configuration
- Runtime configuration updates
- Validation and defaults

## Dependencies

### Core Dependencies
- **objs**: Domain objects and types
- **sqlx**: Database operations
- **tokio**: Async runtime
- **serde**: Serialization
- **uuid**: Unique identifier generation

### External Service Dependencies
- **hf-hub**: HuggingFace Hub integration
- **oauth2**: OAuth2 authentication
- **jsonwebtoken**: JWT token handling
- **keyring**: OS keyring integration

### Caching and Performance
- **mini-moka**: In-memory caching
- **async-trait**: Async trait support

## Service Patterns

### Service Trait Pattern
```rust
#[async_trait]
pub trait SomeService {
    async fn operation(&self, params: Params) -> Result<Output, ApiError>;
}
```

### Dependency Injection
Services are injected into route handlers and other services, enabling:
- Easy testing with mock implementations
- Flexible service composition
- Clear dependency relationships

### Error Propagation
All services use the `ApiError` type from the `objs` crate for consistent error handling across the application.

## Integration Points

- **Routes Layer**: Services are injected into route handlers
- **Commands Layer**: CLI commands use services for operations
- **Database**: Services manage all database interactions
- **External APIs**: Services handle all external service communication
- **Configuration**: Services manage application configuration

## Usage Examples

Services are typically used in route handlers:

```rust
async fn handler(
    Extension(app_service): Extension<Arc<dyn AppService>>,
    Extension(auth_service): Extension<Arc<dyn AuthService>>,
) -> Result<Json<Response>, ApiError> {
    // Use services to implement business logic
}
```

## Testing Strategy

The services layer includes comprehensive testing utilities:
- Mock service implementations
- Test data generators
- Integration test helpers
- Database test utilities

This enables thorough testing of business logic without external dependencies.
