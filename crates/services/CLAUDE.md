# CLAUDE.md - services Crate

See [crates/services/PACKAGE.md](crates/services/PACKAGE.md) for implementation details.

## Purpose

The `services` crate implements BodhiApp's **business logic orchestration layer**, providing 11 interconnected services that coordinate OAuth2 authentication, AI API integrations, model management, user access control, data persistence, and multi-layer security. This crate bridges domain objects from `objs` with external systems while maintaining deployment flexibility across standalone servers, desktop applications, and embedded contexts.

## Architectural Design Rationale

### Why Service Registry Pattern

BodhiApp chose a trait-based service registry pattern over traditional dependency injection frameworks for several critical reasons:

1. **Compile-Time Safety**: The `AppService` trait ensures all service dependencies are satisfied at compile time, preventing runtime surprises in production deployments
2. **Testing Isolation**: Each service trait has `#[mockall::automock]` annotations enabling comprehensive mock generation for unit testing without external dependencies
3. **Deployment Flexibility**: The registry pattern allows different service implementations across deployment contexts (server vs desktop vs embedded) without architectural changes
4. **Thread-Safe Concurrency**: All services implement `Send + Sync + Debug`, enabling safe sharing across async tasks and worker threads
5. **Explicit Dependencies**: Service constructors explicitly declare dependencies through the `derive-new` pattern, making the dependency graph clear and maintainable

### Why Multi-Layer Authentication

The authentication system spans multiple services rather than a monolithic auth module because:

1. **Separation of Concerns**: Each service handles a specific aspect - OAuth2 flows (AuthService), credential encryption (SecretService), session management (SessionService), and persistent storage (KeyringService)
2. **Platform Abstraction**: KeyringService abstracts OS-specific credential storage (Keychain, Secret Service, Windows Credential Manager) behind a unified interface
3. **Security Defense in Depth**: Multiple layers ensure credentials are never exposed - encrypted in database, protected by platform keyring, and transmitted via secure sessions
4. **Token Lifecycle Management**: Complex JWT refresh logic, token exchange protocols, and session coordination require specialized handling across service boundaries
5. **Keycloak Integration**: Custom Bodhi API endpoints for resource management and dynamic admin assignment require coordinated service interactions

### Why Separated Model Management Services

Model management is split between HubService and DataService rather than a single service because:

1. **External vs Local Concerns**: HubService handles Hugging Face API interactions while DataService manages local file system operations
2. **Offline Capability**: DataService can operate without network access, enabling offline model usage and testing
3. **Error Recovery Boundaries**: Network failures in HubService don't affect local model operations in DataService
4. **Cache Coherency**: Separation allows independent caching strategies - API responses in HubService, file metadata in DataService
5. **Testing Isolation**: OfflineHubService enables testing without external API dependencies while DataService tests focus on file operations

## Cross-Crate Coordination Patterns

### Service to HTTP Infrastructure Flow

Services integrate with HTTP infrastructure through specific coordination points:

**Model Resolution Pipeline**:
- HTTP request arrives at routes_oai chat completion endpoint
- Route handler queries DataService.find_alias() for model resolution
- DataService returns Alias object with request parameter overlays
- SharedContext uses HubService.find_local_file() to locate GGUF file
- LLM server process launched with resolved model path

**Error Translation Chain**:
- Service method returns domain-specific error (e.g., HubServiceError::GatedAccess)
- Error implements AppError trait via errmeta_derive
- RouterStateError wraps service error with HTTP context
- Error converts to ApiError with OpenAI-compatible format
- Response includes localized error message from objs LocalizationService

### Service to Authentication Middleware Coordination

Authentication flows span services and middleware with precise coordination:

**Token Exchange Flow**:
1. External client presents access token to auth_middleware
2. Middleware queries AuthService for token validation
3. AuthService checks DbService for cached validation result
4. If expired, AuthService initiates RFC 8693 token exchange with Keycloak
5. New token stored in DbService with expiration tracking
6. SessionService creates/updates HTTP session with user context
7. Middleware attaches authenticated user to request extensions

**Access Control Decision**:
1. Route handler specifies required Role/Scope via auth_middleware
2. Middleware extracts user context from SessionService
3. Role hierarchy checked against required permissions
4. Resource-specific scopes validated through TokenScope parsing
5. Access granted or denied with appropriate error response

### Service to Database Transaction Boundaries

Database operations maintain consistency through coordinated transactions:

**Model Download Transaction**:
```rust
// Conceptual transaction boundary
db.transaction(|tx| {
  // HubService creates download request record
  let request_id = hub_service.create_download_request(tx, &repo)?;

  // DataService reserves local storage path
  let path = data_service.reserve_model_path(tx, &repo)?;

  // HubService performs download with progress tracking
  hub_service.download_with_progress(tx, request_id, &path)?;

  // DataService registers completed model
  data_service.register_model(tx, &repo, &path)?;

  // Transaction commits atomically
  Ok(())
})
```

### Service to CLI Integration Patterns

Services adapt to CLI context through specialized error handling:

**CLI Error Translation**:
- Service errors bubble up to commands crate
- CLI-specific formatting applied (no JSON envelopes)
- Progress bars integrate with HubService download tracking
- Interactive prompts use DataService for model selection
- Configuration updates through SettingService persist across sessions

## Domain-Specific Architecture Patterns

### OAuth2 with Dynamic Client Registration

BodhiApp implements a sophisticated OAuth2 flow with runtime client registration:

**Why Dynamic Registration**:
- Eliminates pre-shared client credentials reducing deployment complexity
- Enables per-installation client isolation for security
- Supports custom Bodhi API endpoints for resource administration
- Allows runtime realm configuration without rebuild

**Registration Sequence**:
1. AuthService detects missing client configuration
2. Registers new OAuth client with Keycloak using Bodhi API
3. Stores encrypted client credentials via SecretService
4. Persists registration metadata in platform keyring
5. Subsequent requests use cached credentials

### AI API Service Testing Architecture

The AI API service implements comprehensive model testing capabilities:

**Test Strategy Rationale**:
- Validates API keys before expensive operations
- Tests model availability with minimal token usage
- Provides configurable test prompts for different providers
- Handles provider-specific rate limiting gracefully
- Categorizes failures for appropriate user guidance

**Error Classification Hierarchy**:
```
AiApiServiceError
├── ApiKeyNotConfigured     → Guides configuration
├── ModelNotFound           → Suggests alternatives
├── AuthenticationFailed    → Prompts credential update
├── RateLimitExceeded      → Implements backoff
└── ApiError               → Provider-specific handling
```

### Platform-Agnostic Credential Storage

The layered credential storage system ensures security across platforms:

**Storage Layers**:
1. **Database**: Encrypted credentials with AES-GCM
2. **Platform Keyring**: OS-specific secure storage
3. **Session Cookies**: Temporary authentication state
4. **Memory Cache**: Short-lived token cache

**Why This Layering**:
- Database encryption protects at-rest credentials
- Platform keyring leverages OS security features
- Session cookies enable stateless HTTP requests
- Memory cache reduces token validation overhead

## Critical Design Decisions

### Time Service Abstraction

All timestamp operations flow through TimeService rather than direct Utc::now() calls:

**Rationale**:
- Enables deterministic testing with FrozenTimeService
- Ensures consistent timestamp format across services
- Removes nanosecond precision for cross-platform compatibility
- Facilitates time-travel testing for token expiration

**Implementation Impact**:
- Service constructors accept TimeService parameter
- Database records use TimeService for created_at/updated_at
- Token validation checks expiration via TimeService
- Tests inject FrozenTimeService for reproducibility

### Event Broadcasting for Reactive Coordination

Database operations broadcast change events enabling reactive testing:

**Design Benefits**:
- Eliminates polling in integration tests
- Enables cache invalidation on data changes
- Supports real-time UI updates in future
- Maintains loose coupling between services

**Current Usage**:
- Test infrastructure listens for operation completion
- Cache service invalidates on model changes
- Session service updates on authentication events

### Offline Testing with Service Stubs

Each external service has an offline stub implementation:

**OfflineHubService**:
- Returns predefined model metadata
- Simulates download progress
- Enables testing without Hugging Face API

**MockAuthService**:
- Provides deterministic token generation
- Simulates OAuth2 flows locally
- Enables auth testing without Keycloak

**Benefits**:
- Fast unit tests without network dependencies
- Deterministic test execution
- Reduced API rate limit consumption
- Simplified CI/CD pipeline

## Security Architecture Decisions

### Why PBKDF2 with 1000 Iterations

The SecretService uses PBKDF2 for key derivation with specific parameters:

**Design Rationale**:
- 1000 iterations balances security with performance for interactive operations
- Random salt per encryption prevents rainbow table attacks
- AES-GCM provides authenticated encryption detecting tampering
- Base64 encoding enables safe storage in text-based formats

**Trade-offs Considered**:
- Higher iteration counts (100,000+) would improve security but impact UX
- Argon2 would provide better security but lacks wide platform support
- Hardware security modules would be ideal but limit deployment flexibility

