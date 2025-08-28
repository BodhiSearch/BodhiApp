# PACKAGE.md - routes_all test_utils

This document provides detailed technical information for the `routes_all` test_utils module, focusing on BodhiApp's route composition testing infrastructure and comprehensive integration testing patterns.

## Route Composition Testing Infrastructure

The `routes_all` test_utils module provides testing infrastructure for comprehensive route composition validation, middleware integration testing, and UI serving configuration testing.

### Testing Architecture Overview

Currently, the routes_all crate implements testing patterns directly within the main source files rather than providing separate test_utils infrastructure. This approach enables:

- **Inline Testing**: Route composition tests integrated directly with implementation for immediate validation
- **Environment Testing**: UI serving configuration testing with different environment scenarios
- **Proxy Testing**: HTTP proxy functionality testing with backend server coordination
- **Integration Validation**: Cross-route boundary testing with authentication and authorization validation

### Current Testing Patterns

#### UI Serving Configuration Testing
Comprehensive testing of environment-specific UI serving with different configuration scenarios:

```rust
// Pattern implemented in src/routes.rs:345-395
#[rstest]
#[case::production_with_static(/* production config */)]
#[case::dev_with_proxy(/* development proxy config */)]
#[case::dev_with_static(/* development static config */)]
#[tokio::test]
async fn test_ui_router_scenarios(
  #[case] config: EnvConfig,
  #[case] static_router: Option<Router>,
  #[case] test_paths: Vec<(&str, bool)>,
) {
  // Test UI serving behavior across different environments
}
```

#### Proxy Router Testing
HTTP proxy functionality testing with backend server coordination:

```rust
// Pattern implemented in src/routes_proxy.rs:45-89
#[rstest]
#[awt]
#[tokio::test]
async fn test_proxy_handler(#[future] backend_server: (SocketAddr, Sender<()>)) -> anyhow::Result<()> {
  // Test proxy request forwarding and error handling
}
```

### Testing Infrastructure Capabilities

#### Route Composition Testing
- **Multi-Route Integration**: Testing route composition across routes_oai and routes_app boundaries
- **Middleware Validation**: Authentication and authorization middleware testing with different role and scope combinations
- **State Management**: RouterState testing with comprehensive AppService and SharedContext mocking
- **Error Handling**: Cross-route error propagation and response consistency validation

#### Authentication Flow Testing
- **Bearer Token Testing**: API authentication testing with role and scope validation
- **Session Authentication**: Session-based authentication testing with secure cookie configuration
- **Authorization Hierarchy**: Role hierarchy testing with User/PowerUser/Admin access control
- **Dual Authentication**: Testing precedence rules between bearer token and session authentication

#### UI Serving Testing
- **Environment Configuration**: Testing production vs development UI serving modes
- **Proxy Integration**: Development proxy testing with localhost:3000 integration
- **Static Asset Serving**: Embedded asset serving testing with fallback handling
- **Configuration Validation**: Environment-specific configuration testing with graceful degradation

### Extension Guidelines for Route Testing

#### Adding Route Composition Tests
When creating tests for new route composition patterns:

1. **Integration Testing**: Test complete route composition with all middleware layers and authentication flows
2. **Authentication Testing**: Validate authentication and authorization across different route types with comprehensive role and scope testing
3. **Error Scenario Testing**: Test error propagation and handling across route composition boundaries with consistent error responses
4. **Performance Testing**: Validate route composition performance under realistic load conditions with middleware overhead analysis
5. **Configuration Testing**: Test environment-specific configuration with different deployment scenarios

#### Extending Authentication Testing
For new authentication and authorization testing patterns:

1. **Role Hierarchy Testing**: Test role-based authorization with hierarchical access control enforcement
2. **Scope Validation Testing**: Test scope-based authorization with TokenScope and UserScope validation
3. **Session Management Testing**: Test session-based authentication with secure cookie configuration and lifecycle management
4. **API Token Testing**: Test bearer token authentication with database-backed validation and status tracking
5. **Cross-Route Security**: Test security boundaries across different route types with consistent authorization enforcement

#### UI Serving Testing Extensions
For new UI serving testing capabilities:

1. **Environment Testing**: Test new UI serving modes with different environment configurations
2. **Proxy Testing**: Test new development proxy patterns with different frontend frameworks and build tools
3. **Asset Serving Testing**: Test static asset serving with different caching strategies and performance optimizations
4. **Fallback Testing**: Test graceful degradation scenarios with comprehensive error handling validation
5. **Configuration Testing**: Test environment-specific configuration validation with startup error prevention

### Future Test Infrastructure Development

The routes_all test_utils module could be extended to provide:

#### Route Composition Test Fixtures
- **Router Builder**: Test router composition with configurable middleware and authentication layers
- **Mock State Management**: Comprehensive RouterState mocking with AppService and SharedContext coordination
- **Authentication Fixtures**: Pre-configured authentication scenarios for different role and scope combinations
- **UI Serving Fixtures**: Environment-specific UI serving configuration for testing different deployment scenarios

#### Integration Testing Utilities
- **Cross-Route Testing**: Utilities for testing interactions across route boundaries with consistent error handling
- **Middleware Testing**: Comprehensive middleware testing infrastructure with proper ordering and interaction validation
- **Performance Testing**: Load testing utilities for route composition performance validation under realistic conditions
- **Security Testing**: Authentication and authorization testing utilities with comprehensive security boundary validation

#### Mock Infrastructure
- **Service Mocking**: Comprehensive service mocking for isolated route composition testing
- **Authentication Mocking**: Authentication middleware mocking with different authentication and authorization scenarios
- **UI Configuration Mocking**: Environment configuration mocking for UI serving testing with different deployment modes
- **Error Scenario Mocking**: Error injection utilities for testing error propagation and recovery across route boundaries

## Commands

**Testing**: `cargo test -p routes_all` (includes route composition and UI serving tests)  
**Integration Testing**: `cargo test -p routes_all --features test-utils` (includes comprehensive route testing infrastructure)  
**Performance Testing**: Custom load testing utilities for route composition performance validation