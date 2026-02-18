# CLAUDE.md

This file provides guidance to Claude Code when working with the `lib_bodhiserver` crate.

See [crates/lib_bodhiserver/PACKAGE.md](crates/lib_bodhiserver/PACKAGE.md) for implementation details.

## Purpose

The `lib_bodhiserver` crate serves as BodhiApp's **embeddable server library orchestration layer**, providing sophisticated service composition, application directory management, and comprehensive configuration systems for embedding BodhiApp functionality into external applications.

## Key Domain Architecture

### Embeddable Server Library System
Advanced library interface for embedding BodhiApp functionality:
- **AppServiceBuilder Architecture**: Sophisticated dependency injection with automatic service resolution and comprehensive error handling
- **Application Directory Management**: Complete filesystem setup with BODHI_HOME, HF_HOME, and configuration directory orchestration
- **Configuration System Integration**: Flexible configuration with environment variables, settings files, and system defaults coordination
- **Service Composition Orchestration**: Complete service registry initialization with all 16 business services and dependency management
- **Resource Lifecycle Management**: Proper initialization, configuration, and cleanup coordination for embedded deployments

### Cross-Crate Service Integration Architecture
Comprehensive service orchestration for embeddable library functionality:
- **Service Registry Composition**: Complete AppService registry with HubService, DataService, AuthService, DbService, SessionService, SecretService, CacheService, and TimeService
- **Configuration Management**: Environment-specific configuration with development/production modes and flexible settings management
- **Database Integration**: SQLite database setup with migration management and connection pooling for embedded scenarios
- **Authentication Coordination**: OAuth2 integration with session management and API token support for embedded authentication
- **Error Message Architecture**: User-friendly error messages via thiserror templates

### Application Bootstrap and Configuration System
Sophisticated application initialization with comprehensive configuration management:
- **AppOptions Builder Pattern**: Flexible configuration with environment variables, app settings, and OAuth credentials management
- **Directory Setup Orchestration**: Automatic creation of BODHI_HOME, aliases, databases, logs, and HuggingFace cache directories
- **Settings Service Integration**: Complete settings management with file-based configuration, environment variable overrides, and system defaults
- **Error Handling Architecture**: Comprehensive error types with user-friendly messages and recovery strategies for configuration failures
- **UI Asset Management**: Embedded static asset serving with Next.js frontend integration for complete application embedding

## Architecture Position

**Upstream dependencies** (crates this depends on):
- [`objs`](../objs/CLAUDE.md) -- domain types, `AppOptions`, `AppType`, `EnvType`
- [`services`](../services/CLAUDE.md) -- all 16 service traits and implementations
- [`server_core`](../server_core/CLAUDE.md) -- `RouterState`, `SharedContext`
- [`auth_middleware`](../auth_middleware/CLAUDE.md) -- authentication middleware
- [`routes_app`](../routes_app/CLAUDE.md) -- route handlers
- [`server_app`](../server_app/CLAUDE.md) -- `ServeCommand`, server lifecycle

**Downstream consumers** (crates that depend on this):
- [`bodhi/src-tauri`](../bodhi/src-tauri/CLAUDE.md) -- Tauri desktop app calls `build_app_service()`, `setup_app_dirs()`
- [`lib_bodhiserver_napi`](../lib_bodhiserver_napi/CLAUDE.md) -- NAPI Node.js bindings call `build_app_service()`, `setup_app_dirs()`

## Cross-Crate Integration Patterns

### Service Layer Composition Coordination
Complex service orchestration for embeddable library functionality:
- **AppServiceBuilder Integration**: Sophisticated dependency injection with automatic service resolution and comprehensive error handling
- **Service Registry Composition**: Complete AppService registry initialization with all 16 business services including authentication, model management, toolsets, MCP, and configuration
- **Database Service Coordination**: SQLite database setup with migration management, connection pooling, and transaction support for embedded scenarios
- **Authentication Service Integration**: OAuth2 flows, session management, and API token support coordinated through SecretService and KeyringService
- **Configuration Service Management**: SettingService integration with environment variables, settings files, and system defaults coordination

### Application Directory Management Integration
Comprehensive filesystem setup coordinated across BodhiApp's architecture:
- **BODHI_HOME Management**: Automatic directory creation with environment-specific paths (development vs production) and configuration validation
- **HuggingFace Cache Integration**: HF_HOME setup with hub directory creation and model cache management coordination
- **Database Directory Setup**: Application and session database creation with proper file permissions and migration support
- **Logs Directory Management**: Centralized logging directory setup with proper permissions and cleanup coordination
- **UI Asset Integration**: Embedded Next.js frontend assets with static file serving for complete application embedding

### Configuration System Integration
Advanced configuration management coordinated across all application layers:
- **AppOptions Builder Pattern**: Flexible configuration with environment variables, app settings, OAuth credentials, and system settings management
- **Settings Service Integration**: Complete settings management with file-based configuration, environment variable overrides, and system defaults coordination
- **Environment Type Management**: Development/production mode coordination with environment-specific configuration and resource management
- **Error Message Support**: User-friendly error messages via thiserror templates with comprehensive error handling

## Embeddable Library Orchestration Workflows

### Multi-Service Application Bootstrap Coordination
Complex application initialization with comprehensive service orchestration:

1. **Configuration Validation**: AppOptions validation with environment variables, system settings, and application configuration verification
2. **Directory Setup**: BODHI_HOME, HF_HOME, aliases, databases, and logs directory creation with proper permissions and error handling
3. **Service Composition**: AppServiceBuilder orchestration with all 16 business services including dependency injection and error recovery
4. **Database Migration**: SQLite database setup with schema migration, connection pooling, and transaction support for embedded scenarios
5. **Service Initialization**: Complete service initialization with error handling and recovery mechanisms