### Session Security Configuration

HTTP sessions use specific security settings:

**Current Configuration**:
- SameSite::Strict prevents CSRF attacks
- SQLite backend enables horizontal scaling
- TODO: Secure flag pending HTTPS deployment

**Future Enhancements**:
- Enable Secure flag when HTTPS is universal
- Implement session rotation on privilege escalation
- Add session timeout with automatic cleanup

## Extension Guidelines

### Adding New Services

When creating new services for the ecosystem:

1. **Define Service Trait**: Create trait with async methods and mockall annotation
2. **Implement Service**: Provide concrete implementation with proper error handling
3. **Add to Registry**: Extend AppService trait and DefaultAppService
4. **Create Test Utils**: Add mock builders in test_utils module
5. **Document Dependencies**: Update service interdependency documentation

### Extending Authentication

For new authentication providers or flows:

1. **Implement Provider Trait**: Define provider-specific operations
2. **Extend AuthService**: Add provider-specific methods
3. **Update Token Handling**: Ensure token refresh logic handles provider
4. **Add Error Types**: Create provider-specific error variants
5. **Test Integration**: Verify session and database coordination

### Adding External Integrations

When integrating new external services:

1. **Create Service Abstraction**: Hide external API behind trait
2. **Implement Offline Stub**: Enable testing without external dependency
3. **Add Error Classification**: Categorize failures appropriately
4. **Implement Retry Logic**: Handle transient failures gracefully
5. **Document Rate Limits**: Specify any API constraints

## Testing Architecture

### Service Testing Patterns

Services use consistent testing patterns:

**Unit Test Structure**:
```rust
#[cfg(test)]
mod tests {
  use super::*;
  use mockall::predicate::*;

  #[tokio::test]
  async fn test_service_operation() {
    // Arrange: Setup mocks
    let mut mock_dep = MockDependency::new();
    mock_dep.expect_method()
      .with(eq("param"))
      .returning(|_| Ok(()));

    // Act: Execute service method
    let service = ServiceImpl::new(Arc::new(mock_dep));
    let result = service.operation("param").await;

    // Assert: Verify behavior
    assert!(result.is_ok());
  }
}
```

**Integration Test Pattern**:
```rust
#[rstest]
#[tokio::test]
async fn test_service_integration(
  #[future] app_service: Arc<dyn AppService>
) {
  let app_service = app_service.await;

  // Test cross-service interaction
  let result = app_service.hub_service()
    .list_models("llama")
    .await;

  assert!(!result.unwrap().is_empty());
}
```

### Mock Service Coordination

Complex operations require coordinated mock expectations:

**Authentication Flow Mocking**:
```rust
// Setup auth service mock
auth_mock.expect_validate_token()
  .returning(|_| Ok(User::test_user()));

// Setup session service mock
session_mock.expect_create_session()
  .returning(|_| Ok(SessionId::new()));

// Setup database mock
db_mock.expect_save_token()
  .returning(|_| Ok(()));

// Coordinate service interactions
let result = coordinate_auth_flow(auth_mock, session_mock, db_mock).await;
```

## Performance Considerations

### Caching Strategy

Services implement multi-level caching:

**Cache Levels**:
1. **Memory (Mini-Moka)**: Hot data with TTL
2. **Database**: Persistent cache with expiration
3. **File System**: Model metadata cache

**Cache Invalidation**:
- Time-based expiration for external data
- Event-based invalidation for local changes
- Manual invalidation for configuration updates

### Connection Pool Management

Database connections are carefully managed:

**Pool Configuration**:
- SQLite WAL mode for concurrent reads
- Limited connection count for embedded scenarios
- Connection timeout for deadlock prevention

### Async Operation Coordination

Services coordinate async operations efficiently:

**Strategies**:
- Buffered streaming for large downloads
- Parallel fetching for independent operations
- Request coalescing for duplicate calls
- Timeout enforcement for external APIs

## Critical Invariants

### Service Initialization Order

Services must initialize in dependency order:
1. TimeService (no dependencies)
2. DbService (depends on TimeService)
3. SecretService (depends on DbService)
4. SettingService (depends on SecretService)
5. AuthService (depends on above)
6. SessionService (depends on AuthService)
7. Remaining services

### Thread Safety Requirements

All services must be thread-safe:
- Implement Send + Sync + Debug
- Use Arc for shared ownership
- Avoid interior mutability without synchronization
- Prefer immutable operations

### Error Context Preservation

Service errors must preserve context:
- Original error as source
- Operation being performed
- Relevant parameters (sanitized)
- Suggested recovery action