# CLAUDE.md

This file provides guidance to Claude Code when working with the bodhi/src-tauri crate.

_For detailed implementation examples and technical depth, see [crates/bodhi/src-tauri/PACKAGE.md](crates/bodhi/src-tauri/PACKAGE.md)_

## Purpose

The `bodhi/src-tauri` crate serves as BodhiApp's **dual-mode application entry point**, providing both native desktop application functionality and container/server deployment capabilities through sophisticated conditional compilation and feature-based architecture switching. It coordinates complete application embedding with lib_bodhiserver while supporting cross-platform desktop deployment and headless server operation.

## Key Domain Architecture

### Dual-Mode Application System

Advanced conditional compilation architecture enabling multiple deployment modes:

- **Native Desktop Mode**: Tauri-based desktop application with system tray, menu integration, and embedded web UI serving
- **Container/Server Mode**: Headless server deployment with HTTP API serving and file-based logging for containerized environments
- **Feature-Based Switching**: Conditional compilation using `native` feature flag to select appropriate initialization and execution paths
- **Unified CLI Interface**: Single clap-based command-line interface that adapts behavior based on compilation features and runtime context
- **Shared Application Logic**: Common application setup, configuration management, and service integration across both deployment modes

### Native Desktop Integration Architecture

Sophisticated desktop application functionality with system-level integration:

- **Tauri Framework Integration**: Cross-platform desktop application with webview-based UI hosting and native system API access
- **System Tray Management**: Background operation with menu-driven controls for homepage access and graceful application shutdown
- **Embedded Server Orchestration**: Complete lib_bodhiserver integration with automatic startup, configuration, and lifecycle management
- **Web Browser Integration**: Automatic browser launching for UI access with fallback handling for browser launch failures
- **Application Lifecycle Management**: Window hide-on-close behavior, server shutdown coordination, and proper resource cleanup

### Container Deployment Architecture

Headless server deployment optimized for containerized and production environments:

- **HTTP Server Mode**: Direct lib_bodhiserver integration with ServeCommand orchestration for API serving
- **File-Based Logging**: Comprehensive logging system with daily rotation, configurable output targets, and structured log management
- **Configuration Override System**: Command-line parameter support for host/port configuration with settings service integration
- **Production Environment Support**: Feature-based configuration switching between development and production authentication endpoints

### CLI Command Architecture

Sophisticated command-line interface with feature-based behavior switching:

- **Clap Integration**: Comprehensive CLI parsing with subcommand support and feature-conditional command availability
- **AppCommand Enum**: Unified command representation supporting both server deployment and native desktop modes
- **Feature-Based CLI**: Conditional compilation enables different CLI interfaces based on `native` feature flag
- **Error Handling**: CLI-specific error translation with actionable user guidance and comprehensive validation

### Application Configuration Management

Environment-specific configuration coordination across deployment modes:

- **Environment Detection**: Development vs production mode switching with appropriate authentication endpoints and resource paths
- **Configuration Builder**: AppOptions pattern with environment variables, OAuth credentials, and system settings coordination
- **Settings Integration**: SettingService coordination for configuration management with file-based and environment variable overrides
- **Resource Path Management**: BODHI_EXEC_LOOKUP_PATH configuration for LLM server binary discovery and execution variant management

## Architecture Position

The `bodhi/src-tauri` crate serves as BodhiApp's **application entry point orchestration layer**:

- **Above lib_bodhiserver**: Coordinates complete application embedding with service composition and configuration management
- **Below deployment infrastructure**: Provides executable entry points for both native desktop and container deployment scenarios
- **Integration with objs**: Uses domain objects for error handling, configuration validation, and CLI parameter management
- **Cross-cutting with all layers**: Implements application-wide concerns like logging configuration, environment management, and resource lifecycle

## Cross-Crate Integration Patterns

### Service Layer Integration Coordination

Application entry point coordinates with BodhiApp's complete service architecture:

- **lib_bodhiserver Integration**: Complete application embedding with AppService registry composition and configuration management
- **Service Orchestration**: Coordinates all 10 business services through lib_bodhiserver's AppServiceBuilder pattern with dependency injection
- **Configuration Coordination**: Environment-specific configuration with development/production mode switching and OAuth endpoint management
- **Error Translation**: Service errors converted to appropriate application-level error messages with localized user guidance

### Deployment Mode Integration Architecture

Sophisticated deployment coordination across different execution environments:

- **Native Desktop Integration**: Tauri framework coordination with system tray, menu management, and embedded web UI serving
- **Container Deployment Integration**: Headless server deployment with HTTP API serving and file-based logging for containerized environments
- **CLI Integration**: Command-line interface coordination with clap parsing and feature-conditional command availability
- **Resource Management**: Application directory setup, logging configuration, and service lifecycle management across deployment modes

## Application Orchestration Workflows

### Dual-Mode Application Initialization

Complex application bootstrap with feature-based execution path selection:

1. **CLI Parsing**: Clap-based command parsing with feature-conditional subcommand availability and comprehensive validation
2. **Command Resolution**: AppCommand enum construction with deployment mode detection and parameter extraction
3. **Feature-Based Dispatch**: Conditional compilation routing to appropriate initialization module (native_init vs server_init)
4. **Configuration Bootstrap**: AppOptions construction with environment detection and OAuth endpoint configuration
5. **Service Composition**: lib_bodhiserver integration with complete AppService registry initialization and dependency injection

