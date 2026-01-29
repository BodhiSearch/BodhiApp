# CLAUDE.md - integration-tests

See [PACKAGE.md](./PACKAGE.md) for implementation details and file references.

This file provides guidance to Claude Code when working with the `integration-tests` crate, which provides end-to-end testing infrastructure for BodhiApp.

## Purpose

The `integration-tests` crate provides comprehensive end-to-end testing infrastructure that validates complete application workflows in production-like environments:

- **Live Server Testing**: Real server startup and shutdown testing with actual llama.cpp integration and model loading
- **OAuth2 Authentication Testing**: Complete authentication flows using real OAuth2 tokens and session management
- **API Integration Testing**: Full HTTP API testing with streaming and non-streaming chat completions
- **Cross-Component Integration**: End-to-end validation of service coordination across all application layers
- **Test Data Management**: Structured test data with HuggingFace cache simulation and model file management

## Key Domain Architecture

### Live Server Testing Infrastructure

The crate provides sophisticated live server testing that validates complete application lifecycle:

- **Real Server Startup**: Full server initialization with actual llama.cpp process management and model loading
- **OAuth2 Integration**: Complete authentication flows using real OAuth2 tokens, session creation, and cookie management
- **Resource Management**: Proper cleanup of temporary directories, server processes, and authentication resources
- **Concurrent Testing**: Serial test execution with resource isolation to prevent conflicts

### Test Data Management System

Comprehensive test data infrastructure that simulates production environments:

- **HuggingFace Cache Simulation**: Complete hub cache structure with model blobs, snapshots, and metadata
- **Model File Management**: Small test models (Llama-68M variants) for fast test execution
- **Configuration Management**: Test-specific Bodhi configuration with aliases and model definitions
- **Environment Isolation**: Temporary directory management with proper cleanup

### Authentication Testing Architecture

Production-like authentication testing using real OAuth2 flows:

- **OAuth2 Token Management**: Real token acquisition using client credentials and password flows
- **Session Management**: Database-backed session creation with proper token storage
- **Cookie Handling**: Secure session cookie creation and management for API testing
- **Multi-Client Testing**: Support for creating test clients and resource management

### Cross-Component Integration Validation

End-to-end testing that validates service coordination across all application layers:

- **Service Dependency Testing**: Validation of service injection and coordination patterns
- **Shared Context Testing**: Server factory and shared context lifecycle management
- **API Endpoint Testing**: Complete HTTP request/response cycles with authentication
- **Streaming Protocol Testing**: Server-sent events and streaming response validation

## Dependencies

### Core Application Components (dev-dependencies with test-utils)

- `server_app` - Complete server application with test utilities for integration testing
- `lib_bodhiserver` - Embeddable server library with test fixtures
- `services` - Business logic services with mock implementations and test utilities
- `routes_app` - Application API endpoints with test routing configurations
- `server_core` - HTTP server infrastructure with test context management
- `auth_middleware` - Authentication middleware with test client and session utilities
- `llama_server_proc` - LLM process management with test server lifecycle utilities
- `objs` - Domain objects with test fixtures

### Testing Infrastructure

- `rstest` - Parameterized testing with fixture management for complex test setups
- `tokio` - Async runtime with full feature set for async test execution
- `serial_test` - Sequential test execution to prevent resource conflicts in live server tests
- `reqwest` - HTTP client for API integration testing with streaming support
- `anyhow` - Error handling with anyhow_trace for detailed error context in tests

### Test Data and Environment Management

- `tempfile` - Temporary directory management for isolated test environments
- `fs_extra` - Extended file system operations for test data copying and management
- `sqlx` - Database operations with SQLite support for session and authentication testing
- `tower-sessions` - Session management testing with database-backed session stores
- `jsonwebtoken` - JWT token validation and testing utilities
- `cookie` - HTTP cookie creation and management for authentication testing
- `maplit` - HashMap literal macros for test data creation and session management

## Architecture Position

The `integration-tests` crate operates at the highest testing layer, providing end-to-end validation of the complete BodhiApp system:

- **System Integration Validation**: Tests the complete application stack from HTTP requests through service coordination to llama.cpp process management
- **Production Environment Simulation**: Uses real OAuth2 authentication, actual model files, and complete server lifecycle management
- **Cross-Crate Coordination Testing**: Validates service dependencies and integration patterns across all application crates
- **Quality Gate Enforcement**: Ensures that changes to any component don't break end-to-end functionality and user workflows

