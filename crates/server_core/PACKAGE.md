# PACKAGE.md - Server Core Crate

*For architectural insights and design decisions, see [crates/server_core/CLAUDE.md](crates/server_core/CLAUDE.md)*

## Implementation Index

The `server_core` crate provides BodhiApp's HTTP infrastructure orchestration layer with sophisticated streaming capabilities, LLM server context management, and intelligent request routing.

### Core Infrastructure Files

**src/lib.rs**
- Module structure and feature-gated test utilities
- Public API exports for HTTP infrastructure components
- L10n resources inclusion for error localization

**src/router_state.rs**
- RouterState trait and DefaultRouterState implementation
- Service registry integration for HTTP route handlers
- Model routing coordination with SharedContext and AiApiService
- Error handling with RouterStateError mapping

**src/shared_rw.rs**
- SharedContext trait for LLM server lifecycle management
- DefaultSharedContext implementation with state synchronization
- ModelLoadStrategy for efficient model switching
- Observer pattern for server state notifications

**src/model_router.rs**
- ModelRouter trait for intelligent request routing
- DefaultModelRouter implementation with DataService integration
- RouteDestination enum for local vs remote routing decisions
- Priority-based alias resolution with comprehensive error handling

**src/direct_sse.rs**
- DirectSSE implementation for application-generated events
- DirectEvent builder pattern with BytesMut optimization
- KeepAlive mechanism for connection stability
- High-performance streaming with memory efficiency

**src/fwd_sse.rs**
- RawSSE implementation for LLM server response proxying
- Channel-based streaming with tokio integration
- ForwardedSSE utilities for proxy streaming scenarios
- Simplified streaming interface for external service integration

**src/server_args_merge.rs**
- Advanced server argument merging with three-tier precedence
- Sophisticated flag parsing with negative number detection
- HashMap-based deduplication for configuration coordination
- Support for complex llama.cpp argument patterns

### HTTP Infrastructure Examples

