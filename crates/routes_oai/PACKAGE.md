# PACKAGE.md - crates/routes_oai

See [crates/routes_oai/CLAUDE.md](crates/routes_oai/CLAUDE.md) for architectural guidance and design rationale.

This document provides detailed technical information for the `crates/routes_oai` crate, focusing on BodhiApp's OpenAI API compatibility implementation patterns, sophisticated streaming capabilities, and dual-format API support architecture.

## Core Implementation Files

### Main Components
- `src/lib.rs` - Library exports with module re-exports and API endpoint constants
- `src/routes_chat.rs` - OpenAI chat completions and embeddings endpoints with streaming support, request validation, and `HttpError` enum
- `src/routes_oai_models.rs` - OpenAI models API with comprehensive alias resolution and deduplication
- `src/routes_ollama.rs` - Ollama compatibility layer with bidirectional format translation
- `src/test_utils/mod.rs` - Testing utilities and fixtures for route testing


### Dependencies

**Core Dependencies** (`Cargo.toml`):
- `async-openai` for OpenAI type compatibility and parameter validation
- `server_core` for RouterState, LlmEndpoint, and HTTP infrastructure coordination
- `services` for business logic coordination and service access
- `objs` for domain objects, error handling (`AppError`, `ErrorType`, `ApiError`), and validation
- `errmeta_derive` for `ErrorMeta` derive macro on `HttpError`
- `axum` and `axum-extra` for HTTP route handling and request extraction (`WithRejection`)
- `serde` and `serde_json` for JSON serialization and deserialization
- `utoipa` for OpenAPI documentation generation

## API Implementation Architecture

### HttpError Enum

The crate defines its own error type for HTTP-layer errors in `src/routes_chat.rs`:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HttpError {
  #[error("Error constructing HTTP response: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),

  #[error("Response serialization failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Serialization(#[from] serde_json::Error),

  #[error("{0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRequest(String),
}
```

**Key design points**:
- `Http` and `Serialization` are internal server errors (500) for response construction failures
- `InvalidRequest` is a bad request error (400) for pre-forwarding validation failures, replacing the removed `BadRequestError` struct from `objs`
- All variants implement `AppError` via `ErrorMeta` derive for automatic `ApiError` conversion

### Chat Completion Request Validation

Structural validation performed before forwarding (`src/routes_chat.rs`):

```rust
fn validate_chat_completion_request(
  request: &serde_json::Value,
) -> Result<(), HttpError> {
  // 1. model: must exist and be a string
  // 2. messages: must exist and be an array
  // 3. stream: if present, must be a boolean
  // Returns HttpError::InvalidRequest on failure
}
```

The handler accepts `Json<serde_json::Value>` (not a typed struct) and forwards the raw JSON directly to the LLM server, preserving vendor-specific fields.

### Chat Completions Handler

Core chat completion endpoint with raw JSON forwarding (`src/routes_chat.rs`):

```rust
pub async fn chat_completions_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, ApiError>,
) -> Result<Response, ApiError> {
  validate_chat_completion_request(&request)?;
  let response = state
    .forward_request(LlmEndpoint::ChatCompletions, request)
    .await?;
  // Proxy status, headers, and body stream directly
}
```

**Key Features**:
- Raw `serde_json::Value` acceptance avoids lossy typed deserialization
- Structural validation via `validate_chat_completion_request()` catches malformed requests early
- Request forwarded through `RouterState::forward_request()` with `LlmEndpoint::ChatCompletions`
- Response status, headers, and body stream proxied directly without intermediate buffering

### Embeddings Handler

Embeddings endpoint with typed deserialization (`src/routes_chat.rs`):

```rust
pub async fn embeddings_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(request), _): WithRejection<Json<CreateEmbeddingRequest>, ApiError>,
) -> Result<Response, ApiError> {
  let request_value = serde_json::to_value(request)
    .map_err(HttpError::Serialization)?;
  let response = state
    .forward_request(LlmEndpoint::Embeddings, request_value)
    .await?;
  // Proxy status, headers, and body stream directly
}
```

Uses typed `CreateEmbeddingRequest` deserialization (unlike chat completions) since embeddings requests have a simpler, more stable schema.

### Models API Implementation

OpenAI-compatible model discovery with alias resolution and deduplication (`src/routes_oai_models.rs`):

- `oai_models_handler` - Lists all models, deduplicating via `HashSet` across User, Model, and Api alias types
- `oai_model_handler` - Retrieves a specific model by ID via `DataService::find_alias()`
- `user_alias_to_oai_model()` - Converts user aliases with filesystem-based creation timestamps
- `model_alias_to_oai_model()` - Converts auto-discovered model aliases from HF cache paths
- `api_model_to_oai_model()` - Converts API aliases with prefixed model IDs and base URL as `owned_by`

**API Alias Cache Refresh**: When `forward_all_with_prefix` is set and the cache is empty or stale, an async task is spawned to refresh model listings from the remote API provider.

## Ollama Ecosystem Integration Architecture

### Request Format Translation

Bidirectional format conversion between Ollama and OpenAI (`src/routes_ollama.rs`):

```rust
impl From<ChatRequest> for CreateChatCompletionRequest {
  fn from(val: ChatRequest) -> Self {
    let options = val.options.unwrap_or_default();
    CreateChatCompletionRequest {
      messages: val.messages.into_iter().map(|i| i.into()).collect(),
      model: val.model,
      frequency_penalty: options.frequency_penalty,
      max_completion_tokens: options.num_predict,
      temperature: options.temperature,
      top_p: options.top_p,
      stop: options.stop.map(StopConfiguration::StringArray),
      stream: val.stream,
      ..Default::default()
    }
  }
}
```

### Response Format Translation

OpenAI to Ollama response conversion with analytics placeholders (`src/routes_ollama.rs`):
- `From<CreateChatCompletionResponse> for ChatResponse` - Non-streaming response conversion
- `From<CreateChatCompletionStreamResponse> for ResponseStream` - Streaming chunk conversion
- Analytics fields (durations, counts) populated from usage data or set to placeholder values

### Ollama Models Integration

Model metadata conversion with filesystem integration (`src/routes_ollama.rs`):
- `user_alias_to_ollama_model()` - Filesystem metadata for creation time, GGUF format details
- `model_alias_to_ollama_model()` - HF cache path construction for auto-discovered models
- `alias_to_ollama_model_show()` - Detailed model info including request/context params serialized as YAML
- API aliases are filtered out of Ollama model listings (Ollama does not support remote API providers)

### Ollama Chat Handler

The Ollama chat handler (`src/routes_ollama.rs`) converts Ollama requests to OpenAI format, forwards via `forward_request()`, and handles two response modes:
- **Non-streaming**: Full response deserialized as `CreateChatCompletionResponse`, converted to `ChatResponse`, returned as JSON
- **Streaming**: SSE chunks parsed from OpenAI format, converted to `ResponseStream`, re-serialized as SSE

## Cross-Crate Integration Implementation

### Service Layer Coordination

Routes coordinate extensively with BodhiApp's service architecture:

```rust
// DataService integration for model operations (src/routes_oai_models.rs)
let aliases = state
  .app_service()
  .data_service()
  .list_aliases()
  .await
  .map_err(ApiError::from)?;