### Configuration Management Orchestration
Sophisticated configuration coordination across application boundaries:

**Environment-Specific Configuration**:
1. **Environment Detection**: Development vs production mode detection with appropriate configuration defaults and resource paths
2. **Settings File Management**: YAML configuration file loading with environment variable overrides and system defaults coordination
3. **OAuth Configuration**: Application registration information and authentication provider configuration with secure credential storage
4. **Resource Path Configuration**: BODHI_HOME, HF_HOME, and other resource path configuration with automatic directory creation

**Service Configuration Coordination**:
1. **Database Configuration**: SQLite connection strings, migration management, and connection pooling configuration for embedded scenarios
2. **Authentication Configuration**: OAuth2 provider settings, JWT configuration, and session management setup with secure credential handling
3. **Cache Configuration**: In-memory cache settings, TTL configuration, and eviction policy management for performance optimization
4. **Logging Configuration**: Log level management, output configuration, and resource cleanup for embedded application integration

### Error Handling and Recovery Orchestration
Comprehensive error management across embeddable library boundaries:
1. **Configuration Error Recovery**: Validation failures, missing directories, and permission issues with actionable error messages
2. **Service Initialization Errors**: Database connection failures, authentication setup errors, and service dependency resolution with recovery strategies
3. **Resource Management Errors**: Directory creation failures, file permission issues, and cleanup coordination with proper error isolation
4. **Integration Error Handling**: External application integration errors with graceful degradation and comprehensive error reporting

## Important Constraints

### Embeddable Library Requirements
- All library operations must use AppServiceBuilder pattern for consistent service composition and dependency injection
- Configuration management must support both programmatic and file-based configuration with environment variable overrides
- Service initialization must handle embedded scenarios with proper resource cleanup and error recovery mechanisms
- Directory management must support different deployment scenarios (Tauri, NAPI, standalone) with appropriate permissions

### Service Composition Standards
- AppServiceBuilder must resolve all service dependencies automatically with comprehensive error handling and validation
- Service registry must provide access to all 16 business services with proper lifecycle management and cleanup coordination
- Database services must support embedded SQLite with migration management and connection pooling for performance
- Authentication services must integrate with platform-specific credential storage and OAuth2 flows for embedded scenarios

### Configuration Management Rules
- AppOptions must validate all required configuration with clear error messages and recovery guidance
- Settings service must support environment-specific configuration with development/production mode coordination
- Directory setup must handle filesystem permissions and creation failures with proper error reporting and recovery
- Error messages should be user-friendly, written in sentence case, and end with a period

## Embeddable Library Extension Patterns

### Adding New Service Integration
When creating new service integration for embeddable library:

1. **AppServiceBuilder Extensions**: Add new service methods with dependency injection and automatic resolution patterns
2. **Configuration Integration**: Extend AppOptions with new configuration options and validation rules for service setup
3. **Error Handling**: Create service-specific errors that implement AppError trait for consistent error reporting and recovery
4. **Resource Management**: Implement proper resource lifecycle management with cleanup and error recovery mechanisms
5. **Testing Infrastructure**: Use comprehensive service mocking for isolated library integration testing scenarios

### Extending Configuration Management
For new configuration capabilities and embedded scenarios:

1. **AppOptions Extensions**: Add new configuration options with builder pattern and validation support for embedded scenarios
2. **Settings Integration**: Coordinate with SettingService for new configuration management and environment variable support
3. **Directory Management**: Extend directory setup for new resource types with proper permissions and error handling
4. **Environment Support**: Support new deployment environments with appropriate configuration defaults and resource management
5. **Configuration Testing**: Test configuration management with different embedded scenarios and validation failures

### New Deployment Context Support
When adding support for new deployment scenarios:

1. **Context Analysis**: Analyze resource constraints, security requirements, and lifecycle management needs for the new context
2. **Adaptation Patterns**: Implement context-specific adaptations while maintaining core functionality consistency
3. **Resource Coordination**: Design resource management strategies that work within the constraints of the new deployment context
4. **Security Considerations**: Implement appropriate security measures for credential storage and data protection in the new context
5. **Integration Testing**: Develop comprehensive testing strategies that validate functionality in realistic deployment scenarios
6. **Performance Optimization**: Optimize resource usage and startup performance for the specific constraints of the new context

## Performance and Scalability Considerations

### Cold Start Optimization
**Startup Performance**: The orchestration layer prioritizes fast cold starts through:
- **Lazy Initialization**: Services are initialized only when needed
- **Parallel Loading**: Independent services can be initialized concurrently
- **Resource Pooling**: Database connections and caches are established early and reused
- **Asset Optimization**: UI assets are pre-compressed and efficiently embedded

### Memory Management
**Resource Efficiency**: In embedded contexts, memory usage is carefully managed:
- **Service Sharing**: Multiple components share singleton services where appropriate
- **Cleanup Coordination**: Proper resource cleanup prevents memory leaks in long-running embedded scenarios
- **Cache Management**: Intelligent cache eviction prevents unbounded memory growth

### Extensibility Architecture
**Future-Proof Design**: The orchestration layer is designed to accommodate:
- **New Service Types**: Additional business services can be added without disrupting existing orchestration
- **Alternative Implementations**: Services can be swapped with alternative implementations (e.g., different databases, auth providers)
- **Context-Specific Optimizations**: New deployment contexts can implement context-specific optimizations while maintaining compatibility