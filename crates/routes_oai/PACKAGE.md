# PACKAGE.md - routes_oai

This document provides detailed technical information for the `routes_oai` crate, focusing on BodhiApp's OpenAI API compatibility implementation patterns, sophisticated streaming capabilities, and dual-format API support architecture.

## OpenAI API Compatibility Architecture

The `routes_oai` crate serves as BodhiApp's **OpenAI API compatibility orchestration layer**, implementing comprehensive OpenAI-compatible endpoints with sophisticated streaming, Ollama ecosystem integration, and multi-format API support.

### Chat Completions Implementation
Advanced chat completion endpoint with streaming and non-streaming support:

```rust
// Core chat completion pattern (see src/routes_chat.rs:45-67 for complete implementation)
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_CHAT_COMPLETIONS,
    tag = API_TAG_OPENAI,
    operation_id = "createChatCompletion",
    // Comprehensive OpenAPI documentation with examples
)]
pub async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<CreateChatCompletionRequest>, ApiError>,
) -> Result<Response, ApiError> {
  // 1. Request forwarding through RouterState to SharedContext
  let response = state.chat_completions(request).await?;
  
  // 2. Response streaming coordination with proper headers
  let mut response_builder = Response::builder().status(response.status());
  if let Some(headers) = response_builder.headers_mut() {
    *headers = response.headers().clone();
  }
  
  // 3. Stream proxying for both streaming and non-streaming responses
  let stream = response.bytes_stream();
  let body = Body::from_stream(stream);
  Ok(response_builder.body(body).map_err(HttpError::Http)?)
}
```

**Chat Completion Features**:
- Complete OpenAI parameter validation using `async-openai` types for ecosystem compatibility
- Streaming and non-streaming response support with proper SSE formatting
- Request forwarding through RouterState to SharedContext for LLM server coordination
- Comprehensive error handling with OpenAI-compatible error responses and HTTP status codes
- Response proxying with proper header management and stream coordination

### Models API Implementation
OpenAI-compatible model discovery and metadata endpoints:

```rust
// Model listing pattern (see src/routes_oai_models.rs:67-89 for complete implementation)
pub async fn oai_models_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<OAIModelListResponse>, ApiError> {
  let models = state
    .app_service()
    .data_service()
    .list_aliases()?
    .into_iter()
    .map(|alias| to_oai_model(state.clone(), alias))
    .collect::<Vec<_>>();
  Ok(Json(OAIModelListResponse {
    object: "list".to_string(),
    data: models,
  }))
}

// Model metadata conversion (see src/routes_oai_models.rs:134-145)
fn to_oai_model(state: Arc<dyn RouterState>, alias: Alias) -> OAIModel {
  let bodhi_home = &state.app_service().setting_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
  let created = state.app_service().time_service().created_at(&path);
  OAIModel {
    id: alias.alias,
    object: "model".to_string(),
    created,
    owned_by: "system".to_string(),
  }
}
```

**Models API Features**:
- DataService integration for model alias discovery and listing
- OpenAI-compatible model object formatting with metadata
- Individual model endpoint support with alias resolution
- TimeService integration for model creation timestamps
- Comprehensive error handling for missing models with localized messages

## Ollama Ecosystem Integration Architecture

### Dual-Format API Support
Sophisticated request/response translation between OpenAI and Ollama formats:

```rust
// Ollama chat request translation (see src/routes_ollama.rs:156-178 for complete implementation)
impl From<ChatRequest> for CreateChatCompletionRequest {
  fn from(val: ChatRequest) -> Self {
    let options = val.options.unwrap_or_default();
    CreateChatCompletionRequest {
      messages: val.messages.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
      model: val.model,
      frequency_penalty: options.frequency_penalty,
      max_completion_tokens: options.num_predict,
      temperature: options.temperature,
      top_p: options.top_p,
      stop: options.stop.map(Stop::StringArray),
      stream: val.stream,
      // Parameter mapping with semantic preservation
      ..Default::default()
    }
  }
}

// Ollama response conversion (see src/routes_ollama.rs:189-212)
impl From<CreateChatCompletionResponse> for ChatResponse {
  fn from(response: CreateChatCompletionResponse) -> Self {
    let first = response.choices.first();
    let message = first.map(|choice| choice.message.clone().into()).unwrap_or_default();
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
      // Additional Ollama-specific fields
    }
  }
}
```

### Ollama Models Integration
Ollama-compatible model listing and information endpoints:

```rust
// Ollama model conversion (see src/routes_ollama.rs:67-89 for complete implementation)
fn to_ollama_model(state: Arc<dyn RouterState>, alias: Alias) -> Model {
  let bodhi_home = &state.app_service().setting_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
  let created = fs::metadata(path)
    .and_then(|m| m.created())
    .and_then(|t| t.duration_since(UNIX_EPOCH))
    .unwrap_or_default()
    .as_secs() as u32;
  Model {
    model: alias.alias,
    modified_at: created,
    size: 0, // Placeholder for future size calculation
    digest: alias.snapshot,
    details: ModelDetails {
      format: GGUF.to_string(),
      family: "unknown".to_string(),
      // Ollama-specific model metadata
    },
  }
}

// Model show implementation with parameter serialization
fn to_ollama_model_show(state: Arc<dyn RouterState>, alias: Alias) -> ShowResponse {
  let request_params = serde_yaml::to_string(&alias.request_params).unwrap_or_default();
  let context_params = serde_yaml::to_string(&alias.context_params).unwrap_or_default();
  let parameters = format!("{context_params}{request_params}");
  // Chat template removed since llama.cpp now handles this internally
  let template = "".to_string();
  
  ShowResponse {
    parameters,
    template,
    // Additional Ollama model information
  }
}
```

