# CLAUDE.md

This file provides guidance to Claude Code when working with the `objs` crate.

*For detailed implementation examples and technical depth, see [crates/objs/PACKAGE.md](crates/objs/PACKAGE.md)*

## Purpose

The `objs` crate serves as BodhiApp's **universal foundation layer**, providing domain objects, centralized error handling with localization, and shared types that enable consistent behavior across the entire application ecosystem including services, routes, CLI, and desktop components. This crate defines the canonical business entities and cross-cutting concerns that maintain architectural consistency across all deployment contexts.

## Key Domain Architecture

### Centralized Error System
BodhiApp's error system provides application-wide consistency:
- **ErrorType enum**: Universal HTTP status code mapping used by all routes and services
- **AppError trait**: Standardized error metadata interface implemented across all crates
- **ApiError envelope**: Converts service layer errors to OpenAI-compatible JSON responses
- **Localized messaging**: Multi-language error support consumed by web UI, CLI, and desktop clients
- **Cross-crate error propagation**: Seamless error flow from services through routes to clients

### GGUF Model File System
Specialized binary format handling for local AI model management:
- **Magic number validation**: Supports GGUF v2-v3 with endian autodetection
- **Metadata extraction**: Key-value parsing for chat templates, tokenization parameters, and model configuration
- **Memory-mapped access**: Safe bounds checking prevents crashes on corrupted files
- **Service integration**: Used by HubService for model validation and DataService for local file management

### Model Ecosystem Architecture
Comprehensive model management spanning Hub integration to local storage:
- **Repo**: Canonical "user/name" format enforced across HubService and DataService interactions
- **HubFile**: Represents cached models with validation against actual Hugging Face cache structure
- **Alias System**: Unified model aliasing supporting user-defined, auto-discovered, and remote API models
  - **UserAlias**: YAML-configured local model aliases with parameter overlays
  - **ModelAlias**: Auto-discovered local models from filesystem scanning
  - **ApiAlias**: Remote API model endpoints with prefix-based routing
- **RemoteFile**: Downloadable model specifications coordinated between services and routes

### OAuth2-Based Access Control
Role and scope system integrated with authentication services:
- **Role hierarchy**: Admin > Manager > PowerUser > User ordering used throughout route authorization
- **TokenScope/UserScope**: OAuth2-style scope parsing coordinated with AuthService and middleware
- **ResourceScope**: Union type enabling flexible authorization across different service contexts
- **Cross-service coordination**: Access control decisions flow from AuthService through middleware to routes

### OpenAI API Compatibility Framework
Complete parameter system for OpenAI API emulation:
- **OAIRequestParams**: Validation ranges enforced consistently across OAI routes and service calls
- **GptContextParams**: llama.cpp configuration coordinated between CLI, services, and llama_server_proc
- **Non-destructive parameter overlay**: Precedence system (request > alias > defaults) used throughout request processing
- **Service coordination**: Parameters flow from routes through services to actual model execution
- **API Format Abstraction**: Extensible ApiFormat enum enabling multiple API protocol support beyond OpenAI

### Configuration and Environment Management System
Comprehensive configuration architecture for multi-environment deployments:
- **EnvType**: Production/Development environment discrimination affecting security and logging behavior
- **AppType**: Native/Container deployment context influencing resource management strategies
- **LogLevel**: Unified logging configuration across all crates with tracing integration
- **Setting System**: Hierarchical configuration with source tracking (System > CommandLine > Environment > User)
- **SettingMetadata**: Configuration mutation tracking and validation requirements

### API Organization and Documentation System
Structured API surface management for OpenAPI generation:
- **API Tags**: Centralized tag constants ensuring consistent OpenAPI documentation grouping
- **Route Organization**: Tags coordinate endpoint categorization across routes_oai and routes_app
- **Documentation Generation**: Tags flow through utoipa for automated API documentation
- **Client SDK Generation**: Consistent tagging enables reliable TypeScript client generation

## Architecture Position

The `objs` crate serves as BodhiApp's **architectural keystone**:
- **Universal Foundation**: All other crates depend on objs for core types and error handling
- **Cross-Crate Consistency**: Ensures unified behavior across services, routes, CLI, and desktop components
- **Integration Coordinator**: Bridges external APIs (OpenAI, Hugging Face, OAuth2) with internal representations
- **Localization Hub**: Provides centralized multi-language support consumed by all user-facing components
- **Domain Authority**: Defines canonical business entities used throughout the application ecosystem

## Cross-Crate Integration Patterns

