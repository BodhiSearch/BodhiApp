# OpenAI-Compatible Embeddings Endpoint Implementation Plan

## Executive Summary

Implement an OpenAI-compatible embeddings endpoint by leveraging the existing request forwarding architecture. Rather than creating separate methods for each endpoint type, we'll generalize the current "chat completions" infrastructure to handle multiple endpoint types through parameterization using Rust enums.

## Architecture Analysis

### Current System Understanding

The system is fundamentally a **request forwarding layer** that routes requests to either:
1. **Local Models**: Via llama.cpp server process
2. **Remote Models**: Via external API providers (OpenAI, etc.)

The current implementation uses "chat_completions" naming throughout, but this is misleading - it's actually a generic request forwarding mechanism that can handle any endpoint type.

### Key Components Analysis

#### 1. HTTP Route Layer (`crates/routes_oai/src/routes_chat.rs`)
- **File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/routes_oai/src/routes_chat.rs`
- **Current Handler**: `chat_completions_handler` (lines 94-106)
- **Purpose**: Receives HTTP requests and delegates to RouterState
- **Key Logic**:
  ```rust
  pub async fn chat_completions_handler(
    State(state): State<Arc<dyn RouterState>>,
    WithRejection(Json(request), _): WithRejection<Json<CreateChatCompletionRequest>, ApiError>,
  ) -> Result<Response, ApiError> {
    let response = state.chat_completions(request).await?;
    // ... response building
  }
  ```

#### 2. RouterState Layer (`crates/server_core/src/router_state.rs`)
- **File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/server_core/src/router_state.rs`
- **Current Method**: `chat_completions` (lines 75-125)
- **Purpose**: Routes requests based on model type (local vs remote)
- **Key Components**:
  - Uses `ModelRouter` to determine routing destination (lines 81-82)
  - Routes to `SharedContext` for local models (lines 84-87)
  - Routes to `AiApiService` for remote models (lines 89-122)

#### 3. Model Router (`crates/server_core/src/model_router.rs`)
- **File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/server_core/src/model_router.rs`
- **Purpose**: Determines whether a model is local or remote
- **Resolution Order**:
  1. User alias (highest priority)
  2. Model alias
  3. API models (lowest priority)
- **Return Type**: `RouteDestination` enum with `Local(Alias)` or `Remote(ApiAlias)`

#### 4. SharedContext Layer (`crates/server_core/src/shared_rw.rs`)
- **File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/server_core/src/shared_rw.rs`
- **Current Method**: `chat_completions` (lines 176-274)
- **Purpose**: Manages local LLM server lifecycle and forwards requests
- **Key Features**:
  - Model loading strategy (Continue/Load/DropAndLoad)
  - Server state management with RwLock
  - Delegates to `Server::chat_completions()`

#### 5. LLM Server Process (`crates/llama_server_proc/src/server.rs`)
- **File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/llama_server_proc/src/server.rs`
- **Existing Methods**:
  - `chat_completions(&self, body: &Value)` (line 94)
  - `embeddings(&self, body: &Value)` (line 96) - **Already exists!**
  - `tokenize(&self, body: &Value)` (line 98)
  - `detokenize(&self, body: &Value)` (line 100)
- **Implementation Pattern**: All methods use `proxy_request(endpoint, body)`

#### 6. AI API Service (`crates/services/src/ai_api_service.rs`)
- **File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/services/src/ai_api_service.rs`
- **Current Method**: `forward_chat_completion` (lines 223-271)
- **Purpose**: Forwards requests to remote API providers
- **Features**:
  - API key management via DbService
  - Response streaming support
  - Error handling and status code mapping

### OpenAI Types Available

From `async-openai` crate:
- **Request**: `CreateEmbeddingRequest` (model, input, encoding_format, user, dimensions)
- **Response**: `CreateEmbeddingResponse` (object, model, data, usage)
- **Input Types**: `EmbeddingInput` (String, Vec<String>, Vec<Vec<i32>>)

## Proposed Solution: Generic Request Forwarding

### Core Concept

Instead of creating separate methods for each endpoint type, we'll:
1. Create an enum to represent endpoint types
2. Rename existing methods to reflect their generic nature
3. Pass the endpoint type as a parameter
4. Add the embeddings handler in the same file as chat completions

