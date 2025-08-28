# CLAUDE.md

This file provides guidance to Claude Code when working with the `services` crate.

*For detailed implementation examples and technical depth, see [crates/services/PACKAGE.md](crates/services/PACKAGE.md)*

## Purpose

The `services` crate implements BodhiApp's business logic layer through a sophisticated service architecture that coordinates OAuth2 authentication, model management, data persistence, and multi-layer security.

## Key Domain Architecture

### Service Registry Pattern
BodhiApp uses a sophisticated trait-based service registry with comprehensive dependency injection:
- **AppService trait**: Central registry providing access to all 10 business services including localization and time services
- **DefaultAppService**: Concrete implementation managing service composition with derive_new pattern
- **Arc<dyn Trait> pattern**: Thread-safe shared ownership across async contexts with mockall integration
- **Service interdependencies**: Complex coordination between authentication, data, hub, database, session, secret, cache, and time services

### Authentication Coordination System
Multi-stage authentication flow with comprehensive service coordination:
- **OAuth2 Client Registration**: Dynamic app registration with Keycloak identity provider using custom Bodhi API endpoints
- **PKCE Authorization Flow**: Authorization code exchange with PKCE security and code verifier validation
- **Token Management**: JWT access/refresh token lifecycle with automatic refresh and expiration handling
- **Token Exchange Protocol**: RFC 8693 token exchange for service-to-service authentication with scope validation
- **Session Integration**: SQLite-backed HTTP session management coordinated with JWT tokens and secure cookie configuration
- **Resource Administration**: Dynamic resource admin assignment and access request management

### Model Management Pipeline
Integrated model discovery, download, and local management with sophisticated error handling:
- **Hub Service Integration**: Hugging Face Hub API with gated repository handling, authentication token management, and comprehensive error categorization
- **Local Storage Coordination**: File system organization with metadata tracking, atomic file operations, and GGUF validation
- **Alias System**: YAML-based model aliases linking friendly names to specific snapshots with filename sanitization
- **Remote Model Registry**: Centralized model metadata with version synchronization and cache invalidation
- **Error Recovery**: Network failure handling with retry logic, partial download recovery, and detailed error categorization (gated access, not found, transport errors)
- **Offline Testing**: OfflineHubService for testing without external dependencies using local test data

### Multi-Layer Security Architecture
Coordinated security services for comprehensive data protection with platform-specific implementations:
- **Secret Service**: AES-GCM encryption with PBKDF2 key derivation (1000 iterations), random salt/nonce generation, and Base64 encoding
- **Keyring Service**: Platform-specific secure credential storage integration (Apple Keychain, Linux Secret Service, Windows Credential Manager)
- **Session Security**: SQLite-backed HTTP sessions with SameSite::Strict cookies, secure configuration, and session isolation
- **Database Security**: Parameterized queries with sqlx, transaction isolation, and migration management
- **Transport Security**: OAuth2 PKCE with code verifier validation, JWT token validation, and comprehensive error handling
- **Credential Flow**: End-to-end credential encryption, storage, and retrieval with platform keyring integration

### Database Transaction System
SQLite-based persistence with versioned schema evolution and event broadcasting:
- **Migration Management**: Versioned database migrations with sqlx migrate support and rollback capabilities
- **Connection Pooling**: SQLite connection pool management for concurrent access with proper lifecycle management
- **Transaction Coordination**: Cross-service transaction support for data consistency with atomic operations
- **Time Service Abstraction**: Testable time operations with frozen time for testing and nanosecond precision removal
- **Request Tracking**: Download request management with status tracking and comprehensive lifecycle management
- **Event Broadcasting**: Database change notification system for reactive testing and service coordination

### Caching and Performance Layer
Performance optimization through strategic caching and resource management:
- **Mini-Moka Integration**: High-performance in-memory cache with TTL support and thread-safe concurrent access
- **Cache Coordination**: Service-level cache invalidation strategies coordinated with database changes
- **Async Operations**: Non-blocking I/O coordination across all services with proper error propagation
- **Resource Management**: Connection pooling and resource lifecycle management with cleanup strategies
- **Performance Testing**: Cache service stubs for testing cache invalidation and consistency scenarios

## Architecture Position

The `services` crate serves as the orchestration layer:

- **Above**: Domain objects (`objs`), infrastructure (`llama_server_proc`), and external APIs
- **Below**: HTTP routes, server middleware, CLI commands, and application entry points
- **Coordination**: Manages complex interactions between external systems and internal state
- **Abstraction**: Provides clean interfaces for business logic testing and composition across HTTP, CLI, and embedded application contexts
- **Deployment Flexibility**: Service architecture supports multiple deployment modes including standalone servers, embedded applications, and dual-mode desktop/server configurations

## Service Interdependencies

### HTTP Infrastructure Service Integration
Services coordinate with HTTP infrastructure for request processing:
- **DataService** ↔ **HTTP RouterState**: Model alias resolution via `find_alias()` for chat completion requests
- **HubService** ↔ **HTTP SharedContext**: Local model file discovery via `find_local_file()` for LLM server coordination
- **SettingService** ↔ **HTTP SharedContext**: Server configuration and execution variant management for LLM server lifecycle
- **Service Error Translation**: All service errors flow through HTTP infrastructure with RouterStateError translation to appropriate HTTP status codes

### Authentication Flow Dependencies
Complex service coordination for complete authentication with comprehensive error handling:
- **AuthService** ↔ **DbService**: Token validation, storage, and API token management with transaction support and auth_middleware integration
- **AuthService** ↔ **SessionService**: Session creation, management, and SQLite-backed persistence coordinated through auth_middleware
- **AuthService** ↔ **SecretService**: Credential encryption, storage, and AppRegInfo management for JWT token validation in auth_middleware
- **AuthService** ↔ **SettingService**: OAuth client configuration and Keycloak endpoint management with auth_middleware token exchange
- **AuthService** ↔ **KeyringService**: Platform-specific credential storage for persistent authentication across HTTP sessions
- **AuthService** ↔ **auth_middleware**: OAuth2 token exchange, refresh operations, and external client token validation with comprehensive caching

