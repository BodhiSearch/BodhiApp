# routes_oai - OpenAI-Compatible API Routes

## Overview

The `routes_oai` crate implements OpenAI-compatible HTTP API endpoints for BodhiApp. It provides a standardized interface that allows existing OpenAI client libraries and tools to work seamlessly with BodhiApp's local LLM infrastructure.

## Purpose

- **OpenAI Compatibility**: Implements OpenAI API specification for seamless integration
- **Chat Completions**: Provides chat completion endpoints with streaming support
- **Model Management**: Exposes available models through OpenAI-compatible endpoints
- **Ollama Compatibility**: Additional compatibility layer for Ollama clients
- **Standards Compliance**: Ensures API responses match OpenAI specifications

## Key Route Modules

### Chat Completions (`routes_chat.rs`)
- **POST /v1/chat/completions**: Main chat completion endpoint
- **Streaming Support**: Server-Sent Events for real-time responses
- **Message History**: Conversation context management
- **Model Selection**: Dynamic model selection per request
- **Parameter Support**: Temperature, max_tokens, top_p, etc.

Key features:
- Full OpenAI chat completions API compatibility
- Streaming and non-streaming responses
- Function calling support (if implemented)
- Message role handling (system, user, assistant)
- Token counting and limits

### Model Information (`routes_oai_models.rs`)
- **GET /v1/models**: List available models
- **GET /v1/models/{model_id}**: Get specific model information
- **Model Metadata**: Provides model capabilities and limits
- **Dynamic Discovery**: Real-time model availability

Provides:
- Model listing with metadata
- Model capability information
- Compatibility information
- Performance characteristics

### Ollama Compatibility (`routes_ollama.rs`)
- **Ollama API Endpoints**: Compatible with Ollama client tools
- **Model Management**: Ollama-style model operations
- **Chat Interface**: Ollama chat completion format
- **Bridge Layer**: Translates between Ollama and BodhiApp formats

## Directory Structure

```
src/
├── lib.rs                    # Main module exports and route registration
├── routes_chat.rs            # Chat completion endpoints
├── routes_oai_models.rs      # Model information endpoints
├── routes_ollama.rs          # Ollama compatibility endpoints
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    └── mod.rs
```

## API Endpoints

### OpenAI-Compatible Endpoints

#### Chat Completions
```
POST /v1/chat/completions
Content-Type: application/json

{
  "model": "model-alias-or-id",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 150,
  "stream": false
}
```

#### Model Listing
```
GET /v1/models

Response:
{
  "object": "list",
  "data": [
    {
      "id": "model-id",
      "object": "model",
      "created": 1677610602,
      "owned_by": "bodhi"
    }
  ]
}
```

#### Model Information
```
GET /v1/models/{model_id}

Response:
{
  "id": "model-id",
  "object": "model",
  "created": 1677610602,
  "owned_by": "bodhi",
  "permission": [...],
  "root": "model-id"
}
```

### Ollama-Compatible Endpoints

#### Ollama Chat
```
POST /api/chat
Content-Type: application/json

{
  "model": "model-name",
  "messages": [...],
  "stream": false
}
```

#### Ollama Models
```
GET /api/tags

Response:
{
  "models": [
    {
      "name": "model-name",
      "modified_at": "2023-12-07T09:32:18.757212583Z",
      "size": 3825819519
    }
  ]
}
```

## Key Features

### Streaming Support
- **Server-Sent Events**: Real-time response streaming
- **Chunked Responses**: Incremental message delivery
- **Connection Management**: Proper stream lifecycle handling
- **Error Handling**: Graceful stream error recovery

### Request Validation
- **Parameter Validation**: Comprehensive input validation
- **Model Verification**: Ensures requested models are available
- **Rate Limiting**: Configurable request rate limits
- **Authentication**: Integration with auth middleware

### Response Formatting
- **OpenAI Compliance**: Exact response format matching
- **Error Responses**: Standardized error format
- **Metadata Inclusion**: Proper response metadata
- **Content Filtering**: Optional content filtering support

## Dependencies

### Core Dependencies
- **objs**: Domain objects and OpenAI types
- **services**: Business logic services
- **server_core**: HTTP server infrastructure
- **auth_middleware**: Authentication and authorization

### HTTP and Streaming
- **axum**: HTTP framework
- **tower**: Middleware support
- **futures**: Stream handling
- **serde**: JSON serialization

## Usage Patterns

### Route Registration
Routes are registered with the Axum router:

```rust
pub fn create_oai_routes() -> Router<RouterState> {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/v1/models/:model_id", get(get_model))
        .layer(ApiAuthMiddleware::new())
}
```

### Handler Implementation
Handlers follow a consistent pattern:

```rust
async fn chat_completions(
    Extension(chat_service): Extension<Arc<dyn ChatService>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, ApiError> {
    // Validate request
    // Call service
    // Format response
}
```

### Streaming Responses
Streaming is implemented using SSE:

```rust
async fn streaming_chat(
    request: ChatCompletionRequest,
) -> Sse<impl Stream<Item = Event>> {
    let stream = chat_service.stream_completion(request).await?;
    let sse_stream = stream.map(|chunk| {
        Event::default().data(serde_json::to_string(&chunk)?)
    });
    Sse::new(sse_stream)
}
```

## Testing Support

The routes_oai crate includes comprehensive testing:
- **Integration Tests**: Full API endpoint testing
- **Mock Services**: Service layer mocking
- **Response Validation**: OpenAI compatibility testing
- **Streaming Tests**: SSE stream validation

## Error Handling

### OpenAI Error Format
Errors follow the OpenAI error response format:

```json
{
  "error": {
    "message": "Error description",
    "type": "invalid_request_error",
    "param": "parameter_name",
    "code": "error_code"
  }
}
```

### HTTP Status Codes
- **200**: Successful completion
- **400**: Bad request (invalid parameters)
- **401**: Unauthorized (invalid API key)
- **404**: Model not found
- **429**: Rate limit exceeded
- **500**: Internal server error

## Performance Considerations

### Streaming Efficiency
- **Chunked Transfer**: Efficient streaming implementation
- **Memory Management**: Proper stream memory handling
- **Connection Pooling**: Efficient connection reuse
- **Backpressure**: Proper flow control

### Caching
- **Model Metadata**: Cached model information
- **Response Caching**: Optional response caching
- **Connection Reuse**: HTTP connection pooling

## Integration Points

- **LLM Services**: Integrates with local LLM inference
- **Model Management**: Uses model service for availability
- **Authentication**: Protected by auth middleware
- **Frontend**: Provides API for web interface
- **External Tools**: Compatible with OpenAI client libraries

## Compatibility

### OpenAI Client Libraries
The API is compatible with:
- Official OpenAI Python library
- OpenAI Node.js library
- Third-party OpenAI clients
- Curl and HTTP clients

### Ollama Tools
The Ollama compatibility layer supports:
- Ollama CLI tools
- Ollama client libraries
- Existing Ollama workflows

## Future Extensions

The routes_oai crate is designed to support:
- Function calling API
- Fine-tuning endpoints
- Embeddings API
- Image generation API
- Audio transcription API
- Additional OpenAI features as they become available
