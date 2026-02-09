# PACKAGE.md - server_app

See [crates/server_app/CLAUDE.md](crates/server_app/CLAUDE.md) for architectural overview and design decisions.

This document provides detailed technical information for the `server_app` crate, focusing on BodhiApp's main HTTP server executable orchestration architecture, sophisticated server lifecycle management, and comprehensive service bootstrap patterns.

## Main HTTP Server Executable Architecture

The `server_app` crate serves as BodhiApp's **main HTTP server executable orchestration layer**, implementing advanced server lifecycle management, graceful shutdown coordination, and comprehensive service bootstrap with sophisticated listener patterns.

### Server Handle Architecture
Advanced server lifecycle management with comprehensive coordination:

```rust
// Pattern structure (see crates/server_app/src/server.rs for complete implementation)
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
// Shutdown callback pattern (see crates/server_app/src/serve.rs for complete implementation)
#[async_trait::async_trait]
pub trait ShutdownCallback: Send + Sync {
  async fn shutdown(&self);
}

pub struct ShutdownContextCallback {
  ctx: Arc<dyn SharedContext>,
}

#[async_trait::async_trait]
impl ShutdownCallback for ShutdownContextCallback {
  async fn shutdown(&self) {
    if let Err(err) = self.ctx.stop().await {
      tracing::warn!(err = ?err, "error stopping llama context");
    }
  }
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
// Keep-alive pattern (see crates/server_app/src/listener_keep_alive.rs for complete implementation)
#[derive(Debug)]
pub struct ServerKeepAlive {
  keep_alive: RwLock<i64>,
  timer_handle: RwLock<Option<JoinHandle<()>>>,
  shared_context: Arc<dyn SharedContext>,
}

impl ServerKeepAlive {
  pub fn new(shared_context: Arc<dyn SharedContext>, keep_alive: i64) -> Self {
    Self {
      keep_alive: RwLock::new(keep_alive),
      timer_handle: RwLock::new(None),
      shared_context,
    }
  }

  fn start_timer(&self) {
    let keep_alive = *self.keep_alive.read().unwrap();
    match keep_alive {
      -1 => { /* Never stop - cancel timer */ }
      0 => { /* Immediate stop - stop server now */ }
      n => { /* Timed stop - start timer for n seconds */ }
    }
  }
}
```

### VariantChangeListener Implementation
Dynamic execution variant switching with SharedContext coordination:

```rust
// Variant change pattern (see crates/server_app/src/listener_variant.rs for complete implementation)
#[derive(Debug, derive_new::new)]
pub struct VariantChangeListener {
  ctx: Arc<dyn SharedContext>,
}

impl SettingsChangeListener for VariantChangeListener {
  fn on_change(&self, key: &str, prev_value: &Option<serde_yaml::Value>, _prev_source: &SettingSource, new_value: &Option<serde_yaml::Value>, _new_source: &SettingSource) {
    if key != BODHI_EXEC_VARIANT { return; }
    if prev_value.is_some() && new_value.is_some() && prev_value.as_ref() == new_value.as_ref() { return; }
    
    let ctx_clone = self.ctx.clone();
    let new_value = if let Some(serde_yaml::Value::String(new_value)) = new_value {
      new_value.to_string()
    } else {
      tracing::error!("BODHI_EXEC_VARIANT is not set, or not a string type, skipping updating the server config");
      return;
    };
    
    task::spawn(async move {
      if let Err(err) = ctx_clone.set_exec_variant(&new_value).await {
        tracing::error!(?err, "failed to set exec variant");
      }
    });
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
Comprehensive service bootstrap with dependency injection and configuration validation:

```rust
// Service bootstrap pattern (see crates/server_app/src/serve.rs for complete implementation)
impl ServeCommand {
  pub async fn get_server_handle(
    &self,
    service: Arc<dyn AppService>,
    static_dir: Option<&'static Dir<'static>>,
  ) -> Result<ServerShutdownHandle> {
    let ServeCommand::ByParams { host, port } = self;
    let setting_service = service.setting_service();
    
    // 1. Create server handle with lifecycle coordination
    let ServerHandle { server, shutdown, ready_rx } = build_server_handle(host, *port);

    // 2. Validate executable path for LLM server
    let exec_path = service.setting_service().exec_path_from();
    if !exec_path.exists() {
      error!("exec not found at {}", exec_path.to_string_lossy());
      return Err(ContextError::ExecNotExists(exec_path.to_string_lossy().to_string()))?;
    }

    // 3. Initialize SharedContext with service coordination
    let ctx = DefaultSharedContext::new(service.hub_service(), service.setting_service());
    let ctx: Arc<dyn SharedContext> = Arc::new(ctx);
    
    // 4. Register advanced listeners for real-time coordination
    setting_service.add_listener(Arc::new(VariantChangeListener::new(ctx.clone())));
    let keep_alive = Arc::new(ServerKeepAlive::new(ctx.clone(), setting_service.keep_alive()));
    setting_service.add_listener(keep_alive.clone());
    ctx.add_state_listener(keep_alive).await;

    // 5. Create static router with environment-specific configuration
    let static_router = static_dir.map(|dir| {
      let static_service = ServeDir::new(dir).append_index_html_on_directories(true);
      Router::new().fallback_service(static_service)
    });

    // 6. Build complete route composition with middleware stack
    let app = build_routes(ctx.clone(), service, static_router);
    
    // 7. Start server with graceful shutdown coordination
    let join_handle = tokio::spawn(async move {
      let callback = Box::new(ShutdownContextCallback { ctx });
      server.start_new(app, Some(callback)).await
    });
    
    // 8. Wait for ready notification and provide server URL information
    ready_rx.await?;
    let server_url = format!("{}://{host}:{port}", setting_service.scheme());
    let public_url = setting_service.public_server_url();
    println!("server started on server_url={server_url}, public_url={public_url}");
    
    Ok(ServerShutdownHandle { join_handle, shutdown })
  }
}
```

### Static Asset Serving Architecture
Dynamic UI serving with environment-specific configuration:

```rust
// Static asset serving pattern with environment detection
let static_router = static_dir.map(|dir| {
  let static_service = ServeDir::new(dir).append_index_html_on_directories(true);
  Router::new().fallback_service(static_service)
});

