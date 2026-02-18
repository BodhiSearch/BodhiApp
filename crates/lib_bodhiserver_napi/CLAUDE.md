# CLAUDE.md

This file provides guidance to Claude Code when working with the `lib_bodhiserver_napi` crate.

*For detailed implementation examples and technical depth, see [crates/lib_bodhiserver_napi/PACKAGE.md](crates/lib_bodhiserver_napi/PACKAGE.md)*

## Purpose

The `lib_bodhiserver_napi` crate serves as BodhiApp's **Node.js binding orchestration layer**, implementing sophisticated NAPI-based bindings for embedding BodhiApp server functionality into Node.js applications with comprehensive configuration management and cross-platform support.

## Key Domain Architecture

### NAPI Binding System
Advanced Node.js integration with sophisticated configuration management:
- **BodhiServer Class**: Main NAPI wrapper providing complete server lifecycle management with async/await support
- **NapiAppOptions Configuration**: Flexible configuration system supporting environment variables, app settings, and system settings
- **Cross-Platform Bindings**: Native module support for Windows, macOS (Intel/ARM), and Linux with automated build pipeline
- **TypeScript Integration**: Complete type definitions with auto-generated interfaces and comprehensive type safety
- **Memory Management**: Proper resource cleanup with Drop trait implementation and automatic garbage collection coordination

### Configuration Management Architecture
Sophisticated configuration system bridging Node.js and Rust environments:
- **Multi-Layer Configuration**: Environment variables, app settings, and system settings with proper precedence handling
- **OAuth2 Integration**: Client credentials management with secure configuration and authentication flow support
- **Application Status Management**: Dynamic app status coordination with validation and state management
- **Settings Validation**: Comprehensive configuration validation with error handling and recovery guidance
- **Environment Coordination**: Development/production mode support with environment-specific defaults and resource management

### Server Lifecycle Management System
Advanced server orchestration with comprehensive lifecycle coordination:
- **Async Server Operations**: Promise-based server start/stop operations with proper error handling and state management
- **Health Monitoring**: Server ping functionality with HTTP health checks and connection validation
- **Resource Cleanup**: Automatic cleanup of temporary directories, log guards, and server handles with proper Drop implementation
- **State Synchronization**: Thread-safe server state management with Mutex coordination and concurrent access support
- **Logging Integration**: Comprehensive logging setup with file and stdout output, configurable log levels, and proper cleanup

## Architecture Position

The `lib_bodhiserver_napi` crate is a **leaf crate** providing Node.js bindings for the BodhiApp server.

**Upstream dependencies** (crates this depends on):
- [`lib_bodhiserver`](../lib_bodhiserver/CLAUDE.md) -- `build_app_service()`, `setup_app_dirs()`, `ServeCommand`, embedded UI assets

**Downstream consumers**: None (this is a leaf crate consumed by Node.js applications)

**Associated test infrastructure**: See [`tests-js/CLAUDE.md`](tests-js/CLAUDE.md) for the Playwright E2E test suite that validates BodhiApp via these NAPI bindings.

**npm package**: `@bodhiapp/app-bindings` -- published NAPI bindings package (requires Node.js >= 22)

## Cross-Crate Integration Patterns

### Embeddable Server Integration
Complex integration with lib_bodhiserver for complete Node.js embedding:
- **AppServiceBuilder Coordination**: NAPI configuration translated to AppOptions for service composition and dependency injection
- **Directory Management Integration**: Automatic BODHI_HOME, HF_HOME, and resource directory setup coordinated with lib_bodhiserver
- **Service Registry Access**: Complete AppService registry functionality exposed through NAPI bindings with proper error translation
- **Configuration Management**: Multi-layer configuration system coordinated with lib_bodhiserver settings and environment management
- **Server Lifecycle Coordination**: ServeCommand integration for HTTP server management with embedded UI assets and graceful shutdown

### Node.js Runtime Integration
Advanced NAPI integration with Node.js runtime and ecosystem:
- **Async Runtime Coordination**: Tokio async runtime integration with Node.js event loop and Promise-based API
- **Memory Management**: Proper resource cleanup coordination between Rust Drop trait and JavaScript garbage collection
- **Error Translation**: Rust error types converted to JavaScript Error objects with proper stack traces and error messages
- **Type Safety**: Auto-generated TypeScript definitions with comprehensive interface coverage and type validation
- **Cross-Platform Support**: Native module compilation for multiple platforms with automated build pipeline and distribution

### Configuration System Integration
Sophisticated configuration management bridging Node.js and Rust environments:
- **Environment Variable Coordination**: Node.js process.env integration with Rust environment variable management
- **Settings File Integration**: YAML configuration file support with Node.js file system access and Rust settings management
- **OAuth2 Configuration**: Client credentials management with secure storage and authentication flow coordination
- **Application Status Management**: Dynamic status coordination with validation and state synchronization across language boundaries

## NAPI Binding Orchestration Workflows

### Multi-Layer Configuration Coordination
Complex configuration management across Node.js and Rust boundaries:

1. **Configuration Creation**: NapiAppOptions creation with empty configuration structure for flexible setup
2. **Environment Variable Integration**: Node.js environment variables mapped to Rust configuration with proper type conversion
3. **Settings Management**: App settings and system settings coordination with validation and precedence handling
4. **OAuth2 Configuration**: Client credentials management with secure storage and authentication flow setup
5. **Configuration Validation**: Comprehensive validation with error translation and recovery guidance for Node.js applications

### Server Lifecycle Orchestration
Sophisticated server management with async coordination:

**Server Startup Workflow**:
1. **Configuration Translation**: NapiAppOptions converted to AppOptions for lib_bodhiserver integration
2. **Directory Setup**: Automatic BODHI_HOME, HF_HOME, and resource directory creation with proper permissions
3. **Service Composition**: AppServiceBuilder coordination for complete service registry initialization
4. **Logging Setup**: Comprehensive logging configuration with file and stdout output coordination
5. **HTTP Server Launch**: ServeCommand execution with embedded UI assets and graceful startup coordination

**Server Shutdown Workflow**:
1. **Graceful Shutdown**: ServerShutdownHandle coordination for proper resource cleanup
2. **Resource Cleanup**: Temporary directory cleanup, log guard disposal, and memory management
3. **State Synchronization**: Thread-safe shutdown coordination with Mutex-based state management
4. **Error Handling**: Comprehensive error handling with proper JavaScript error translation

### Cross-Platform Integration Orchestration
Advanced NAPI integration across multiple platforms:
1. **Native Module Compilation**: Cross-platform build pipeline for Windows, macOS (Intel/ARM), and Linux
2. **Runtime Integration**: Tokio async runtime coordination with Node.js event loop and Promise-based API
3. **Memory Management**: Proper resource lifecycle management between Rust and JavaScript garbage collection
4. **Type Safety**: Auto-generated TypeScript definitions with comprehensive interface coverage and validation

## Important Constraints

### NAPI Binding Requirements
- All NAPI operations must use proper async/await patterns with Promise-based API for Node.js integration
- Configuration management must support multi-layer configuration with environment variables, app settings, and system settings
- Server lifecycle operations must coordinate with lib_bodhiserver for complete service composition and resource management
- Memory management must properly coordinate between Rust Drop trait and JavaScript garbage collection for resource cleanup

### Cross-Platform Integration Standards
- Native module compilation must support Windows, macOS (Intel/ARM), and Linux with automated build pipeline
- TypeScript definitions must be auto-generated with comprehensive interface coverage and type validation
- Error handling must translate Rust error types to JavaScript Error objects with proper stack traces and messages
- Async runtime must coordinate Tokio with Node.js event loop for proper Promise-based API integration

### Configuration Management Rules
- NapiAppOptions must validate all configuration with clear error messages and recovery guidance
- Environment variable integration must coordinate with Node.js process.env and Rust environment management
- OAuth2 configuration must support secure client credentials management with authentication flow coordination
- Settings validation must occur at configuration creation with comprehensive error handling and type checking

### Server Integration Coordination Requirements
- Server lifecycle must coordinate with lib_bodhiserver for complete application bootstrap and service composition
- Directory management must support automatic BODHI_HOME, HF_HOME, and resource directory creation with proper permissions
- Logging integration must support both file and stdout output with configurable log levels and proper cleanup
- HTTP server integration must coordinate with ServeCommand for embedded UI assets and graceful shutdown handling

## NAPI Extension Patterns

### Adding New NAPI Bindings
When creating new Node.js bindings for BodhiApp functionality:

1. **NAPI Function Design**: Use proper NAPI annotations with async support and error handling for JavaScript integration
2. **Configuration Integration**: Extend NapiAppOptions with new configuration options and validation rules for Node.js applications
3. **Type Safety**: Update TypeScript definitions with new interfaces and comprehensive type coverage for development support
4. **Error Handling**: Create NAPI-specific errors that translate to JavaScript Error objects with proper context and stack traces
5. **Testing Infrastructure**: Use comprehensive JavaScript and Rust testing for isolated NAPI binding validation

### Extending Configuration Management
For new configuration capabilities and Node.js integration patterns:

1. **Multi-Layer Configuration**: Extend configuration system with new environment variables, app settings, and system settings
2. **Validation Integration**: Add comprehensive validation with error translation and recovery guidance for Node.js applications
3. **Cross-Platform Support**: Ensure new configuration works across Windows, macOS, and Linux with proper platform-specific handling
4. **TypeScript Integration**: Update type definitions with new configuration options and comprehensive interface coverage
5. **Configuration Testing**: Test configuration management with different Node.js scenarios and validation failures

### Cross-Application Integration Patterns
For new Node.js embedding scenarios and application integration:

1. **Runtime Coordination**: Design async runtime coordination that integrates Tokio with Node.js event loop efficiently
2. **Memory Management**: Implement proper resource lifecycle management between Rust and JavaScript garbage collection
3. **Error Boundaries**: Provide comprehensive error handling with proper isolation and recovery for Node.js applications
4. **Performance Optimization**: Optimize NAPI bindings and resource management for Node.js scenarios with minimal overhead
5. **Integration Testing**: Support comprehensive Node.js integration testing with realistic application scenarios and use cases

