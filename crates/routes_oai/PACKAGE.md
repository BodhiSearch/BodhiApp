# PACKAGE.md - routes_oai

This document provides detailed technical information for the `routes_oai` crate, focusing on BodhiApp's OpenAI API compatibility implementation patterns, sophisticated streaming capabilities, and dual-format API support architecture.

## Crate Structure

**Main Library**: `crates/routes_oai/src/lib.rs:1-25`
- Module exports for chat, models, and Ollama routes
- API endpoint constants for OpenAI and Ollama compatibility
- Localization resources integration

**Core Route Modules**:
- `crates/routes_oai/src/routes_chat.rs:1-303` - OpenAI chat completions endpoint with streaming
- `crates/routes_oai/src/routes_oai_models.rs:1-427` - OpenAI models API with alias resolution
- `crates/routes_oai/src/routes_ollama.rs:1-765` - Ollama compatibility layer with format translation

**Dependencies** (`crates/routes_oai/Cargo.toml:6-27`):
- `async-openai` for OpenAI type compatibility
- `server_core` for RouterState and HTTP infrastructure
- `services` for business logic coordination
- `objs` for domain objects and error handling

## OpenAI API Compatibility Architecture

### Chat Completions Implementation

Core chat completion handler with streaming support:

```rust
// Chat completion endpoint (crates/routes_oai/src/routes_chat.rs:109-121)
pub async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<CreateChatCompletionRequest>, ApiError>,
) -> Result<Response, ApiError> {
  let response = state.chat_completions(request).await?;
  let mut response_builder = Response::builder().status(response.status());
  if let Some(headers) = response_builder.headers_mut() {
    *headers = response.headers().clone();
  }
  let stream = response.bytes_stream();
  let body = Body::from_stream(stream);
  Ok(response_builder.body(body).map_err(HttpError::Http)?)
}
```

**Key Features**:
- Complete OpenAI parameter validation using `async-openai` types (`crates/routes_oai/src/routes_chat.rs:111`)
- Streaming and non-streaming response support with proper SSE formatting
- Request forwarding through RouterState to SharedContext for LLM coordination (`crates/routes_oai/src/routes_chat.rs:113`)
- Comprehensive OpenAPI documentation with examples (`crates/routes_oai/src/routes_chat.rs:18-108`)

### Models API Implementation

OpenAI-compatible model discovery with alias resolution:

```rust
// Model listing handler (crates/routes_oai/src/routes_oai_models.rs:127-170)
pub async fn oai_models_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<OAIModelListResponse>, ApiError> {
  let aliases = state
    .app_service()
    .data_service()
    .list_aliases()
    .await
    .map_err(ApiError::from)?;

  let mut seen_models = HashSet::new();
  let mut models = Vec::new();

  // Process aliases in priority order: User > Model > API
  for alias in aliases {
    match alias {
      Alias::User(user_alias) => {
        if seen_models.insert(user_alias.alias.clone()) {
          models.push(user_alias_to_oai_model(state.clone(), user_alias));
        }
      }
      // Additional alias handling...
    }
  }

  Ok(Json(OAIModelListResponse {
    object: "list".to_string(),
    data: models,
  }))
}
```

**Model Conversion Functions**:
- User alias conversion: `crates/routes_oai/src/routes_oai_models.rs:239-249`
- Model alias conversion: `crates/routes_oai/src/routes_oai_models.rs:251-267`  
- API alias conversion: `crates/routes_oai/src/routes_oai_models.rs:269-277`

## Ollama Ecosystem Integration Architecture

### Request Format Translation

Sophisticated bidirectional format conversion between Ollama and OpenAI:

```rust
// Ollama to OpenAI request conversion (crates/routes_oai/src/routes_ollama.rs:345-368)
impl From<ChatRequest> for CreateChatCompletionRequest {
  fn from(val: ChatRequest) -> Self {
    let options = val.options.unwrap_or_default();
    CreateChatCompletionRequest {
      messages: val
        .messages
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<_>>(),
      model: val.model,
      frequency_penalty: options.frequency_penalty,
      max_completion_tokens: options.num_predict,
      temperature: options.temperature,
      top_p: options.top_p,
      stop: options.stop.map(Stop::StringArray),
      stream: val.stream,
      ..Default::default()
    }
  }
}
```

### Response Format Translation

OpenAI to Ollama response conversion with analytics placeholders:

```rust
// OpenAI to Ollama response conversion (crates/routes_oai/src/routes_ollama.rs:387-412)
impl From<CreateChatCompletionResponse> for ChatResponse {
  fn from(response: CreateChatCompletionResponse) -> Self {
    let first = response.choices.first();
    let message = first
      .map(|choice| choice.message.clone().into())
      .unwrap_or_default();
    let done_reason = first.map(|choice| choice.finish_reason).unwrap_or(None);
    let done = done_reason.is_some();
    let usage = response.usage;

    ChatResponse {
      model: response.model,
      created_at: response.created,
      message,
      done_reason,
      done,
      // Analytics placeholders for future implementation
      total_duration: 0 as f64,
      load_duration: "-1".to_string(),
      prompt_eval_count: usage.as_ref().map(|u| u.prompt_tokens as i32).unwrap_or(-1),
      eval_count: usage.map(|u| u.completion_tokens as i32).unwrap_or(-1),
    }
  }
}
```

### Ollama Models Integration

Model metadata conversion with filesystem integration:

```rust
// User alias to Ollama model conversion (crates/routes_oai/src/routes_ollama.rs:123-146)
fn user_alias_to_ollama_model(state: Arc<dyn RouterState>, alias: UserAlias) -> Model {
  let bodhi_home = &state.app_service().setting_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
  let created = fs::metadata(path)
    .map_err(|e| e.to_string())
    .and_then(|m| m.created().map_err(|e| e.to_string()))
    .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
    .unwrap_or_default()
    .as_secs() as u32;
  Model {
    model: alias.alias,
    modified_at: created,
    size: 0,
    digest: alias.snapshot,
    details: ModelDetails {
      parent_model: None,
      format: GGUF.to_string(),
      family: "unknown".to_string(),
      // Additional model metadata fields
    },
  }
}
```

## HTTP Route Orchestration Implementation

### Error Handling Architecture

Comprehensive error handling with API-specific response formatting:

```rust
// HTTP error types with localization (crates/routes_oai/src/routes_chat.rs:9-15)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HttpError {
  #[error("http_error")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),
}
```

**Error Resources**: `crates/routes_oai/src/resources/en-US/messages.ftl:1`
- Localized error messages for HTTP errors
- Integration with objs error system for consistent responses

### Streaming Response Coordination

Advanced streaming with format-specific transformation:

```rust
// Ollama streaming transformation (crates/routes_oai/src/routes_ollama.rs:644-669)
let stream = response.bytes_stream().map(move |chunk| {
  let chunk = chunk.map_err(|e| format!("error reading chunk: {e}"))?;
  let text = String::from_utf8_lossy(&chunk);

  if text.starts_with("data: ") {
    let msg = text
      .strip_prefix("data: ")
      .unwrap()
      .strip_suffix("\n\n")
      .unwrap();

    let oai_chunk: CreateChatCompletionStreamResponse =
      serde_json::from_str(msg).map_err(|e| format!("error parsing chunk: {e}"))?;

    let data: ResponseStream = oai_chunk.into();
    serde_json::to_string(&data)
      .map(|s| format!("data: {s}\n\n"))
      .map_err(|e| format!("error serializing chunk: {e}"))
  } else {
    Ok(text.into_owned())
  }
});
```

## Cross-Crate Integration Implementation

### Service Layer Coordination

Routes coordinate extensively with BodhiApp's service architecture:

```rust
// DataService integration for model operations (crates/routes_oai/src/routes_oai_models.rs:131-136)
let aliases = state
  .app_service()
  .data_service()
  .list_aliases()
  .await
  .map_err(ApiError::from)?;

// Model alias resolution with error handling (crates/routes_oai/src/routes_oai_models.rs:221)
if let Some(alias) = state.app_service().data_service().find_alias(&id).await {
  // Handle resolved alias
} else {
  Err(ApiError::from(AliasNotFoundError(id)))
}
```

### Domain Object Integration

Extensive use of objs crate throughout API operations:

```rust
// Domain object imports (crates/routes_oai/src/routes_oai_models.rs:7)
use objs::{Alias, ApiAlias, ApiError, ModelAlias, UserAlias, OpenAIApiError};

// Error system integration (crates/routes_oai/src/routes_chat.rs:5)
use objs::{ApiError, AppError, ErrorType, OpenAIApiError};
```

## API Testing Architecture

### OpenAI Compatibility Testing

Comprehensive testing with OpenAI SDK validation:

```rust
// Non-streaming chat completion test (crates/routes_oai/src/routes_chat.rs:166-211)
#[rstest]
#[awt]
#[tokio::test]
async fn test_routes_chat_completions_non_stream(
  #[future] app_service_stub: AppServiceStub,
) -> anyhow::Result<()> {
  let request = CreateChatCompletionRequestArgs::default()
    .model("testalias-exists:instruct")
    .messages(vec![ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessageArgs::default()
        .content("What day comes after Monday?")
        .build()?,
    )])
    .build()?;
  
  let response = app.oneshot(Request::post("/v1/chat/completions").json(request)?).await?;
  assert_eq!(StatusCode::OK, response.status());
  
  let result: CreateChatCompletionResponse = response.json().await?;
  assert_eq!(
    "The day that comes after Monday is Tuesday.",
    result.choices.first().unwrap().message.content.as_ref().unwrap()
  );
}
```

### Streaming Response Testing

Advanced streaming validation with SSE format testing:

```rust
// Streaming response test (crates/routes_oai/src/routes_chat.rs:253-301)
#[rstest]
#[awt]
#[tokio::test]
async fn test_routes_chat_completions_stream(
  #[future] app_service_stub: AppServiceStub,
) -> anyhow::Result<()> {
  let request = CreateChatCompletionRequestArgs::default()
    .stream(true)
    .build()?;
  
  let response = app.oneshot(Request::post("/v1/chat/completions").json(request)?).await?;
  let response: Vec<CreateChatCompletionStreamResponse> = response.sse().await?;
  let content = response.into_iter().fold(String::new(), |mut f, r| {
    let content = r.choices.first().unwrap().delta.content.as_deref().unwrap_or_default();
    f.push_str(content);
    f
  });
  
  assert_eq!("  After Monday, the next day is Tuesday.", content);
}
```

### Ollama Ecosystem Testing

Ollama compatibility validation:

```rust
// Ollama model listing test (crates/routes_oai/src/routes_ollama.rs:699-723)
#[rstest]
#[awt]
#[tokio::test]
async fn test_ollama_routes_models_list(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = app
    .oneshot(Request::get("/api/tags").body(Body::empty())?)
    .await?
    .json::<Value>()
    .await?;
  
  assert!(response["models"].as_array().length().unwrap() >= 6);
  let llama3 = response["models"]
    .as_array()
    .unwrap()
    .iter()
    .find(|item| item["model"] == "llama3:instruct")
    .unwrap();
  assert_eq!("5007652f7a641fe7170e0bad4f63839419bd9213", llama3["digest"]);
}
```

## Extension Guidelines

### Adding New OpenAI Endpoints

When implementing additional OpenAI API endpoints:

1. **Specification Compliance**: Use `async-openai` types for exact OpenAI parameter validation and response formatting
2. **Service Coordination**: Implement RouterState integration for consistent AppService access and business logic coordination
3. **Error Handling**: Create API-specific error types that convert to OpenAI-compatible responses with proper status codes
4. **Documentation**: Add comprehensive `utoipa` OpenAPI documentation with examples and proper schemas
5. **Testing**: Create compatibility tests using OpenAI SDK types for validation

### Extending Ollama Compatibility

For new Ollama API endpoints:

1. **Format Translation**: Design bidirectional `From` implementations between Ollama and OpenAI formats
2. **Parameter Mapping**: Create semantic parameter mapping preserving functionality across API specifications
3. **Response Formatting**: Implement Ollama-specific response types with proper field mapping and serialization
4. **Ecosystem Integration**: Validate compatibility with Ollama CLI tools and ecosystem integrations
5. **Testing**: Add Ollama ecosystem compatibility tests with format validation

### Performance Optimization Patterns

For efficient API operations:

1. **Stream Proxying**: Use `Body::from_stream()` for efficient streaming without intermediate buffering
2. **Alias Deduplication**: Use `HashSet` for efficient model deduplication in listing operations
3. **Format Abstraction**: Create unified internal processing supporting both API formats
4. **Error Boundaries**: Implement proper error handling for streaming and non-streaming contexts
5. **Resource Management**: Coordinate with TimeService for efficient metadata operations

## Commands

**Full Test Suite**: `cargo test -p routes_oai`  
**Chat Endpoints**: `cargo test -p routes_oai routes_chat`  
**Model Endpoints**: `cargo test -p routes_oai routes_oai_models`  
**Ollama Compatibility**: `cargo test -p routes_oai routes_ollama`  
**Streaming Tests**: `cargo test -p routes_oai stream`  
**Format Validation**: `cargo test -p routes_oai --test integration`