#### RouterState Service Coordination

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

  impl RouterState for DefaultRouterState {
    fn chat_completions(
      &self,
      request: CreateChatCompletionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>> {
      Box::pin(async move {
        // Use ModelRouter to determine routing destination
        let destination = self.model_router().route_request(&request.model).await?;

        match destination {
          RouteDestination::Local(alias) => {
            // Route to local model via SharedContext
            let response = self.ctx.chat_completions(request, alias).await?;
            Ok(response)
          }
          RouteDestination::Remote(api_alias) => {
            // Route to remote API via AiApiService
            let axum_response = self
              .ai_api_service()
              .forward_chat_completion(&api_alias.id, request)
              .await?;
            // Convert axum::Response to reqwest::Response
            Ok(reqwest::Response::from(axum_response))
          }
        }
      })
    }
  }
```

#### Model Request Routing

```rust
  #[async_trait]
  pub trait ModelRouter: Send + Sync + std::fmt::Debug {
    /// Route a model request to the appropriate destination
    /// Resolution order: 1. User alias, 2. Model alias, 3. API models
    async fn route_request(&self, model: &str) -> Result<RouteDestination>;
  }

  /// Represents the destination for a model request
  #[derive(Debug, Clone)]
  pub enum RouteDestination {
    /// Route to local model via existing SharedContext
    Local(Alias),
    /// Route to remote API via AiApiService
    Remote(ApiAlias),
  }

  #[derive(Debug, Clone)]
  pub struct DefaultModelRouter {
    data_service: Arc<dyn DataService>,
  }

  #[async_trait]
  impl ModelRouter for DefaultModelRouter {
    async fn route_request(&self, model: &str) -> Result<RouteDestination> {
      // Use unified DataService to find alias across all types
      if let Some(alias) = self.data_service.find_alias(model).await {
        match alias {
          // Local models route to SharedContext
          Alias::User(_) | Alias::Model(_) => Ok(RouteDestination::Local(alias)),
          // Remote API models route to AiApiService
          Alias::Api(api_alias) => Ok(RouteDestination::Remote(api_alias)),
        }
      } else {
        Err(ModelRouterError::ApiModelNotFound(model.to_string()))
      }
    }
  }
```

## Server-Sent Events Architecture

### DirectSSE for Application Events

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

### RawSSE for Service Proxying

```rust
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

## SharedContext LLM Server Management

### Server Lifecycle Coordination

```rust
  #[derive(Debug, Clone)]
  pub enum ServerState {
    Start,
    Stop,
    ChatCompletions { alias: String },
    Variant { variant: String },
  }

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
```

### Model Load Strategy Implementation

```rust
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

### SharedContext Implementation

```rust
  pub struct DefaultSharedContext {
    hub_service: Arc<dyn HubService>,
    setting_service: Arc<dyn SettingService>,
    factory: Box<dyn ServerFactory>,
    exec_variant: ExecVariant,
    server: RwLock<Option<Box<dyn Server>>>,
    state_listeners: RwLock<Vec<Arc<dyn ServerStateListener>>>,
  }

  #[async_trait::async_trait]
  impl SharedContext for DefaultSharedContext {
    async fn chat_completions(
      &self,
      mut request: CreateChatCompletionRequest,
      alias: Alias,
    ) -> Result<reqwest::Response> {
      // Extract alias information and reject API aliases
      let (alias_name, repo, filename, snapshot, context_params) = match &alias {
        Alias::User(user_alias) => (
          &user_alias.alias,
          &user_alias.repo,
          &user_alias.filename,
          &user_alias.snapshot,
          &user_alias.context_params,
        ),
        Alias::Model(model_alias) => (
          &model_alias.alias,
          &model_alias.repo,
          &model_alias.filename,
          &model_alias.snapshot,
          &Vec::<String>::new(),
        ),
        Alias::Api(_) => {
          return Err(ContextError::Unreachable(
            "API aliases cannot be processed by SharedContext".to_string(),
          ));
        }
      };

      let lock = self.server.read().await;
      let server = lock.as_ref();
      let loaded_alias = server.map(|server| server.get_server_args().alias);

      // Apply request parameters if this is a user alias
      if let Alias::User(user_alias) = &alias {
        user_alias.request_params.update(&mut request);
      }

      let result = match ModelLoadStrategy::choose(loaded_alias, alias_name) {
        ModelLoadStrategy::Continue => {
          let response = server
            .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
            .chat_completions(&serde_json::to_value(request)?)
            .await?;
          Ok(response)
        }
        ModelLoadStrategy::DropAndLoad | ModelLoadStrategy::Load => {
          drop(lock);
          let variant = self.exec_variant.get().await;
          let setting_args = self.get_setting_args();
          let setting_variant_args = self.get_setting_variant_args(&variant);
          let merged_args = merge_server_args(&setting_args, &setting_variant_args, context_params);

          let server_args = LlamaServerArgsBuilder::default()
            .alias(alias_name.clone())
            .model(model_file.to_string_lossy().to_string())
            .server_args(merged_args)
            .build()?;

          self.reload(Some(server_args)).await?;
          let lock = self.server.read().await;
          let server = lock.as_ref();
          let response = server
            .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
            .chat_completions(&serde_json::to_value(request)?)
            .await?;
          Ok(response)
        }
      };

      self
        .notify_state_listeners(ServerState::ChatCompletions {
          alias: alias_name.clone(),
        })
        .await;
      result
    }
  }
```

## Server Arguments Merging System

### Three-Tier Precedence Configuration

```rust
  pub fn merge_server_args(
    setting_args: &[String],
    setting_variant_args: &[String],
    alias_args: &[String],
  ) -> Vec<String> {
    let mut arg_map: HashMap<String, Option<String>> = HashMap::new();

    // Parse and add base args first (lowest precedence)
    let parsed_setting_args = parse_args_from_strings(setting_args);
    for (key, value) in parsed_setting_args {
      arg_map.insert(key, value);
    }

    // Parse and add variant-specific args (medium precedence)
    let parsed_variant_args = parse_args_from_strings(setting_variant_args);
    for (key, value) in parsed_variant_args {
      arg_map.insert(key, value);
    }

    // Parse and add alias args (highest precedence)
    let parsed_alias_args = parse_args_from_strings(alias_args);
    for (key, value) in parsed_alias_args {
      arg_map.insert(key, value);
    }

    // Convert back to Vec<String> maintaining flag format
    arg_map
      .into_iter()
      .map(|(key, value)| match value {
        Some(val) => format!("{} {}", key, val),
        None => key,
      })
      .collect()
  }
```

### Advanced Flag Parsing

```rust
  fn is_flag(token: &str) -> bool {
    if !token.starts_with('-') {
      return false;
    }

    if token.len() == 1 || token.starts_with("--") {
      return token.starts_with("--");
    }

    // Single dash: distinguish flags from negative numbers
    let second_char = token.chars().nth(1).unwrap();
    if second_char.is_ascii_digit() {
      let number_part = &token[1..];
      number_part.parse::<i64>().is_err() && number_part.parse::<f64>().is_err()
    } else {
      true
    }
  }

  fn parse_args_from_strings(args: &[String]) -> Vec<(String, Option<String>)> {
    let mut result = Vec::new();
    let mut tokens = Vec::new();

    // Flatten all argument strings into tokens
    for arg_string in args {
      tokens.extend(split_respecting_quotes(arg_string));
    }

    let mut i = 0;
    while i < tokens.len() {
      let token = &tokens[i];

      if is_flag(token) {
        if i + 1 < tokens.len() && !is_flag(&tokens[i + 1]) {
          // Flag with value
          result.push((token.clone(), Some(tokens[i + 1].clone())));
          i += 2;
        } else {
          // Boolean flag
          result.push((token.clone(), None));
          i += 1;
        }
      } else {
        i += 1;
      }
    }

    result
  }
```

## Error Handling Architecture

### HTTP Error Translation

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
    #[error(transparent)]
    ModelRouter(#[from] ModelRouterError),
    #[error(transparent)]
    AiApiService(#[from] AiApiServiceError),
  }

  #[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
  #[error_meta(trait_to_impl = objs::AppError)]
  pub enum ContextError {
    #[error("server_not_loaded")]
    #[error_meta(error_type = objs::ErrorType::InternalServerError)]
    ServerNotLoaded,

    #[error("executable_not_exists")]
    #[error_meta(error_type = objs::ErrorType::InternalServerError)]
    ExecNotExists(String),

    #[error(transparent)]
    Server(#[from] llama_server_proc::ServerError),

    #[error("json_error")]
    #[error_meta(error_type = objs::ErrorType::InternalError)]
    JsonError(#[from] serde_json::Error),

    #[error("unreachable_state")]
    #[error_meta(error_type = objs::ErrorType::InternalError)]
    Unreachable(String),
  }
```

## Testing Infrastructure

### Service Mock Coordination

```rust
  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_delegate_to_context_with_alias(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_ctx = MockSharedContext::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias-exists:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;

    mock_ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(Alias::User(UserAlias::testalias_exists())),
      )
      .times(1)
      .return_once(|_, _| Ok(mock_response("")));

    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_hub_service()
      .with_data_service()
      .await
      .build()?;

    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    state.chat_completions(request).await?;
    Ok(())
  }
```

### SharedContext Testing

```rust
  #[rstest]
  #[tokio::test]
  async fn test_chat_completions_continue_strategy(
    mut mock_server: MockServer,
    mut app_service_stub_builder: AppServiceStubBuilder,
    bin_path: TempDir,
  ) -> anyhow::Result<()> {
    let expected_input: Value = serde_json::from_str(
      r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct"}"#,
    )?;

    mock_server
      .expect_chat_completions()
      .with(eq(expected_input))
      .times(1)
      .return_once(|_| async { Ok(mock_response("")) }.boxed());

    let server_args = LlamaServerArgsBuilder::default()
      .alias("testalias:instruct")
      .model(model_file.path())
      .build()?;

    let shared_ctx = DefaultSharedContext::with_args(
      app_service_stub.hub_service(),
      app_service_stub.setting_service(),
      Box::new(ServerFactoryStub::new(Box::new(mock_server))),
    );

    shared_ctx.reload(Some(server_args)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;

    shared_ctx
      .chat_completions(request, Alias::User(UserAlias::testalias()))
      .await?;

    Ok(())
  }
```

## Build Commands

```bash
# Test server_core crate
cargo test -p server_core

# Test with features
cargo test -p server_core --features test-utils

# Build server_core crate
cargo build -p server_core

# Run clippy
cargo clippy -p server_core

# Format code
cargo fmt -p server_core
```

## Extension Patterns

### Adding New HTTP Streaming Capabilities

1. Choose appropriate SSE type (DirectSSE vs RawSSE) based on use case
2. Implement proper connection lifecycle management with cleanup
3. Use BytesMut for efficient memory management in DirectSSE
4. Add comprehensive error handling with graceful degradation
5. Test with realistic network conditions and client behavior

### Extending SharedContext

1. Design proper async lifecycle coordination for new operations
2. Implement observer pattern for state change notifications
3. Ensure thread-safe state management with RwLock coordination
4. Add comprehensive error handling with context preservation
5. Support comprehensive testing with server factory mocking

### Router State Integration

1. Coordinate with AppService registry for business logic access
2. Implement proper error translation to HTTP responses
3. Design efficient request processing with service coordination
4. Add dependency injection patterns for HTTP handlers
5. Support comprehensive HTTP integration testing

## Recent Architecture Changes

### Model Router Enhancement
- Intelligent request routing with local vs remote API detection
- Priority-based alias resolution (User > Model > API aliases)
- Comprehensive error handling for routing failures
- Support for unified alias resolution across all alias types

### Server Arguments Evolution
- Advanced three-tier precedence system for configuration merging
- Sophisticated flag parsing with negative number detection
- Support for complex llama.cpp argument patterns
- Cross-string boundary parsing for flexible configuration

### Enhanced Error Coordination
- Comprehensive error translation from services to HTTP responses
- Context-aware error handling with proper HTTP status mapping
- Transparent error wrapping with service context preservation
- Localized error messages with errmeta_derive integration