# PACKAGE.md - server_app

See [crates/server_app/CLAUDE.md](crates/server_app/CLAUDE.md) for architectural overview and design decisions.

This document provides detailed technical information for the `server_app` crate, focusing on BodhiApp's main HTTP server executable orchestration architecture, sophisticated server lifecycle management, and comprehensive service bootstrap patterns.

## Main HTTP Server Executable Architecture

The `server_app` crate serves as BodhiApp's **main HTTP server executable orchestration layer**, implementing advanced server lifecycle management, graceful shutdown coordination, and comprehensive service bootstrap with sophisticated listener patterns.

### Server Handle Architecture
Advanced server lifecycle management with comprehensive coordination:

```rust
// Pattern structure (see crates/server_app/src/server.rs)
pub struct Server {
  host: String,
  port: u16,
  ready: Sender<()>,
  shutdown_rx: Receiver<()>,
}

pub struct ServerHandle {
  pub server: Server,
  pub shutdown: oneshot::Sender<()>,
  pub ready_rx: oneshot::Receiver<()>,
}

pub fn build_server_handle(host: &str, port: u16) -> ServerHandle {
  let (shutdown, shutdown_rx) = oneshot::channel::<()>();
  let (ready, ready_rx) = oneshot::channel::<()>();
  let server = Server::new(host, port, ready, shutdown_rx);
  ServerHandle { server, shutdown, ready_rx }
}
```

### Graceful Shutdown Integration
Sophisticated shutdown coordination with resource cleanup:

```rust
// Shutdown callback pattern (see crates/server_app/src/serve.rs)
#[async_trait::async_trait]
pub trait ShutdownCallback: Send + Sync {
  async fn shutdown(&self);
}

pub struct ShutdownContextCallback {
  ctx: Arc<dyn SharedContext>,
}
```

**Server Lifecycle Features**:
- Ready notification channel for startup coordination with external systems
- Graceful shutdown channel for clean resource cleanup and service termination
- Cross-platform signal handling (Unix SIGTERM, Windows Ctrl+Break) with proper signal registration
- ShutdownCallback trait for custom resource cleanup during server termination
- TCP listener management with port conflict detection and error reporting

## Advanced Listener Pattern Implementation

### ServerKeepAlive Listener Architecture
Intelligent server lifecycle management with configurable keep-alive coordination:

```rust
// Keep-alive pattern (see crates/server_app/src/listener_keep_alive.rs)
pub struct ServerKeepAlive {
  keep_alive: RwLock<i64>,
  timer_handle: RwLock<Option<JoinHandle<()>>>,
  shared_context: Arc<dyn SharedContext>,
}
// Values: -1 = never stop, 0 = immediate stop, n = stop after n seconds
```

### VariantChangeListener Implementation
Dynamic execution variant switching with SharedContext coordination:

```rust
// Variant change pattern (see crates/server_app/src/listener_variant.rs)
pub struct VariantChangeListener {
  ctx: Arc<dyn SharedContext>,
}

impl SettingsChangeListener for VariantChangeListener {
  fn on_change(&self, key: &str, ...) {
    if key != BODHI_EXEC_VARIANT { return; }
    // Spawns async task to update SharedContext exec variant
  }
}
```

**Advanced Listener Features**:
- Real-time configuration updates without service restart through SettingsChangeListener pattern
- Server state change notifications with ServerStateListener for LLM server lifecycle events
- Timer management with automatic reset on activity and cancellation on server stop
- Async processing for non-blocking configuration updates with proper error handling
- Cross-service coordination with SharedContext for execution variant switching

## Service Bootstrap Orchestration Architecture

### Complete Service Initialization Pattern
The bootstrap sequence in `crates/server_app/src/serve.rs`:

1. Create server handle with lifecycle coordination channels
2. Validate executable path for LLM server binary
3. Initialize SharedContext with HubService and SettingService
4. Register VariantChangeListener and ServerKeepAlive listeners
5. Build static router with environment-specific configuration
6. Compose routes via `build_routes()` integration with routes_app
7. Spawn server task with ShutdownContextCallback
8. Wait for ready notification and report server URL

