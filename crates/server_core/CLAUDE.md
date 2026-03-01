# CLAUDE.md - server_core Crate

See [crates/server_core/PACKAGE.md](crates/server_core/PACKAGE.md) for implementation details.

## Purpose

The `server_core` crate implements BodhiApp's **HTTP infrastructure orchestration layer**, providing server-sent event streaming, LLM server context management, model request routing, and HTTP state coordination. This crate bridges the service layer with HTTP route handlers while managing the complex lifecycle of LLM server processes and real-time streaming connections.

## Architecture Position

**Upstream dependencies** (crates this depends on):
- [`services`](../services/CLAUDE.md) -- domain types, `ApiError`, `AppError` trait, `ErrorType`, `AppService` trait, business service traits
- [`llama_server_proc`](../llama_server_proc/CLAUDE.md) -- LLM server process management

**Downstream consumers** (crates that depend on this):
- [`auth_middleware`](../auth_middleware/CLAUDE.md) -- uses `RouterState` for middleware integration
- [`routes_app`](../routes_app/CLAUDE.md) -- uses `RouterState`, SSE streaming, `SharedContext`
- [`server_app`](../server_app/CLAUDE.md) -- constructs `DefaultRouterState` and `DefaultSharedContext`
- [`lib_bodhiserver`](../lib_bodhiserver/CLAUDE.md) -- constructs and embeds server infrastructure

## Architectural Design Rationale

### Why RouterState Dependency Injection

BodhiApp uses RouterState as a centralized dependency injection container for HTTP handlers rather than global state or individual extractors because:

1. **Type-Safe Service Access**: RouterState provides compile-time guaranteed access to all services through the AppService registry
2. **Request Context Isolation**: Each request handler receives a consistent view of services without global state pollution
3. **Testing Flexibility**: RouterState can be easily mocked with test services for comprehensive HTTP handler testing
4. **Middleware Integration**: Authentication and authorization middleware can inject context into RouterState for downstream handlers
5. **Performance Optimization**: Arc-based sharing eliminates service instantiation overhead per request

### Why Dual SSE Implementation

The crate provides two distinct SSE implementations (DirectSSE and ForwardedSSE) because:

1. **Different Event Sources**: DirectSSE handles application-generated events while ForwardedSSE proxies external service streams
2. **Memory Optimization**: DirectSSE uses BytesMut for efficient event construction while ForwardedSSE streams raw bytes
3. **Error Handling Boundaries**: Application events have different error semantics than proxied LLM server streams
4. **Connection Management**: Keep-alive strategies differ between self-generated and proxied event streams
5. **Format Flexibility**: DirectSSE supports custom event formatting while ForwardedSSE preserves original stream format

### Why SharedContext for LLM Server Management

SharedContext provides a sophisticated abstraction for LLM server lifecycle management because:

1. **Process Isolation**: Each LLM server runs as a separate process requiring careful lifecycle coordination
2. **State Synchronization**: Multiple concurrent HTTP requests need consistent views of server state
3. **Resource Management**: LLM servers consume significant resources requiring proper cleanup on shutdown
4. **Hot Reloading**: Model switching without service interruption requires sophisticated state management
5. **Observer Pattern**: Multiple components need notifications of server state changes for coordination

## Cross-Crate Coordination Patterns

### HTTP to Service Layer Bridge

The server_core crate orchestrates complex interactions between HTTP handlers and business services:

**Request Processing Pipeline**:
```rust
// Conceptual flow through server_core
HTTP Request → RouterState → AppService Registry → Business Service
                    ↓                                      ↓
              SharedContext → LLM Server Process → Response Stream
                    ↓                                      ↓
               DirectSSE/ForwardedSSE → HTTP Response → Client
```

**Service Access Pattern**:
- RouterState provides `app_service()` method for handler access to services
- Services accessed through Arc<dyn Trait> for thread-safe sharing
- Error translation through RouterStateError for consistent HTTP responses
- Authentication context injected via auth_middleware integration

### LLM Server Coordination Flow

SharedContext coordinates LLM server processes with HTTP infrastructure:

