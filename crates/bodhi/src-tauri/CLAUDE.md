# CLAUDE.md

This file provides guidance to Claude Code when working with the bodhi/src-tauri crate.

See [crates/bodhi/src-tauri/PACKAGE.md](crates/bodhi/src-tauri/PACKAGE.md) for implementation details and technical depth.

## Purpose

The `crates/bodhi/src-tauri` crate serves as BodhiApp's **unified application entry point**, providing sophisticated dual-mode deployment capabilities through feature-based conditional compilation. It orchestrates complete application embedding with lib_bodhiserver while supporting both native Tauri desktop applications and headless container deployments with comprehensive CLI interface abstraction.

## Key Domain Architecture

### Dual-Mode Application System

Sophisticated conditional compilation architecture enabling deployment flexibility:

- **Native Desktop Mode**: Tauri-based desktop application with system tray integration, menu-driven controls, and embedded web UI hosting
- **Container/Server Mode**: Headless server deployment optimized for containerized environments with comprehensive file-based logging
- **Feature-Based Architecture**: Conditional compilation using `native` feature flag determines initialization module selection and CLI behavior
- **Unified CLI Abstraction**: Single clap-based command interface adapts functionality based on compilation features and deployment context
- **Shared Configuration Logic**: Common AppOptions construction, environment detection, and service composition across deployment modes

### Native Desktop Integration Architecture

Comprehensive desktop application functionality with system-level integration:

- **Tauri Framework Coordination**: Cross-platform desktop application with webview embedding, plugin integration, and native system API access
- **System Tray Orchestration**: Background operation with menu-driven controls for homepage access, server management, and graceful shutdown
- **Embedded Server Management**: Complete lib_bodhiserver coordination with ServeCommand orchestration and automatic lifecycle management
- **Browser Launch Coordination**: Automatic browser launching with web URL construction and fallback error handling for accessibility
- **Window Lifecycle Management**: Hide-on-close behavior, proper window state management, and coordinated server shutdown processes

### Container Deployment Architecture

Optimized headless server deployment for containerized and production environments:

- **HTTP Server Orchestration**: Direct lib_bodhiserver integration with ServeCommand execution and comprehensive API serving coordination
- **Structured Logging Infrastructure**: File-based logging with daily rotation, configurable output targets, and environment-specific filtering
- **Dynamic Configuration Override**: Command-line parameter processing with SettingService integration for host/port configuration management
- **Environment-Specific Configuration**: Feature-based switching between development and production authentication endpoints and resource paths

### CLI Command Architecture

Unified command-line interface with sophisticated feature-based behavior switching:

- **Clap Framework Integration**: Comprehensive CLI parsing with subcommand support and feature-conditional availability based on compilation targets
- **AppCommand Enum Abstraction**: Unified command representation coordinating server deployment parameters and native desktop initialization
- **Feature-Conditional Interface**: Conditional compilation enables different CLI subcommands based on `native` feature flag compilation
- **Comprehensive Validation**: CLI parameter validation with type checking, range validation, and actionable error guidance

### Application Configuration Management

Sophisticated environment-specific configuration coordination across deployment modes:

- **Environment Detection**: Development vs production mode switching with appropriate authentication endpoints, OAuth realms, and resource paths
- **Configuration Builder Coordination**: AppOptions pattern with environment variables, OAuth credentials, and system settings composition
- **Settings Service Integration**: SettingService coordination for configuration management with command-line overrides and environment variable precedence
- **Resource Path Orchestration**: BODHI_EXEC_LOOKUP_PATH configuration for LLM server binary discovery, execution variant management, and platform-specific handling

## Architecture Position

The `crates/bodhi/src-tauri` crate serves as BodhiApp's **unified application orchestration layer**:

- **Above lib_bodhiserver**: Coordinates complete application embedding with service composition, configuration management, and deployment coordination
- **Below deployment infrastructure**: Provides executable entry points for native desktop and container deployment scenarios with feature-based compilation
- **Integration with objs**: Uses domain objects for error handling, configuration validation, CLI parameter management, and localization support
- **Cross-cutting coordination**: Implements application-wide concerns including logging configuration, environment management, resource lifecycle, and deployment orchestration

