# CLAUDE.md

This file provides guidance to Claude Code when working with the `server_core` crate.

*For detailed implementation examples and technical depth, see [crates/server_core/PACKAGE.md](crates/server_core/PACKAGE.md)*

## Purpose

The `server_core` crate serves as BodhiApp's **HTTP infrastructure orchestration layer**, providing sophisticated server-sent event streaming, LLM server context management, and HTTP route coordination with comprehensive async operations and error handling.

## Key Domain Architecture

### HTTP Infrastructure Orchestration System
Advanced HTTP server infrastructure with streaming and context coordination:
- **RouterState Management**: Centralized dependency injection container for HTTP route handlers
- **Server-Sent Events Architecture**: Dual SSE implementation (direct + forwarded) for real-time chat streaming  
- **SharedContext Coordination**: LLM server instance lifecycle management with state synchronization
- **Request Orchestration**: Intelligent request routing and proxy forwarding to llama.cpp servers
- **Async Stream Processing**: High-performance streaming with connection management and error recovery

### Cross-Service HTTP Coordination Architecture  
Sophisticated HTTP infrastructure coordinating across BodhiApp's service layer:
- **Services ↔ RouterState**: Dependency injection providing `AppService` registry access to HTTP handlers
- **SharedContext ↔ LlamaServerProc**: LLM server lifecycle management with process coordination
- **RouterState ↔ Routes**: State management for HTTP route handlers with error translation
- **SSE ↔ Streaming**: Real-time event streaming coordination with connection management
- **Context ↔ Services**: LLM server state synchronization with service layer operations

### Server-Sent Events Streaming Architecture
Dual SSE implementation for different streaming scenarios:
- **DirectSSE**: Application-generated event streaming with custom event formatting and keep-alive support
- **RawSSE/ForwardedSSE**: Proxy streaming from external LLM services with efficient request forwarding
- **Stream Management**: Connection lifecycle management with automatic cleanup and error recovery
- **Event Formatting**: BytesMut-based event construction with optimized memory usage and DirectEvent builder pattern
- **Axum Integration**: Native Axum response streaming with HTTP header management and proper MIME type handling

### LLM Server Context Management
Advanced context coordination for LLM server instances:
- **SharedContext Trait**: Interface for LLM server lifecycle operations (start/stop/reload/set_exec_variant)
- **Server State Management**: State synchronization across HTTP requests with RwLock-based async coordination
- **Request Routing**: Intelligent routing with ModelLoadStrategy (Continue/DropAndLoad/Load) for efficient model switching
- **Resource Lifecycle**: Proper startup/shutdown coordination with cleanup and error handling via ServerFactory pattern
- **State Listeners**: Observer pattern for server state change notifications with async notification broadcasting
- **Execution Variant Management**: Dynamic server variant switching with ExecVariant coordination

### Model Request Routing Infrastructure
Sophisticated model request routing system for local vs remote API coordination:
- **ModelRouter Trait**: Interface for intelligent model request routing with RouteDestination determination
- **Local vs Remote Routing**: Automatic detection and routing of User/Model aliases to SharedContext vs API aliases to AiApiService
- **Precedence Resolution**: User aliases override Model aliases, Model aliases override API models for consistent request handling
- **Response Format Conversion**: Seamless conversion between axum::Response and reqwest::Response for API compatibility
- **Error Handling**: Comprehensive error handling with ModelRouterError for routing failures and not found scenarios

### Server Configuration Merging System
Advanced LLM server argument merging with hierarchical precedence:
- **Three-Tier Precedence**: Setting-level, variant-level, and alias-level argument coordination with HashMap deduplication
- **Advanced Flag Parsing**: Sophisticated argument parsing with negative number detection and complex value handling
- **LLM-Specific Patterns**: Support for llama.cpp argument patterns including logit-bias, override-kv, and lora-scaled configurations
- **Cross-String Parsing**: Robust parsing across configuration string boundaries for flexible argument specification
- **Configuration Flexibility**: Dynamic server configuration with runtime argument override capabilities

