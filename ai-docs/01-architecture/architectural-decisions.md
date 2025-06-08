# Architectural Decisions

This document captures key architectural decisions, their rationale, and the benefits they provide to the Bodhi App system.

## Required Documentation References

**MUST READ for context:**
- `ai-docs/01-architecture/system-overview.md` - System architecture overview
- `ai-docs/01-architecture/tauri-desktop.md` - Desktop application architecture

## Key Architectural Decisions

### 1. Embedded Web Server in Tauri

**Decision**: Use an embedded web server instead of Tauri's native IPC for desktop application communication.

**Rationale**:
- **API Compatibility**: Maintains full compatibility with OpenAI and Ollama client libraries
- **Standard Debugging**: Enables use of standard web debugging tools (browser dev tools, Postman, curl)
- **Unified Codebase**: Single API implementation serves both web and desktop clients
- **Third-party Integration**: Existing tools and libraries work without modification
- **Development Experience**: Familiar HTTP/REST patterns for all developers

**Benefits**:
- Reduced complexity in maintaining separate communication protocols
- Better testability with standard HTTP testing tools
- Easier integration with external monitoring and logging tools
- Consistent behavior across deployment modes

### 2. Configuration Loading Priority

**Decision**: Implement a hierarchical configuration system with clear precedence rules.

**Priority Order** (highest to lowest):
1. **Command line arguments** - Deployment and runtime overrides
2. **Environment variables** - Container and deployment configuration
3. **Configuration files** - Persistent local settings
4. **Database settings** - User-specific preferences
5. **Default values** - Fallback values

**Rationale**:
- **Operational Flexibility**: Allows runtime configuration without code changes
- **Environment Isolation**: Different settings for development, staging, production
- **User Customization**: Persistent user preferences stored in database
- **Deployment Safety**: Command-line overrides for emergency configuration changes

**Benefits**:
- Clear configuration precedence eliminates ambiguity
- Supports both automated deployment and manual configuration
- Enables per-user customization without affecting system defaults

### 3. Server-Sent Events Over WebSockets

**Decision**: Use Server-Sent Events (SSE) for real-time chat streaming instead of WebSockets.

**Current Implementation**:
- SSE for unidirectional streaming (server to client)
- HTTP requests for client to server communication
- WebSocket support planned for future bidirectional features

**Rationale**:
- **Simplicity**: SSE is simpler to implement and debug than WebSockets
- **HTTP Compatibility**: Works with standard HTTP infrastructure (proxies, load balancers)
- **OpenAI Compatibility**: Matches OpenAI's streaming API implementation
- **Browser Support**: Native browser support without additional libraries

**Future WebSocket Use Cases**:
- Real-time notifications
- Collaborative features
- System status updates
- Bidirectional communication needs

### 4. Multi-Crate Architecture

**Decision**: Organize backend code into focused, single-responsibility crates.

**Crate Organization**:
- **Domain Separation**: `objs` for types, `services` for business logic
- **API Separation**: `routes_oai` vs `routes_app` for different API contracts
- **Infrastructure Separation**: `auth_middleware`, `server_core` for cross-cutting concerns
- **Deployment Separation**: `server_app` vs `bodhi/src-tauri` for different deployment modes

**Rationale**:
- **Modularity**: Clear boundaries between different system concerns
- **Testability**: Each crate can be tested in isolation
- **Reusability**: Crates can be composed differently for different deployment modes
- **Team Scalability**: Different teams can work on different crates

**Benefits**:
- Faster compilation times (only changed crates rebuild)
- Clear dependency management
- Easier to reason about system boundaries
- Supports microservice decomposition in the future

### 5. Type-Safe Error Handling

**Decision**: Use `errmeta_derive` for centralized error metadata and type-safe error handling.

**Implementation**:
- Error types with embedded metadata (HTTP status, error codes, messages)
- Automatic conversion between error types
- Localization support for error messages
- Structured error responses for APIs

**Rationale**:
- **Consistency**: Uniform error handling across all system components
- **Type Safety**: Compile-time verification of error handling
- **Debugging**: Rich error context for troubleshooting
- **API Quality**: Consistent error responses for client applications

**Benefits**:
- Reduced boilerplate for error handling
- Better error messages for users and developers
- Easier maintenance of error handling logic

## Design Patterns

### Dependency Injection Pattern

**Implementation**: Services injected into route handlers via Axum extensions.

**Benefits**:
- **Testability**: Easy to mock services for unit testing
- **Flexibility**: Different service implementations for different environments
- **Separation of Concerns**: Route handlers focus on HTTP concerns, services handle business logic

### Repository Pattern

**Implementation**: Database operations abstracted behind service traits.

**Benefits**:
- **Database Independence**: Can switch database implementations
- **Testing**: Mock database operations for unit tests
- **Business Logic Isolation**: Domain logic separated from data access

### Builder Pattern

**Implementation**: Used for complex object construction (models, configurations).

**Benefits**:
- **Flexibility**: Optional parameters and default values
- **Readability**: Clear, self-documenting object construction
- **Validation**: Construction-time validation of required fields

## Technology Choices

### SQLite for Local Storage

**Decision**: Use SQLite as the primary database for local data storage.

**Rationale**:
- **Zero Configuration**: No separate database server required
- **Portability**: Single file database, easy to backup and move
- **Performance**: Excellent performance for local applications
- **ACID Compliance**: Full transaction support for data integrity

**Trade-offs**:
- Limited concurrent write performance (acceptable for single-user desktop app)
- No built-in replication (not needed for local-first architecture)

### React Query for State Management

**Decision**: Use React Query for server state management instead of Redux or Zustand.

**Rationale**:
- **Server State Focus**: Designed specifically for server state management
- **Caching**: Intelligent caching and background updates
- **Developer Experience**: Excellent debugging tools and error handling
- **Performance**: Automatic request deduplication and background refetching

**Benefits**:
- Reduced boilerplate compared to Redux
- Better handling of loading and error states
- Automatic cache invalidation and updates

### Axum Web Framework

**Decision**: Use Axum instead of other Rust web frameworks (Actix, Warp, Rocket).

**Rationale**:
- **Performance**: Built on Tokio for excellent async performance
- **Type Safety**: Compile-time request/response validation
- **Ecosystem**: Good integration with other Rust libraries
- **Maintainability**: Clear, composable API design

**Benefits**:
- Excellent performance characteristics
- Strong type safety prevents runtime errors
- Good documentation and community support

## Related Documentation

- **[System Overview](system-overview.md)** - High-level system architecture
- **[Tauri Desktop](tauri-desktop.md)** - Desktop application implementation
- **[Rust Backend](rust-backend.md)** - Backend implementation patterns
- **[Development Conventions](development-conventions.md)** - Coding standards and practices

---

*These architectural decisions provide the foundation for a maintainable, scalable, and user-friendly application. They should be revisited as the system evolves and new requirements emerge.*