// Integration with routes_app for complete HTTP stack
let app = build_routes(ctx.clone(), service, static_router);
```

**Service Bootstrap Features**:
- Complete AppService registry initialization with all 10 business services
- SharedContext bootstrap with HubService and SettingService coordination for LLM server management
- Advanced listener registration with ServerKeepAlive and VariantChangeListener for real-time coordination
- Static asset serving with development proxy support and production embedded assets
- Route composition integration with routes_app for complete HTTP middleware stack

## Cross-Platform Signal Handling Implementation

### Signal Coordination Architecture
Comprehensive signal handling with cross-platform support:

```rust
// Signal handling pattern (see crates/server_app/src/shutdown.rs for complete implementation)
pub async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
    tracing::info!("received Ctrl+C, stopping server");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
    tracing::info!("received SIGTERM, stopping server");
  };

  #[cfg(not(unix))]
  let terminate = async {
    signal::windows::ctrl_break()
      .expect("failed to install Ctrl+Break handler")
      .recv()
      .await;
    tracing::info!("received Ctrl+Break, stopping server");
  };

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}
```

**Signal Handling Features**:
- Cross-platform signal support (Unix SIGTERM, Windows Ctrl+Break) with proper signal registration
- Graceful shutdown coordination with resource cleanup and service termination
- Signal handler installation with error handling and fallback mechanisms
- Integration with server lifecycle for clean shutdown coordination

## Error Handling Architecture

### Server-Specific Error Types
Comprehensive error handling with domain-specific error types:

```rust
// Server error types (see crates/server_app/src/error.rs for TaskJoinError, crates/server_app/src/serve.rs for ServeError)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("task_join_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct TaskJoinError {
  #[from]
  source: JoinError,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServeError {
  #[error(transparent)]
  Setting(#[from] SettingServiceError),
  #[error(transparent)]
  Join(#[from] TaskJoinError),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  Server(#[from] ServerError),
  #[error("unknown")]
  #[error_meta(error_type = ErrorType::Unknown)]
  Unknown,
}
```

### Error Coordination Patterns
Sophisticated error handling across service boundaries:

```rust
// Error handling with transparent wrapping and context preservation
impl_error_from!(tokio::task::JoinError, ServeError::Join, TaskJoinError);
impl_error_from!(::std::io::Error, ServerError::Io, ::objs::IoError);
```

**Error Handling Features**:
- Transparent error wrapping preserves original service error context
- Server-specific error types with user-friendly messages via errmeta_derive integration
- Comprehensive error boundaries for server lifecycle, service bootstrap, and listener operations
- Error isolation prevents listener failures from interrupting server operation

## Extension Guidelines for Server Orchestration

### Adding New Server Lifecycle Components
When creating new server lifecycle management features:

1. **Listener Pattern Integration**: Implement SettingsChangeListener or ServerStateListener for real-time configuration and state management
2. **Service Bootstrap Extensions**: Coordinate with AppService registry for new service initialization and dependency injection
3. **Shutdown Callback Integration**: Implement ShutdownCallback trait for proper resource cleanup during graceful shutdown
4. **Error Handling**: Create server-specific errors that implement AppError trait for consistent error reporting and recovery
5. **Testing Infrastructure**: Use comprehensive service mocking for isolated server lifecycle testing scenarios

### Extending Configuration Management
For new configuration and settings management patterns:

1. **Settings Listener Integration**: Implement SettingsChangeListener for real-time configuration updates without service restart
2. **Environment Validation**: Add configuration validation during server bootstrap with clear error reporting
3. **Service Coordination**: Coordinate configuration changes across service boundaries with proper error handling
4. **Dynamic Updates**: Support runtime configuration updates through listener pattern with validation and rollback
5. **Configuration Testing**: Test configuration management with different environment scenarios and validation failures

## Server Testing Architecture

### Server Lifecycle Testing Patterns
Comprehensive testing of server orchestration and lifecycle management:

```rust
// Server testing pattern (see crates/server_app/src/serve.rs for complete test implementation)
#[rstest]
#[tokio::test]
async fn test_server_fails_when_port_already_in_use(temp_dir: TempDir) -> anyhow::Result<()> {
  // Bind to a random port first to ensure it's in use
  let port = 8000 + (rand::random::<u16>() % 1000);
  let host = "127.0.0.1";
  let addr = format!("{}:{}", host, port);
  let _listener = TcpListener::bind(&addr).await?;
  
  let app_service = Arc::new(
    AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_session_service().await
      .with_db_service().await
      .build().unwrap(),
  );
  
  let serve_command = ServeCommand::ByParams { host: host.to_string(), port };
  let result = serve_command.get_server_handle(app_service, None).await;
  
  assert!(result.is_err());
  match result.unwrap_err() {
    ServeError::Server(ServerError::Io(_)) => {
      // Expected - binding to port should fail
    }
    other => panic!("Expected IO error for port conflict, got: {:?}", other),
  }
  Ok(())
}
```

### Listener Integration Testing
Advanced listener pattern testing with service coordination:

```rust
// Listener testing patterns with comprehensive mock coordination
#[rstest]
#[tokio::test]
async fn test_keep_alive_setting_changes_starts_timer(#[case] from: i64, #[case] to: i64) {
  let mut mock_ctx = MockSharedContext::new();
  mock_ctx.expect_stop().never();

  let keep_alive = ServerKeepAlive::new(Arc::new(mock_ctx), from);
  keep_alive.on_state_change(ServerState::Start).await;

  keep_alive.on_change(
    BODHI_KEEP_ALIVE_SECS,
    &Some(Value::Number(from.into())),
    &SettingSource::SettingsFile,
    &Some(Value::Number(to.into())),
    &SettingSource::SettingsFile,
  );
  
  tokio::time::sleep(Duration::from_millis(100)).await;
  assert_eq!(*keep_alive.keep_alive.read().unwrap(), to);
  assert!(keep_alive.timer_handle.read().unwrap().is_some());
}
```

**Testing Infrastructure Features**:
- Server lifecycle testing with AppServiceStubBuilder for comprehensive service mocking
- Listener pattern testing with MockSharedContext for isolated listener behavior validation
- Error scenario testing with port conflicts, service failures, and configuration errors
- Integration testing with realistic service interactions and resource management

## Commands

**Testing**: `cargo test -p server_app` (includes server lifecycle and listener integration tests)  
**Building**: Standard `cargo build -p server_app`  
**Server Testing**: `cargo test -p server_app --features test-utils` (includes comprehensive server testing infrastructure)