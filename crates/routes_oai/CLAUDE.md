# CLAUDE.md

See [crates/routes_oai/PACKAGE.md](crates/routes_oai/PACKAGE.md) for implementation details and technical depth.

## Purpose

The `routes_oai` crate serves as BodhiApp's **OpenAI API compatibility orchestration layer**, implementing comprehensive OpenAI-compatible endpoints with sophisticated streaming capabilities, Ollama ecosystem integration, and dual-format API support for maximum ecosystem compatibility and tool integration.

## Key Domain Architecture

### OpenAI API Compatibility Architecture
Comprehensive OpenAI API implementation ensuring full ecosystem compatibility:

- **Chat Completions Orchestration**: `/v1/chat/completions` with streaming and non-streaming support, pre-deserialization validation, and response proxying
- **Embeddings API**: `/v1/embeddings` endpoint for text embedding generation with the same request-forwarding architecture as chat completions
- **Models Discovery API**: `/v1/models` and `/v1/models/{id}` endpoints with model alias resolution, deduplication, and metadata coordination across user, model, and API alias types
- **Pre-Deserialization Validation**: Chat completion requests are accepted as raw `serde_json::Value` and validated before forwarding, avoiding lossy re-serialization through typed structs
- **Response Streaming Infrastructure**: Server-sent events with proper SSE formatting, connection management, and streaming response proxying via `Body::from_stream`
- **Error Handling Integration**: Crate-local `HttpError` enum with `InvalidRequest` variant for 400 errors, `Http` for response construction failures, and `Serialization` for JSON errors -- all implementing `AppError` for consistent HTTP response generation

### Ollama Ecosystem Integration Architecture
Sophisticated dual API compatibility for broader ecosystem support:

- **Ollama Models API**: `/api/tags` and `/api/show` endpoints with Ollama-specific response formats and model metadata conversion
- **Ollama Chat Interface**: `/api/chat` endpoint with bidirectional request/response format translation and parameter mapping
- **Cross-Format Translation System**: Seamless conversion between OpenAI and Ollama formats using `From` trait implementations with semantic parameter preservation
- **Parameter Mapping Intelligence**: Intelligent parameter translation between API specifications maintaining functionality and compatibility
- **Tool Integration Support**: Full compatibility with Ollama CLI tools, ecosystem integrations, and third-party applications

### Request Validation Architecture
The crate implements a deliberate two-phase validation strategy for chat completions:

- **Phase 1 -- Structural Validation**: The `validate_chat_completion_request()` function checks raw JSON structure before forwarding, catching malformed requests early with user-friendly error messages. Three checks are performed: model field presence/type, messages field presence/type, and stream field type (if present).
- **Phase 2 -- Downstream Validation**: The LLM server performs full semantic validation (model existence, parameter ranges, message content validity) after the request is forwarded.
- **Design Rationale**: By accepting requests as `serde_json::Value` rather than deserializing into `CreateChatCompletionRequest`, the crate avoids the problem of async-openai's typed struct discarding unknown or extended fields during re-serialization. The raw JSON is forwarded directly to the LLM server, preserving vendor-specific extensions and parameters that async-openai types would otherwise strip.
- **Error Consolidation**: All validation errors use the `HttpError::InvalidRequest(String)` variant, which maps to `ErrorType::BadRequest` (HTTP 400). This replaced the previous `BadRequestError` struct from the `objs` crate, consolidating request validation errors within the crate that owns the validation logic.

### HTTP Route Orchestration Architecture
Advanced HTTP request processing with comprehensive service coordination:

- **RouterState Integration**: Dependency injection providing unified access to AppService registry, SharedContext, and service composition
- **Model Alias Resolution**: DataService coordination for model alias lookup, validation, and priority-based resolution with deduplication via `HashSet`
- **Request Forwarding Infrastructure**: Requests routed through `RouterState::forward_request()` with `LlmEndpoint` discriminator to appropriate LLM server instances
- **Response Proxying**: Response headers and body streams are proxied directly from the LLM server response, avoiding intermediate buffering
- **Error Translation Coordination**: Service errors converted to appropriate API-specific error responses with localization and user guidance

## Architecture Position

The `routes_oai` crate serves as BodhiApp's **OpenAI API compatibility orchestration layer**:

- **Above server_core and services**: Coordinates HTTP infrastructure and business services for comprehensive API operations and service integration
- **Below client applications**: Provides OpenAI-compatible API surface for external tools, SDKs, and ecosystem integrations
- **Parallel to routes_app**: Similar HTTP orchestration role but specialized for OpenAI/Ollama compatibility rather than application management
- **Integration with auth_middleware**: Comprehensive authentication middleware integration for API security, authorization, and access control

## Cross-Crate Integration Patterns

### Service Layer API Coordination
Comprehensive API operations coordinated across BodhiApp's complete service architecture:

- **RouterState Integration**: HTTP handlers access complete AppService registry for business logic operations with dependency injection
- **DataService Coordination**: Model alias resolution, listing, validation, and priority-based selection for API endpoints; handles three alias types (User, Model, Api) with distinct conversion logic per alias type
- **SharedContext Integration**: LLM server request routing via `forward_request()` with `LlmEndpoint` discriminator, response streaming coordination
- **Error Service Translation**: Service errors converted to OpenAI/Ollama-compatible API responses with user-friendly messages
- **Authentication Integration**: Comprehensive API security through auth_middleware with bearer tokens, session management, and access control

### HTTP Infrastructure Integration
Advanced API route coordination with comprehensive HTTP infrastructure:

- **Request Validation Infrastructure**: Structural JSON validation for chat completions using raw `serde_json::Value`, with `HttpError::InvalidRequest` for user-facing 400 errors
- **Response Streaming Coordination**: Server-sent events coordinated with server_core streaming infrastructure for real-time response delivery
- **Error Handling System**: HTTP error responses with proper status codes and API-specific error formats via `errmeta_derive::ErrorMeta` macro
- **Content Negotiation**: Multi-format response handling for OpenAI and Ollama compatibility with automatic format detection
- **Route Composition Integration**: Coordinated with routes_all for unified route composition, hierarchical authorization, and middleware orchestration

### Domain Object Integration
Comprehensive integration with objs crate for consistent API operations:

- **Alias Resolution System**: Model alias lookup, validation, and priority-based resolution using objs domain objects (`UserAlias`, `ModelAlias`, `ApiAlias`) with comprehensive error handling
- **Error System Integration**: `HttpError` enum implements `AppError` trait via `errmeta_derive::ErrorMeta` for consistent HTTP response generation with status code mapping. `InvalidRequest` maps to `ErrorType::BadRequest`, while `Http` and `Serialization` map to `ErrorType::InternalServer`.
- **Error Message Support**: User-friendly error messages via thiserror templates with format-specific message formatting

## API Orchestration Workflows

### Multi-Service Chat Completion Coordination
Sophisticated request processing with comprehensive service orchestration:

1. **Request Reception**: HTTP route accepts raw JSON body via `WithRejection<Json<serde_json::Value>, ApiError>` for flexible parsing
2. **Structural Validation**: `validate_chat_completion_request()` checks model, messages, and stream fields for correct types, returning `HttpError::InvalidRequest` on failure
3. **Request Forwarding**: Raw `serde_json::Value` forwarded via `state.forward_request(LlmEndpoint::ChatCompletions, request)` preserving all original fields
4. **Response Proxying**: Response status, headers, and body stream are proxied directly without intermediate deserialization
5. **Error Translation**: Service errors (`ContextError`, `ApiError`) and construction errors (`HttpError::Http`) converted to appropriate HTTP responses

### Dual-Format API Support Orchestration
Comprehensive API compatibility coordination with sophisticated format handling:

**OpenAI API Workflow**:
1. **Structural Validation**: Raw JSON validation for required fields before forwarding
2. **Request Forwarding**: Direct `serde_json::Value` forwarding preserving vendor extensions
3. **Response Proxying**: Headers and body stream proxied directly from LLM server
4. **Streaming Support**: SSE streaming proxied through `Body::from_stream` without format transformation

**Ollama API Workflow**:
1. **Format Translation**: Ollama `ChatRequest` deserialized and converted to OpenAI `CreateChatCompletionRequest` via `From` impl
2. **Parameter Mapping**: Ollama-specific parameters (num_predict, tfs_z, etc.) mapped to OpenAI equivalents with semantic preservation
3. **Response Conversion**: OpenAI responses converted to Ollama-compatible format (`ChatResponse`, `ResponseStream`) with proper field mapping
4. **Streaming Transformation**: SSE chunks parsed from OpenAI format, converted to Ollama `ResponseStream`, and re-serialized

### Cross-Format Error Coordination
Sophisticated error handling across different API formats:

- **OpenAI Error Format**: Standard OpenAI error responses via `ApiError` with proper error types, codes, and HTTP status codes
- **Ollama Error Format**: Ollama-compatible `OllamaError` struct with simplified error string for Ollama ecosystem compatibility
- **Crate-Local Errors**: `HttpError` enum consolidates request validation (`InvalidRequest`), HTTP construction (`Http`), and serialization (`Serialization`) errors
- **Error Message Support**: User-friendly error messages via thiserror derive with descriptive format strings

## Important Constraints

### OpenAI API Compatibility Requirements

- **Specification Compliance**: All endpoints must maintain strict OpenAI API specification compliance for ecosystem compatibility and tool integration
- **Raw JSON Forwarding**: Chat completion requests must be forwarded as raw JSON to preserve vendor-specific extensions that typed deserialization would strip
- **Response Format Compliance**: Response formats must match OpenAI specification exactly for SDK compatibility, tool integration, and ecosystem support
- **Error Response Standards**: Error responses must follow OpenAI error format via `ApiError` with proper error types, HTTP status codes, and detailed error information
- **Streaming Response Requirements**: Streaming responses must proxy SSE format directly from the LLM server without unnecessary transformation

### Ollama Ecosystem Integration Standards

- **CLI Tool Compatibility**: Ollama endpoints must provide full support for Ollama CLI tools and ecosystem integrations with proper format handling
- **Bidirectional Translation**: Request/response format translation must be bidirectional and lossless where possible with semantic preservation
- **Parameter Mapping Standards**: Parameter mapping between Ollama and OpenAI formats must preserve semantic meaning and functionality
- **Error Handling Consistency**: Error handling must provide Ollama-compatible error responses (`Json<OllamaError>`) while maintaining internal consistency
- **Model Metadata Compatibility**: Model metadata must be compatible with Ollama model discovery, management tools, and ecosystem expectations

### HTTP Infrastructure Coordination Rules

- **RouterState Integration**: All routes must use RouterState dependency injection for consistent service access and comprehensive dependency management
- **Model Resolution Coordination**: Model alias resolution must coordinate with DataService for consistent model discovery and priority-based selection
- **Streaming Infrastructure Integration**: Response streaming must proxy through `Body::from_stream` for efficient pass-through without intermediate buffering
- **Error Translation Requirements**: `HttpError` variants must implement `AppError` via `ErrorMeta` derive for automatic HTTP response conversion
- **Authentication Integration Standards**: Authentication must integrate with auth_middleware for consistent API security, access control, and authorization across endpoints

## API Extension Patterns

### Adding New OpenAI Endpoints
When implementing additional OpenAI API endpoints:

1. **Raw JSON Pattern**: Consider accepting `serde_json::Value` for forward-proxy endpoints to preserve vendor extensions, or typed structs for endpoints requiring local processing
2. **Service Coordination**: Use RouterState for consistent AppService access, business logic coordination, and dependency management
3. **Error Handling Integration**: Add variants to `HttpError` for endpoint-specific errors, using `ErrorMeta` derive for `AppError` trait implementation
4. **Documentation Standards**: Add comprehensive `utoipa` OpenAPI documentation with examples, proper schemas, and endpoint specifications
5. **Testing Infrastructure**: Create comprehensive API compatibility tests with OpenAI SDK validation and ecosystem integration testing

### Extending Ollama Compatibility
For new Ollama API endpoints and features:

1. **Format Translation Design**: Design comprehensive bidirectional translation between Ollama and OpenAI formats using `From` trait implementations
2. **Parameter Mapping Intelligence**: Create intelligent parameter mapping that preserves functionality and maintains compatibility
3. **Ecosystem Integration Testing**: Validate compatibility with Ollama CLI tools, ecosystem integrations, and third-party applications
4. **Error Consistency Management**: Use `Json<OllamaError>` as the error type for Ollama routes to provide Ollama-compatible error responses
5. **Documentation Standards**: Document Ollama-specific behavior, compatibility considerations, and ecosystem integration patterns

### Cross-Format Integration Patterns
For features that span both API formats:

1. **Unified Processing Architecture**: Design internal processing that supports both API formats efficiently with minimal overhead
2. **Format Abstraction Layers**: Create sophisticated abstraction layers that handle format-specific concerns while maintaining consistency
3. **Response Transformation Pipeline**: Implement efficient response transformation for different API formats with proper streaming support
4. **Error Coordination System**: Ensure consistent error handling across different API format requirements
5. **Performance Optimization Strategy**: Optimize for minimal overhead in format translation, processing, and response generation