**Service Bootstrap Features**:
- Complete AppService registry initialization with all business services
- SharedContext bootstrap with HubService and SettingService coordination for LLM server management
- Advanced listener registration with ServerKeepAlive and VariantChangeListener for real-time coordination
- Static asset serving with development proxy support and production embedded assets
- Route composition integration with routes_app for complete HTTP middleware stack

## Cross-Platform Signal Handling Implementation

### Signal Coordination Architecture
See `crates/server_app/src/shutdown.rs` for the `shutdown_signal()` function implementing:
- Unix: Ctrl+C and SIGTERM via `tokio::signal::unix::signal(SignalKind::terminate())`
- Windows: Ctrl+C and Ctrl+Break via `tokio::signal::windows::ctrl_break()`
- Uses `tokio::select!` to await whichever signal arrives first

## Error Handling Architecture

### Server-Specific Error Types
See `crates/server_app/src/error.rs` for `TaskJoinError` and `crates/server_app/src/serve.rs` for `ServeError`:

```rust
// ServeError aggregates all server-layer errors
pub enum ServeError {
  Setting(SettingServiceError),
  Join(TaskJoinError),
  Context(ContextError),
  Server(ServerError),
  Unknown,
}
```

**Error Handling Features**:
- Transparent error wrapping preserves original service error context
- Server-specific error types with user-friendly messages via errmeta_derive integration
- Comprehensive error boundaries for server lifecycle, service bootstrap, and listener operations
- Error isolation prevents listener failures from interrupting server operation

## Live Integration Test Infrastructure

### File Structure

| File | Purpose |
|------|---------|
| `crates/server_app/tests/utils/live_server_utils.rs` | Inline AppService setup, `live_server` fixture, OAuth2 helpers |
| `crates/server_app/tests/utils/tool_call.rs` | SSE stream parsing, weather tool definitions |
| `crates/server_app/tests/utils/mod.rs` | Re-exports |
| `crates/server_app/tests/resources/.env.test` | OAuth2 credentials (gitignored) |
| `crates/server_app/tests/resources/.env.test.example` | Template for required env vars |

### TestServerHandle
The central test fixture struct returned by the `live_server` rstest fixture:

```rust
// See crates/server_app/tests/utils/live_server_utils.rs
pub struct TestServerHandle {
  pub temp_cache_dir: TempDir,
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
  pub app_service: Arc<dyn AppService>,
}
```

Each test receives a fully bootstrapped HTTP server on a random port with:
- Temp directory for `BODHI_HOME` (app DB, session DB, secrets, settings)
- Real HuggingFace cache at `~/.cache/huggingface` via `OfflineHubService`
- Real OAuth2 resource client created dynamically via Keycloak admin API
- Real llama.cpp binary from `crates/llama_server_proc/bin/`

### Authentication Helpers

```rust
// Get OAuth2 tokens via resource-owner password grant
pub async fn get_oauth_tokens(app_service: &dyn AppService)
  -> anyhow::Result<(String, String)>

// Create session record in SQLite session store
pub async fn create_authenticated_session(
  app_service: &Arc<dyn AppService>,
  access_token: &str,
  refresh_token: &str,
) -> anyhow::Result<String>

// Build session cookie for reqwest client
pub fn create_session_cookie(session_id: &str) -> Cookie
```

### SSE Stream Parsing Utilities
See `crates/server_app/tests/utils/tool_call.rs`:

- `get_weather_tool()` -- Returns `ChatCompletionTools` with `get_current_temperature` function definition
- `parse_streaming_tool_calls(response_text)` -- Parses SSE `data:` lines, accumulates tool call arguments by index, returns `(Vec<Value>, String)` of tool calls and finish reason
- `parse_streaming_content(response_text)` -- Parses SSE `data:` lines, concatenates `delta.content` strings, returns `(String, String)` of content and finish reason

### Live Test Files