### Endpoint Enum Design

```rust
#[derive(Debug, Clone, Copy)]
pub enum LlmEndpoint {
    ChatCompletions,
    Embeddings,
}

impl LlmEndpoint {
    pub fn path(&self) -> &str {
        match self {
            Self::ChatCompletions => "/v1/chat/completions",
            Self::Embeddings => "/v1/embeddings",
        }
    }

    pub fn api_path(&self) -> &str {
        match self {
            Self::ChatCompletions => "/chat/completions",
            Self::Embeddings => "/embeddings",
        }
    }
}
```

### Method Renaming Strategy

Consistent use of `forward_request` throughout the stack:

1. **RouterState trait**:
   - `chat_completions()` → `forward_request(endpoint: LlmEndpoint, request: impl Serialize)`

2. **SharedContext trait**:
   - `chat_completions()` → `forward_request(endpoint: LlmEndpoint, request: Value, alias: Alias)`

3. **AiApiService trait**:
   - `forward_chat_completion()` → `forward_request(endpoint: LlmEndpoint, id: &str, request: impl Serialize)`

4. **Keep as-is** (already endpoint-specific):
   - `Server::chat_completions()`
   - `Server::embeddings()`

## Implementation Plan - Phase-Wise

### Phase 1: Core Infrastructure Refactoring
**Objective**: Generalize the request forwarding infrastructure

1. **Create Endpoint Enum** (`crates/server_core/src/lib.rs`):
   - Define `LlmEndpoint` enum
   - Add path helper methods
   - Export from server_core

2. **Update SharedContext** (`crates/server_core/src/shared_rw.rs`):
   - Rename `chat_completions` to `forward_request`
   - Add `endpoint: LlmEndpoint` parameter
   - Update method to dispatch to appropriate Server method based on endpoint
   - Update all call sites in tests

3. **Update RouterState** (`crates/server_core/src/router_state.rs`):
   - Rename `chat_completions` to `forward_request`
   - Add `endpoint: LlmEndpoint` parameter
   - Pass endpoint through to SharedContext and AiApiService
   - Update tests

4. **Update AiApiService** (`crates/services/src/ai_api_service.rs`):
   - Rename `forward_chat_completion` to `forward_request`
   - Add `endpoint: LlmEndpoint` parameter
   - Use endpoint to construct API path
   - Update tests

**Tests**: Run `cargo test -p server_core` and `cargo test -p services`

### Phase 2: Add Embeddings Handler
**Objective**: Add embeddings endpoint in the same file as chat completions

1. **Update routes_chat.rs** (`crates/routes_oai/src/routes_chat.rs`):
   - Keep existing `chat_completions_handler`
   - Add `embeddings_handler` function
   - Both use the generalized `forward_request` method
   - Add OpenAPI documentation for embeddings

2. **Register Route** (`crates/routes_oai/src/lib.rs`):
   - Add embeddings route at `/v1/embeddings`
   - Import embeddings handler

3. **Add Tests**:
   - Test embeddings with local model alias
   - Test embeddings with remote API alias
   - Test error cases

**Tests**: Run `cargo test -p routes_oai`

### Phase 3: Integration Testing
**Objective**: Ensure end-to-end functionality

1. **Rust Integration Tests**:
   - Add embeddings tests in routes_chat.rs test module
   - Test complete request flow
   - Verify response format

2. **JavaScript/NAPI Tests** (`crates/lib_bodhiserver_napi/tests-js/`):
   - Add embeddings tests to `live-server.test.js`
   - Test with actual llama.cpp server
   - Verify OpenAI compatibility

**Tests**: Run `make test`

### Phase 4: Documentation and Client Updates
**Objective**: Update OpenAPI spec and TypeScript client

1. **Generate OpenAPI Spec**:
   - Run `cargo run --package xtask openapi`
   - Verify embeddings endpoint documented

2. **Update TypeScript Client**:
   - Run `cd ts-client && npm run generate`
   - Verify embeddings types generated

3. **Verify Synchronization**:
   - Run `make ci.ts-client-check`

**Tests**: Verify generated files are correct

## Testing Strategy

### Unit Tests
Each layer will have comprehensive unit tests:
- Mock dependencies using mockall
- Test both success and error paths
- Verify request/response transformation