## Important Constraints

### Test Environment Requirements

- **OAuth2 Server Access**: Tests require access to a live OAuth2 server with configured test clients and users
- **Model File Dependencies**: Tests depend on specific small model files (Llama-68M variants) for fast execution
- **Serial Execution**: Live server tests must run serially due to resource conflicts and port binding
- **Environment Configuration**: Tests require specific environment variables for OAuth2 configuration and test credentials

### Resource Management Constraints

- **Temporary Directory Cleanup**: All tests must properly clean up temporary directories and test data
- **Server Process Lifecycle**: Live server tests must ensure proper server shutdown to prevent resource leaks
- **Authentication Session Management**: Tests must properly manage OAuth2 tokens and session cleanup
- **Port Allocation**: Live server tests use random port allocation to prevent conflicts

### Test Data Management Constraints

- **HuggingFace Cache Structure**: Test data must maintain proper HuggingFace hub cache structure with blobs and snapshots
- **Model File Integrity**: Test model files must be valid GGUF format files for successful llama.cpp integration
- **Configuration Consistency**: Test configuration files must match production structure for accurate testing

## Cross-Crate Integration Patterns

### Live Server Testing with OAuth2 Authentication

The integration tests demonstrate complete authentication flows using real OAuth2 tokens and session management:

- **OAuth2 Token Acquisition**: Tests acquire real access and refresh tokens using client credentials flow
- **Session Creation**: Tests create database-backed sessions with proper token storage using tower-sessions
- **Cookie-Based Authentication**: Tests use secure HTTP cookies for session-based authentication rather than Bearer tokens
- **Multi-Service Coordination**: Tests validate coordination between auth_middleware, services, and server_core components

### Test Data Management and Environment Isolation

The crate implements sophisticated test data management that simulates production environments:

- **HuggingFace Cache Simulation**: Complete hub cache structure with model blobs, snapshots, and metadata files
- **Temporary Environment Setup**: Each test creates isolated temporary directories with proper Bodhi configuration
- **Model File Management**: Small test models (Llama-68M variants) enable fast test execution while maintaining realism
- **Configuration Copying**: Test configuration files are copied from version-controlled test data to temporary environments

### Service Integration and Dependency Injection

Integration tests validate complex service coordination patterns across the application:

- **Mock Service Integration**: Tests use OfflineHubService and MockSettingService for controlled testing environments
- **Service Builder Patterns**: Tests demonstrate proper service construction using AppServiceBuilder with custom implementations
- **Shared Context Management**: Tests validate DefaultSharedContext lifecycle with server factory coordination
- **Cross-Service Communication**: Tests ensure proper data flow between services, middleware, and route handlers

### Live Server Testing Infrastructure Implementation

The crate provides comprehensive live server testing utilities that manage complete server lifecycle:

**TestServerHandle Structure** (see `tests/utils/live_server_utils.rs`):

- **Temporary Environment Management**: Each test gets isolated temporary directories with proper cleanup
- **Random Port Allocation**: Tests use random ports to prevent conflicts during parallel test execution
- **Server Lifecycle Management**: Proper server startup, operation, and shutdown with resource cleanup
- **Service Integration**: Complete service dependency injection with test-specific configurations

**Authentication Testing Utilities**:

- **OAuth2 Token Management**: Real token acquisition using environment-configured OAuth2 servers
- **Session Management**: Database-backed session creation with proper token storage and expiration
- **Cookie Handling**: Secure session cookie creation for HTTP-based authentication testing
- **Multi-Client Support**: Support for creating test OAuth2 clients and resource management

**Test Data Management Patterns**:

- **HuggingFace Cache Structure**: Complete simulation of HuggingFace hub cache with proper blob and snapshot organization
- **Model File Management**: Small but realistic GGUF model files for fast test execution
- **Configuration Management**: Test-specific Bodhi configuration with aliases and model definitions
- **Environment Variable Handling**: Proper loading of test environment configuration from `.env.test` files

### Test Execution Patterns and Resource Management

**Serial Test Execution** (using `#[serial_test::serial(live)]`):

- All live server tests run serially to prevent resource conflicts
- Tests use the `serial(live)` attribute to ensure proper resource isolation
- Each test gets exclusive access to server ports and authentication resources

