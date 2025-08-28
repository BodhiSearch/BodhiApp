# CLAUDE.md

This file provides guidance to Claude Code when working with the `routes_app` crate.

*For detailed implementation examples and technical depth, see [crates/routes_app/PACKAGE.md](crates/routes_app/PACKAGE.md)*

## Purpose

The `routes_app` crate serves as BodhiApp's **application API orchestration layer**, implementing comprehensive HTTP endpoints for model management, authentication, API token management, and application configuration with sophisticated OpenAPI documentation generation and multi-service coordination.

## Key Domain Architecture

### Application API Orchestration System
Comprehensive HTTP endpoint implementation with sophisticated service coordination:
- **Model Management API**: Create, pull, and manage model aliases with command orchestration
- **Authentication API**: OAuth2 flows, session management, and user profile operations
- **API Token Management**: JWT token generation, validation, and lifecycle management
- **Application Configuration**: Settings management, setup flows, and system configuration
- **OpenAPI Documentation**: Comprehensive API specification generation with Utoipa integration
- **Development Tools**: Debug endpoints and developer utilities for system introspection

### Multi-Service HTTP Coordination Architecture
Sophisticated HTTP request processing with cross-service orchestration:
- **RouterState Integration**: Dependency injection providing access to AppService registry and SharedContext
- **Command Layer Integration**: Direct integration with commands crate for CLI operation orchestration
- **Service Layer Coordination**: Complex service interactions for authentication, model management, and configuration
- **Error Translation**: Service errors converted to appropriate HTTP responses with OpenAI-compatible error formats
- **Request Validation**: Comprehensive input validation using validator crate and custom validation rules

### OpenAPI Documentation Generation System
Advanced API documentation with comprehensive specification generation:
- **Utoipa Integration**: Automatic OpenAPI 3.0 specification generation with schema validation
- **Environment-Specific Configuration**: Dynamic server URL and security scheme configuration
- **Comprehensive Schema Coverage**: Complete request/response object documentation with examples
- **Interactive Documentation**: API exploration interface with authentication flow documentation
- **Multi-Format Support**: JSON schema generation with validation and example generation

## Architecture Position

The `routes_app` crate serves as BodhiApp's **application API orchestration layer**:
- **Above server_core and services**: Coordinates HTTP infrastructure and business services for application API operations
- **Below client applications**: Provides comprehensive API surface for web UI, mobile apps, and external integrations
- **Parallel to routes_oai**: Similar HTTP orchestration role but focused on application management instead of OpenAI compatibility
- **Integration with auth_middleware**: Leverages authentication middleware for comprehensive API security and authorization

## Cross-Crate Integration Patterns

### Service Layer API Coordination
Complex API operations coordinated across BodhiApp's service layer:
- **RouterState Integration**: HTTP handlers access AppService registry for comprehensive business logic operations
- **Command Layer Orchestration**: Direct integration with commands crate for CLI operation execution through HTTP endpoints
- **Authentication Service Coordination**: OAuth2 flows, token management, and user profile operations with comprehensive error handling
- **Model Management Integration**: DataService and HubService coordination for model alias management and file operations
- **Configuration Management**: SettingService integration for application configuration and system setup workflows

### HTTP Infrastructure Integration
Application API routes coordinate with HTTP infrastructure for request processing:
- **Request Validation**: Comprehensive input validation using validator crate with custom validation rules
- **Response Formatting**: Consistent API response formats with pagination, error handling, and OpenAI compatibility
- **Error Handling**: Service errors converted to appropriate HTTP status codes with localized messages
- **Authentication Integration**: Seamless integration with auth_middleware for API security and authorization
- **Route Composition Integration**: Coordinated with routes_all for unified route composition with session-based authentication and role-based authorization

### Domain Object Integration
Extensive use of objs crate for API operations:
- **Request/Response Objects**: Comprehensive API object definitions with validation and serialization
- **Error System Integration**: API errors implement AppError trait for consistent HTTP response generation
- **Parameter Validation**: Domain object validation rules applied consistently across API endpoints
- **Localization Support**: Multi-language error messages via objs LocalizationService integration

## API Orchestration Workflows

### Multi-Service Model Management Coordination
Complex model management operations with service orchestration:

1. **Model Alias Creation**: HTTP endpoints coordinate with commands crate for CreateCommand execution
2. **Model Pull Operations**: PullCommand orchestration through HTTP with progress tracking and error handling
3. **Model Discovery**: DataService and HubService coordination for model listing and metadata retrieval
4. **Alias Management**: CRUD operations for model aliases with validation and conflict resolution
5. **File Management**: Local model file operations coordinated with HubService and file system management

### Authentication Flow Orchestration
Sophisticated authentication coordination across multiple services:

**OAuth2 Authentication Workflow**:
1. **Login Initiation**: OAuth2 authorization URL generation with PKCE challenge and state validation
2. **Callback Processing**: Authorization code exchange with token validation and session creation
3. **Session Management**: HTTP session coordination with Tower Sessions and secure cookie handling
4. **Token Refresh**: Automatic token refresh with session updates and error recovery
5. **User Profile**: User information retrieval and profile management through authentication services

**API Token Management Workflow**:
1. **Token Creation**: JWT token generation with scope validation and database storage
2. **Token Validation**: Bearer token authentication with database lookup and status checking
3. **Token Lifecycle**: Token activation, deactivation, and expiration management
4. **Authorization**: Role-based and scope-based authorization for API endpoints

### Application Configuration Orchestration
Comprehensive application setup and configuration management:
1. **Initial Setup**: Application registration and configuration with OAuth provider integration
2. **Settings Management**: Configuration CRUD operations with validation and persistence
3. **System Status**: Application health and status monitoring with comprehensive diagnostics
4. **Development Tools**: Debug endpoints for system introspection and troubleshooting

## Important Constraints

### API Compatibility Requirements
- All endpoints must maintain consistent request/response formats for client application compatibility
- Request validation must use validator crate with comprehensive validation rules and error reporting
- Response formats must support pagination, sorting, and filtering for large data sets
- Error responses must follow OpenAI-compatible error format for ecosystem integration
- API versioning must be maintained through URL paths with backward compatibility considerations

### Authentication and Authorization Standards
- All protected endpoints must integrate with auth_middleware for consistent security
- OAuth2 flows must follow PKCE standards with proper state validation and CSRF protection
- API token management must support role-based and scope-based authorization with hierarchical access control
- Session management must use secure cookie configuration with SameSite and secure flags
- Authentication errors must provide actionable guidance without leaking sensitive information

### Service Coordination Rules
- All routes must use RouterState dependency injection for consistent service access
- Command orchestration must handle async operations with proper error propagation and recovery
- Service errors must be translated to appropriate HTTP status codes with localized messages
- Long-running operations must provide progress feedback and cancellation support
- Database operations must maintain transactional consistency across service boundaries

## Application API Extension Patterns

### Adding New Application Endpoints
When implementing additional application API endpoints:

1. **Service Coordination Design**: Use RouterState for consistent AppService access and business logic coordination
2. **Request/Response Objects**: Define comprehensive API objects with validation using validator crate and ToSchema for OpenAPI
3. **Command Integration**: Leverage commands crate for complex operations that require multi-service coordination
4. **Error Handling**: Implement API-specific errors that convert to appropriate HTTP responses with localization
5. **OpenAPI Documentation**: Add comprehensive Utoipa annotations with examples and proper schema definitions

### Extending Authentication Flows
For new authentication and authorization patterns:

1. **OAuth2 Integration**: Follow established PKCE patterns with proper state validation and CSRF protection
2. **Session Management**: Integrate with Tower Sessions for consistent session handling and secure cookie configuration
3. **API Token Extensions**: Extend JWT token management with new scopes and authorization patterns
4. **Authorization Middleware**: Coordinate with auth_middleware for consistent security across endpoints
5. **User Management**: Design user profile and account management features with proper privacy controls

### Cross-Service Integration Patterns
For features that require coordination across multiple services:

1. **Command Orchestration**: Use commands crate for complex multi-service workflows with proper error boundaries
2. **Service Registry**: Coordinate through AppService registry for consistent business logic access
3. **Error Translation**: Convert service errors to appropriate HTTP responses with OpenAI-compatible error formats
4. **Progress Tracking**: Implement progress feedback for long-running operations with cancellation support
5. **Transaction Management**: Ensure data consistency across service boundaries with proper rollback capabilities