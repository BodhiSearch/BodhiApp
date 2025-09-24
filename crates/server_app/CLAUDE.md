# CLAUDE.md

This file provides guidance to Claude Code when working with the `server_app` crate.

See [PACKAGE.md](./PACKAGE.md) for implementation details

## Purpose

The `server_app` crate serves as BodhiApp's **main HTTP server executable orchestration layer**, implementing sophisticated server lifecycle management, graceful shutdown coordination, and comprehensive service bootstrap with advanced listener patterns and resource management.

## Key Domain Architecture

### HTTP Server Lifecycle Management System
Advanced server orchestration with comprehensive lifecycle coordination:
- **Server Handle Architecture**: Sophisticated server startup/shutdown coordination with ready notification and graceful shutdown channels
- **Graceful Shutdown Integration**: Signal handling (Ctrl+C, SIGTERM) with proper resource cleanup and context shutdown callbacks
- **Service Bootstrap Orchestration**: Complete application service initialization with dependency injection and configuration validation
- **Static Asset Serving**: Dynamic UI serving with environment-specific configuration (development proxy vs production embedded assets)
- **Port Binding Management**: TCP listener coordination with error handling for port conflicts and binding failures

### Advanced Listener Pattern Architecture
Sophisticated event-driven coordination with cross-service integration:
- **ServerKeepAlive Listener**: Intelligent server lifecycle management with configurable keep-alive timers and automatic shutdown coordination
- **VariantChangeListener**: Dynamic execution variant switching with SharedContext coordination for CPU/CUDA/ROCm configurations
- **Settings Change Integration**: Real-time configuration updates with listener pattern coordination across service boundaries
- **State Change Broadcasting**: Observer pattern implementation for server state notifications with async event handling

### Service Orchestration and Bootstrap System
Comprehensive service initialization with sophisticated dependency management:
- **AppService Registry Bootstrap**: Complete service composition with DefaultAppService initialization and dependency injection
- **SharedContext Integration**: LLM server context management with DefaultSharedContext initialization and listener registration
- **Route Composition Coordination**: Integration with routes_all for complete HTTP route and middleware stack composition
- **Resource Validation**: Executable path validation, database connectivity checks, and service health verification

## Architecture Position

The `server_app` crate serves as BodhiApp's **main HTTP server executable orchestration layer**:
- **Above all other crates**: Coordinates complete application bootstrap including services, routes, server_core, and infrastructure
- **Below deployment infrastructure**: Provides production-ready server executable for Docker, systemd, and cloud deployments
- **Integration with llama_server_proc**: Manages LLM server process lifecycle through SharedContext coordination
- **Cross-cutting with all layers**: Implements application-wide concerns like graceful shutdown, configuration management, and service health monitoring

## Cross-Crate Integration Patterns

### Service Layer Bootstrap Coordination
Complex service initialization coordinated across BodhiApp's entire architecture:
- **AppService Registry Initialization**: Complete service composition with all 10 business services including authentication, model management, and configuration
- **SharedContext Bootstrap**: LLM server context initialization with HubService and SettingService coordination for model management
- **Route Integration**: routes_all coordination for complete HTTP route composition with middleware stack and static asset serving
- **Error Translation**: Service errors converted to appropriate HTTP responses with comprehensive error handling and graceful degradation

### Infrastructure Integration Architecture
Server executable coordinates with infrastructure and deployment layers:
- **Signal Handling Integration**: Cross-platform signal handling (Unix SIGTERM, Windows Ctrl+Break) with graceful shutdown coordination
- **Resource Management**: TCP listener management, database connection validation, and file system permission checking
- **Configuration Validation**: Environment variable validation, executable path verification, and service health checking
- **Static Asset Coordination**: Dynamic UI serving with development proxy support and production embedded asset serving

### Listener Pattern Integration
Advanced event-driven coordination across service boundaries:
- **Settings Change Propagation**: Real-time configuration updates with SettingsChangeListener pattern across service boundaries
- **Server State Coordination**: ServerStateListener pattern for LLM server lifecycle events with keep-alive timer management
- **Context State Management**: SharedContext state change notifications with observer pattern implementation
- **Service Health Monitoring**: Cross-service health checking with automatic recovery and error reporting