**Model Loading Strategy**:
1. Request arrives with model identifier
2. ModelRouter determines if model is local or remote API
3. For local models, SharedContext checks current loaded model
4. ModelLoadStrategy determines action (Continue/DropAndLoad/Load)
5. Server args merged from setting/variant/alias levels
6. LLM server started/restarted as needed
7. Request forwarded to appropriate server instance

**State Management Flow**:
```rust
// State synchronization across requests
SharedContext {
  state: RwLock<ServerState>,
  listeners: Vec<StateListener>,
  server_factory: Arc<dyn ServerFactory>
}
```

### Streaming Infrastructure Coordination

SSE implementations coordinate with different parts of the architecture:

**DirectSSE Integration**:
- Routes generate application events (progress, status updates)
- DirectEvent builder creates formatted SSE messages
- BytesMut optimization for memory-efficient streaming
- Axum response integration with proper headers

**ForwardedSSE Integration**:
- LLM server responses proxied through HTTP infrastructure
- reqwest::Response converted to axum::Response
- Stream interruption handling with automatic recovery
- Connection cleanup coordinated with LLM server lifecycle

## Domain-Specific Architecture Patterns

### Model Request Routing Architecture

The ModelRouter trait enables intelligent routing between local and remote models:

**Routing Decision Tree**:
```
Request with model_id
    ├→ Check UserAlias (highest priority)
    │   └→ Route to SharedContext (local)
    ├→ Check ModelAlias (medium priority)
    │   └→ Route to SharedContext (local)
    ├→ Check ApiAlias (low priority)
    │   └→ Route to AiApiService (remote)
    └→ Not Found Error
```

**Why This Hierarchy**:
- User aliases allow custom model configurations
- Model aliases provide auto-discovered local models
- API aliases enable remote model access
- Precedence ensures local models preferred over remote

### Server Arguments Merging Strategy

Complex argument merging supports flexible LLM server configuration:

**Three-Tier Precedence**:
1. **Setting Level**: Global server arguments from configuration
2. **Variant Level**: Execution variant specific arguments (CPU/CUDA/etc)
3. **Alias Level**: Model-specific parameter overrides

**Merging Algorithm**:
```rust
// Conceptual merging process
let mut args = HashMap::new();
args.extend(setting_args);      // Base configuration
args.extend(variant_args);       // Variant overrides
args.extend(alias_args);         // Model overrides
deduplicate_and_order(args)      // Final arguments
```

**Complex Argument Patterns**:
- Negative numbers: `--temp -0.5`
- Key-value pairs: `--override-kv tokenizer.ggml.add_bos_token=bool:false`
- Scaled values: `--lora-scaled model.gguf 0.5`
- JSON arrays: `--logit-bias 29871-2`

### Connection Lifecycle Management

SSE connections require sophisticated lifecycle management:

**Keep-Alive Strategy**:
- Periodic comment events prevent proxy timeouts
- Configurable keep-alive intervals (default 30s)
- Automatic cleanup on client disconnect
- Resource tracking for connection limits

**Error Recovery Pattern**:
```rust
// Stream error handling
match stream_result {
  Ok(chunk) => forward_chunk(),
  Err(Timeout) => send_keep_alive(),
  Err(Disconnect) => cleanup_resources(),
  Err(ServerError) => send_error_event()
}
```

## Critical Design Decisions

### Why RwLock for SharedContext State

SharedContext uses async RwLock for state management rather than Mutex:

**Rationale**:
- Multiple readers (status checks) with occasional writers (state changes)
- Read operations don't block other reads improving concurrency
- Write operations ensure exclusive access during state transitions
- Async lock prevents blocking runtime threads

**Trade-offs**:
- More complex than Mutex but better read concurrency
- Potential writer starvation under heavy read load
- Deadlock prevention requires careful lock ordering

### Server Factory Abstraction

LLM server creation abstracted behind ServerFactory trait:

**Benefits**:
- Testing with mock servers without real processes
- Different server implementations (llama.cpp, alternatives)
- Process management strategies (direct, containerized)
- Resource limit enforcement per deployment context

**Implementation**:
```rust
#[async_trait]
pub trait ServerFactory: Send + Sync {
  async fn create(&self, args: Vec<String>) -> Result<Box<dyn LlamaServerInterface>>;
}
```

### Error Translation Architecture

Service errors are translated to HTTP responses through a single `ApiError` response path:

**Translation Layers**:
1. Service returns domain error (e.g., `HubServiceError`, `DataServiceError`)
2. `ContextError` aggregates service-level errors with transparent delegation via `#[error(transparent)]`
3. `ApiError` (from the `services` crate) converts domain errors directly to OpenAI-compatible HTTP responses
4. Error type determines HTTP status code via the `AppError` trait's `error_type()` method
5. User-friendly message extracted via `error.to_string()` for response body

**Why a Single ApiError Path**:
- Eliminates unnecessary generic wrapper types -- `ApiError` is the sole conversion point from domain errors to HTTP responses
- The `AppError` trait (implemented via `errmeta_derive::ErrorMeta`) provides `error_type()` and `code()` for consistent categorization
- `ContextError` uses `#[error(transparent)]` to delegate display and error metadata to the underlying service errors
- Direct conversion avoids intermediate wrapper allocation and simplifies the error flow
- API compatibility with OpenAI clients is maintained at the `ApiError` level, not in server_core

## Security Architecture Decisions

### Authentication Context Propagation

RouterState integrates with auth_middleware for security:

**Context Flow**:
1. auth_middleware validates request credentials
2. User/role/scope information attached to request extensions
3. RouterState provides authenticated context to handlers
4. Services receive security context for authorization

**Header Injection**:
- `X-Resource-Token`: Internal service authentication
- `X-Resource-Role`: User role for authorization
- `X-Resource-Scope`: OAuth2 scopes for fine-grained access

### Stream Security Considerations

SSE streams require special security handling:

**Challenges**:
- Long-lived connections bypass typical request timeouts
- Authentication tokens may expire during streaming
- Resource consumption attacks via connection exhaustion

**Mitigations**:
- Connection limits per client
- Periodic authentication revalidation
- Resource usage monitoring
- Automatic cleanup on suspicious patterns

## Performance Optimization Strategies

### Memory-Efficient Streaming

SSE implementations optimize memory usage:

**BytesMut Optimization**:
- Reusable buffers for event construction
- Minimal allocations during streaming
- Efficient UTF-8 validation
- Zero-copy where possible

**Chunked Transfer**:
- Streaming responses without buffering entire content
- Backpressure handling for slow clients
- Automatic flow control with async streams

### Connection Pool Management

HTTP client connections efficiently managed:

**Pooling Strategy**:
- Reusable connections to LLM servers
- Connection timeout configuration
- Automatic retry with exponential backoff
- Circuit breaker for failing servers

### Concurrent Request Handling

SharedContext enables high concurrency:

**Optimization Techniques**:
- Read-write lock minimizes contention
- State observers avoid polling
- Async operations throughout
- Resource pooling for efficiency

## Extension Guidelines

### Adding New Streaming Formats

To support new streaming protocols:

1. **Define Stream Type**: Create new SSE variant or separate implementation
2. **Handle Framing**: Implement protocol-specific message framing
3. **Error Semantics**: Define error handling and recovery
4. **Connection Management**: Implement lifecycle and cleanup
5. **Test Coverage**: Add streaming tests with various scenarios

### Extending SharedContext

For new LLM server capabilities:

1. **Define Operations**: Add methods to SharedContext trait
2. **State Management**: Update state model for new operations
3. **Observer Notifications**: Emit events for state changes
4. **Resource Cleanup**: Ensure proper cleanup in all paths
5. **Concurrency Safety**: Verify thread-safe operation

### Custom Model Routing

To add routing strategies:

1. **Implement ModelRouter**: Create custom routing logic
2. **Priority Rules**: Define precedence for route selection
3. **Caching Strategy**: Implement route caching if needed
4. **Error Handling**: Provide clear routing failure errors
5. **Testing**: Verify routing decisions with tests

## Testing Architecture

### Two-Tier Testing Strategy

The crate employs a deliberate two-tier testing approach that separates unit-level mock testing from live integration testing:

**Unit Tests (in-module, `src/shared_rw.rs`)**: Use `MockServer` and `ServerFactoryStub` to validate SharedContext logic (ModelLoadStrategy decisions, forward_request routing, state transitions) without real LLM server processes. These tests use `#[serial(BodhiServerContext)]` for sequential execution of mock-based context tests.