**Timeout Management** (using `#[timeout(Duration::from_secs(5 * 60))]`):

- Tests have 5-minute timeouts to prevent hanging in CI environments
- Timeout handling ensures proper resource cleanup even on test failures

**Fixture-Based Test Setup** (using `rstest` fixtures):

- `llama2_7b_setup` fixture provides complete application service setup with OAuth2 configuration
- `live_server` fixture builds on the setup to provide running server instances
- Fixtures handle complex dependency injection and service coordination

**Error Handling and Cleanup**:

- Tests use `anyhow::Result` for comprehensive error handling
- Proper cleanup of temporary directories, server processes, and authentication sessions
- Resource cleanup occurs even when tests fail or timeout

## Test Infrastructure Implementation

### TestServerHandle Structure

The actual test infrastructure uses `TestServerHandle` (see `tests/utils/live_server_utils.rs:139-145`):

```rust
pub struct TestServerHandle {
  pub temp_cache_dir: TempDir,
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
  pub app_service: Arc<dyn AppService>,
}
```

**Key Infrastructure Components**:

- **Temporary Directory Management**: Each test gets isolated temporary cache directories with proper cleanup
- **Random Port Allocation**: Tests use random ports (2000-60000 range) to prevent conflicts
- **Server Lifecycle Management**: Uses `ServerShutdownHandle` from server_app for proper shutdown coordination
- **Service Access**: Provides access to the complete `AppService` for authentication and testing utilities

### Authentication Testing Infrastructure

**OAuth2 Token Management** (see `get_oauth_tokens` function):

- Real OAuth2 token acquisition using client credentials and password flows
- Environment-based configuration for test OAuth2 servers and credentials
- Support for access token and refresh token management

**Session Management** (see `create_authenticated_session` function):

- Database-backed session creation using tower-sessions
- Proper token storage in session data with expiration management
- Session cookie creation for HTTP-based authentication testing

### Test Data Structure and Management

**HuggingFace Cache Simulation**:

```
tests/data/live/huggingface/hub/
├── models--afrideva--Llama-68M-Chat-v1-GGUF/
│   ├── blobs/cdd6bad08258f53c637c233309c3b41ccd91907359364aaa02e18df54c34b836
│   ├── refs/main
│   └── snapshots/4bcbc666d2f0d2b04d06f046d6baccdab79eac61/
│       └── llama-68m-chat-v1.q8_0.gguf
└── models--TheBloke--TinyLlama-1.1B-Chat-v1.0-GGUF/
    └── blobs/da3087fb14aede55fde6eb81a0e55e886810e43509ec82ecdc7aa5d62a03b556
```

**Bodhi Configuration Structure**:

```
tests/data/live/bodhi/
├── aliases/qwen3--1.7b-instruct.yaml
├── logs/
└── models.yaml
```

### Environment Configuration and Test Setup

**Environment Variable Management** (see `tests/resources/.env.test`):

- OAuth2 server configuration with realm and client credentials
- Test user credentials for authentication flows
- Environment isolation using temporary directories and custom paths

**Test Fixture Patterns** (see `tests/utils/live_server_utils.rs`):

- `llama2_7b_setup` fixture provides complete application service setup
- `live_server` fixture builds running server instances with authentication
- Fixtures handle complex dependency injection and service coordination

**Test Data Copying and Management** (see `copy_test_dir` function):

- Efficient copying of test data to temporary directories using `fs_extra`
- Proper HuggingFace cache structure maintenance
- Configuration file management for test-specific settings

## Testing Categories and Implementation

### Live Server Integration Tests

**API Ping Testing** (see `test_live_api_ping.rs`):

- Basic server connectivity and health check validation
- Server startup and shutdown lifecycle testing
- HTTP status code validation for basic endpoints

**Chat Completion Testing** (see `test_live_chat_completions_*.rs`):

- Non-streaming chat completion API validation with content verification
- Streaming chat completion testing with server-sent events parsing
- OAuth2 authentication integration with session-based authentication
- Model-specific testing using qwen3:1.7b-instruct alias

### Library Integration Tests

**LLM Server Process Testing** (see `test_live_lib.rs`):

- Direct llama.cpp server startup and shutdown testing
- Model loading with various file formats (symlinks, direct files, blobs)
- Shared context testing with server factory coordination
- Resource cleanup and proper shutdown validation

### Authentication and Session Management Tests

