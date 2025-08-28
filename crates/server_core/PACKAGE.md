# PACKAGE.md - server_core

This document provides detailed technical information for the `server_core` crate, focusing on BodhiApp's HTTP infrastructure orchestration architecture, sophisticated streaming capabilities, and LLM server context management patterns.

## HTTP Infrastructure Orchestration Architecture

The `server_core` crate serves as BodhiApp's **HTTP infrastructure orchestration layer**, implementing advanced server-sent event streaming, LLM server context coordination, and HTTP route infrastructure with comprehensive async operations.

### RouterState Dependency Injection Architecture
Sophisticated state management for HTTP route handlers with service coordination:

```rust
pub trait RouterState: std::fmt::Debug + Send + Sync {
    fn app_service(&self) -> Arc<dyn AppService>;
    
    fn chat_completions(
        &self,
        request: CreateChatCompletionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>>;
}

#[derive(Debug, Clone)]
pub struct DefaultRouterState {
    pub(crate) ctx: Arc<dyn SharedContext>,
    pub(crate) app_service: Arc<dyn AppService>,
}

impl DefaultRouterState {
    pub fn new(ctx: Arc<dyn SharedContext>, app_service: Arc<dyn AppService>) -> Self {
        Self { ctx, app_service }
    }
}
```

### Cross-Service HTTP Coordination Implementation
RouterState orchestrates complex service interactions for HTTP operations:

```rust
impl RouterState for DefaultRouterState {
    fn app_service(&self) -> Arc<dyn AppService> {
        self.app_service.clone()
    }
    
    fn chat_completions(
        &self,
        request: CreateChatCompletionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>> {
        Box::pin(async move {
            // 1. Extract model alias from request
            let model_alias = extract_alias_from_request(&request)?;
            
            // 2. Resolve alias via DataService
            let data_service = self.app_service.data_service();
            let alias = data_service.load_alias(&model_alias)
                .map_err(RouterStateError::AliasNotFound)?;
            
            // 3. Route request through SharedContext
            let response = self.ctx.chat_completions(request, alias).await
                .map_err(RouterStateError::ContextError)?;
            
            Ok(response)
        })
    }
}
```

**Key HTTP Coordination Features**:
- Service registry access through `AppService` trait for business logic coordination
- Model alias resolution via `DataService` for HTTP request processing
- LLM server context coordination for request routing and processing
- Comprehensive error handling with HTTP status code mapping

## Server-Sent Events Streaming Architecture

### DirectSSE Implementation for Application Events
Advanced SSE implementation for application-generated event streaming:

```rust
#[derive(Debug, Clone, Default)]
pub struct DirectEvent {
    buffer: BytesMut,
}

impl DirectEvent {
    pub fn new() -> Self {
        DirectEvent::default()
    }
    
    pub fn data<T>(mut self, data: T) -> Self
    where
        T: AsRef<str>,
    {
        for line in data.as_ref().split('\n') {
            self.buffer.extend_from_slice(line.as_bytes());
        }
        self
    }
    
    pub fn finalize(mut self) -> Bytes {
        self.buffer.put_u8(b'\n');
        self.buffer.freeze()
    }
}

#[derive(Clone)]
#[must_use]
pub struct DirectSse<S> {
    stream: S,
    keep_alive: Option<KeepAlive>,
}

impl<S> DirectSse<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            keep_alive: None,
        }
    }
    
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}
```

### RawSSE for LLM Server Proxying
Simplified proxy streaming for external LLM service integration:

```rust
// Pattern structure (see src/fwd_sse.rs:5-25 for complete implementation)
pub struct RawSSE<S>(S);

impl<S> RawSSE<S>
where
  S: Stream<Item = String> + Send + 'static,
{
  pub fn new(stream: S) -> Self {
    RawSSE(stream)
  }

  pub fn into_response(self) -> Response {
    let body = Body::from_stream(self.0.map(Ok::<_, Infallible>));

    Response::builder()
      .header("Content-Type", "text/event-stream")
      .header("Cache-Control", "no-cache")
      .body(body)
      .unwrap()
  }
}

// Convenience function for channel-based SSE streaming
pub fn fwd_sse(rx: Receiver<String>) -> Response {
  let stream = ReceiverStream::new(rx);
  RawSSE::new(stream).into_response()
}
```