**Live Integration Tests (`tests/test_live_shared_rw.rs`)**: Exercise `DefaultSharedContext` with the real llama.cpp server binary and actual GGUF model files. These validate that the server lifecycle (start, reload, stop) works end-to-end without an HTTP layer.

### Why Live SharedContext Tests Exist Separately

The live tests fill a critical gap between mock-based unit tests and full HTTP integration tests:

1. **Real Process Lifecycle Validation**: Mock tests cannot verify that `DefaultServerFactory` correctly spawns and manages an actual llama.cpp process. The live tests confirm that the executable path resolution, server startup, and graceful shutdown work with real binaries.
2. **Model File Resolution**: HuggingFace hub layouts use symlinks from snapshot directories to blob files. The live tests verify that `OfflineHubService` wrapping `HfHubService` correctly resolves both symlinked model files and direct blob references -- a subtlety that mocks cannot capture.
3. **No HTTP Overhead**: Unlike routes_app live tests that require a full HTTP server, these tests exercise SharedContext directly, isolating server process management from HTTP concerns.
4. **Sequential Execution**: Live tests use `serial_test::serial(live)` to prevent concurrent llama.cpp processes from conflicting over resources (ports, GPU memory).

### Live Test Infrastructure Requirements

The live tests depend on:

- **llama.cpp server binary**: Must be pre-built at `crates/llama_server_proc/bin/{BUILD_TARGET}/{DEFAULT_VARIANT}/{EXEC_NAME}`
- **Test model files**: Small GGUF models (Llama-68M) stored in `tests/data/live/huggingface/hub/` following the HuggingFace cache directory layout
- **OfflineHubService**: A services test utility that wraps `HfHubService` for local-only hub operations without network access
- **MockSettingService**: Provides executable path and variant configuration pointing to the real binary

### Test Data Layout for Live Tests

The `tests/data/live/huggingface/hub/` directory mirrors a real HuggingFace cache:

```
tests/data/live/huggingface/hub/
  models--afrideva--Llama-68M-Chat-v1-GGUF/
    blobs/          # Actual GGUF model binary (content-addressed)
    snapshots/      # Symlinks from snapshot hash to blob files
    refs/main       # Points to current snapshot hash
```

This layout exercises the full HuggingFace file resolution path: ref -> snapshot -> symlink -> blob. The symlink test (`test_live_shared_rw_reload_with_model_as_symlink`) validates that SharedContext handles symlinked model paths, while `test_live_shared_rw_reload_with_actual_file` validates direct blob paths.

### HTTP Infrastructure Testing

Comprehensive test utilities for HTTP handlers:

**Test RouterState**:
```rust
#[fixture]
pub fn test_router_state() -> RouterState {
  let app_service = create_test_app_service();
  RouterState::new(app_service, test_context())
}
```

**SSE Testing**:
```rust
#[tokio::test]
async fn test_sse_streaming() {
  let events = vec![
    DirectEvent::new("message", "data"),
    DirectEvent::new("error", "failed"),
  ];

  let stream = direct_sse(events);
  let response = stream.into_response();

  assert_eq!(response.headers()["content-type"], "text/event-stream");
}
```

### LLM Server Mocking

Mock implementations for testing without real processes:

**Mock ServerFactory**:
```rust
struct MockServerFactory;

impl ServerFactory for MockServerFactory {
  async fn create(&self, _args: Vec<String>) -> Result<Box<dyn LlamaServerInterface>> {
    Ok(Box::new(MockLlamaServer::new()))
  }
}
```

## Critical Invariants

### State Consistency Requirements

SharedContext must maintain consistent state:
- Server state transitions must be atomic
- Observers notified after state committed
- No partial state visible to requests
- Cleanup guaranteed even on panic

### Streaming Invariants

SSE streams must maintain protocol compliance:
- Events formatted with `data:` prefix
- Double newline between events
- UTF-8 encoding throughout
- Proper connection cleanup

### Resource Management Rules

All resources must be properly managed:
- LLM processes terminated on shutdown
- Connections closed on errors
- Memory freed on stream completion
- Temporary files cleaned up

### Thread Safety Guarantees

All shared state must be thread-safe:
- RouterState immutable after creation
- SharedContext uses proper synchronization
- Service references use Arc
- No data races possible