// Model alias resolution with error handling (src/routes_oai_models.rs)
if let Some(alias) = state.app_service().data_service().find_alias(&id).await {
  // Handle resolved alias
} else {
  Err(ApiError::from(AliasNotFoundError(id)))
}
```

### Domain Object Integration

Extensive use of objs crate throughout API operations:

```rust
// Domain object imports (src/routes_oai_models.rs)
use objs::{Alias, ApiAlias, ApiError, ModelAlias, UserAlias, OpenAIApiError};

// Error system integration (src/routes_chat.rs)
use objs::{ApiError, AppError, ErrorType};
```

## API Testing Architecture

### Chat Completion Testing

Tests in `src/routes_chat.rs` cover:
- **Non-streaming**: `test_routes_chat_completions_non_stream` - Full request/response cycle with model alias resolution via `MockSharedContext`
- **Streaming**: `test_routes_chat_completions_stream` - SSE stream parsing and content aggregation across multiple chunks
- **Embeddings**: `test_routes_embeddings_non_stream` - Typed embedding request with response validation

### Models API Testing

Tests in `src/routes_oai_models.rs` cover:
- `test_oai_models_handler` - Full model listing with expected alias resolution
- `test_oai_model_handler` - Single model retrieval by ID
- `test_oai_model_handler_not_found` - 404 for non-existent models
- API alias tests with and without prefix for both list and single model endpoints

### Ollama Compatibility Testing

Tests in `src/routes_ollama.rs` cover:
- `test_ollama_routes_models_list` - Ollama model listing format validation
- `test_ollama_model_show` - Detailed model information with parameters and template fields

## Extension Guidelines

### Adding New OpenAI Endpoints

When implementing additional OpenAI API endpoints:

1. **Request Strategy**: Decide between raw JSON forwarding (for proxy endpoints preserving vendor extensions) or typed deserialization (for endpoints needing local processing)
2. **Error Handling**: Add variants to `HttpError` if the endpoint has unique failure modes; use `ErrorMeta` derive for `AppError` implementation
3. **Service Coordination**: Implement RouterState integration for consistent AppService access
4. **Documentation**: Add comprehensive `utoipa` OpenAPI documentation with examples and proper schemas
5. **Testing**: Create compatibility tests using `MockSharedContext` for request forwarding validation

### Extending Ollama Compatibility

For new Ollama API endpoints:

1. **Format Translation**: Design bidirectional `From` implementations between Ollama and OpenAI formats
2. **Error Handling**: Use `Json<OllamaError>` as the error type for Ollama-compatible error responses
3. **Response Formatting**: Implement Ollama-specific response types with proper field mapping and serialization
4. **Ecosystem Integration**: Validate compatibility with Ollama CLI tools and ecosystem integrations
5. **Testing**: Add Ollama ecosystem compatibility tests with format validation

### Performance Optimization Patterns

For efficient API operations:

1. **Stream Proxying**: Use `Body::from_stream()` for efficient streaming without intermediate buffering
2. **Raw JSON Forwarding**: Accept `serde_json::Value` for proxy endpoints to avoid lossy re-serialization
3. **Alias Deduplication**: Use `HashSet` for efficient model deduplication in listing operations
4. **Error Boundaries**: Implement proper error handling for streaming and non-streaming contexts
5. **Async Cache Refresh**: Spawn background tasks for API alias cache updates to avoid blocking request processing

## Commands

**Full Test Suite**: `cargo test -p routes_oai`
**Chat Endpoints**: `cargo test -p routes_oai routes_chat`
**Model Endpoints**: `cargo test -p routes_oai routes_oai_models`
**Ollama Compatibility**: `cargo test -p routes_oai routes_ollama`
**Streaming Tests**: `cargo test -p routes_oai stream`
