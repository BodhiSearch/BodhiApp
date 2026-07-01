# Architectural Decisions

This document captures key architectural decisions, their rationale, and the benefits they provide to the Bodhi App system.

## Required Documentation References

**MUST READ for context:**
- [`system-overview.md`](system-overview.md) - System architecture overview
- [`tauri-desktop.md`](tauri-desktop.md) - Desktop application architecture

## Key Architectural Decisions

### 1. Embedded Web Server in Tauri

**Decision**: Use an embedded web server instead of Tauri's native IPC for desktop application communication.

**Rationale**:
- **API Compatibility**: Maintains full compatibility with OpenAI client libraries
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
- **Domain + Logic**: `services` is the single hub for all domain types and business logic (the former `objs` crate was merged into `services`)
- **HTTP Infrastructure**: `server_core` for SharedContext, SSE streaming, InferenceService
- **API + Middleware**: `routes_app` hosts all HTTP endpoints (OpenAI/Anthropic/app) plus auth middleware
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

### 6. Session Cookie Same-Origin Enforcement

**Decision**: Accept session cookies **only** for requests that explicitly declare a `Sec-Fetch-Site` header value of `same-origin`, and set the session cookie itself to `SameSite::Strict`. Any cross-site request (including those originating from browser extensions) will therefore be denied access to the user's authenticated session.

**Rationale**:
- **CSRF Hardening**: The combination of the `SameSite::Strict` cookie attribute and runtime verification of `Sec-Fetch-Site` eliminates cross-site request-forgery vectors that might otherwise bypass relaxed SameSite implementations.
- **Extension Isolation**: Prevents Chrome/Firefox extensions from silently reusing the authenticated browser session when performing background XHR/fetch requests to the Bodhi backend.
- **Defense in Depth**: Adds a server-side guard in addition to client-side cookie attributes, ensuring older or non-compliant user agents cannot abuse cookies.
- **Standards-Based**: Relies on Fetch Metadata Request Headers, a widely-supported web standard designed specifically for this purpose.

**Implementation**:
- Middleware helper `is_same_site` in `crates/routes_app/src/middleware/auth/auth_middleware.rs` checks the request headers and short-circuits with `401 Unauthorized` when the header is missing or not `same-origin`.
- Session cookie builder uses `SameSite::Strict` in `crates/services/src/auth/session_service.rs`.
- Unit tests in `crates/routes_app/src/middleware/auth/` and live tests in `crates/server_app` cover both positive (same-origin) and negative (cross-site) scenarios to guarantee enforcement.

**Benefits**:
- Eliminates remaining CSRF attack surface for authenticated endpoints.
- Protects user data from malicious third-party sites and untrusted browser extensions.
- Aligns with the project's zero-trust security posture and "dumb frontend" design, ensuring that only the official Bodhi UI can leverage the session.

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

### SeaORM over SQLite and PostgreSQL

**Decision**: Use SeaORM as the data-access layer, backed by SQLite for dev/desktop and PostgreSQL for production/Docker (multi-tenant).

**Rationale**:
- **Zero Configuration (SQLite)**: No separate database server for desktop/local use; single portable file
- **Multi-Tenant Isolation (PostgreSQL)**: Row-Level Security via `SET LOCAL app.current_tenant_id` for production
- **Single Codebase**: Same SeaORM entities and migrations target both backends
- **ACID Compliance**: Full transaction support for data integrity

**Trade-offs**:
- SQLite has limited concurrent write performance (acceptable for single-user desktop)
- PostgreSQL required for the multi-tenant web deployment

### TanStack Query (v5) for State Management

**Decision**: Use TanStack Query v5 (formerly React Query) for server state management instead of Redux or Zustand.

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
- **Backend implementation patterns** - [`crates/services/CLAUDE.md`](../../crates/services/CLAUDE.md)
- **Coding standards and cross-crate patterns** - [`crates/CLAUDE.md`](../../crates/CLAUDE.md)

---

*These architectural decisions provide the foundation for a maintainable, scalable, and user-friendly application. They should be revisited as the system evolves and new requirements emerge.*