### Integration Tests
- Test complete flow from HTTP to llama.cpp/API
- Verify model routing logic
- Test with actual test fixtures

### Live Server Tests
- Start real llama.cpp server
- Send actual embeddings requests
- Verify OpenAI-compatible responses

## Error Handling

### Error Types to Handle
1. **Model not found**: When requested model doesn't exist
2. **Server not running**: When local llama.cpp isn't started
3. **API errors**: Rate limiting, authentication failures
4. **Network errors**: Timeouts, connection failures

### Error Propagation
- Service errors → RouterStateError → ApiError → HTTP status codes
- Maintain error context through the stack
- Provide meaningful error messages to clients

## Migration Path

### Backward Compatibility
During the refactoring, we'll:
1. Keep old method names as deprecated wrappers initially
2. Update all internal call sites
3. Remove deprecated methods in a follow-up PR

### Gradual Rollout
1. Phase 1: Internal refactoring (no API changes)
2. Phase 2: Add new embeddings endpoint
3. Phase 3: Deprecate old naming (if exposing internal APIs)
4. Phase 4: Remove deprecated code

## Success Criteria

1. ✅ Embeddings endpoint works with OpenAI-compatible requests
2. ✅ Local models route through llama.cpp server
3. ✅ Remote models route through external APIs
4. ✅ Consistent `forward_request` naming throughout
5. ✅ No code duplication between endpoints
6. ✅ All existing tests pass
7. ✅ New embeddings tests pass
8. ✅ OpenAPI documentation updated
9. ✅ TypeScript client includes embeddings support
10. ✅ Performance comparable to chat completions

## Future Extensibility

This architecture will make it trivial to add:
- `/v1/tokenize` endpoint
- `/v1/detokenize` endpoint
- Any other llama.cpp endpoints
- Custom routing logic per endpoint type

Simply add new variants to `LlmEndpoint` enum and handlers in routes file.

## Code Examples

### Generalized SharedContext Method
```rust
async fn forward_request(
    &self,
    endpoint: LlmEndpoint,
    mut request: Value,
    alias: Alias,
) -> Result<reqwest::Response> {
    // ... existing model loading logic ...

    // Dispatch to appropriate server method
    let response = match endpoint {
        LlmEndpoint::ChatCompletions => {
            server.chat_completions(&request).await?
        }
        LlmEndpoint::Embeddings => {
            server.embeddings(&request).await?
        }
    };

    Ok(response)
}
```

### Embeddings Handler
```rust
pub async fn embeddings_handler(
    State(state): State<Arc<dyn RouterState>>,
    WithRejection(Json(request), _): WithRejection<Json<CreateEmbeddingRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let response = state.forward_request(
        LlmEndpoint::Embeddings,
        serde_json::to_value(request)?
    ).await?;

    // Build response (same as chat_completions_handler)
    let mut response_builder = Response::builder().status(response.status());
    if let Some(headers) = response_builder.headers_mut() {
        *headers = response.headers().clone();
    }
    let stream = response.bytes_stream();
    let body = Body::from_stream(stream);
    Ok(response_builder.body(body).map_err(HttpError::Http)?)
}
```

## Appendix: File References

### Primary Files to Modify
1. `crates/routes_oai/src/routes_chat.rs` - Add embeddings handler
2. `crates/server_core/src/router_state.rs` - Generalize forward_request
3. `crates/server_core/src/shared_rw.rs` - Generalize forward_request
4. `crates/services/src/ai_api_service.rs` - Generalize forward_request
5. `crates/routes_oai/src/lib.rs` - Register embeddings route

### Test Files to Update/Create
1. `crates/routes_oai/src/routes_chat.rs` (test module) - Add embeddings tests
2. `crates/server_core/src/router_state.rs` (test module) - Update tests
3. `crates/server_core/src/shared_rw.rs` (test module) - Update tests
4. `crates/services/src/ai_api_service.rs` (test module) - Update tests
5. `crates/lib_bodhiserver_napi/tests-js/live-server.test.js` - Add JS tests

### Supporting Files
1. `crates/server_core/src/lib.rs` - Add LlmEndpoint enum
2. `crates/llama_server_proc/src/server.rs` - Already has embeddings() method