**SSE Architecture Features**:
- `DirectSSE`: Custom event formatting with BytesMut optimization and DirectEvent builder pattern for application events
- `RawSSE`: Simplified proxy streaming with tokio channel integration for LLM server responses
- Connection lifecycle management with automatic cleanup and keep-alive support via KeepAliveStream
- Axum integration for native HTTP response streaming with proper MIME type and header management

## SharedContext LLM Server Management

### LLM Server Lifecycle Coordination Architecture
Advanced context management for LLM server instances with state synchronization:

```rust
#[derive(Debug, Clone)]
pub enum ServerState {
    Start,
    Stop,
    ChatCompletions { alias: String },
    Variant { variant: String },
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SharedContext: std::fmt::Debug + Send + Sync {
    async fn set_exec_variant(&self, variant: &str) -> Result<()>;
    async fn reload(&self, server_args: Option<LlamaServerArgs>) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn is_loaded(&self) -> bool;
    
    async fn chat_completions(
        &self,
        mut request: CreateChatCompletionRequest,
        alias: Alias,
    ) -> Result<reqwest::Response>;
    
    async fn add_state_listener(&self, listener: Arc<dyn ServerStateListener>);
    async fn notify_state_listeners(&self, state: ServerState);
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ServerStateListener: Debug + Send + Sync {
    async fn on_state_change(&self, state: ServerState);
}
```

### LLM Server Context Implementation Pattern
Sophisticated LLM server coordination with process management:

```rust
// Pattern structure (see src/shared_rw.rs:85-95 for complete implementation)
pub struct DefaultSharedContext {
  hub_service: Arc<dyn HubService>,
  setting_service: Arc<dyn SettingService>,
  factory: Box<dyn ServerFactory>,
  exec_variant: ExecVariant,
  server: RwLock<Option<Box<dyn Server>>>,
  state_listeners: RwLock<Vec<Arc<dyn ServerStateListener>>>,
}

impl DefaultSharedContext {
  pub fn new(hub_service: Arc<dyn HubService>, setting_service: Arc<dyn SettingService>) -> Self {
    Self::with_args(hub_service, setting_service, Box::new(DefaultServerFactory))
  }

  pub fn with_args(
    hub_service: Arc<dyn HubService>,
    setting_service: Arc<dyn SettingService>,
    factory: Box<dyn ServerFactory>,
  ) -> Self {
    let exec_variant = setting_service.exec_variant();
    Self {
      hub_service,
      setting_service,
      factory,
      exec_variant: ExecVariant::new(exec_variant),
      server: RwLock::new(None),
      state_listeners: RwLock::new(Vec::new()),
    }
  }
}

#[async_trait::async_trait]
impl SharedContext for DefaultSharedContext {
  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
  ) -> Result<reqwest::Response> {
    let lock = self.server.read().await;
    let server = lock.as_ref();
    let loaded_alias = server.map(|server| server.get_server_args().alias);
    let request_alias = &alias.alias;
    let model_file = self
      .hub_service
      .find_local_file(&alias.repo, &alias.filename, Some(alias.snapshot.clone()))?
      .path();
    alias.request_params.update(&mut request);
    let input_value = serde_json::to_value(request)?;
    let alias_name = alias.alias.clone();
    let setting_args = self.get_setting_args();
    let result = match ModelLoadStrategy::choose(loaded_alias, request_alias) {
      ModelLoadStrategy::Continue => {
        let response = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .chat_completions(&input_value)
          .await?;
        Ok(response)
      }
      ModelLoadStrategy::DropAndLoad => {
        drop(lock);
        let variant = self.exec_variant.get().await;
        let setting_variant_args = self.get_setting_variant_args(&variant);
        let merged_args =
          merge_server_args(&setting_args, &setting_variant_args, &alias.context_params);
        let server_args = LlamaServerArgsBuilder::default()
          .alias(alias.alias)
          .model(model_file.to_string_lossy().to_string())
          .server_args(merged_args)
          .build()?;
        self.reload(Some(server_args)).await?;
        let lock = self.server.read().await;
        let server = lock.as_ref();
        let response = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .chat_completions(&input_value)
          .await?;
        Ok(response)
      }
      ModelLoadStrategy::Load => {
        let variant = self.exec_variant.get().await;
        let setting_variant_args = self.get_setting_variant_args(&variant);
        let merged_args =
          merge_server_args(&setting_args, &setting_variant_args, &alias.context_params);
        let server_args = LlamaServerArgsBuilder::default()
          .alias(alias.alias)
          .model(model_file.to_string_lossy().to_string())
          .server_args(merged_args)
          .build()?;
        drop(lock);
        self.reload(Some(server_args)).await?;
        let lock = self.server.read().await;
        let server = lock.as_ref();
        let response = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .chat_completions(&input_value)
          .await?;
        Ok(response)
      }
    };
    self
      .notify_state_listeners(ServerState::ChatCompletions { alias: alias_name })
      .await;
    result
  }
}
```

