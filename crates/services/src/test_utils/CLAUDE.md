# CLAUDE.md - Services Test Utilities

See [PACKAGE.md](./PACKAGE.md) for implementation details and navigation guide.

## Architectural Purpose

The `services/test_utils` module serves as the foundational testing infrastructure for BodhiApp's service layer, providing sophisticated mock orchestration, deterministic test environments, and comprehensive service integration testing capabilities. This module enables testing complex multi-service interactions while maintaining complete isolation from external dependencies.

## Core Testing Architecture Patterns

### Service Composition Engine
The test utilities implement a comprehensive service composition pattern through `AppServiceStub`, which orchestrates complex service dependency graphs:

- **Builder Pattern Integration**: `AppServiceStubBuilder` provides flexible service composition with automatic dependency resolution and configurable real/mock service mixing
- **Service Registry Pattern**: Complete implementation of the `AppService` trait enables seamless substitution in production code paths
- **Dependency Injection Architecture**: Services are injected as `Arc<dyn Trait>` allowing for dynamic service composition and shared ownership across test scenarios
- **Temporal Isolation**: `FrozenTimeService` provides deterministic time operations crucial for testing time-sensitive business logic

### Database Testing Infrastructure
Advanced database testing architecture supporting complex transactional scenarios:

- **Isolation Through Virtualization**: Each test receives a dedicated SQLite database in temporary directories, preventing cross-test contamination
- **Event-Driven Testing**: `TestDbService` broadcasts database operations enabling reactive test validation and operation sequencing verification
- **Migration Testing**: Automatic schema migration in isolated environments with rollback testing capabilities
- **Encrypted Storage Simulation**: Built-in encryption key management for testing secure data persistence without real cryptographic overhead

### Authentication Flow Testing
Comprehensive OAuth2/JWT testing infrastructure simulating real-world authentication scenarios:

- **Deterministic Cryptography**: Embedded RSA key pairs provide consistent JWT signing and validation without external key management
- **Multi-Token Lifecycle**: Support for Bearer tokens, Offline tokens, and Refresh token flows with proper expiration handling
- **Keycloak Integration Testing**: Complete simulation of Keycloak realm configuration, client registration, and resource access patterns
- **Session Coordination**: HTTP session state management synchronized with JWT authentication state for end-to-end validation

### External Service Mocking Strategy
Sophisticated external service mocking enabling offline testing without sacrificing realism:

- **Hybrid Service Architecture**: `TestHfService` and `OfflineHubService` provide configurable real/mock operation modes
- **Test Data Ecosystem**: Automatic copying of realistic test data from `crates/services/tests/data/` ensuring comprehensive offline scenarios
- **Network Isolation**: Prevention of external network calls during testing while maintaining local file system operations
- **Error Scenario Simulation**: Comprehensive simulation of network failures, API rate limits, and service unavailability

## Service Integration Patterns

### Cross-Service Transaction Testing
Complex business flow validation across service boundaries:

- **Download Lifecycle Management**: End-to-end testing of model download requests involving Hub service, Database persistence, and Data service file management
- **Authentication + Database + Session Integration**: OAuth2 token exchange with encrypted database storage and HTTP session coordination
- **API Alias Management**: File system alias creation synchronized with database metadata and Hub service model resolution
- **User Access Request Flows**: Complete user access request lifecycle from creation through approval with multi-service coordination

### Error Propagation Architecture
Cross-service error handling and recovery validation:

- **Error Type Preservation**: Validation that error types are properly converted and contextual information is preserved across service boundaries
- **Rollback Transaction Testing**: Verification of proper cleanup when multi-service transactions fail mid-process
- **Circuit Breaker Pattern Testing**: Testing service degradation scenarios and fallback mechanisms
- **Retry Logic Validation**: Testing exponential backoff and retry strategies across service calls

### Mock Service Coordination
Advanced mock service orchestration for complex testing scenarios:

- **MockAll Integration**: Seamless integration with mockall-generated mocks providing expectation-driven testing
- **Behavior Configuration**: Configurable mock behaviors supporting various success, failure, and edge case scenarios
- **State Coordination**: Mock services that maintain internal state for multi-call interaction testing
- **Selective Mocking**: Ability to mix real and mock services for targeted testing of specific service interactions

## Domain-Specific Testing Patterns

### Hub Service Testing Architecture
Comprehensive model management testing without external dependencies:

- **Repository Access Simulation**: Complete simulation of HuggingFace repository access patterns including gated repositories
- **Download Progress Testing**: Simulation of download progress callbacks and cancellation scenarios
- **Model Cache Management**: Testing local model cache consistency and cleanup operations
- **Tokenizer Configuration Testing**: Validation of tokenizer configuration loading and chat template resolution

### Secret Management Testing
Secure credential management testing without real cryptographic operations:

- **In-Memory Secret Storage**: `SecretServiceStub` provides deterministic secret storage with application registration information
- **Keyring Integration Testing**: `KeyringStoreStub` simulates cross-platform credential storage without system keyring dependencies
- **Application Status Management**: Testing application setup/ready state transitions with secure credential flows
- **OAuth Registration Simulation**: Complete OAuth client registration testing with realistic configuration

### Session Management Testing
HTTP session security and lifecycle testing:

- **Session Security Configuration**: Validation of secure cookie settings, SameSite policies, and session isolation
- **Session Data Persistence**: Testing session data storage and retrieval across HTTP requests
- **Session Expiration Handling**: Testing session timeout and renewal scenarios
- **Cross-Request State Management**: Validation of session state consistency across multiple HTTP transactions

## Testing Infrastructure Constraints

### Service Trait Architecture Requirements
- All service traits must support `#[mockall::automock]` annotation for automatic mock generation
- Service implementations must preserve semantic behavior including proper error condition handling
- Arc-wrapped service storage requires careful lifetime management and clone operations for shared ownership
- Service builder methods must handle lazy initialization and proper dependency injection ordering

### Database Testing Requirements
- SQLite-based testing infrastructure requires temporary database cleanup and migration validation
- Event broadcasting system must handle concurrent access and proper event ordering
- Encryption key management must be deterministic for consistent encrypted data testing across test runs
- Database connection pooling requires proper cleanup to prevent resource leaks in test environments

### Authentication Testing Requirements
- RSA key pair management requires embedded test keys for consistent JWT signing without external key dependencies
- OAuth2 flow simulation must handle realistic error scenarios including network failures and invalid credentials
- JWT claims validation must include proper resource_access role checking and token type differentiation
- Session security testing must validate cookie settings and CSRF protection mechanisms

### External Service Isolation Requirements
- Network call prevention mechanisms must be comprehensive to avoid accidental external dependencies
- Test data copying must handle various file types and directory structures from the test data ecosystem
- Mock service behavior must be configurable for different test scenarios while maintaining realistic API signatures
- Offline mode operation must preserve all functional capabilities except network-dependent operations

## Extension Architecture Guidelines

### Adding New Service Test Support
When extending the test utilities for new services:

1. **Service Registration**: Add service to `AppServiceStub` with proper Arc wrapping and builder pattern integration
2. **Mock Implementation**: Create comprehensive mock using `#[mockall::automock]` with expectation coverage for all service methods
3. **Stub Implementation**: Implement simplified stub version for offline testing with in-memory storage
4. **Integration Testing**: Add cross-service interaction tests validating proper error propagation and state management
5. **Fixture Creation**: Provide rstest fixtures with proper async handling and dependency ordering

### Database Integration Testing Patterns
For services requiring database integration:

1. **Isolation Strategy**: Use `TestDbService` wrapper for event monitoring and database operation isolation
2. **Temporal Control**: Integrate `FrozenTimeService` for deterministic timestamp generation in database records
3. **Migration Testing**: Validate schema changes with realistic data and proper rollback scenarios
4. **Transaction Testing**: Test multi-table operations with proper rollback and consistency validation
5. **Concurrency Testing**: Validate connection pooling and concurrent access patterns

### Security Testing Integration
For authentication and security-related services:

1. **Mock Provider Integration**: Simulate various OAuth provider responses including errors and edge cases
2. **Token Lifecycle Testing**: Comprehensive testing of token creation, validation, refresh, and expiration flows
3. **Session Coordination**: HTTP session state synchronized with authentication state for end-to-end validation
4. **Security Configuration Validation**: Test secure cookie settings, session isolation, and CSRF protection
5. **Cross-Service Authentication**: Validate token exchange and service-to-service authentication patterns

This testing infrastructure enables comprehensive service layer validation while maintaining complete isolation from external dependencies, providing the foundation for reliable integration testing across BodhiApp's complex service architecture.