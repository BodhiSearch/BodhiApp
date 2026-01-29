# CLAUDE.md

This file provides guidance to Claude Code when working with the `routes_all` crate.

*For detailed implementation examples and technical depth, see [PACKAGE.md](./PACKAGE.md)*

## Purpose

The `routes_all` crate serves as BodhiApp's **HTTP route composition and middleware orchestration layer**, implementing sophisticated route unification, multi-layered authentication, and comprehensive API documentation with advanced UI serving capabilities.

## Key Domain Architecture

### Route Composition and Unification System
Sophisticated HTTP route orchestration with comprehensive middleware integration:
- **Multi-Route Integration**: Combines routes_oai (OpenAI/Ollama compatibility) and routes_app (application management) with unified RouterState dependency injection
- **Hierarchical Authorization**: Five-tier authorization system (Public → Optional Auth → User → PowerUser/Admin → Manager) with role and scope-based access control
- **Middleware Orchestration**: Layered middleware with auth_middleware, session management, canonical URL handling, and comprehensive tracing
- **Development Mode Support**: Dynamic UI serving with environment-based configuration supporting proxy mode, embedded assets, and graceful fallbacks
- **Cross-Origin Resource Sharing**: Permissive CORS configuration optimized for development workflows and web client integration

### Multi-Layer Authentication Architecture
Advanced authentication system supporting multiple authentication flows:
- **Dual Authentication Support**: Bearer token authentication for API endpoints and session-based authentication for web management interfaces
- **Role-Based Authorization**: Hierarchical role system (Admin > Manager > PowerUser > User) with fine-grained endpoint protection and has_access_to() validation
- **Scope-Based Authorization**: Dual scope system with TokenScope (API token capabilities) and UserScope (session user capabilities) validation
- **Session Integration**: Tower Sessions with secure cookie configuration, CSRF protection, and automatic lifecycle management
- **Optional Authentication Layer**: inject_optional_auth_info middleware for endpoints requiring conditional authentication

### OpenAPI Documentation and UI Serving System
Comprehensive API specification with interactive documentation and dynamic UI serving:
- **Unified Documentation**: BodhiOpenAPIDoc combines specifications from routes_oai and routes_app with OpenAPIEnvModifier for environment adaptation
- **Interactive Interface**: Swagger UI at /swagger-ui with /api-docs/openapi.json endpoint for comprehensive API exploration
- **Dynamic UI Serving**: Environment-based UI serving with production embedded assets, development proxy to localhost:3000, and fallback handling
- **Error Message Resources**: User-friendly error messages via thiserror templates

## Architecture Position

The `routes_all` crate serves as BodhiApp's **HTTP route composition and middleware orchestration layer**:
- **Above server_core and services**: Coordinates HTTP infrastructure and business services for complete route composition
- **Below client applications**: Provides unified HTTP API surface for web UI, mobile apps, CLI tools, and external integrations
- **Integration with auth_middleware**: Leverages authentication middleware for comprehensive multi-layer security across all endpoints
- **Coordination with routes_oai and routes_app**: Unifies OpenAI/Ollama compatibility and application management endpoints with consistent middleware

## Cross-Crate Integration Patterns

### Route Layer Integration Architecture
Complex route composition coordinated across BodhiApp's HTTP layer:
- **RouterState Integration**: Unified state management providing access to AppService registry and SharedContext for all route handlers
- **routes_oai Integration**: OpenAI and Ollama API endpoints with bearer token authentication and comprehensive parameter validation
- **routes_app Integration**: Application management endpoints with session-based authentication and role-based authorization
- **auth_middleware Coordination**: Multi-layer authentication with role hierarchy enforcement and scope-based access control
- **Error Translation**: Consistent error handling across all routes with OpenAI-compatible error responses

### Service Layer HTTP Coordination
Route composition coordinates with BodhiApp's service layer:
- **AppService Registry Access**: All routes access business services through RouterState dependency injection
- **Authentication Service Integration**: OAuth2 flows, session management, and API token validation coordinated across route boundaries
- **Model Management Coordination**: DataService and HubService integration for model operations across OpenAI and application endpoints
- **Configuration Management**: SettingService integration for environment-specific routing and UI serving configuration

### Middleware Integration Architecture
Sophisticated middleware orchestration across route boundaries:
- **Authentication Middleware**: Role-based and scope-based authorization with hierarchical access control enforcement
- **Session Management**: Tower Sessions integration with secure cookie configuration and automatic session lifecycle
- **Canonical URL Middleware**: SEO optimization and security benefits through automatic canonical URL redirection
- **CORS Configuration**: Cross-origin resource sharing with comprehensive header and method support for web client integration

## Route Composition Orchestration Workflows

### Multi-Layer Route Integration Architecture
Sophisticated route composition with hierarchical authorization:

1. **Public Route Layer**: Health checks, app info, setup, and logout endpoints with no authentication requirements
2. **Dev Route Layer**: Development-only endpoints (secrets, envs) conditionally added in non-production environments
3. **Optional Authentication Layer**: User info, OAuth2 flows, and access request endpoints with inject_optional_auth_info middleware
4. **User-Level API Layer**: OpenAI/Ollama endpoints and basic model operations requiring User role and scope authorization
5. **PowerUser API Layer**: Model management, API model operations, and download handling requiring PowerUser role and scope
6. **PowerUser Session Layer**: Token management operations requiring PowerUser role with session-only authentication
7. **Admin Session Layer**: Settings management and user removal requiring Admin role with session-only authentication
8. **Manager Session Layer**: Access request management and user role changes requiring Manager role with session-only authentication