## Server Orchestration Workflows

### Multi-Service Bootstrap Coordination
Complex application initialization with comprehensive service orchestration:

1. **Service Registry Initialization**: AppService registry bootstrap with all 10 business services including dependency injection and configuration validation
2. **SharedContext Bootstrap**: LLM server context initialization with HubService and SettingService coordination for model management capabilities
3. **Listener Registration**: Advanced listener pattern setup with ServerKeepAlive and VariantChangeListener for real-time configuration management
4. **Route Composition**: routes_all integration for complete HTTP route and middleware stack with static asset serving configuration
5. **Server Lifecycle Management**: TCP listener binding, ready notification, and graceful shutdown coordination with signal handling

### Advanced Listener Orchestration Workflows
Sophisticated event-driven coordination across service boundaries:

**ServerKeepAlive Workflow**:
1. **Timer Management**: Configurable keep-alive timer with automatic server shutdown coordination based on inactivity
2. **Settings Integration**: Real-time keep-alive configuration updates with timer reset and cancellation logic
3. **State Coordination**: Server state change notifications with timer reset on chat completions and cancellation on server stop
4. **Resource Cleanup**: Automatic LLM server shutdown coordination when keep-alive timer expires

**VariantChangeListener Workflow**:
1. **Configuration Monitoring**: Real-time BODHI_EXEC_VARIANT setting changes with validation and error handling
2. **Context Coordination**: SharedContext execution variant updates for CPU/CUDA/ROCm configuration switching
3. **Async Processing**: Non-blocking variant updates with proper error logging and recovery
4. **Service Integration**: Seamless integration with SettingService for configuration management

### Graceful Shutdown Orchestration
Comprehensive shutdown coordination with resource cleanup:
1. **Signal Reception**: Cross-platform signal handling (Ctrl+C, SIGTERM, Ctrl+Break) with proper signal registration
2. **Shutdown Propagation**: Graceful shutdown signal propagation through server handle architecture
3. **Context Cleanup**: SharedContext shutdown coordination with LLM server process termination
4. **Resource Cleanup**: TCP listener cleanup, service shutdown, and resource deallocation with error handling

## Important Constraints

### Server Lifecycle Management Requirements
- All server operations must use Server handle architecture for consistent lifecycle management with ready notification and graceful shutdown
- Signal handling must support cross-platform operation (Unix SIGTERM, Windows Ctrl+Break) with proper signal registration and cleanup
- Graceful shutdown must coordinate across all services with proper resource cleanup and error handling
- TCP listener binding must handle port conflicts with appropriate error reporting and recovery mechanisms

### Service Bootstrap Coordination Standards
- All services must be initialized through AppService registry pattern for consistent dependency injection and configuration management
- SharedContext initialization must coordinate with HubService and SettingService for proper LLM server management capabilities
- Listener registration must follow observer pattern with proper error handling and async coordination
- Route composition must integrate with routes_all for complete HTTP stack with middleware and static asset serving

### Listener Pattern Integration Rules
- ServerKeepAlive must coordinate with both SettingsChangeListener and ServerStateListener for comprehensive timer management
- VariantChangeListener must handle async SharedContext updates with proper error logging and recovery
- All listeners must support real-time configuration updates without service interruption
- Listener error handling must not interrupt server operation with proper error isolation and logging

### Resource Management and Validation Requirements
- Executable path validation must occur during startup with clear error reporting for missing LLM server binaries
- Static asset serving must support both development proxy mode and production embedded assets with environment detection
- Service health validation must occur during bootstrap with comprehensive connectivity and permission checking
- Error handling must provide actionable guidance for common deployment and configuration issues

## Server Extension Patterns

### Adding New Server Lifecycle Components
When creating new server lifecycle management features:

1. **Listener Pattern Integration**: Implement SettingsChangeListener or ServerStateListener for real-time configuration and state management
2. **Service Bootstrap Extensions**: Coordinate with AppService registry for new service initialization and dependency injection
3. **Shutdown Callback Integration**: Implement ShutdownCallback trait for proper resource cleanup during graceful shutdown
4. **Error Handling**: Create server-specific errors that implement AppError trait for consistent error reporting and recovery
5. **Testing Infrastructure**: Use comprehensive service mocking for isolated server lifecycle testing scenarios

### Extending Configuration Management
For new configuration and settings management patterns:

1. **Settings Listener Integration**: Implement SettingsChangeListener for real-time configuration updates without service restart
2. **Environment Validation**: Add configuration validation during server bootstrap with clear error reporting
3. **Service Coordination**: Coordinate configuration changes across service boundaries with proper error handling
4. **Dynamic Updates**: Support runtime configuration updates through listener pattern with validation and rollback
5. **Configuration Testing**: Test configuration management with different environment scenarios and validation failures

### Server Orchestration Extensions
For new server orchestration and coordination patterns:

1. **Service Integration**: Coordinate with AppService registry for consistent business logic access and service composition
2. **Context Management**: Integrate with SharedContext for LLM server lifecycle coordination and state management
3. **Route Composition**: Coordinate with routes_all for HTTP route and middleware integration with proper error boundaries
4. **Resource Management**: Implement proper resource lifecycle management with cleanup and error recovery
5. **Integration Testing**: Support comprehensive server orchestration testing with realistic service interactions

## Server Lifecycle Error Coordination

### Service Bootstrap Error Handling
Comprehensive error management during server initialization:
- **Service Initialization Failures**: AppService registry initialization errors with detailed service-specific error reporting
- **SharedContext Bootstrap Errors**: LLM server context initialization failures with executable path validation and resource checking
- **Listener Registration Failures**: Observer pattern setup errors with proper error isolation and service degradation
- **Route Composition Errors**: routes_all integration failures with middleware and static asset serving error handling

### Runtime Error Management
Advanced error handling during server operation:
- **Listener Error Isolation**: ServerKeepAlive and VariantChangeListener errors isolated from server operation with proper logging
- **Configuration Update Failures**: Settings change errors handled gracefully without service interruption
- **Context State Errors**: SharedContext state change errors with proper recovery and error reporting
- **Signal Handling Errors**: Graceful shutdown signal processing errors with fallback mechanisms

### Resource Management Error Recovery
Sophisticated error recovery for resource management:
- **TCP Listener Binding Failures**: Port conflict detection with clear error reporting and alternative port suggestions
- **Static Asset Serving Errors**: Development proxy and production asset serving errors with graceful fallback
- **Service Health Check Failures**: Comprehensive service health validation with detailed error reporting and recovery guidance
- **Shutdown Coordination Errors**: Graceful shutdown error handling with resource cleanup and error isolation

## Server Testing Architecture

### Server Lifecycle Testing
Comprehensive testing of server orchestration and lifecycle management:
- **Bootstrap Testing**: Complete service initialization testing with AppService registry and SharedContext coordination
- **Listener Integration Testing**: ServerKeepAlive and VariantChangeListener testing with mock service coordination
- **Graceful Shutdown Testing**: Signal handling and shutdown coordination testing with resource cleanup validation
- **Error Scenario Testing**: Server startup and runtime error handling with comprehensive error recovery validation

### Service Integration Testing
Server-level integration testing with service mock coordination:
- **Service Registry Testing**: AppService registry initialization with comprehensive service mocking
- **Context Management Testing**: SharedContext integration testing with LLM server lifecycle coordination
- **Route Composition Testing**: routes_all integration testing with HTTP stack and middleware validation
- **Configuration Testing**: Settings management and listener pattern testing with real-time configuration updates

### Resource Management Testing
Server resource management and validation testing:
- **TCP Listener Testing**: Port binding, conflict detection, and error handling with realistic network scenarios
- **Static Asset Testing**: Development proxy and production asset serving with environment-specific configuration
- **Service Health Testing**: Comprehensive service health validation with connectivity and permission checking
- **Performance Testing**: Server performance under load with resource usage monitoring and optimization