**OAuth2 Integration Testing**:

- Real OAuth2 token acquisition using client credentials flow
- Session creation and management with database-backed storage
- Cookie-based authentication for HTTP API testing
- Multi-client resource management and cleanup

### Cross-Component Integration Validation

**Service Coordination Testing**:

- Service dependency injection with mock and real implementations
- Cross-service communication validation
- Shared context lifecycle management
- Error propagation across service boundaries

## Development Guidelines

### Adding New Integration Tests

1. **Use Serial Execution**: Add `#[serial_test::serial(live)]` for tests that use live servers or shared resources
2. **Implement Proper Timeouts**: Use `#[timeout(Duration::from_secs(5 * 60))]` for tests that may hang
3. **Leverage Existing Fixtures**: Use `live_server` fixture for full server testing with authentication
4. **Follow Authentication Patterns**: Use `get_oauth_tokens` and `create_authenticated_session` for authenticated API testing
5. **Ensure Resource Cleanup**: Tests must properly clean up temporary directories, server processes, and authentication sessions

### Test Data Management Best Practices

- **Use Small Model Files**: Tests use Llama-68M variants for fast execution while maintaining realism
- **Maintain HuggingFace Structure**: Test data must follow proper hub cache structure with blobs and snapshots
- **Version Control Test Data**: All test data is maintained in version control for consistency
- **Environment Isolation**: Each test gets isolated temporary directories with proper cleanup

### Error Handling and Debugging

- **Use anyhow_trace**: Add `#[anyhow_trace::anyhow_trace]` for detailed error context in test failures
- **Descriptive Assertions**: Use `pretty_assertions::assert_eq` for better test failure output
- **Proper Error Propagation**: Tests use `anyhow::Result` for comprehensive error handling
- **Resource Cleanup on Failure**: Ensure cleanup occurs even when tests fail or timeout

### Performance and Resource Management

- **Serial Test Execution**: Live server tests run serially to prevent resource conflicts
- **Random Port Allocation**: Tests use random ports to prevent conflicts during test execution
- **Efficient Test Data**: Small model files and efficient copying reduce test execution time
- **Proper Resource Isolation**: Each test gets isolated environments to prevent interference

## Testing Strategy and Execution

### Test Execution Commands

```bash
# Run all integration tests (note: requires OAuth2 server access and environment configuration)
cargo test --package integration-tests

# Run specific test files
cargo test --package integration-tests test_live_api_ping
cargo test --package integration-tests test_live_chat_completions_non_streamed

# Run tests with output (useful for debugging authentication issues)
cargo test --package integration-tests -- --nocapture

# Tests automatically run serially due to #[serial_test::serial(live)] attributes
```

### Environment Setup Requirements

- **OAuth2 Server Configuration**: Tests require access to a live OAuth2 server with proper client and user setup
- **Environment Variables**: Must configure `.env.test` file with OAuth2 server details and test credentials
- **Model Files**: Tests depend on specific model files in the test data structure
- **Temporary Directory Access**: Tests need write access for temporary directory creation and cleanup

### Test Environment Characteristics

- **Isolated Temporary Environments**: Each test creates isolated temporary directories with proper cleanup
- **Real Authentication**: Tests use actual OAuth2 flows rather than mocked authentication
- **Live Server Testing**: Tests start real server instances with actual llama.cpp integration
- **Database-Backed Sessions**: Tests use real session storage with proper cleanup

## Current Test Coverage

### API Integration Coverage

- **Basic Connectivity**: API ping endpoint testing with server lifecycle validation
- **Chat Completions**: Both streaming and non-streaming chat completion testing with content validation
- **Authentication Integration**: Complete OAuth2 flow testing with session management
- **Error Handling**: Proper error response validation and resource cleanup

### Library Integration Coverage

- **LLM Server Process**: Direct llama.cpp server testing with various model file formats
- **Shared Context Management**: Server factory and shared context lifecycle testing
- **Service Coordination**: Cross-service integration with mock and real service implementations
- **Resource Management**: Proper cleanup and shutdown testing across all components

### Authentication and Security Coverage

- **OAuth2 Token Management**: Real token acquisition and validation testing
- **Session Management**: Database-backed session creation and cleanup testing
- **Cookie-Based Authentication**: Secure session cookie handling for HTTP API testing
- **Multi-Client Support**: OAuth2 client creation and resource management testing