### Authentication Flow Orchestration
Complex authentication coordination across route boundaries:

**Bearer Token Authentication Flow**:
1. **API Route Access**: OpenAI/Ollama and model management endpoints require bearer token authentication
2. **Role Validation**: Hierarchical role checking (User/PowerUser/Admin) with has_access_to() validation
3. **Scope Authorization**: Token scope validation (TokenScope::User/PowerUser) and user scope validation (UserScope::User/PowerUser)
4. **Database Token Verification**: API token status checking with digest-based lookup and active/inactive validation

**Session-Based Authentication Flow**:
1. **Session Route Access**: Application management and token management endpoints use session authentication
2. **OAuth2 Integration**: Authorization code exchange with PKCE security and session creation
3. **Session Management**: Tower Sessions with secure cookie configuration and automatic lifecycle management
4. **Role Extraction**: Role extraction from session data with consistent authorization validation

### UI Serving Orchestration
Dynamic UI serving with environment-specific configuration via apply_ui_router:
1. **Production Mode**: Embedded static assets serving with optimized performance and proper caching headers
2. **Development Proxy Mode**: BODHI_DEV_PROXY_UI=true enables proxy_router forwarding to localhost:3000 for hot reload
3. **Development Static Mode**: BODHI_DEV_PROXY_UI=false serves embedded assets in development for production build testing
4. **Fallback Handling**: Graceful degradation when static_router is None, maintaining API functionality without UI

## Important Constraints

### Route Composition Requirements
- All routes must use RouterState dependency injection for consistent service access across route boundaries
- Route-specific middleware must be applied in correct order for proper authentication and authorization flow
- Error handling must provide consistent error responses across OpenAI and application endpoints
- CORS configuration must support web client integration while maintaining security boundaries

### Authentication and Authorization Standards
- Bearer token authentication required for OpenAI/Ollama API endpoints with role and scope validation
- Session-based authentication required for application management endpoints with secure cookie configuration
- Role hierarchy must be enforced consistently (Admin > Manager > PowerUser > User) across all protected endpoints
- Scope-based authorization must support both TokenScope and UserScope with proper precedence rules

### Middleware Integration Rules
- Authentication middleware must be applied after session management for proper session injection
- Canonical URL middleware must be applied early in the stack for proper request processing
- Tracing middleware must be configured with appropriate log levels for production performance
- Session layer must use secure cookie configuration with SameSite and secure flags

### UI Serving Coordination Requirements
- Production builds must serve embedded static assets with proper caching headers
- Development mode must support proxy configuration for hot reload development workflows
- Environment-specific configuration must be validated at startup to prevent runtime errors
- Fallback handling must provide graceful degradation when UI assets are unavailable

## Route Composition Extension Patterns

### Adding New Route Groups
When integrating new route implementations:

1. **Route Integration Design**: Import route handlers and endpoints from new route crates with proper dependency management
2. **Authentication Layer Selection**: Choose appropriate authentication middleware (api_auth_middleware vs auth_middleware) based on endpoint requirements
3. **Authorization Configuration**: Configure role and scope requirements using Role and TokenScope/UserScope enums from objs crate
4. **Middleware Ordering**: Apply middleware in correct order (authentication → authorization → route-specific) for proper request processing
5. **OpenAPI Integration**: Merge OpenAPI documentation from new route crates with environment-specific configuration

### Extending Authentication Patterns
For new authentication and authorization requirements:

1. **Role Hierarchy Integration**: Ensure new authorization logic follows established role hierarchy with has_access_to() validation
2. **Scope Extension**: Extend TokenScope and UserScope enums in objs crate for new authorization contexts
3. **Middleware Composition**: Layer authentication middleware appropriately for different endpoint security requirements
4. **Session Integration**: Coordinate with Tower Sessions for new session-based authentication patterns
5. **Error Handling**: Provide consistent authentication error responses across all route boundaries

### UI Serving Extensions
For new UI serving capabilities and development workflows:

1. **Environment Configuration**: Extend SettingService configuration for new UI serving modes and proxy configurations
2. **Static Asset Integration**: Configure embedded asset serving for new UI components with proper caching strategies
3. **Proxy Configuration**: Support new development proxy patterns for different frontend frameworks and build tools
4. **Fallback Handling**: Implement graceful degradation for new UI serving scenarios with appropriate error responses
5. **Performance Optimization**: Optimize static asset serving and proxy performance for production deployments

## Route Composition Testing Architecture

### Multi-Route Integration Testing
Route composition requires comprehensive testing across route boundaries:
- **Router Configuration Testing**: Validate complete router composition with all middleware layers and route integration
- **Authentication Flow Testing**: Test authentication and authorization across different route types with role and scope validation
- **UI Serving Testing**: Validate environment-specific UI serving with proxy and static asset configurations
- **Middleware Interaction Testing**: Test middleware ordering and interaction across route boundaries with error handling validation
- **Cross-Route Error Handling**: Validate consistent error responses across OpenAI and application endpoints

### Route Composition Mock Coordination
Testing route composition with service mock coordination:
- **RouterState Mocking**: Mock RouterState with comprehensive AppService and SharedContext mocking for isolated testing
- **Authentication Middleware Testing**: Test authentication middleware with different role and scope combinations
- **Service Integration Testing**: Validate service coordination across route boundaries with realistic service interactions
- **UI Configuration Testing**: Test UI serving configuration with different environment settings and proxy configurations
- **Error Scenario Testing**: Test error propagation and handling across route composition boundaries