### Native Desktop Application Orchestration

Sophisticated desktop application coordination with system integration:

**Native Mode Workflow**:

1. **Tauri Framework Bootstrap**: Application builder configuration with plugin integration and logging setup
2. **System Integration Setup**: System tray creation, menu configuration, and platform-specific activation policy
3. **Embedded Server Coordination**: lib_bodhiserver integration with ServeCommand orchestration and automatic startup
4. **UI Integration**: Web browser launching for UI access with embedded asset serving and error handling
5. **Lifecycle Management**: Window hide-on-close behavior, server shutdown coordination, and graceful application exit

### Container Deployment Orchestration

Headless server deployment with comprehensive logging and configuration management:

**Container Mode Workflow**:

1. **Configuration Override**: Command-line parameter processing with settings service integration and environment variable coordination
2. **Logging Infrastructure**: File-based logging setup with daily rotation, configurable output targets, and structured log management
3. **Service Bootstrap**: lib_bodhiserver integration with ServeCommand execution and HTTP server coordination
4. **Resource Management**: Proper resource lifecycle management with cleanup coordination and error handling
5. **Graceful Shutdown**: Signal handling and resource cleanup with comprehensive error recovery

## Important Constraints

### Dual-Mode Architecture Requirements

- All application logic must support both native desktop and container deployment modes through feature-based conditional compilation
- CLI interface must adapt behavior based on compilation features with appropriate subcommand availability and validation
- Configuration management must coordinate environment-specific settings with development/production mode switching
- Error handling must provide appropriate user guidance for both desktop and server deployment scenarios

### Service Integration Standards

- All service access must coordinate through lib_bodhiserver's AppServiceBuilder pattern for consistent dependency injection
- Application bootstrap must handle complete service composition with all 10 business services and comprehensive error recovery
- Configuration validation must occur during application startup with clear error reporting and recovery guidance
- Resource lifecycle management must coordinate across deployment modes with proper cleanup and error handling

### Native Desktop Integration Rules

- Tauri framework integration must support system tray functionality with menu-driven controls and proper event handling
- Desktop application must coordinate embedded server lifecycle with automatic startup, configuration, and graceful shutdown
- Web browser integration must handle launch failures gracefully with fallback error reporting and user guidance
- Platform-specific features must be properly abstracted with conditional compilation for cross-platform compatibility

### Container Deployment Standards

- Headless server mode must support comprehensive file-based logging with daily rotation and configurable output targets
- Command-line parameter processing must integrate with settings service for configuration override and validation
- HTTP server coordination must leverage lib_bodhiserver's ServeCommand pattern with proper error handling and resource management
- Signal handling must support graceful shutdown with comprehensive resource cleanup and error recovery

## Application Extension Patterns

### Adding New Deployment Modes

When creating new deployment scenarios for the dual-mode application:

1. **Feature Flag Design**: Create new feature flags with appropriate conditional compilation for deployment-specific functionality
2. **CLI Integration**: Extend clap command structure with new subcommands and parameter validation for deployment scenarios
3. **Configuration Extensions**: Add new AppOptions configuration with environment-specific settings and validation rules
4. **Service Coordination**: Coordinate with lib_bodhiserver for new service composition patterns and resource management
5. **Error Handling**: Implement deployment-specific errors that provide actionable guidance for configuration and setup issues

### Extending Native Desktop Features

For new native desktop functionality and system integration:

1. **Tauri Integration**: Leverage Tauri framework capabilities with proper plugin integration and system API access
2. **System Integration**: Design system-level features with cross-platform compatibility and proper error handling
3. **UI Coordination**: Coordinate embedded web UI with native desktop features through proper event handling and state management
4. **Resource Management**: Implement proper resource lifecycle management with cleanup coordination and error recovery
5. **Platform Testing**: Test desktop features across different operating systems with platform-specific validation

### Container Deployment Extensions

For new container and server deployment capabilities:

1. **Configuration Management**: Extend command-line parameter processing with settings service integration and validation
2. **Logging Infrastructure**: Design comprehensive logging strategies with file-based output and structured log management
3. **Service Orchestration**: Coordinate with lib_bodhiserver for new HTTP server patterns and service composition
4. **Resource Optimization**: Optimize resource usage for containerized environments with proper cleanup and error handling
5. **Deployment Testing**: Test container deployment scenarios with realistic infrastructure and configuration validation

## Critical System Constraints

### Application Architecture Requirements

- Dual-mode architecture must maintain clean separation between native desktop and container deployment with feature-based conditional compilation
- CLI interface must provide consistent user experience across deployment modes with appropriate command availability and validation
- Configuration management must support environment-specific settings with proper validation and error recovery mechanisms
- Service integration must coordinate through lib_bodhiserver for consistent application composition and dependency injection

### Error Handling and Recovery Standards

- All application errors must implement AppError trait for consistent error reporting and localization support
- Error messages must provide actionable guidance appropriate for the deployment mode and user context
- Error recovery must coordinate across service boundaries with proper resource cleanup and state management
- Application startup errors must provide clear diagnostic information for configuration and deployment issues

### Resource Management and Lifecycle Rules

- Application lifecycle must coordinate resource management across deployment modes with proper initialization and cleanup
- Service composition must handle dependency injection and error recovery through lib_bodhiserver coordination
- Configuration validation must occur during application startup with comprehensive error reporting and recovery guidance
- Resource cleanup must coordinate across all application components with proper error handling and graceful degradation