### Service Layer Integration
The objs crate enables sophisticated service coordination through comprehensive domain object usage:
- **Error Propagation**: Service errors implement AppError via errmeta_derive for consistent HTTP response generation with localized messages
- **Domain Validation**: Services use objs validation for request parameters, business rules, and cross-service data consistency
- **Model Coordination**: HubService and DataService coordinate via shared Repo, HubFile, and Alias types with atomic file operations
- **Authentication Integration**: AuthService uses Role and Scope types for authorization decisions with hierarchical access control
- **Database Integration**: DbService uses objs error types for transaction management and migration support
- **Secret Management**: SecretService integrates with objs error system for encryption/decryption error handling
- **Session Coordination**: SessionService uses objs types for HTTP session management with secure cookie configuration

### Route Layer Integration  
Routes depend on objs for request/response handling:
- **Parameter Validation**: OAIRequestParams used across OpenAI-compatible endpoints with alias.request_params.update() pattern
- **Error Response Generation**: ApiError converts service errors to OpenAI-compatible JSON via RouterStateError translation
- **Authentication Middleware**: Role and Scope types enable fine-grained access control through auth_middleware with hierarchical authorization
- **Authorization Headers**: ResourceScope union type supports both TokenScope and UserScope authorization contexts in HTTP middleware
- **Localized Error Messages**: Multi-language error support for web UI and API clients through HTTP error responses with auth_middleware integration
- **OpenAI API Compatibility**: Complete parameter system enables OpenAI API emulation through routes_oai with non-destructive parameter overlay
- **Application API Integration**: routes_app uses domain objects for model management, authentication, and configuration with comprehensive validation
- **API Error Translation**: Service errors converted to OpenAI-compatible error responses with proper error types and HTTP status codes

### HTTP Infrastructure Integration
Domain objects flow through HTTP infrastructure with server_core coordination:
- **Alias Resolution**: DataService.find_alias() provides Alias objects for HTTP chat completion requests
- **Model File Discovery**: HubService.find_local_file() returns HubFile objects for SharedContext LLM server coordination
- **Error Translation**: All domain errors implement AppError trait for consistent HTTP status code mapping via RouterStateError
- **Parameter Application**: Alias.request_params.update() applies domain parameters to HTTP requests in SharedContext

### Cross-Component Data Flow
Domain objects flow throughout the application with comprehensive service integration:

- **CLI → Services**: Command parameters validated and converted via objs types with comprehensive error handling and CLI-specific error translation
- **Services → Routes**: Business logic results converted to API responses via objs with localized error messages
- **Routes → Frontend**: Consistent error format and localized messages via ApiError with OpenAI compatibility
- **Application API → Services**: routes_app coordinates model management, authentication, and configuration through objs domain objects
- **Desktop ↔ Services**: Shared domain objects ensure consistency between web and desktop clients
- **Service ↔ Service**: Cross-service coordination via shared domain objects (Repo, HubFile, Alias, Role, Scope)
- **Database ↔ Services**: Domain objects provide consistent data validation and error handling across persistence boundaries
- **Authentication Flow**: OAuth2 types flow from AuthService through SecretService to SessionService with comprehensive error propagation
- **Model Management**: Model domain objects coordinate between HubService, DataService, and CacheService with validation and error recovery

### Deployment Context Integration
Domain objects maintain consistency across multiple deployment contexts:

- **Dual-Mode Application Support**: Domain objects provide consistent validation and behavior across desktop and server deployment modes without architectural changes
- **Embedded Application Integration**: Domain objects designed for safe usage across embedded application boundaries including desktop applications and library integrations
- **Context-Agnostic Design**: All domain objects implement deployment-neutral interfaces enabling flexible application composition and embedding scenarios
- **Cross-Deployment Consistency**: Error handling, validation, and serialization behavior remains consistent across different deployment contexts (standalone, embedded, desktop, container)

### Embedded Application Architecture
Domain objects support multiple embedding contexts:
- **Tauri Desktop Integration**: Objects designed for safe IPC serialization between Rust core and JavaScript frontend
- **NAPI Bindings**: Domain types support Node.js integration through lib_bodhiserver_napi
- **Library Embedding**: Objects maintain thread-safety for embedding in other Rust applications
- **WASM Compatibility**: Serialization formats chosen for potential WebAssembly deployment

## Critical System Constraints

### Application-Wide Error Handling
- **Universal Implementation**: All crates must implement AppError for error types to ensure consistent behavior
- **Localization Requirement**: All user-facing errors must have Fluent message templates in en-US and fr-FR
- **Cross-Crate Propagation**: Errors must flow cleanly from services through routes to clients
- **Fallback Safety**: ApiError provides graceful degradation when localization fails

