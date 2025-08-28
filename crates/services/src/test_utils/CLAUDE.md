# CLAUDE.md

This file provides guidance to Claude Code when working with the `test_utils` module for services.

*For implementation details and extension patterns, see [crates/services/src/test_utils/PACKAGE.md](crates/services/src/test_utils/PACKAGE.md)*

## Purpose

The `test_utils` module provides specialized testing infrastructure for BodhiApp's service layer, enabling complex multi-service integration testing with sophisticated mock coordination and realistic service composition.

## Key Testing Infrastructure Categories

### Service Composition Testing
Comprehensive service integration testing capabilities:
- **AppServiceStub**: Complete service registry with configurable implementations
- **AppServiceStubBuilder**: Flexible service composition for different test scenarios
- **Service interdependency testing**: Validates complex authentication and model management flows
- **Mock service coordination**: Ensures service interactions work correctly across boundaries

### Authentication Flow Testing
Multi-stage authentication testing infrastructure:
- **OAuth2 flow simulation**: Complete authorization code to token exchange testing
- **JWT token lifecycle testing**: Token creation, validation, refresh, and expiration scenarios
- **Session integration testing**: HTTP session coordination with authentication state
- **Token exchange testing**: Service-to-service authentication validation
- **Multi-provider testing**: Support for different OAuth2 identity providers

### Database Integration Testing
Sophisticated database testing with realistic scenarios:
- **TestDbService**: SQLite database with temporary storage and migration support
- **FrozenTimeService**: Deterministic time operations for reproducible testing
- **Transaction testing**: Cross-service transaction coordination and rollback scenarios
- **Migration testing**: Database schema evolution testing with data preservation
- **Event broadcasting**: Database change notification testing for reactive systems

### Mock Service Orchestration
Advanced mock service coordination for complex scenarios:
- **MockHubService**: Hugging Face Hub API simulation with various response scenarios
- **MockAuthService**: OAuth2 and JWT service mocking with configurable flows
- **OfflineHubService**: Local-only model management testing without external dependencies
- **Service stub composition**: Mix of real and mock services for targeted testing

### Security Testing Infrastructure
Comprehensive security validation testing:
- **SecretServiceStub**: Encryption/decryption testing with deterministic keys
- **Keyring testing**: Platform keyring integration testing with mock implementations
- **Session security testing**: Cookie security and session isolation validation
- **Credential flow testing**: End-to-end credential storage and retrieval testing

## Architecture Position

The `test_utils` module serves as:
- **Service Integration Hub**: Enables testing of complex service interactions
- **Mock Coordination Layer**: Provides sophisticated service mocking capabilities
- **Database Testing Foundation**: Supports realistic database integration testing
- **Authentication Testing Infrastructure**: Validates complete authentication flows

## Service Testing Patterns

### Multi-Service Integration Testing
Complex scenarios requiring multiple services:
- **Authentication + Database**: OAuth flow with persistent session storage
- **Hub + Data**: Model download with local alias creation and metadata storage
- **Auth + Secret**: Credential encryption and secure storage coordination
- **Cache + Database**: Data consistency across cache and persistent storage

### Realistic Service Composition
Testing with mixed real and mock services:
- **Offline testing**: Mock external services (Hub, OAuth) with real local services
- **Integration testing**: Real database with mock external dependencies
- **Performance testing**: Real caching with synthetic data loads
- **Error scenario testing**: Coordinated error injection across service boundaries

### Cross-Service Transaction Testing
Database transaction coordination across services:
- **Download request tracking**: Multi-service participation in download lifecycle
- **Authentication state synchronization**: Token validation coordinated with session management
- **Alias management**: File system operations coordinated with database metadata
- **Error recovery**: Service failure and recovery coordination testing

## Important Constraints

### Service Mock Requirements
- All service traits must have `#[mockall::automock]` for comprehensive mocking
- Mock services must maintain behavioral compatibility with real implementations
- Service composition testing requires careful mock expectation coordination
- Cross-service error propagation must be validated in mock scenarios

### Database Testing Requirements
- Each test must use isolated temporary SQLite database
- Migration testing requires validation with realistic data sets
- Transaction testing must validate rollback scenarios and error handling
- Time service abstraction required for deterministic testing

### Authentication Testing Requirements  
- OAuth2 flows must be tested with realistic token formats and expiration
- JWT validation must include signature verification and claims validation
- Session testing must validate secure cookie configuration and isolation
- Token exchange testing must validate scope and audience restrictions

### External Service Mocking Requirements
- Hugging Face Hub mocking must simulate gated repositories and network errors
- OAuth provider mocking must handle various error conditions and edge cases
- Network error simulation must include timeout and retry scenarios
- Rate limiting simulation must validate backoff and recovery strategies

### Service Composition Constraints
- Service registration must happen in dependency order during test setup
- Arc<dyn Trait> pattern requires careful lifetime management in tests
- Service interdependencies must be validated with integration tests
- Mock service expectations must be configured before service composition