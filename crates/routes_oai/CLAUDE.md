# CLAUDE.md

See [PACKAGE.md](./PACKAGE.md) for implementation details and technical depth.

## Purpose

The `routes_oai` crate serves as BodhiApp's **OpenAI API compatibility layer**, implementing comprehensive OpenAI-compatible endpoints with sophisticated streaming capabilities, Ollama ecosystem integration, and multi-format API support for broad ecosystem compatibility.

## Key Domain Architecture

### OpenAI API Compatibility System
Comprehensive OpenAI API implementation with full ecosystem compatibility:
- **Chat Completions Endpoint**: `/v1/chat/completions` with streaming and non-streaming support
- **Models API**: `/v1/models` and `/v1/models/{id}` for model discovery and metadata
- **Request Validation**: Complete OpenAI parameter validation with range checking and format compliance
- **Response Streaming**: Server-sent events with proper SSE formatting and connection management
- **Error Handling**: OpenAI-compatible error responses with proper HTTP status codes and error types

### Ollama Ecosystem Integration
Dual API compatibility for broader ecosystem support:
- **Ollama Models API**: `/api/tags` and `/api/show` endpoints with Ollama-specific response formats
- **Ollama Chat Interface**: `/api/chat` endpoint with request/response format translation
- **Cross-Format Translation**: Seamless conversion between OpenAI and Ollama request/response formats
- **Parameter Mapping**: Intelligent parameter translation between different API specifications
- **Ecosystem Compatibility**: Support for Ollama CLI tools and ecosystem integrations

### HTTP Route Orchestration Architecture
Sophisticated HTTP request processing with service coordination:
- **RouterState Integration**: Dependency injection providing access to AppService registry and SharedContext
- **Model Alias Resolution**: DataService coordination for model alias lookup and validation
- **Request Forwarding**: Intelligent request routing through SharedContext to LLM server instances
- **Response Transformation**: Format-specific response processing for OpenAI and Ollama compatibility
- **Error Translation**: Service errors converted to appropriate API-specific error responses with localization

## Architecture Position

The `routes_oai` crate serves as BodhiApp's **OpenAI API compatibility orchestration layer**:
- **Above server_core and services**: Coordinates HTTP infrastructure and business services for API operations
- **Below client applications**: Provides OpenAI-compatible API surface for external tools and SDKs
- **Parallel to routes_app**: Similar HTTP orchestration role but focused on OpenAI/Ollama compatibility instead of application management
- **Integration with auth_middleware**: Leverages authentication middleware for API security and authorization

## Cross-Crate Integration Patterns

### Service Layer API Coordination
Complex API operations coordinated across BodhiApp's service layer:
- **RouterState Integration**: HTTP handlers access AppService registry for business logic operations
- **DataService Coordination**: Model alias resolution, listing, and validation for API endpoints
- **SharedContext Integration**: LLM server request routing and response streaming coordination
- **Error Service Translation**: Service errors converted to OpenAI/Ollama-compatible API responses with localization
- **Authentication Integration**: API security coordinated through auth_middleware with bearer token and session support

### HTTP Infrastructure Integration
API routes coordinate with HTTP infrastructure for request processing:
- **Request Validation**: Comprehensive OpenAI parameter validation using async-openai types
- **Response Streaming**: Server-sent events coordinated with server_core streaming infrastructure
- **Error Handling**: HTTP error responses with proper status codes and API-specific error formats
- **Content Negotiation**: Multi-format response handling for OpenAI and Ollama compatibility
- **Route Composition Integration**: Coordinated with routes_all for unified route composition with hierarchical authorization and middleware orchestration

### Domain Object Integration
Extensive use of objs crate for API operations:
- **Alias Resolution**: Model alias lookup and validation using objs domain objects
- **Error System Integration**: API errors implement AppError trait for consistent HTTP response generation
- **Parameter Validation**: OpenAI parameter validation using objs validation rules and error handling
- **Localization Support**: Multi-language error messages via objs LocalizationService integration

## API Orchestration Workflows

### Multi-Service Chat Completion Coordination
Complex request processing with service orchestration:

1. **Request Reception**: HTTP routes receive OpenAI-compatible requests with comprehensive validation
2. **Model Alias Resolution**: DataService resolves model aliases for chat completion requests
3. **Context Coordination**: SharedContext manages LLM server instances for request processing
4. **Response Streaming**: Server-sent events handle real-time streaming responses with proper formatting
5. **Error Translation**: Service errors converted to OpenAI-compatible HTTP responses with localization

### Dual-Format API Support Orchestration
Sophisticated API compatibility coordination:

**OpenAI API Workflow**:
1. **Parameter Validation**: Complete OpenAI parameter validation using async-openai types
2. **Request Processing**: Standard OpenAI request processing with model alias resolution
3. **Response Formatting**: OpenAI-compatible response formatting with proper JSON structure
4. **Streaming Support**: SSE streaming with OpenAI chunk format and connection management

**Ollama API Workflow**:
1. **Format Translation**: Ollama request format converted to OpenAI internal format
2. **Parameter Mapping**: Ollama-specific parameters mapped to OpenAI equivalents
3. **Response Conversion**: OpenAI responses converted to Ollama-compatible format
4. **Ecosystem Integration**: Support for Ollama CLI tools and ecosystem compatibility

### Cross-Format Error Coordination
Error handling across different API formats:
- **OpenAI Error Format**: Standard OpenAI error responses with proper error types and codes
- **Ollama Error Format**: Ollama-compatible error responses with simplified error structure
- **Service Error Translation**: Service errors converted to appropriate API-specific formats
- **Localization Support**: Multi-language error messages coordinated with objs error system

## Important Constraints

### OpenAI API Compatibility Requirements
- All endpoints must maintain strict OpenAI API specification compliance for ecosystem compatibility
- Request validation must use async-openai types for parameter validation and format consistency
- Response formats must match OpenAI specification exactly for SDK and tool compatibility
- Error responses must follow OpenAI error format with proper error types and HTTP status codes
- Streaming responses must use proper SSE formatting with OpenAI chunk structure

### Ollama Ecosystem Integration Standards
- Ollama endpoints must support Ollama CLI tools and ecosystem integrations
- Request/response format translation must be bidirectional and lossless where possible
- Parameter mapping between Ollama and OpenAI formats must preserve semantic meaning
- Error handling must provide Ollama-compatible error responses while maintaining internal consistency
- Model metadata must be compatible with Ollama model discovery and management tools

### HTTP Infrastructure Coordination Rules
- All routes must use RouterState dependency injection for consistent service access
- Model alias resolution must coordinate with DataService for consistent model discovery
- Response streaming must integrate with server_core streaming infrastructure for performance
- Error handling must translate service errors to appropriate API-specific formats with localization
- Authentication must integrate with auth_middleware for consistent API security across endpoints

## API Extension Patterns

### Adding New OpenAI Endpoints
When implementing additional OpenAI API endpoints:

1. **Specification Compliance**: Follow OpenAI API specification exactly for parameter validation and response formats
2. **Service Coordination**: Use RouterState for consistent AppService access and business logic coordination
3. **Error Handling**: Implement API-specific errors that convert to OpenAI-compatible responses
4. **Documentation**: Add comprehensive OpenAPI documentation with examples and proper schemas
5. **Testing Infrastructure**: Create comprehensive API compatibility tests with OpenAI SDK validation

### Extending Ollama Compatibility
For new Ollama API endpoints and features:

1. **Format Translation**: Design bidirectional translation between Ollama and OpenAI formats
2. **Parameter Mapping**: Create semantic parameter mapping that preserves functionality
3. **Ecosystem Testing**: Validate compatibility with Ollama CLI tools and ecosystem integrations
4. **Error Consistency**: Provide Ollama-compatible error responses while maintaining internal error handling
5. **Documentation**: Document Ollama-specific behavior and compatibility considerations

### Cross-Format Integration Patterns
For features that span both API formats:

1. **Unified Processing**: Design internal processing that supports both API formats efficiently
2. **Format Abstraction**: Create abstraction layers that handle format-specific concerns
3. **Response Transformation**: Implement efficient response transformation for different API formats
4. **Error Coordination**: Ensure consistent error handling across different API format requirements
5. **Performance Optimization**: Optimize for minimal overhead in format translation and processing