## Cross-Crate Integration Patterns

### Service Layer Integration Coordination

Application entry point orchestrates BodhiApp's complete service architecture:

- **lib_bodhiserver Embedding**: Complete application service registry composition with AppServiceBuilder pattern and configuration management coordination
- **Service Composition**: Coordinates all business services through lib_bodhiserver's dependency injection with proper initialization ordering and error handling
- **Configuration Management**: Environment-specific configuration with development/production mode switching, OAuth endpoint coordination, and settings service integration
- **Error Translation Coordination**: Service errors converted to appropriate application-level error messages with localization support and user-actionable guidance

### Deployment Mode Integration Architecture

Comprehensive deployment coordination across different execution environments:

- **Native Desktop Coordination**: Tauri framework integration with system tray management, menu event handling, and embedded web UI hosting
- **Container Deployment Coordination**: Headless server deployment with HTTP API serving, structured logging, and configuration override management
- **CLI Interface Coordination**: Command-line interface integration with clap parsing, feature-conditional subcommand availability, and parameter validation
- **Resource Lifecycle Management**: Application directory setup, logging infrastructure configuration, and service coordination across deployment modes

## Application Orchestration Workflows

### Dual-Mode Application Initialization

Sophisticated application bootstrap with feature-based execution path selection:

1. **CLI Command Processing**: Clap-based command parsing with feature-conditional subcommand availability and comprehensive parameter validation
2. **Command Resolution**: AppCommand enum construction with deployment mode detection, parameter extraction, and configuration preparation
3. **Feature-Based Dispatch**: Conditional compilation routing to appropriate initialization module (native_init vs server_init) based on compilation features
4. **Configuration Bootstrap**: AppOptions construction with environment detection, OAuth endpoint configuration, and settings service coordination
5. **Service Composition**: lib_bodhiserver integration with complete AppService registry initialization, dependency injection, and error handling coordination

### Native Desktop Application Orchestration

Comprehensive desktop application coordination with system-level integration:

**Native Mode Workflow**:

1. **Tauri Framework Bootstrap**: Application builder configuration with plugin integration, logging setup, and system-specific policies
2. **System Integration Setup**: System tray creation with menu configuration, event handling coordination, and platform-specific activation policies
3. **Embedded Server Coordination**: lib_bodhiserver integration with ServeCommand orchestration, automatic startup, and health monitoring
4. **UI Integration**: Web browser launching with URL construction, embedded asset serving, and fallback error handling
5. **Lifecycle Management**: Window hide-on-close behavior, coordinated server shutdown, graceful resource cleanup, and application exit handling

### Container Deployment Orchestration

Optimized headless server deployment with comprehensive logging and configuration management:

**Container Mode Workflow**:

1. **Configuration Override**: Command-line parameter processing with settings service integration, environment variable coordination, and validation
2. **Logging Infrastructure**: File-based logging setup with daily rotation, configurable output targets, structured log management, and filtering
3. **Service Bootstrap**: lib_bodhiserver integration with ServeCommand execution, HTTP server coordination, and service composition
4. **Resource Management**: Proper resource lifecycle management with cleanup coordination, error handling, and graceful degradation
5. **Graceful Shutdown**: Signal handling coordination, resource cleanup, comprehensive error recovery, and proper exit handling

## Important Constraints

### Dual-Mode Architecture Requirements

- All application logic must support both native desktop and container deployment modes through feature-based conditional compilation with clean separation
- CLI interface must adapt behavior based on compilation features with appropriate subcommand availability, parameter validation, and error handling
- Configuration management must coordinate environment-specific settings with development/production mode switching and proper precedence handling
- Error handling must provide appropriate user guidance for both desktop and server deployment scenarios with localization support

### Service Integration Standards

- All service access must coordinate through lib_bodhiserver's AppServiceBuilder pattern for consistent dependency injection and initialization ordering
- Application bootstrap must handle complete service composition with all business services, dependency resolution, and comprehensive error recovery
- Configuration validation must occur during application startup with clear error reporting, diagnostic information, and recovery guidance
- Resource lifecycle management must coordinate across deployment modes with proper initialization, cleanup, and error handling coordination