### Model Management Consistency
- **Canonical Format**: Repo "user/name" format enforced across all model-handling components
- **File System Safety**: Alias filename sanitization prevents path traversal and file system issues
- **Cache Validation**: HubFile validation ensures integrity of Hugging Face cache structure
- **Binary Safety**: GGUF parsing bounds checking prevents crashes across service and CLI usage

### Authentication System Integration
- **Role Hierarchy Consistency**: Ordering must be maintained across all authorization contexts with auth_middleware enforcing hierarchical access control
- **Scope Standard Compliance**: OAuth2-style scope parsing ensures compatibility with external identity providers and auth_middleware token exchange
- **Cross-Service Authorization**: TokenScope and ResourceScope enable authorization decisions across service boundaries with middleware precedence rules
- **Security Enforcement**: Case-sensitive parsing and "offline_access" requirements maintain security standards for JWT token validation
- **Middleware Integration**: Role and scope types flow through auth_middleware for HTTP request authorization with consistent domain validation

### API Compatibility Guarantees
- **Parameter Range Enforcement**: OpenAI parameter validation ensures API compatibility across all endpoints
- **Non-Destructive Layering**: Parameter precedence system maintained consistently across request processing
- **CLI Integration**: clap integration ensures command-line interface consistency with API parameters, with CLI-specific builder patterns and error translation
- **Serialization Optimization**: Default value handling maintains performance across JSON serialization boundaries
- **API Evolution Strategy**: ApiFormat enum enables backward-compatible API protocol additions

### Type Safety and Validation Invariants
- **Builder Pattern Consistency**: All complex domain objects provide builders with compile-time validation
- **String Parsing Safety**: FromStr implementations must validate all constraints before construction
- **Serialization Roundtrip**: All domain objects guarantee lossless serialization/deserialization cycles
- **Regex Validation**: Repository and filename validation prevents injection attacks and filesystem issues
- **Timestamp Consistency**: All timestamps use UTC with chrono for cross-timezone reliability

## Test Utilities Architecture

### Overview
The `test_utils` module provides a comprehensive testing infrastructure supporting all downstream crates through the `test-utils` feature flag. This module implements sophisticated mock objects, test fixtures, and domain-specific test builders that enable reliable, maintainable testing across BodhiApp's entire ecosystem.

### Testing Strategy and Patterns

#### Rstest-Based Fixture Architecture
BodhiApp's test utilities leverage `rstest` fixtures for dependency injection and test isolation:
- **Deterministic Environment Setup**: Fixtures provide consistent test environments across all crates
- **Resource Isolation**: Each test receives isolated temporary directories and mock services
- **Cross-Crate Consistency**: Shared fixtures ensure uniform testing patterns across service, route, and CLI tests
- **Performance Optimization**: `#[once]` fixtures minimize expensive setup operations like Python data generation

#### Domain-Specific Mock Objects
Test utilities provide sophisticated mock implementations covering all major domain areas:
- **Model Management Mocks**: Realistic Hugging Face cache structures with valid GGUF files
- **Authentication Fixtures**: Role-based authorization testing with OAuth2 flow simulation
- **Localization Testing**: Isolated FluentLocalizationService instances with mock resource loading
- **HTTP Response Utilities**: Type-safe response parsing for integration test scenarios
- **Temporary Environment Management**: Safe filesystem operations with automatic cleanup

#### Data Generation and Test Isolation
The test utilities implement multi-layered data generation for comprehensive testing:
- **Python Script Integration**: Automated GGUF file generation with controlled metadata structures
- **Snapshot Consistency**: Deterministic test data using fixed snapshot identifiers for reproducible tests
- **Directory Structure Mocking**: Realistic Hugging Face and Bodhi home directory simulation
- **Binary Format Testing**: Endian-specific GGUF files for cross-platform validation
- **Template and Tokenizer Mocking**: Complete chat template and tokenization testing infrastructure

### Cross-Crate Testing Integration

#### Service Layer Testing Support
Test utilities enable sophisticated service testing through domain-specific builders:
- **HubService Testing**: Mock repository structures with realistic file hierarchies and metadata
- **DataService Testing**: Temporary directory fixtures with alias configuration management
- **AuthService Testing**: Role and scope testing with realistic OAuth2 flow simulation
- **Error Handling Testing**: Localized error message validation across all service boundaries
- **Database Testing**: Transaction isolation and migration testing support

#### Route Layer Testing Infrastructure
Test utilities support comprehensive HTTP endpoint testing:
- **Request/Response Parsing**: Type-safe HTTP body parsing for JSON and text responses
- **Authentication Context**: Role-based authorization testing with middleware integration
- **Parameter Validation**: OpenAI parameter testing across all compatibility endpoints
- **Error Response Validation**: Localized error message testing for API responses
- **Integration Test Support**: End-to-end request flow testing with realistic mock data