### Model Management Dependencies
Coordinated model discovery, storage, and validation:
- **HubService** ↔ **DataService**: Downloaded model registration, alias creation, and GGUF validation
- **HubService** ↔ **CacheService**: Repository metadata caching with TTL and invalidation strategies
- **DataService** ↔ **SettingService**: Storage path configuration and Hugging Face cache management
- **DataService** ↔ **SecretService**: Repository access token management for gated models
- **DataService** ↔ **HubService**: Model file validation and metadata extraction coordination

### Database Transaction Boundaries
Cross-service transaction coordination with event broadcasting:
- **Download tracking**: Multiple services participate in download request lifecycle with status management
- **Access control**: Authentication state synchronized with database permissions and role management
- **Alias management**: File system operations coordinated with database metadata and atomic writes
- **Session management**: HTTP sessions synchronized with authentication state and secure cookie handling
- **Event coordination**: Database change notifications broadcast to reactive services for consistency

### Time Service Integration
Deterministic time operations across all services:

- **DbService** ↔ **TimeService**: Consistent timestamp generation for database records
- **AuthService** ↔ **TimeService**: Token expiration and refresh timing coordination
- **TestDbService** ↔ **FrozenTimeService**: Deterministic testing with frozen time for reproducible results

### Deployment Context Coordination
Service architecture supports multiple deployment contexts with consistent behavior:

- **Embedded Application Integration**: Services coordinate seamlessly across embedded contexts including desktop applications and library integrations
- **Dual-Mode Service Coordination**: Service registry pattern enables consistent service access across different deployment modes (desktop, server, CLI) without architectural changes
- **Context-Agnostic Service Design**: All services implement deployment-neutral interfaces enabling flexible application composition and embedding scenarios
- **Resource Management Coordination**: Services adapt resource management strategies based on deployment context while maintaining consistent business logic and data integrity

## Important Constraints

### Service Lifecycle Management
- All services must be thread-safe and implement Send + Sync + Debug for comprehensive debugging
- Services use Arc<dyn Trait> for shared ownership across async contexts with proper lifetime management
- Service registration happens at application startup with dependency injection via DefaultAppService::new
- Mock services available via `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]` for comprehensive testing
- Service composition testing requires careful mock expectation coordination and dependency ordering

### Authentication Security Requirements
- OAuth2 PKCE required for all authorization flows with code verifier validation
- JWT tokens must have expiration and refresh mechanisms with automatic refresh handling
- Session cookies configured with SameSite::Strict and secure settings (TODO: enable secure flag for HTTPS)
- All authentication state changes must be atomic across service boundaries with transaction support
- Keycloak integration requires custom Bodhi API endpoints for resource management and admin assignment
- Token exchange must follow RFC 8693 standards with proper scope and audience validation

### Data Consistency Requirements
- Database operations must use parameterized queries with sqlx to prevent SQL injection
- File system operations must be atomic with rollback capabilities using temporary files and rename operations
- Cache invalidation must be coordinated with database changes through service-level coordination
- Cross-service operations must maintain transactional consistency with proper error propagation
- Migration management must support both up and down migrations with data preservation
- Event broadcasting must maintain consistency across reactive service boundaries

### External Integration Constraints
- Hugging Face Hub API rate limiting and error handling required with comprehensive error categorization
- OAuth provider integration must handle various error conditions including gated access and invalid tokens
- Network operations must implement retry logic with exponential backoff for transient failures
- All external API calls must have configurable timeout handling with proper error propagation
- Keycloak integration requires specific realm configuration and custom Bodhi API endpoint support
- Platform keyring integration must handle different credential storage mechanisms across operating systems

### Security and Privacy Requirements
- Secrets must never be stored in plaintext or logged with comprehensive masking in HTTP request logging
- Encryption keys derived using PBKDF2 with 1000 iterations and random salts for each encryption operation
- Platform keyring integration required for persistent credentials with OS-specific implementations
- All sensitive operations must be auditable and traceable with proper error context preservation
- AES-GCM encryption with unique nonces per operation and Base64 encoding for storage
- Session security with SQLite backend and SameSite::Strict cookie configuration

## Error Handling Strategy

### Service-Specific Error Domains
Each service defines localized error types with comprehensive handling using errmeta_derive:
- **AuthServiceError**: OAuth flow failures, token validation, client registration errors, and Keycloak API errors
- **HubServiceError**: API failures, gated repository access, network errors with retry logic, and comprehensive error categorization
- **DataServiceError**: File system operations, alias conflicts, YAML parsing errors, and atomic operation failures
- **DbError**: SQL operations, migration failures, transaction rollbacks, and connection pool management
- **SecretServiceError**: Encryption/decryption failures, key derivation errors, and platform keyring integration issues
- **SessionServiceError**: HTTP session management, SQLite store operations, and cookie configuration errors

### Cross-Service Error Propagation
Complex error handling across service boundaries with comprehensive context preservation:
- Transparent error wrapping maintains original context using impl_error_from! macro patterns
- Service errors map to appropriate HTTP status codes via ErrorType enum from objs crate
- Localized error messages support multiple languages through objs LocalizationService integration
- Error recovery strategies coordinated across affected services with proper transaction rollback
- HTTP request/response logging with parameter masking for security-sensitive operations
- Comprehensive error categorization for external API failures (gated access, not found, transport errors)