| Test File | Tests | Key Assertions |
|-----------|-------|----------------|
| `test_live_chat_completions_non_streamed.rs` | `test_live_chat_completions_non_streamed` | Model available in /v1/models, response contains "Tuesday", finish_reason is "stop" |
| `test_live_chat_completions_streamed.rs` | `test_live_chat_completions_stream` | SSE stream parses correctly, content contains "Tuesday", last chunk has finish_reason "stop" |
| `test_live_tool_calling_non_streamed.rs` | `test_live_tool_calling_non_streamed`, `test_live_tool_calling_multi_turn_non_streamed` | finish_reason "tool_calls", function name is "get_current_temperature", multi-turn returns "stop" with temperature info |
| `test_live_tool_calling_streamed.rs` | `test_live_tool_calling_streamed`, `test_live_tool_calling_multi_turn_streamed` | Streaming tool call accumulation, multi-turn streaming with tool result follow-up |
| `test_live_thinking_disabled.rs` | `test_live_chat_completions_thinking_disabled`, `test_live_chat_completions_reasoning_format_none`, `test_live_chat_completions_thinking_enabled_default` | `reasoning_content` absent when thinking disabled, present by default |
| `test_live_agentic_chat_with_exa.rs` | `test_live_agentic_chat_with_exa_toolset` | Exa toolset enablement, user config, qualified tool names, backend tool execution, multi-turn with real Exa API |

### Common Test Pattern
All live tests follow the same structure:

1. Destructure `live_server` fixture into `TestServerHandle` components
2. Obtain OAuth2 tokens via `get_oauth_tokens`
3. Create authenticated session via `create_authenticated_session`
4. Build session cookie via `create_session_cookie`
5. Use `reqwest::Client` to make HTTP requests with cookie header and `Sec-Fetch-Site: same-origin`
6. Assert response status, parse JSON or SSE stream
7. Call `handle.shutdown().await?` before final assertions

### Agentic Chat Test (Exa) Workflow
The `test_live_agentic_chat_with_exa_toolset` test exercises the full agentic pipeline:

1. **Enable toolset type**: `PUT /bodhi/v1/toolset_types/{scope_uuid}/app-config`
2. **Configure user toolset**: `POST /bodhi/v1/toolsets` with API key
3. **Verify toolset**: `GET /bodhi/v1/toolsets` confirms `enabled`, `has_api_key`, 4 tools
4. **Chat with tools**: `POST /v1/chat/completions` with qualified tool names (`toolset__builtin-exa-web-search__<method>`)
5. **Execute tool**: `POST /bodhi/v1/toolsets/{id}/execute/{method}` with parsed arguments
6. **Follow-up**: Second chat completion with tool result produces final "stop" response

## Source File Index

| File | Purpose |
|------|---------|
| `crates/server_app/src/lib.rs` | Module declarations and public re-exports |
| `crates/server_app/src/server.rs` | Server, ServerHandle, build_server_handle, TCP binding |
| `crates/server_app/src/serve.rs` | ServeCommand, get_server_handle, ServerShutdownHandle, ServeError |
| `crates/server_app/src/shutdown.rs` | shutdown_signal() cross-platform signal handling |
| `crates/server_app/src/listener_keep_alive.rs` | ServerKeepAlive listener with timer management |
| `crates/server_app/src/listener_variant.rs` | VariantChangeListener for exec variant switching |
| `crates/server_app/src/error.rs` | TaskJoinError, ServerError types |
| `crates/server_app/src/test_utils/mod.rs` | Feature-gated test utilities (currently empty) |

## Commands

### Unit Tests
```
cargo test -p server_app
```
Runs server lifecycle, listener integration, and error handling tests.

### Live Integration Tests
```
cargo test -p server_app -- --ignored 2>/dev/null || \
cargo test -p server_app --test test_live_chat_completions_non_streamed
```

Run a specific live test:
```
cargo test -p server_app --test test_live_chat_completions_non_streamed
cargo test -p server_app --test test_live_chat_completions_streamed
cargo test -p server_app --test test_live_tool_calling_non_streamed
cargo test -p server_app --test test_live_tool_calling_streamed
cargo test -p server_app --test test_live_thinking_disabled
cargo test -p server_app --test test_live_agentic_chat_with_exa
```

### Prerequisites for Live Tests
1. Download the model:
   ```
   huggingface-cli download ggml-org/Qwen3-1.7B-GGUF
   ```
2. Ensure llama.cpp binary exists at `crates/llama_server_proc/bin/`
3. Copy `tests/resources/.env.test.example` to `tests/resources/.env.test` and fill in OAuth2 credentials
4. For Exa test: set `INTEG_TEST_EXA_API_KEY` environment variable

### Building
```
cargo build -p server_app
```