## Architecture Position

The `server_core` crate serves as BodhiApp's **HTTP infrastructure orchestration layer**:

- **Above services and objs**: Coordinates service layer operations and domain objects for HTTP operations
- **Below route implementations**: Provides HTTP infrastructure foundation for routes_oai and routes_app
- **Parallel to commands**: Similar orchestration role but optimized for HTTP/streaming instead of CLI
- **Integration with llama_server_proc**: Manages LLM server process lifecycle and request coordination
- **Embedded Deployment Support**: HTTP infrastructure designed for embedding in various application contexts including desktop applications, library integrations, and containerized deployments

## Cross-Crate Integration Patterns

### Service Layer HTTP Coordination
HTTP infrastructure orchestrates complex service interactions:
- **AppService Registry Integration**: RouterState provides HTTP handlers access to all business services through dependency injection including auth_middleware
- **DataService Coordination**: Model alias resolution via `find_alias()` for HTTP request processing with error translation and authentication context
- **HubService Integration**: Local model file discovery via `find_local_file()` coordinated through HTTP context for chat completions with authorization
- **AuthService HTTP Integration**: Authentication middleware coordination with HTTP state management through auth_middleware layer
- **Authentication Header Management**: RouterState coordinates with auth_middleware for X-Resource-Token, X-Resource-Role, and X-Resource-Scope injection
- **Error Service Coordination**: Service errors translated to appropriate HTTP status codes with localization via RouterStateError and auth_middleware integration
- **OpenAI API Integration**: RouterState coordinates with routes_oai for OpenAI-compatible API endpoints with streaming support and error translation
- **Application API Integration**: RouterState coordinates with routes_app for model management, authentication, and configuration endpoints
- **API Response Streaming**: HTTP infrastructure supports both OpenAI and Ollama streaming formats through routes_oai coordination
- **Route Composition Integration**: HTTP infrastructure coordinated through routes_all for unified route composition with comprehensive middleware orchestration and UI serving

### LLM Process Integration Architecture
Sophisticated coordination with llama.cpp server processes:
- **LlamaServerProc Management**: SharedContext coordinates LLM server lifecycle with HTTP request handling via ServerFactory abstraction
- **Process State Synchronization**: HTTP context manages LLM server state across concurrent requests with RwLock coordination
- **Request Routing**: Intelligent routing with ModelLoadStrategy for efficient model switching and resource management
- **Stream Coordination**: SSE streaming coordinated with LLM server response streaming through reqwest::Response proxying
- **Resource Management**: Proper HTTP resource cleanup coordinated with LLM process management and server args merging

### HTTP Streaming Integration
Advanced streaming coordination across BodhiApp's architecture:

- **DirectSSE ↔ Routes**: Application-generated events streamed through HTTP responses
- **ForwardedSSE ↔ LLM**: LLM server responses proxied through HTTP streaming infrastructure
- **Connection Management**: HTTP connection lifecycle coordinated with service operations
- **Error Propagation**: Streaming errors properly handled and translated for HTTP responses

### Embedded Deployment Integration
HTTP infrastructure coordination for embedded application contexts:

- **Application Lifecycle Coordination**: HTTP server infrastructure integrates with external application lifecycle management including desktop applications and embedded library contexts
- **Resource Sharing**: SharedContext and RouterState designed for safe sharing across embedded application boundaries with proper resource isolation and cleanup
- **Deployment Context Adaptation**: HTTP infrastructure adapts to different deployment contexts (standalone server, embedded desktop, library integration) while maintaining consistent API behavior
- **External Integration Points**: HTTP infrastructure provides clean integration points for external applications including server handle management and graceful shutdown coordination

## HTTP Infrastructure Orchestration Workflows

### Multi-Service HTTP Request Coordination
Complex HTTP request processing with service orchestration:

1. **Request Reception**: HTTP routes receive requests with RouterState dependency injection
2. **Service Access**: RouterState provides access to AppService registry for business logic
3. **Alias Resolution**: DataService resolves model aliases for chat completion requests
4. **Context Coordination**: SharedContext manages LLM server instances for request processing
5. **Response Streaming**: SSE architecture handles real-time streaming responses
6. **Error Translation**: Service errors converted to appropriate HTTP status codes with localization

### Server-Sent Events Streaming Orchestration
Sophisticated streaming coordination for real-time communication:

**DirectSSE Workflow**:
1. **Event Generation**: Application generates events for streaming to clients
2. **Stream Creation**: DirectEvent formatting with BytesMut optimization
3. **Connection Management**: Keep-alive and connection lifecycle management
4. **Response Integration**: Axum HTTP response streaming with proper headers

**ForwardedSSE Workflow**:
1. **Request Proxying**: HTTP requests forwarded to LLM server instances
2. **Stream Proxying**: LLM server response streams forwarded to clients
3. **Error Handling**: Stream interruption and error recovery coordination
4. **Connection Cleanup**: Proper resource cleanup on client disconnect

### LLM Server Context Orchestration
Advanced LLM server lifecycle management:
1. **Context Initialization**: SharedContext manages LLM server startup and configuration
2. **State Synchronization**: Server state coordinated across concurrent HTTP requests
3. **Request Routing**: Intelligent routing of requests to appropriate LLM instances
4. **Resource Management**: Proper shutdown and cleanup coordination
5. **State Notification**: Observer pattern for server state change notifications

## Important Constraints

### HTTP Infrastructure Requirements
- All HTTP operations must use RouterState dependency injection for consistent service access
- SSE streaming must properly handle connection lifecycle management and cleanup
- SharedContext operations must be thread-safe and support concurrent HTTP request processing
- Error handling must translate service errors to appropriate HTTP status codes with localization

### LLM Server Coordination Standards  
- SharedContext must manage LLM server lifecycle with proper startup/shutdown coordination
- Request routing must intelligently handle LLM server instance selection and load balancing
- Server state must be synchronized across concurrent HTTP requests with async coordination
- Context operations must support observer pattern for state change notifications

### Streaming Infrastructure Rules
- DirectSSE must use BytesMut for efficient event formatting and memory management
- ForwardedSSE must properly proxy LLM server streams with error handling and recovery
- Connection management must handle client disconnects with automatic resource cleanup
- Keep-alive mechanisms must maintain connection stability for long-lived streaming operations

## HTTP Infrastructure Extension Patterns

### Adding New HTTP Streaming Capabilities
When creating new streaming functionality:

1. **SSE Type Selection**: Choose DirectSSE for application events or ForwardedSSE for service proxying
2. **Connection Lifecycle**: Design proper connection management with automatic cleanup
3. **Error Recovery**: Implement comprehensive error handling with graceful degradation
4. **Performance Optimization**: Use efficient memory management and streaming patterns
5. **Integration Testing**: Test streaming with realistic network conditions and client behavior

### SharedContext Extensions  
For new LLM server context management:

1. **Lifecycle Management**: Implement proper async startup/shutdown coordination
2. **State Synchronization**: Design thread-safe state management for concurrent operations
3. **Resource Management**: Ensure proper cleanup and resource lifecycle management
4. **Observer Integration**: Support state change notifications with observer pattern
5. **Error Coordination**: Provide comprehensive error handling with context preservation

### RouterState Integration Patterns
For new HTTP infrastructure coordination:

1. **Service Integration**: Coordinate with AppService registry for business logic access
2. **Error Translation**: Convert service errors to appropriate HTTP responses with localization
3. **Dependency Injection**: Provide proper service access patterns for HTTP handlers
4. **Request Processing**: Design efficient request processing with service coordination
5. **Testing Infrastructure**: Support comprehensive HTTP infrastructure testing with service mocking