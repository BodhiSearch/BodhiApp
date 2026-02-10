# CLAUDE.md

This file provides guidance to Claude Code when working with the `server_app` crate.

See [crates/server_app/PACKAGE.md](crates/server_app/PACKAGE.md) for implementation details and file references.

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
- **Route Composition Coordination**: Integration with routes_app for complete HTTP route and middleware stack composition
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
- **AppService Registry Initialization**: Complete service composition with all business services including authentication, model management, and configuration
- **SharedContext Bootstrap**: LLM server context initialization with HubService and SettingService coordination for model management
- **Route Integration**: routes_app coordination for complete HTTP route composition with middleware stack and static asset serving
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

1. **Service Registry Initialization**: AppService registry bootstrap with all business services including dependency injection and configuration validation
2. **SharedContext Bootstrap**: LLM server context initialization with HubService and SettingService coordination for model management capabilities
3. **Listener Registration**: Advanced listener pattern setup with ServerKeepAlive and VariantChangeListener for real-time configuration management
4. **Route Composition**: routes_app integration for complete HTTP route and middleware stack with static asset serving configuration
5. **Server Lifecycle Management**: TCP listener binding, ready notification, and graceful shutdown coordination with signal handling

### Advanced Listener Orchestration Workflows

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
- Route composition must integrate with routes_app for complete HTTP stack with middleware and static asset serving

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

## Live Integration Test Architecture

### Design Philosophy
The live integration tests validate the full end-to-end stack: real HTTP server, real llama.cpp inference, real OAuth2 authentication, and real API responses. They intentionally avoid mocks to verify that the complete system works correctly when all components are wired together. This catches integration issues that unit tests with mocked services cannot detect.

### Why Inline AppService Setup (No lib_bodhiserver Dependency)
The `setup_minimal_app_service` function in `tests/utils/live_server_utils.rs` manually constructs a `DefaultAppService` with all real service implementations rather than depending on `lib_bodhiserver`. This design decision exists because:
- **Avoiding circular dependencies**: `lib_bodhiserver` depends on `server_app`, so `server_app` cannot depend back on it
- **Test isolation**: The inline setup gives precise control over which services are real vs stubbed (e.g., `OfflineHubService` wraps real `HfHubService` to prevent network downloads during tests)
- **Transparency**: Every service dependency is visible in the test setup code, making failures easier to diagnose

### OAuth2 Authentication Flow for Tests
Live tests require real OAuth2 authentication because the server enforces auth middleware. The test infrastructure:
1. Creates a **resource client** via Keycloak admin API using dev-console credentials
2. Makes the test user an **admin** of the newly created resource client
3. Obtains **access/refresh tokens** via OAuth2 resource-owner password grant
4. Injects tokens into a **session record** in the SQLite session store
5. Creates a **session cookie** that the HTTP client sends with every request

This mirrors the production auth flow except it uses the password grant for automation (production uses authorization code flow).

### Serial Execution Constraint
All live tests use `#[serial_test::serial(live)]` because they share the same llama.cpp server binary and model file. Running them in parallel would cause port conflicts and race conditions on the LLM process. The `live` group name ensures only tests within this crate are serialized -- other crate tests can still run in parallel.

### Test Coverage Categories
The live tests cover distinct LLM inference scenarios:
- **Basic chat completion**: Non-streamed and streamed responses validating OpenAI-compatible response format
- **Tool calling**: Single-turn and multi-turn tool invocations (both non-streamed and streamed) verifying the `tool_calls` finish reason, function name extraction, argument parsing, and follow-up response generation
- **Thinking/reasoning**: Verifying `chat_template_kwargs.enable_thinking`, `reasoning_format: "none"`, and default thinking behavior with `reasoning_content` field presence/absence
- **Agentic chat with Exa**: End-to-end agentic workflow including toolset type enablement, user-level toolset configuration with API key, qualified tool name generation, backend tool execution via `/bodhi/v1/toolsets/:id/execute/:method`, and multi-turn completion with tool results

### Environment Requirements
Live tests require external resources that cannot be committed to the repository:
- **Model**: Pre-downloaded `ggml-org/Qwen3-1.7B-GGUF` in `~/.cache/huggingface/hub/`
- **llama.cpp binary**: Present at `crates/llama_server_proc/bin/`
- **OAuth2 config**: `tests/resources/.env.test` with Keycloak credentials (see `.env.test.example`)
- **Exa API key**: `INTEG_TEST_EXA_API_KEY` environment variable (for agentic chat test only)

## Server Lifecycle Error Coordination

### Service Bootstrap Error Handling
Comprehensive error management during server initialization:
- **Service Initialization Failures**: AppService registry initialization errors with detailed service-specific error reporting
- **SharedContext Bootstrap Errors**: LLM server context initialization failures with executable path validation and resource checking
- **Listener Registration Failures**: Observer pattern setup errors with proper error isolation and service degradation
- **Route Composition Errors**: routes_app integration failures with middleware and static asset serving error handling

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
