# CLAUDE.md - routes_oai

This file provides guidance to Claude Code when working with the `routes_oai` crate, which implements OpenAI-compatible API endpoints for BodhiApp.

## Purpose

The `routes_oai` crate provides OpenAI API compatibility for BodhiApp:

- **OpenAI Chat Completions**: `/v1/chat/completions` endpoint with streaming support
- **Model Listing**: `/v1/models` endpoint for listing available models
- **Ollama Compatibility**: Ollama API endpoints for broader ecosystem compatibility
- **Request Validation**: Comprehensive request validation and error handling
- **Response Streaming**: Server-Sent Events for real-time chat completions
- **API Documentation**: OpenAPI specifications with Utoipa integration

## Key Components

### Chat Completions (`src/routes_chat.rs`)
- `create_chat_completion` - Main chat completion endpoint handler
- OpenAI API compatibility with `CreateChatCompletionRequest`
- Streaming and non-streaming response support
- Model alias resolution and request forwarding
- Comprehensive error handling with proper HTTP status codes

### Models API (`src/routes_oai_models.rs`)
- `list_models` - Lists all available model aliases
- OpenAI-compatible model objects with metadata
- Model filtering and presentation logic
- Integration with data service for model discovery

### Ollama Compatibility (`src/routes_ollama.rs`)
- `/api/tags` - Ollama-style model listing
- `/api/show` - Model information endpoint
- `/api/chat` - Ollama-style chat interface
- Cross-compatibility with Ollama ecosystem tools

## Dependencies

### Core Infrastructure
- `objs` - Domain objects, validation, and error handling
- `server_core` - HTTP server infrastructure and router state
- `services` - Business logic and data access services

### HTTP Framework
- `axum` - Web framework with extractors and handlers
- `axum-extra` - Additional extractors and utilities
- `http` - HTTP types and utilities

### OpenAI Integration
- `async-openai` - OpenAI API types and client integration
- `serde_json` - JSON serialization for request/response handling

### API Documentation
- `utoipa` - OpenAPI specification generation
- API documentation with examples and schemas

## Architecture Position

The `routes_oai` crate sits at the HTTP API layer:
- **Above**: Server core infrastructure and middleware
- **Below**: Client applications and OpenAI-compatible tools
- **Implements**: OpenAI API specification for broad compatibility
- **Coordinates**: Request validation, model resolution, and response formatting

## Usage Patterns

### Route Registration
```rust
use routes_oai::{create_chat_completion, list_models};
use axum::{routing::{get, post}, Router};

let app = Router::new()
    .route("/v1/chat/completions", post(create_chat_completion))
    .route("/v1/models", get(list_models))
    .with_state(router_state);
```

### Chat Completion Request Handling
```rust
use async_openai::types::CreateChatCompletionRequest;
use routes_oai::create_chat_completion;

// Handler automatically validates request and forwards to LLM server
let response = create_chat_completion(
    State(router_state),
    WithRejection(Json(request), ApiError),
).await?;
```

### Model Listing
```rust
use routes_oai::list_models;

// Returns OpenAI-compatible model list
let models_response = list_models(State(router_state)).await?;
```

### Ollama Compatibility
```rust
use routes_oai::{ollama_tags, ollama_show, ollama_chat};

let app = Router::new()
    .route("/api/tags", get(ollama_tags))
    .route("/api/show", post(ollama_show))
    .route("/api/chat", post(ollama_chat));
```

## Integration Points

### With Server Core
- Router state provides access to application services
- Shared context for managing LLM server connections
- Error handling integration with HTTP response mapping

### With Services Layer
- Data service for model alias resolution and listing
- Hub service for model metadata and availability
- Authentication via middleware integration

### With OpenAI Ecosystem
- Full compatibility with OpenAI Python SDK
- Support for popular AI tools and frameworks
- Standard request/response format adherence

## API Endpoints

### OpenAI Compatibility

#### POST `/v1/chat/completions`
- Creates chat completions with streaming support
- Validates request against OpenAI specification
- Supports all standard OpenAI parameters
- Returns OpenAI-compatible response format

#### GET `/v1/models`
- Lists all available model aliases
- Returns OpenAI-compatible model objects
- Includes model metadata and capabilities

### Ollama Compatibility

#### GET `/api/tags`
- Lists models in Ollama format
- Provides model size and modification timestamps
- Compatible with Ollama CLI tools

#### POST `/api/show`
- Shows detailed model information
- Includes model parameters and configuration
- Ollama-compatible response format

#### POST `/api/chat`
- Chat completion in Ollama format
- Alternative interface for Ollama ecosystem integration

## Request Validation

### Chat Completion Validation
- Required fields: `model`, `messages`
- Optional parameters with proper defaults
- Parameter range validation (temperature, top_p, etc.)
- Message format and role validation

### Error Handling
- Comprehensive validation with detailed error messages
- HTTP status code mapping for different error types
- OpenAI-compatible error response format

### Model Resolution
- Alias lookup and validation
- Model availability checking
- Graceful handling of missing models

## Response Streaming

### Server-Sent Events
- Real-time streaming for chat completions
- Proper SSE formatting with event data
- Connection management and cleanup

### Non-Streaming Responses
- Standard JSON responses for non-streaming requests
- Proper content-type headers and formatting
- Complete response buffering and formatting

## Performance Considerations

### Request Processing
- Efficient request validation and parsing
- Minimal overhead for request forwarding
- Asynchronous processing for all operations

### Streaming Optimization
- Memory-efficient streaming with proper buffering
- Connection pooling for upstream services
- Proper error propagation in streaming context

### Caching
- Model list caching for performance
- Request validation result caching where appropriate

## Development Guidelines

### Adding New Endpoints
1. Define endpoint handler with proper OpenAI compatibility
2. Add comprehensive request validation
3. Implement proper error handling with HTTP status mapping
4. Add OpenAPI documentation with examples
5. Include unit tests and integration tests

### Maintaining OpenAI Compatibility
- Follow OpenAI API specification exactly
- Test with OpenAI Python SDK and popular tools
- Validate response formats against official examples
- Handle edge cases and error conditions properly

### Error Handling Best Practices
- Use appropriate HTTP status codes
- Provide clear, actionable error messages
- Include request context in error responses
- Log errors appropriately for debugging

## Testing Strategy

### Unit Tests
- Request validation logic
- Response formatting and serialization
- Error handling and edge cases
- Model resolution and filtering

### Integration Tests
- End-to-end API compatibility testing
- Streaming response validation
- Error propagation through middleware stack
- Performance testing under load

### Compatibility Testing
- OpenAI Python SDK integration tests
- Popular tool compatibility validation
- Ollama ecosystem compatibility testing

## Monitoring and Observability

### Request Metrics
- Request count and response time per endpoint
- Success/failure rates and error types
- Model usage statistics and patterns

### Streaming Metrics
- Connection duration and stability
- Streaming throughput and latency
- Client disconnect patterns

### Error Tracking
- Validation failure patterns
- Upstream service errors
- Client compatibility issues

## Future Extensions

The routes_oai crate is designed for extensibility:
- Additional OpenAI API endpoints (embeddings, fine-tuning)
- Enhanced streaming capabilities (WebSocket support)
- Advanced model routing and load balancing
- Metrics and analytics endpoints
- Rate limiting and quota management