**Ollama Integration Features**:
- Bidirectional format translation between OpenAI and Ollama request/response formats
- Parameter mapping with semantic preservation across different API specifications
- Model metadata conversion with Ollama-specific fields and formatting
- Streaming response translation with proper Ollama chunk formatting
- Ecosystem compatibility with Ollama CLI tools and integrations

## HTTP Route Orchestration Implementation

### Service Coordination Architecture
Routes coordinate extensively with BodhiApp's service layer for API operations:

```rust
// RouterState integration pattern used across all endpoints
impl RouterState for DefaultRouterState {
  fn app_service(&self) -> Arc<dyn AppService> {
    self.app_service.clone()
  }
  
  fn chat_completions(&self, request: CreateChatCompletionRequest) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>> {
    Box::pin(async move {
      // 1. Model alias resolution via DataService
      let alias = self.app_service.data_service().find_alias(&request.model)?;
      
      // 2. Request forwarding through SharedContext
      let response = self.ctx.chat_completions(request, alias).await?;
      
      Ok(response)
    })
  }
}
```

### Error Handling Architecture
Comprehensive error handling with API-specific response formatting:

```rust
// HTTP error types with localization support (see src/routes_chat.rs:12-18)
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HttpError {
  #[error("http_error")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),
}

// OpenAI error response integration
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let api_error = objs::ApiError::from(self);
        let status_code = match api_error.error_type {
            objs::ErrorType::NotFound => StatusCode::NOT_FOUND,
            objs::ErrorType::BadRequest => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status_code, Json(api_error)).into_response()
    }
}

// Ollama error response format
#[derive(Serialize, Deserialize, ToSchema)]
pub struct OllamaError {
  error: String,
}
```

**Error Handling Features**:
- Service errors transparently wrapped with API-specific error formatting
- OpenAI-compatible error responses with proper error types and HTTP status codes
- Ollama-compatible error responses with simplified error structure
- Localized error messages via `errmeta_derive` integration with objs error system
- Comprehensive error boundaries for streaming and non-streaming responses

## Streaming Architecture Implementation

### Server-Sent Events Coordination
Advanced streaming coordination with server_core infrastructure:

```rust
// Streaming response handling pattern
pub async fn chat_completions_handler(/* ... */) -> Result<Response, ApiError> {
  let response = state.chat_completions(request).await?;
  
  // Stream proxying with proper header management
  let mut response_builder = Response::builder().status(response.status());
  if let Some(headers) = response_builder.headers_mut() {
    *headers = response.headers().clone();
  }
  
  // Efficient stream proxying without buffering
  let stream = response.bytes_stream();
  let body = Body::from_stream(stream);
  Ok(response_builder.body(body)?)
}

// Ollama streaming transformation
let stream = response.bytes_stream().map(move |chunk| {
  let chunk = chunk.map_err(|e| format!("error reading chunk: {e}"))?;
  let text = String::from_utf8_lossy(&chunk);

  if text.starts_with("data: ") {
    let msg = text.strip_prefix("data: ").unwrap().strip_suffix("\n\n").unwrap();
    
    // OpenAI to Ollama format conversion
    let oai_chunk: CreateChatCompletionStreamResponse = serde_json::from_str(msg)?;
    let data: ResponseStream = oai_chunk.into();
    serde_json::to_string(&data).map(|s| format!("data: {s}\n\n"))
  } else {
    Ok(text.into_owned())
  }
});
```

**Streaming Features**:
- Efficient stream proxying without intermediate buffering for performance
- Format-specific streaming with OpenAI and Ollama chunk formatting
- Connection management coordinated with server_core streaming infrastructure
- Error propagation through streaming context with proper error recovery
- Header management for proper SSE formatting and client compatibility

## Cross-Crate Integration Implementation

### Service Layer Integration
Routes coordinate extensively with BodhiApp's service layer:

```rust
// DataService integration for model operations
let models = state
  .app_service()
  .data_service()
  .list_aliases()?
  .into_iter()
  .map(|alias| to_oai_model(state.clone(), alias))
  .collect::<Vec<_>>();

// Model alias resolution with error handling
let alias = state
  .app_service()
  .data_service()
  .find_alias(&id)
  .ok_or(AliasNotFoundError(id))?;

// TimeService integration for metadata
let created = state.app_service().time_service().created_at(&path);
```

### Domain Object Integration
Extensive use of objs crate for API operations:

```rust
// Alias domain object usage throughout API endpoints
use objs::{Alias, ApiError, AppError, ErrorType, OpenAIApiError};

// Error system integration with localization
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HttpError {
  #[error("http_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Http(#[from] http::Error),
}
```

