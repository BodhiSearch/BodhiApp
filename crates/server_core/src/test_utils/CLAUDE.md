# CLAUDE.md

This file provides guidance to Claude Code when working with the `test_utils` module for HTTP server infrastructure.

*For implementation details and extension patterns, see [./PACKAGE.md](./PACKAGE.md)*

## Purpose

The `test_utils` module provides specialized testing infrastructure for BodhiApp's HTTP server infrastructure layer, enabling comprehensive testing of server-sent event streaming, LLM server context management, and HTTP route coordination with sophisticated mock orchestration.

**Key Test Utilities** (see `crates/server_core/src/test_utils/`):
- **HTTP Response Extensions**: Advanced response parsing utilities for JSON, text, and SSE streams
- **RouterState Testing**: Pre-configured RouterState fixtures with MockSharedContext integration  
- **ServerFactory Mocking**: LLM server factory stubs for HTTP context testing
- **Mock Server Integration**: rstest fixtures for LLM server process testing coordination

## HTTP Infrastructure Testing Architecture

### RouterState Testing Foundation
Sophisticated HTTP infrastructure testing with service coordination (`crates/server_core/src/test_utils/state.rs`):
- **router_state_stub()**: Pre-configured RouterState with MockSharedContext and AppServiceStub integration
- **ServerFactoryStub**: LLM server factory test implementation with configurable server instances
- **Service Mock Coordination**: HTTP route testing with comprehensive service registry mocking
- **Context Management Testing**: LLM server context testing with lifecycle and state management validation
- **Error Propagation Testing**: HTTP error translation testing with service error coordination
- **Dependency Injection Testing**: RouterState dependency injection validation for HTTP handlers

### SharedContext Mock Orchestration
Advanced LLM server context testing with process coordination:
- **MockSharedContext**: Comprehensive LLM server context mocking for HTTP infrastructure testing
- **Server State Management**: LLM server lifecycle testing with state synchronization validation
- **Request Routing Testing**: Chat completion request routing through context management
- **Observer Pattern Testing**: Server state listener testing with notification coordination
- **Resource Lifecycle Testing**: LLM server startup/shutdown testing with proper cleanup validation

### HTTP Response Testing Extensions
Comprehensive HTTP response parsing utilities (`crates/server_core/src/test_utils/http.rs`):
- **ResponseTestExt**: Trait extension for Axum Response with JSON, text, and SSE parsing
- **RequestTestExt**: Trait extension for HTTP request builders with JSON serialization
- **SSE Stream Parsing**: Specialized parsing for server-sent event streams with data extraction
- **Connection Management Testing**: HTTP connection lifecycle testing with automatic cleanup
- **Stream Error Testing**: Connection interruption and recovery testing with error propagation

## Cross-Crate Testing Integration

### Services Layer HTTP Testing
HTTP infrastructure testing extensively coordinates with services testing:
- **AppServiceStub Integration**: HTTP testing uses services test utilities for realistic service coordination
- **Service Error Testing**: HTTP error translation testing with comprehensive service error scenarios
- **Database Integration Testing**: HTTP context testing with database service coordination
- **Authentication Testing**: HTTP infrastructure testing with authentication service integration
- **Streaming Service Testing**: Real-time streaming testing with service operation coordination

### LlamaServerProc Testing Coordination
HTTP infrastructure testing coordinates with LLM server process testing:
- **ServerFactory Mocking**: LLM server factory testing with HTTP context coordination
- **Server Lifecycle Testing**: HTTP context testing with LLM server process management
- **Request Proxy Testing**: HTTP request routing testing with LLM server communication
- **Process State Testing**: HTTP context testing with LLM server state synchronization
- **Error Recovery Testing**: HTTP error handling testing with LLM server failure scenarios

### Routes Layer Testing Support
HTTP infrastructure testing provides foundation for route testing:
- **RouterState Injection**: HTTP route testing with dependency injection validation
- **SSE Response Testing**: Route streaming response testing with SSE infrastructure
- **Error Response Testing**: HTTP route error handling testing with infrastructure error translation
- **Service Access Testing**: Route service access testing through RouterState infrastructure
- **Integration Testing**: End-to-end HTTP testing across infrastructure and route layers

## Architecture Position

The `test_utils` module serves as:
- **HTTP Infrastructure Testing Foundation**: Provides specialized testing for server-sent events and context management
- **Service Mock Orchestration**: Enables complex HTTP testing with comprehensive service mocking
- **LLM Server Testing Integration**: Coordinates HTTP testing with LLM server process testing
- **Streaming Testing Infrastructure**: Supports comprehensive real-time communication testing