#### CLI and Desktop Testing Support
Test utilities provide cross-deployment testing capabilities:
- **Environment Variable Mocking**: Consistent environment setup across CLI and desktop contexts
- **Configuration Testing**: Temporary configuration directories with realistic settings
- **Model Management Testing**: Complete model lifecycle testing from discovery to execution
- **Cross-Platform Validation**: Endian-specific binary format testing for different architectures

### Testing Invariants and Constraints

#### Deterministic Test Data Requirements
- **Fixed Snapshots**: All test models use consistent snapshot identifiers to ensure reproducible results
- **Predictable File Sizes**: Mock files have deterministic sizes for consistent validation testing
- **Controlled Metadata**: GGUF test files contain predictable metadata structures for parsing validation
- **Isolated Environments**: Each test receives completely isolated temporary environments

#### Mock Service Behavior Consistency
- **Realistic Error Scenarios**: Mock services simulate realistic failure conditions with proper error types
- **Cross-Crate Compatibility**: Mock objects behave identically across all consuming crate tests
- **Resource Lifecycle Management**: Automatic cleanup prevents test resource leaks and interference
- **Thread Safety**: All mock objects support concurrent access for parallel test execution

#### Python Integration Requirements
- **Script Execution Safety**: Python data generation scripts are isolated and validated before execution
- **Cross-Platform Compatibility**: Data generation works consistently across development environments
- **Dependency Management**: Python scripts handle missing dependencies gracefully with clear error messages
- **Output Validation**: Generated test data is validated for correctness before use in Rust tests

### Extension Guidelines for Test Utilities

#### Adding New Test Fixtures
When creating new test utilities for domain objects:
1. **Follow Rstest Patterns**: Use `#[fixture]` annotations with proper dependency injection
2. **Ensure Isolation**: Each fixture should provide completely isolated test environments
3. **Add Builder Methods**: Provide fluent builders for complex test object construction
4. **Include Error Cases**: Create fixtures for testing error conditions and edge cases
5. **Document Usage**: Add clear examples showing fixture usage patterns

#### Extending Mock Objects
For new domain objects requiring test support:
1. **Implement Realistic Behavior**: Mock objects should behave identically to real implementations
2. **Support Error Injection**: Enable testing of error conditions through configurable mock behavior
3. **Maintain State Consistency**: Mock state changes should reflect real object behavior patterns
4. **Enable Cross-Crate Usage**: Design mocks for consumption across multiple crate test suites
5. **Provide Clear APIs**: Mock configuration should be intuitive and well-documented

#### Data Generation Best Practices
When adding new test data generation:
1. **Use Deterministic Generation**: All generated data should be reproducible across test runs
2. **Validate Output**: Generated data should be validated for correctness before use
3. **Handle Dependencies**: Generation scripts should gracefully handle missing dependencies
4. **Support Multiple Formats**: Consider both little-endian and big-endian variants for binary data
5. **Document Generation Process**: Clear documentation of how to regenerate test data

### Critical Testing System Constraints

#### Feature Flag Management
- **Conditional Compilation**: Test utilities are only available with the `test-utils` feature flag
- **Development Dependencies**: Test-specific dependencies are isolated from production builds
- **Cross-Crate Coordination**: Downstream crates must explicitly enable `test-utils` feature for access
- **Build Performance**: Test utilities don't impact production build times or binary size

#### Resource Management Requirements
- **Automatic Cleanup**: All temporary resources must be automatically cleaned up after test completion
- **Memory Safety**: Mock objects must not introduce memory leaks or unsafe behavior
- **Thread Safety**: All test utilities must support concurrent access for parallel test execution
- **Resource Limits**: Test data generation must respect system resource constraints

#### Cross-Platform Testing Guarantees
- **Endian Independence**: Binary format tests work correctly on both little-endian and big-endian systems
- **Path Handling**: File system operations work consistently across Windows, macOS, and Linux
- **Python Integration**: Data generation scripts work across different Python environments
- **Temporary Directory Management**: Cleanup works correctly across all supported operating systems

### Test Utilities Feature Flag
The `test-utils` feature flag enables comprehensive testing infrastructure:
- **Conditional Availability**: Test utilities are only compiled when the feature is enabled
- **Development Dependencies**: Isolates testing-specific dependencies from production builds
- **Cross-Crate Testing**: Downstream crates enable this feature to access testing infrastructure
- **Performance Impact**: Production builds are unaffected by test utility compilation