### ModelLoadStrategy Implementation
Intelligent model switching strategy for efficient resource management:

```rust
// Pattern structure (see src/shared_rw.rs:280-295 for complete implementation)
#[derive(Debug, PartialEq)]
enum ModelLoadStrategy {
  Continue,    // Same model already loaded - continue using existing server
  DropAndLoad, // Different model - stop current server and load new model
  Load,        // No server loaded - start new server with model
}

impl ModelLoadStrategy {
  fn choose(loaded_alias: Option<String>, request_alias: &str) -> ModelLoadStrategy {
    if let Some(loaded_model) = loaded_alias {
      if loaded_model.eq(request_alias) {
        ModelLoadStrategy::Continue
      } else {
        ModelLoadStrategy::DropAndLoad
      }
    } else {
      ModelLoadStrategy::Load
    }
  }
}
```

### ExecVariant Management
Dynamic execution variant switching for different LLM server configurations:

```rust
// Pattern structure (see src/shared_rw.rs:305-325 for complete implementation)
#[derive(Debug)]
pub struct ExecVariant {
  inner: RwLock<String>,
}

impl ExecVariant {
  pub fn new(variant: String) -> Self {
    Self {
      inner: RwLock::new(variant),
    }
  }

  pub async fn get(&self) -> String {
    self.inner.read().await.clone()
  }

  pub async fn set(&self, variant: String) {
    *self.inner.write().await = variant;
  }
}
```

**SharedContext Architecture Features**:
- Thread-safe LLM server state management with RwLock coordination
- Observer pattern implementation for server state change notifications with async broadcasting
- Dynamic server configuration and reloading based on model aliases with ModelLoadStrategy optimization
- Comprehensive error handling with context preservation and recovery
- Execution variant management for different server configurations (CPU, CUDA, etc.)
- Server args merging with setting-level, variant-level, and alias-level parameter coordination

## HTTP Error Coordination Architecture