### Native Desktop Integration Rules

- Tauri framework integration must support system tray functionality with menu-driven controls, proper event handling, and platform-specific behavior
- Desktop application must coordinate embedded server lifecycle with automatic startup, configuration validation, health monitoring, and graceful shutdown
- Web browser integration must handle launch failures gracefully with fallback error reporting, user guidance, and alternative access methods
- Platform-specific features must be properly abstracted with conditional compilation, feature flags, and cross-platform compatibility validation

### Container Deployment Standards

- Headless server mode must support comprehensive file-based logging with daily rotation, configurable output targets, and structured log formatting
- Command-line parameter processing must integrate with settings service for configuration override, validation, and precedence management
- HTTP server coordination must leverage lib_bodhiserver's ServeCommand pattern with proper error handling, resource management, and lifecycle coordination
- Signal handling must support graceful shutdown with comprehensive resource cleanup, error recovery, and proper exit status handling

## Application Extension Patterns

### Adding New Deployment Modes

When creating new deployment scenarios for the multi-mode application:

1. **Feature Flag Design**: Create new feature flags with appropriate conditional compilation for deployment-specific functionality and clean separation
2. **Initialization Module**: Implement new initialization modules following the pattern established by native_init and server_init with proper abstraction
3. **CLI Integration**: Extend clap command structure with new subcommands, parameter validation, and deployment-specific configuration handling
4. **Configuration Extensions**: Add new AppOptions configuration with environment-specific settings, validation rules, and precedence handling
5. **Service Coordination**: Coordinate with lib_bodhiserver for new service composition patterns, resource management, and lifecycle handling

### Extending Native Desktop Features

For new native desktop functionality and system integration:

1. **Tauri Integration**: Leverage Tauri framework capabilities with proper plugin integration, system API access, and cross-platform abstractions
2. **System Integration**: Design system-level features with cross-platform compatibility, proper error handling, and platform-specific behavior
3. **UI Coordination**: Coordinate embedded web UI with native desktop features through proper event handling, state management, and messaging
4. **Resource Management**: Implement proper resource lifecycle management with initialization, cleanup coordination, and error recovery
5. **Platform Testing**: Test desktop features across different operating systems with platform-specific validation and behavior verification

### Container Deployment Extensions

For new container and server deployment capabilities:

1. **Configuration Management**: Extend command-line parameter processing with settings service integration, validation, and precedence handling
2. **Logging Infrastructure**: Design comprehensive logging strategies with file-based output, structured log management, filtering, and rotation
3. **Service Orchestration**: Coordinate with lib_bodhiserver for new HTTP server patterns, service composition, and lifecycle management
4. **Resource Optimization**: Optimize resource usage for containerized environments with proper cleanup, error handling, and graceful degradation
5. **Deployment Testing**: Test container deployment scenarios with realistic infrastructure, configuration validation, and operational verification

## Critical System Constraints

### Application Architecture Requirements

- Multi-mode architecture must maintain clean separation between native desktop and container deployment with feature-based conditional compilation and clear boundaries
- CLI interface must provide consistent user experience across deployment modes with appropriate command availability, parameter validation, and error handling
- Configuration management must support environment-specific settings with proper validation, precedence handling, and error recovery mechanisms
- Service integration must coordinate through lib_bodhiserver for consistent application composition, dependency injection, and initialization ordering

### Error Handling and Recovery Standards

- All application errors must implement AppError trait for consistent error reporting, localization support, and error type classification
- Error messages must provide actionable guidance appropriate for the deployment mode, user context, and operational environment
- Error recovery must coordinate across service boundaries with proper resource cleanup, state management, and graceful degradation
- Application startup errors must provide clear diagnostic information, configuration guidance, and deployment troubleshooting assistance

### Resource Management and Lifecycle Rules

- Application lifecycle must coordinate resource management across deployment modes with proper initialization, cleanup, and error handling
- Service composition must handle dependency injection, initialization ordering, and error recovery through lib_bodhiserver coordination
- Configuration validation must occur during application startup with comprehensive error reporting, diagnostic information, and recovery guidance
- Resource cleanup must coordinate across all application components with proper error handling, graceful degradation, and exit status management
