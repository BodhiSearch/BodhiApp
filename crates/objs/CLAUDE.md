# CLAUDE.md

This file provides guidance to Claude Code when working with the `objs` crate.

*For detailed implementation examples and technical depth, see [crates/objs/PACKAGE.md](crates/objs/PACKAGE.md)*

## Purpose

The `objs` crate serves as BodhiApp's **universal foundation layer**, providing domain objects, centralized error handling with localization, and shared types that enable consistent behavior across the entire application ecosystem including services, routes, CLI, and desktop components.

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
- **Alias**: YAML configuration system linking user-friendly names to specific model snapshots
- **RemoteModel**: Downloadable model specifications coordinated between services and routes

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