### Service Error Translation to HTTP Responses
Sophisticated error handling with HTTP status code mapping and localization:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = objs::AppError)]
pub enum RouterStateError {
    #[error(transparent)]
    ObjValidationError(#[from] ObjValidationError),
    #[error(transparent)]
    AliasNotFound(#[from] AliasNotFoundError),
    #[error(transparent)]
    HubService(#[from] HubServiceError),
    #[error(transparent)]
    ContextError(#[from] ContextError),
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = objs::AppError)]
pub enum ContextError {
    #[error("server_not_loaded")]
    #[error_meta(error_type = objs::ErrorType::InternalServerError)]
    ServerNotLoaded,
    
    #[error("server_startup_failed")]
    #[error_meta(error_type = objs::ErrorType::InternalServerError)]
    ServerStartupFailed { details: String },
    
    #[error("server_communication_error")]
    #[error_meta(error_type = objs::ErrorType::BadGateway)]
    ServerError { source: Box<dyn std::error::Error + Send + Sync> },
}
```

### HTTP Response Error Integration
Error translation maintains service context while providing appropriate HTTP responses:

```rust
// Axum error handling integration
impl IntoResponse for RouterStateError {
    fn into_response(self) -> Response {
        let api_error = objs::ApiError::from(self);
        let status_code = match api_error.error_type {
            objs::ErrorType::NotFound => StatusCode::NOT_FOUND,
            objs::ErrorType::BadRequest => StatusCode::BAD_REQUEST,
            objs::ErrorType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            objs::ErrorType::BadGateway => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let response_body = serde_json::to_string(&api_error)
            .unwrap_or_else(|_| r#"{"error":"Failed to serialize error"}"#.to_string());
        
        (status_code, response_body).into_response()
    }
}
```

**Error Coordination Features**:
- Transparent error wrapping preserves original service error context
- HTTP status code mapping coordinated with objs error system
- Localized error messages via `errmeta_derive` integration
- Comprehensive error boundaries for streaming and context operations

## Cross-Crate Integration Implementation

### Service Layer HTTP Integration
HTTP infrastructure coordinates extensively with BodhiApp's service layer:

```rust
// RouterState provides HTTP handlers access to service registry (see src/router_state.rs:45-55)
impl RouterState for DefaultRouterState {
  fn app_service(&self) -> Arc<dyn AppService> {
    self.app_service.clone()
  }
  
  fn chat_completions(&self, request: CreateChatCompletionRequest) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>> {
    Box::pin(async move {
      let alias = self
        .app_service
        .data_service()
        .find_alias(&request.model)
        .ok_or_else(|| AliasNotFoundError(request.model.clone()))?;
      let response = self.ctx.chat_completions(request, alias).await?;
      Ok(response)
    })
  }
}
```
```

### LlamaServerProc Integration Architecture
SharedContext coordinates with LLM server process management:

```rust
pub trait ServerFactory: std::fmt::Debug + Send + Sync {
    fn create_server(
        &self,
        executable_path: &Path,
        server_args: &LlamaServerArgs,
    ) -> Result<Box<dyn Server>, ContextError>;
}

// SharedContext uses ServerFactory for LLM server lifecycle management
impl DefaultSharedContext {
    async fn reload(&self, server_args: Option<LlamaServerArgs>) -> Result<()> {
        // 1. Stop existing server instance
        if let Some(server) = self.server_state.write().await.take() {
            server.stop().await.map_err(ContextError::ServerError)?;
        }
        
        // 2. Create new server instance via factory
        let server_args = server_args.unwrap_or_else(|| self.default_server_args());
        let executable_path = self.setting_service.llama_server_executable_path()?;
        
        let server = self.server_factory.create_server(&executable_path, &server_args)?;
        server.start().await.map_err(ContextError::ServerStartupFailed)?;
        
        // 3. Update server state
        *self.server_state.write().await = Some(server);
        
        // 4. Notify state listeners
        self.notify_state_listeners(ServerState::Start).await;
        
        Ok(())
    }
}
```

## HTTP Streaming Performance Architecture

### Memory-Efficient Event Processing
DirectSSE uses optimized memory management for high-performance streaming:

```rust
// BytesMut-based event construction for efficient memory usage
impl DirectEvent {
    pub fn data<T>(mut self, data: T) -> Self
    where
        T: AsRef<str>,
    {
        // Direct byte buffer manipulation for performance
        for line in data.as_ref().split('\n') {
            self.buffer.extend_from_slice(line.as_bytes());
        }
        self
    }
    
    pub fn finalize(mut self) -> Bytes {
        self.buffer.put_u8(b'\n');
        self.buffer.freeze() // Zero-copy conversion to immutable bytes
    }
}

// Stream processing with minimal buffering
impl<S, T, E> IntoResponse for DirectSse<S>
where
    S: TryStream<Ok = T, Error = E> + Send + 'static,
    T: Into<DirectEvent>,
    E: Into<BoxError>,
{
    fn into_response(self) -> Response {
        let stream = self.stream.map_ok(Into::into).map_ok(DirectEvent::finalize);
        
        let body = Body::from_stream(stream);
        
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::CONNECTION, "keep-alive")
            .body(body)
            .unwrap()
    }
}
```

### Connection Lifecycle Management
Sophisticated connection management with automatic cleanup and error recovery:

```rust
// Keep-alive mechanism for connection stability
#[derive(Debug, Clone)]
pub struct KeepAlive {
    interval: Duration,
    text: Cow<'static, str>,
}

impl KeepAlive {
    pub fn new(interval: Duration, text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            interval,
            text: text.into(),
        }
    }
}

// Automatic connection cleanup and error handling
pin_project! {
    pub struct SseKeepAlive<S> {
        #[pin]
        stream: S,
        keep_alive: KeepAlive,
        #[pin]
        keep_alive_timer: Sleep,
    }
}

impl<S, T, E> Stream for SseKeepAlive<S>
where
    S: TryStream<Ok = T, Error = E>,
    T: Into<DirectEvent>,
    E: Into<BoxError>,
{
    type Item = Result<Bytes, BoxError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        
        // Poll the underlying stream for events
        match ready!(this.stream.try_poll_next(cx)) {
            Some(Ok(event)) => {
                // Reset keep-alive timer on successful event
                this.keep_alive_timer.reset(Instant::now() + this.keep_alive.interval);
                Poll::Ready(Some(Ok(event.into().finalize())))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
            None => Poll::Ready(None),
        }
    }
}
```

## Extension Guidelines for HTTP Infrastructure

### Adding New Streaming Endpoints
When creating new HTTP streaming functionality:

1. **SSE Type Selection**: Choose DirectSSE for application-generated events or ForwardedSSE for service proxying
2. **Event Format Design**: Use BytesMut for efficient event construction and memory management
3. **Connection Management**: Implement proper lifecycle management with automatic cleanup
4. **Error Recovery**: Design comprehensive error handling with graceful degradation
5. **Performance Testing**: Validate streaming performance under realistic load conditions

### SharedContext Extensions
For new LLM server context management patterns:

1. **Lifecycle Coordination**: Design proper async startup/shutdown with state synchronization
2. **Observer Pattern**: Implement state change notifications with proper listener management
3. **Resource Management**: Ensure proper cleanup and resource lifecycle coordination
4. **Error Boundaries**: Provide comprehensive error handling with context preservation
5. **Testing Infrastructure**: Support comprehensive context testing with realistic server mocking

### RouterState Integration Patterns
For new HTTP infrastructure coordination:

1. **Service Registry**: Coordinate with AppService for consistent business logic access
2. **Dependency Injection**: Design proper service access patterns for HTTP handlers
3. **Error Translation**: Convert service errors to appropriate HTTP responses with localization
4. **Request Processing**: Implement efficient request processing with service coordination
5. **Integration Testing**: Support HTTP infrastructure testing with comprehensive service mocking

## Commands for HTTP Infrastructure Testing

**HTTP Infrastructure Tests**: `cargo test -p server_core` (includes streaming and context testing)  
**SSE Streaming Tests**: `cargo test -p server_core sse` (includes DirectSSE and ForwardedSSE testing)  
**Context Management Tests**: `cargo test -p server_core context` (includes SharedContext lifecycle testing)