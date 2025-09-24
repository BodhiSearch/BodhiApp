# CLAUDE.md

See [crates/routes_oai/PACKAGE.md](crates/routes_oai/PACKAGE.md) for implementation details and technical depth.

## Purpose

The `routes_oai` crate serves as BodhiApp's **OpenAI API compatibility orchestration layer**, implementing comprehensive OpenAI-compatible endpoints with sophisticated streaming capabilities, Ollama ecosystem integration, and dual-format API support for maximum ecosystem compatibility and tool integration.

## Key Domain Architecture

### OpenAI API Compatibility Architecture
Comprehensive OpenAI API implementation ensuring full ecosystem compatibility:

- **Chat Completions Orchestration**: `/v1/chat/completions` with streaming and non-streaming support, request validation, and response formatting
- **Models Discovery API**: `/v1/models` and `/v1/models/{id}` endpoints with model alias resolution and metadata coordination
- **Parameter Validation System**: Complete OpenAI parameter validation using async-openai types with range checking and format compliance
- **Response Streaming Infrastructure**: Server-sent events with proper SSE formatting, connection management, and streaming response coordination
- **Error Handling Integration**: OpenAI-compatible error responses with proper HTTP status codes, error types, and localization support

### Ollama Ecosystem Integration Architecture
Sophisticated dual API compatibility for broader ecosystem support:

- **Ollama Models API**: `/api/tags` and `/api/show` endpoints with Ollama-specific response formats and model metadata conversion
- **Ollama Chat Interface**: `/api/chat` endpoint with bidirectional request/response format translation and parameter mapping
- **Cross-Format Translation System**: Seamless conversion between OpenAI and Ollama formats with semantic parameter preservation
- **Parameter Mapping Intelligence**: Intelligent parameter translation between API specifications maintaining functionality and compatibility
- **Tool Integration Support**: Full compatibility with Ollama CLI tools, ecosystem integrations, and third-party applications

### HTTP Route Orchestration Architecture
Advanced HTTP request processing with comprehensive service coordination:

- **RouterState Integration**: Dependency injection providing unified access to AppService registry, SharedContext, and service composition
- **Model Alias Resolution**: DataService coordination for model alias lookup, validation, and priority-based resolution
- **Request Forwarding Infrastructure**: Intelligent request routing through SharedContext to LLM server instances with load balancing
- **Response Transformation Pipeline**: Format-specific response processing for OpenAI and Ollama compatibility with streaming support
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
- **DataService Coordination**: Model alias resolution, listing, validation, and priority-based selection for API endpoints
- **SharedContext Integration**: LLM server request routing, response streaming coordination, and load balancing management
- **Error Service Translation**: Service errors converted to OpenAI/Ollama-compatible API responses with localization and user guidance
- **Authentication Integration**: Comprehensive API security through auth_middleware with bearer tokens, session management, and access control

### HTTP Infrastructure Integration
Advanced API route coordination with comprehensive HTTP infrastructure:

- **Request Validation Infrastructure**: Comprehensive OpenAI parameter validation using async-openai types with range checking and format compliance
- **Response Streaming Coordination**: Server-sent events coordinated with server_core streaming infrastructure for real-time response delivery
- **Error Handling System**: HTTP error responses with proper status codes, API-specific error formats, and localization support
- **Content Negotiation**: Multi-format response handling for OpenAI and Ollama compatibility with automatic format detection
- **Route Composition Integration**: Coordinated with routes_all for unified route composition, hierarchical authorization, and middleware orchestration

### Domain Object Integration
Comprehensive integration with objs crate for consistent API operations:

- **Alias Resolution System**: Model alias lookup, validation, and priority-based resolution using objs domain objects with comprehensive error handling
- **Error System Integration**: API errors implement AppError trait for consistent HTTP response generation with status code mapping
- **Parameter Validation Framework**: OpenAI parameter validation using objs validation rules, error handling, and user-actionable feedback
- **Localization Support**: Multi-language error messages via objs LocalizationService integration with format-specific message formatting

## API Orchestration Workflows

### Multi-Service Chat Completion Coordination
Sophisticated request processing with comprehensive service orchestration:

1. **Request Reception and Validation**: HTTP routes receive OpenAI-compatible requests with comprehensive parameter validation and format checking
2. **Model Alias Resolution**: DataService resolves model aliases with priority-based selection and availability validation
3. **Context Coordination**: SharedContext manages LLM server instances with load balancing and health monitoring for request processing
4. **Response Streaming Orchestration**: Server-sent events handle real-time streaming responses with proper formatting and connection management
5. **Error Translation and Recovery**: Service errors converted to OpenAI-compatible HTTP responses with localization and recovery guidance

### Dual-Format API Support Orchestration
Comprehensive API compatibility coordination with sophisticated format handling:

**OpenAI API Workflow**:
1. **Parameter Validation**: Complete OpenAI parameter validation using async-openai types with comprehensive error handling
2. **Request Processing**: Standard OpenAI request processing with model alias resolution and service coordination
3. **Response Formatting**: OpenAI-compatible response formatting with proper JSON structure and metadata
4. **Streaming Support**: SSE streaming with OpenAI chunk format, connection management, and error recovery

**Ollama API Workflow**:
1. **Format Translation**: Bidirectional Ollama request format conversion to OpenAI internal format with parameter preservation
2. **Parameter Mapping**: Intelligent Ollama-specific parameter mapping to OpenAI equivalents with semantic preservation
3. **Response Conversion**: OpenAI responses converted to Ollama-compatible format with proper field mapping
4. **Ecosystem Integration**: Full support for Ollama CLI tools, ecosystem compatibility, and third-party integrations

### Cross-Format Error Coordination
Sophisticated error handling across different API formats:

- **OpenAI Error Format**: Standard OpenAI error responses with proper error types, codes, and detailed error information
- **Ollama Error Format**: Ollama-compatible error responses with simplified error structure and appropriate formatting
- **Service Error Translation**: Service errors converted to appropriate API-specific formats with consistent error handling
- **Localization Support**: Multi-language error messages coordinated with objs error system and format-specific message formatting

## Important Constraints

### OpenAI API Compatibility Requirements

- **Specification Compliance**: All endpoints must maintain strict OpenAI API specification compliance for ecosystem compatibility and tool integration
- **Parameter Validation**: Request validation must use async-openai types for parameter validation, format consistency, and comprehensive error handling
- **Response Format Compliance**: Response formats must match OpenAI specification exactly for SDK compatibility, tool integration, and ecosystem support
- **Error Response Standards**: Error responses must follow OpenAI error format with proper error types, HTTP status codes, and detailed error information
- **Streaming Response Requirements**: Streaming responses must use proper SSE formatting with OpenAI chunk structure and connection management

### Ollama Ecosystem Integration Standards

- **CLI Tool Compatibility**: Ollama endpoints must provide full support for Ollama CLI tools and ecosystem integrations with proper format handling
- **Bidirectional Translation**: Request/response format translation must be bidirectional and lossless where possible with semantic preservation
- **Parameter Mapping Standards**: Parameter mapping between Ollama and OpenAI formats must preserve semantic meaning and functionality
- **Error Handling Consistency**: Error handling must provide Ollama-compatible error responses while maintaining internal consistency and proper error reporting
- **Model Metadata Compatibility**: Model metadata must be compatible with Ollama model discovery, management tools, and ecosystem expectations

### HTTP Infrastructure Coordination Rules

- **RouterState Integration**: All routes must use RouterState dependency injection for consistent service access and comprehensive dependency management
- **Model Resolution Coordination**: Model alias resolution must coordinate with DataService for consistent model discovery and priority-based selection
- **Streaming Infrastructure Integration**: Response streaming must integrate with server_core streaming infrastructure for performance and connection management
- **Error Translation Requirements**: Error handling must translate service errors to appropriate API-specific formats with localization and user guidance
- **Authentication Integration Standards**: Authentication must integrate with auth_middleware for consistent API security, access control, and authorization across endpoints

## API Extension Patterns

### Adding New OpenAI Endpoints
When implementing additional OpenAI API endpoints:

1. **Specification Compliance**: Follow OpenAI API specification exactly for parameter validation, response formats, and error handling
2. **Service Coordination**: Use RouterState for consistent AppService access, business logic coordination, and dependency management
3. **Error Handling Integration**: Implement API-specific errors that convert to OpenAI-compatible responses with proper localization
4. **Documentation Standards**: Add comprehensive OpenAPI documentation with examples, proper schemas, and endpoint specifications
5. **Testing Infrastructure**: Create comprehensive API compatibility tests with OpenAI SDK validation and ecosystem integration testing

### Extending Ollama Compatibility
For new Ollama API endpoints and features:

1. **Format Translation Design**: Design comprehensive bidirectional translation between Ollama and OpenAI formats with semantic preservation
2. **Parameter Mapping Intelligence**: Create intelligent parameter mapping that preserves functionality and maintains compatibility
3. **Ecosystem Integration Testing**: Validate compatibility with Ollama CLI tools, ecosystem integrations, and third-party applications
4. **Error Consistency Management**: Provide Ollama-compatible error responses while maintaining internal error handling and consistency
5. **Documentation Standards**: Document Ollama-specific behavior, compatibility considerations, and ecosystem integration patterns

### Cross-Format Integration Patterns
For features that span both API formats:

1. **Unified Processing Architecture**: Design internal processing that supports both API formats efficiently with minimal overhead
2. **Format Abstraction Layers**: Create sophisticated abstraction layers that handle format-specific concerns while maintaining consistency
3. **Response Transformation Pipeline**: Implement efficient response transformation for different API formats with proper streaming support
4. **Error Coordination System**: Ensure consistent error handling across different API format requirements with proper localization
5. **Performance Optimization Strategy**: Optimize for minimal overhead in format translation, processing, and response generation