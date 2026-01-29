# PACKAGE.md - server_core Crate Implementation Index

*For architectural documentation and design rationale, see [crates/server_core/CLAUDE.md](crates/server_core/CLAUDE.md)*

## Module Structure

### Core HTTP Infrastructure
- `src/lib.rs` - Crate root with module exports
- `src/router_state.rs` - Dependency injection container for HTTP handlers
- `src/shared_rw.rs` - SharedContext trait and LLM server management
- `src/model_router.rs` - Model request routing between local and remote
- `src/server_args_merge.rs` - LLM server argument merging logic

### Streaming Infrastructure
- `src/direct_sse.rs` - Application-generated server-sent events
- `src/fwd_sse.rs` - Forwarded SSE from external services
- `src/middleware/mod.rs` - HTTP middleware components

### Error Handling
- `src/error.rs` - HTTP infrastructure error types

### Test Utilities (`test-utils` feature)
- `src/test_utils/mod.rs` - Test fixture exports
- `src/test_utils/http.rs` - HTTP testing utilities
- `src/test_utils/state.rs` - RouterState test fixtures
- `src/test_utils/server.rs` - Mock LLM server implementations

## Key Implementation Examples

### RouterState Dependency Injection
```rust
// src/router_state.rs
#[derive(Clone)]
pub struct RouterState {
  app_service: Arc<dyn AppService>,
  shared_context: Arc<dyn SharedContext>,
}

impl RouterState {
  pub fn new(
    app_service: Arc<dyn AppService>,
    shared_context: Arc<dyn SharedContext>,
  ) -> Self {
    Self { app_service, shared_context }
  }

  pub fn app_service(&self) -> &Arc<dyn AppService> {
    &self.app_service
  }

  pub fn shared_context(&self) -> &Arc<dyn SharedContext> {
    &self.shared_context
  }
}
```

### SharedContext LLM Server Management
```rust
// src/shared_rw.rs
#[async_trait]
pub trait SharedContext: Send + Sync + std::fmt::Debug {
  async fn start_server(&self, args: Vec<String>) -> Result<(), Error>;
  async fn stop_server(&self) -> Result<(), Error>;
  async fn reload_server(&self, args: Vec<String>) -> Result<(), Error>;
  async fn set_exec_variant(&self, variant: ExecVariant) -> Result<(), Error>;
  async fn get_state(&self) -> ServerState;
  async fn add_listener(&self, listener: StateListener);
}

pub enum ModelLoadStrategy {
  Continue,        // Use current loaded model
  DropAndLoad,     // Stop current and load new
  Load,           // Load when no model active
}
```

### Server-Sent Events Streaming
```rust
// src/direct_sse.rs
pub struct DirectEvent {
  event: Option<String>,
  data: String,
  id: Option<String>,
  retry: Option<Duration>,
}

impl DirectEvent {
  pub fn new(event: impl Into<String>, data: impl Into<String>) -> Self {
    Self {
      event: Some(event.into()),
      data: data.into(),
      id: None,
      retry: None,
    }
  }

  pub fn to_bytes(&self) -> BytesMut {
    let mut buf = BytesMut::new();

    if let Some(event) = &self.event {
      buf.extend_from_slice(b"event: ");
      buf.extend_from_slice(event.as_bytes());
      buf.extend_from_slice(b"\n");
    }

    buf.extend_from_slice(b"data: ");
    buf.extend_from_slice(self.data.as_bytes());
    buf.extend_from_slice(b"\n\n");

    buf
  }
}
```

### Model Request Routing
```rust
// src/model_router.rs
#[async_trait]
pub trait ModelRouter: Send + Sync + std::fmt::Debug {
  async fn route_request(
    &self,
    model_id: &str,
  ) -> Result<RouteDestination, ModelRouterError>;
}

pub enum RouteDestination {
  LocalModel(Alias),      // Route to SharedContext
  RemoteApi(ApiAlias),    // Route to AiApiService
}

impl DefaultModelRouter {
  async fn route_request(&self, model_id: &str) -> Result<RouteDestination> {
    // Check UserAlias first (highest priority)
    if let Some(alias) = self.data_service.find_user_alias(model_id)? {
      return Ok(RouteDestination::LocalModel(Alias::UserAlias(alias)));
    }

    // Check ModelAlias (medium priority)
    if let Some(alias) = self.data_service.find_model_alias(model_id)? {
      return Ok(RouteDestination::LocalModel(Alias::ModelAlias(alias)));
    }

    // Check ApiAlias (lowest priority)
    if let Some(alias) = self.data_service.find_api_alias(model_id)? {
      return Ok(RouteDestination::RemoteApi(alias));
    }

    Err(ModelRouterError::NotFound)
  }
}
```