## API Testing Architecture

### OpenAI Compatibility Testing
Comprehensive testing with OpenAI SDK compatibility validation:

```rust
// OpenAI API compatibility testing pattern (see src/routes_chat.rs:67-89 for complete test)
#[rstest]
#[awt]
#[tokio::test]
async fn test_routes_chat_completions_non_stream(
  #[future] app_service_stub: AppServiceStub,
) -> anyhow::Result<()> {
  let request = CreateChatCompletionRequestArgs::default()
    .model("testalias-exists:instruct")
    .messages(vec![ChatCompletionRequestMessage::User(/* ... */)])
    .build()?;
  
  let alias = Alias::testalias_exists();
  let mut ctx = MockSharedContext::default();
  ctx.expect_chat_completions()
    .with(eq(request.clone()), eq(alias))
    .times(1)
    .return_once(move |_, _| Ok(non_streamed_response()));
  
  let router_state = DefaultRouterState::new(Arc::new(ctx), Arc::new(app_service_stub));
  let app = Router::new()
    .route("/v1/chat/completions", post(chat_completions_handler))
    .with_state(Arc::new(router_state));
  
  let response = app.oneshot(Request::post("/v1/chat/completions").json(request)?).await?;
  assert_eq!(StatusCode::OK, response.status());
  
  let result: CreateChatCompletionResponse = response.json().await?;
  // Validate OpenAI response format compliance
}
```

### Streaming Response Testing
Advanced streaming validation with SSE format testing:

```rust
// Streaming response testing with format validation
#[rstest]
#[awt]
#[tokio::test]
async fn test_routes_chat_completions_stream(/* ... */) -> anyhow::Result<()> {
  let request = CreateChatCompletionRequestArgs::default()
    .stream(true)
    .build()?;
  
  // Mock streaming response with proper SSE formatting
  let response = app.oneshot(Request::post("/v1/chat/completions").json(request)?).await?;
  assert_eq!(StatusCode::OK, response.status());
  
  // SSE response parsing and validation
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
Ollama compatibility validation with ecosystem tool testing:

```rust
// Ollama API compatibility testing
#[rstest]
#[awt]
#[tokio::test]
async fn test_ollama_routes_models_list(/* ... */) -> anyhow::Result<()> {
  let app = Router::new()
    .route("/api/tags", get(ollama_models_handler))
    .with_state(Arc::new(router_state_stub));
  
  let response = app.oneshot(Request::get("/api/tags").body(Body::empty())?).await?
    .json::<Value>().await?;
  
  // Validate Ollama response format
  assert!(response["models"].as_array().length().unwrap() >= 6);
  let llama3 = response["models"].as_array().unwrap()
    .iter().find(|item| item["model"] == "llama3:instruct").unwrap();
  assert_eq!("5007652f7a641fe7170e0bad4f63839419bd9213", llama3["digest"]);
}
```

**Testing Features**:
- Comprehensive OpenAI SDK compatibility validation with real request/response testing
- Streaming response testing with SSE format validation and content verification
- Ollama ecosystem compatibility testing with CLI tool format validation
- Service mock coordination for isolated API endpoint testing
- Error scenario testing with proper HTTP status code and error format validation

## Extension Guidelines

### Adding New OpenAI Endpoints
When implementing additional OpenAI API endpoints:

1. **Specification Compliance**: Follow OpenAI API specification exactly using `async-openai` types
2. **Service Coordination**: Use RouterState for consistent AppService access and business logic
3. **Error Handling**: Implement API-specific errors with OpenAI-compatible response formatting
4. **Documentation**: Add comprehensive OpenAPI documentation with examples and schemas
5. **Testing**: Create compatibility tests with OpenAI SDK validation and ecosystem tool testing

### Extending Ollama Compatibility
For new Ollama API endpoints and ecosystem features:

1. **Format Translation**: Design bidirectional translation between Ollama and OpenAI formats
2. **Parameter Mapping**: Create semantic parameter mapping preserving functionality across formats
3. **Ecosystem Integration**: Validate compatibility with Ollama CLI tools and ecosystem integrations
4. **Response Formatting**: Implement Ollama-specific response formatting with proper field mapping
5. **Testing**: Add Ollama ecosystem compatibility tests with CLI tool validation

### Cross-Format API Patterns
For features spanning both OpenAI and Ollama formats:

1. **Unified Processing**: Design internal processing supporting both API formats efficiently
2. **Format Abstraction**: Create abstraction layers handling format-specific concerns
3. **Response Transformation**: Implement efficient response transformation for different formats
4. **Error Coordination**: Ensure consistent error handling across API format requirements
5. **Performance Optimization**: Optimize for minimal overhead in format translation and processing

## Commands

**API Endpoint Tests**: `cargo test -p routes_oai` (includes OpenAI and Ollama compatibility testing)  
**Streaming Tests**: `cargo test -p routes_oai stream` (includes SSE streaming and format validation)  
**Compatibility Tests**: `cargo test -p routes_oai compatibility` (includes ecosystem tool validation)