## HTTP Testing Infrastructure Patterns

### SSE Streaming Mock Coordination
Testing patterns for server-sent event streaming:
- **Event Generation Testing**: DirectSSE testing with application event generation and formatting validation
- **Proxy Streaming Testing**: ForwardedSSE testing with LLM server response proxying validation
- **Connection Lifecycle Testing**: HTTP connection management testing with cleanup and error recovery
- **Performance Testing**: Streaming performance validation under realistic network conditions

### Context Management Testing Requirements
HTTP context testing must validate LLM server coordination:
- **Lifecycle Management Testing**: SharedContext testing with proper startup/shutdown coordination
- **State Synchronization Testing**: HTTP context testing with concurrent request handling validation
- **Observer Pattern Testing**: Server state listener testing with notification coordination
- **Resource Management Testing**: HTTP context testing with proper cleanup and lifecycle management

### RouterState Testing Coordination
HTTP infrastructure testing patterns for route coordination:
- **Dependency Injection Testing**: RouterState testing with service registry access validation
- **Service Coordination Testing**: HTTP request processing testing with multi-service operations
- **Error Translation Testing**: HTTP error handling testing with service error conversion
- **Request Processing Testing**: HTTP infrastructure testing with realistic request workflows

## Important Constraints

### HTTP Infrastructure Testing Requirements
HTTP infrastructure testing must validate comprehensive HTTP functionality:
- **RouterState Testing**: All HTTP infrastructure tests must use RouterState dependency injection patterns
- **SSE Streaming Testing**: Streaming tests must validate both DirectSSE and ForwardedSSE functionality
- **Context Management Testing**: SharedContext tests must validate thread-safe operations and lifecycle management
- **Error Handling Testing**: HTTP error tests must validate service error translation and user-friendly messages

### Service Mock Coordination Requirements
HTTP testing requires sophisticated service mocking:
- **AppService Integration**: HTTP tests must coordinate with services test utilities for realistic mocking
- **Service Error Scenarios**: HTTP error testing must validate comprehensive service failure scenarios
- **Authentication Integration**: HTTP context testing must coordinate with authentication service mocking
- **Database Integration**: HTTP infrastructure testing must validate database service coordination

### LLM Server Testing Integration Requirements
HTTP testing must coordinate with LLM server process testing:
- **ServerFactory Mocking**: HTTP context tests must use realistic LLM server factory mocking
- **Process Lifecycle Testing**: HTTP tests must validate LLM server process coordination
- **Request Routing Testing**: HTTP infrastructure tests must validate request proxy functionality
- **State Management Testing**: HTTP context tests must validate LLM server state synchronization

### Streaming Testing Standards
SSE streaming testing must validate real-time communication functionality:
- **Connection Management Testing**: Streaming tests must validate connection lifecycle and cleanup
- **Error Recovery Testing**: SSE tests must validate stream interruption and recovery scenarios
- **Performance Testing**: Streaming tests must validate performance under realistic load conditions
- **Integration Testing**: SSE tests must validate integration with HTTP infrastructure and route layers

## Testing Infrastructure Extensions

### Adding New HTTP Infrastructure Tests
When creating tests for new HTTP infrastructure functionality:
- **RouterState Integration**: New HTTP tests must integrate with RouterState dependency injection patterns
- **Service Mock Coordination**: HTTP tests must coordinate with services test utilities for realistic mocking
- **Context Management Testing**: LLM server context tests must validate lifecycle and state management
- **Streaming Integration**: SSE tests must validate both application events and proxy streaming scenarios
- **Error Scenario Testing**: HTTP error tests must validate comprehensive failure and recovery scenarios

### Cross-Infrastructure Testing Coordination
HTTP infrastructure testing must coordinate across multiple testing systems:
- **Service Testing Integration**: HTTP tests must coordinate with services test utilities for realistic scenarios
- **Process Testing Integration**: HTTP context tests must coordinate with LLM server process testing
- **Route Testing Support**: HTTP infrastructure tests must provide foundation for route testing
- **Integration Test Patterns**: HTTP tests must validate end-to-end functionality across infrastructure layers

### HTTP Testing Performance Requirements
HTTP infrastructure testing must validate performance characteristics:
- **Streaming Performance Testing**: SSE tests must validate performance under realistic network conditions
- **Connection Scalability Testing**: HTTP infrastructure tests must validate concurrent connection handling
- **Memory Usage Testing**: HTTP streaming tests must validate memory management and cleanup
- **Error Recovery Testing**: HTTP infrastructure tests must validate error recovery and resilience patterns