### Server Arguments Merging
```rust
// src/server_args_merge.rs
pub fn merge_server_args(
  setting_args: &str,
  variant_args: &str,
  alias_args: &str,
) -> Vec<String> {
  let mut args_map = HashMap::new();

  // Parse and merge in precedence order
  parse_args_into_map(setting_args, &mut args_map);
  parse_args_into_map(variant_args, &mut args_map);
  parse_args_into_map(alias_args, &mut args_map);

  // Convert back to argument list
  args_map.into_iter()
    .flat_map(|(key, values)| {
      std::iter::once(key).chain(values)
    })
    .collect()
}

fn parse_args_into_map(args: &str, map: &mut HashMap<String, Vec<String>>) {
  let mut current_flag = None;

  for token in args.split_whitespace() {
    if token.starts_with("--") {
      current_flag = Some(token.to_string());
      map.insert(token.to_string(), vec![]);
    } else if let Some(flag) = current_flag.as_ref() {
      map.get_mut(flag).unwrap().push(token.to_string());
    }
  }
}
```

### Forwarded SSE Streaming
```rust
// src/fwd_sse.rs
pub async fn forward_sse(response: reqwest::Response) -> axum::Response {
  let stream = response.bytes_stream()
    .map(|chunk| {
      chunk.map(|bytes| Event::default().data(String::from_utf8_lossy(&bytes)))
    })
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

  axum::Response::builder()
    .header("content-type", "text/event-stream")
    .header("cache-control", "no-cache")
    .body(Body::from_stream(stream))
    .unwrap()
}
```

## Crate Commands

### Building
```bash
cargo build -p server_core
cargo build -p server_core --features test-utils
```

### Testing
```bash
cargo test -p server_core
cargo test -p server_core --features test-utils
```

### Documentation
```bash
cargo doc -p server_core --open
```

## Usage Examples

### RouterState in HTTP Handlers
```rust
use server_core::RouterState;
use axum::extract::State;

async fn handler(
  State(state): State<RouterState>,
  // other extractors
) -> Result<Response> {
  // Access services
  let data_service = state.app_service().data_service();
  let alias = data_service.find_alias("model-name")?;

  // Access LLM context
  let context = state.shared_context();
  context.reload_server(args).await?;

  Ok(Response::new())
}
```

### SSE Streaming
```rust
use server_core::{DirectEvent, direct_sse};

async fn progress_handler() -> Response {
  let events = vec![
    DirectEvent::new("progress", "0"),
    DirectEvent::new("progress", "50"),
    DirectEvent::new("progress", "100"),
    DirectEvent::new("complete", "done"),
  ];

  direct_sse(events).into_response()
}
```

### Model Routing
```rust
use server_core::{ModelRouter, RouteDestination};

async fn route_model(router: &dyn ModelRouter, model: &str) -> Result<Response> {
  match router.route_request(model).await? {
    RouteDestination::LocalModel(alias) => {
      // Handle local model with SharedContext
      handle_local_model(alias).await
    }
    RouteDestination::RemoteApi(api_alias) => {
      // Forward to external API
      forward_to_api(api_alias).await
    }
  }
}
```

## Feature Flags

- `test-utils`: Enables test utilities and mock implementations

## Dependencies

### Core Dependencies
- `async-trait`: Async trait support
- `axum`: HTTP framework
- `bytes`: Efficient byte buffers
- `futures`: Async stream utilities
- `reqwest`: HTTP client
- `tokio`: Async runtime
- `tower`: Middleware framework

### From Workspace
- `objs`: Domain objects and errors
- `services`: Business service layer
- `llama_server_proc`: LLM server process management

## File References

See individual module files for complete implementation details:
- HTTP state: `src/router_state.rs`
- LLM management: `src/shared_rw.rs`
- SSE streaming: `src/direct_sse.rs`, `src/fwd_sse.rs`
- Model routing: `src/model_router.rs`
- Argument merging: `src/server_args_merge.rs`
- Test utilities: `src/test_utils/*.rs`