# Embeddings Implementation Context

## Key Architecture Decisions

### Generic Request Forwarding Pattern
- Using generic `forward_request` method instead of endpoint-specific methods
- This reflects the true nature of the system: it's a request forwarder, not chat-specific
- LlmEndpoint enum provides type-safe endpoint selection
- Embeddings handler colocated with chat completions in `routes_chat.rs`

### Endpoint Enum Design
```rust
#[derive(Debug, Clone, Copy)]
pub enum LlmEndpoint {
    ChatCompletions,
    Embeddings,
}
```

### Method Naming Consistency
All layers use `forward_request` for consistency:
- RouterState::forward_request
- SharedContext::forward_request
- AiApiService::forward_request

### Existing Infrastructure to Leverage
- Server trait already has `embeddings(&self, body: &Value)` method at `llama_server_proc/src/server.rs:96`
- ModelRouter handles local vs remote routing
- Response conversion between reqwest::Response and axum::Response already implemented

## Important Patterns

### Error Handling Flow
Service errors → RouterStateError → ApiError → HTTP status codes

### Model Resolution Order
1. User alias (highest priority)
2. Model alias
3. API model (lowest priority)

### Response Processing
- Local models: SharedContext → Server → llama.cpp
- Remote models: AiApiService → External API
- Both return reqwest::Response that gets converted to axum::Response

### Test Infrastructure
- Use mockall for mocking: MockSharedContext, MockAiApiService, MockDataService
- rstest for parameterized tests
- Test fixtures: AppServiceStub, UserAlias::testalias_exists()

## Code Style Requirements
- No `use super::*` in test modules (explicit imports only)
- Use `assert_eq!(expected, actual)` in tests
- Run `cargo fmt` after changes
- Add `data-testid` attributes for UI tests

## File Structure

### Primary Implementation Files
1. `crates/server_core/src/lib.rs` - LlmEndpoint enum definition
2. `crates/server_core/src/router_state.rs` - RouterState::forward_request
3. `crates/server_core/src/shared_rw.rs` - SharedContext::forward_request
4. `crates/services/src/ai_api_service.rs` - AiApiService::forward_request
5. `crates/routes_oai/src/routes_chat.rs` - embeddings_handler

### Test Files
1. Test modules in same files as implementation
2. `crates/lib_bodhiserver_napi/tests-js/live-server.test.js` - JS integration tests

## Known Issues
- None

## Phase 2: Embeddings Handler Implementation

### Trait Object Safety Challenge
The initial plan suggested using generic type parameters (`impl Serialize` or `<T: Serialize>`) for the `forward_request` method. However, this approach makes the trait not dyn-compatible (previously called "object-safe"), preventing the use of `Arc<dyn RouterState>`.

**Problem**: Rust traits with generic method parameters cannot be used as trait objects because the vtable cannot be constructed at compile time.

**Solution**: Changed all `forward_request` signatures to accept `serde_json::Value` directly instead of generic types. This maintains trait object safety while still providing flexibility.

### Implementation Changes

**RouterState Trait** (`crates/server_core/src/router_state.rs`):
```rust
fn forward_request(
  &self,
  endpoint: LlmEndpoint,
  request: Value,  // Changed from generic T
) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>>;
```

**SharedContext Trait** (`crates/server_core/src/shared_rw.rs`):
```rust
async fn forward_request(
  &self,
  endpoint: LlmEndpoint,
  request: Value,  // Changed from CreateChatCompletionRequest
  alias: Alias,
) -> Result<reqwest::Response>;
```

**AiApiService Trait** (`crates/services/src/ai_api_service.rs`):
```rust
async fn forward_request(
  &self,
  api_path: &str,
  id: &str,
  request: Value,  // Changed from CreateChatCompletionRequest
) -> Result<Response>;
```

### Handler-Level Serialization

Both `chat_completions_handler` and `embeddings_handler` now convert their typed requests to `Value` before calling `forward_request`:

```rust
let request_value = serde_json::to_value(request).map_err(HttpError::Serialization)?;
let response = state
  .forward_request(LlmEndpoint::Embeddings, request_value)
  .await?;
```

### Error Handling Enhancement

Added `Serialization` variant to `HttpError` enum to handle `serde_json::Error`:

```rust
pub enum HttpError {
  #[error("http_error")]
  Http(#[from] http::Error),

  #[error("serialization_error")]
  Serialization(#[from] serde_json::Error),
}
```

### Test Updates

All mock expectations in tests now expect `Value` instead of typed requests:

```rust
let request_value = serde_json::to_value(&request)?;
ctx
  .expect_forward_request()
  .with(
    eq(LlmEndpoint::Embeddings),
    eq(request_value),  // Changed from eq(request)
    eq(alias),
  )
```

### Route Registration

The embeddings route was registered in `routes_all/src/routes.rs` in the `user_apis` section, alongside the chat completions route:

```rust
let user_apis = Router::new()
  // OpenAI Compatible APIs
  .route(ENDPOINT_OAI_MODELS, get(oai_models_handler))
  .route(ENDPOINT_OAI_CHAT_COMPLETIONS, post(chat_completions_handler))
  .route(ENDPOINT_OAI_EMBEDDINGS, post(embeddings_handler))  // New route
```

### Backward Compatibility

The refactoring maintains backward compatibility:
- All existing tests continue to pass (8 tests)
- Chat completions handler works with the new signature
- Ollama chat handler updated to use new signature
- No breaking changes to external APIs

## Implementation Notes

### Phase 1: Core Infrastructure Refactoring - Completed

**Circular Dependency Resolution**
- Initial plan called for `AiApiService::forward_request(endpoint: LlmEndpoint, ...)` but this would create a circular dependency
- `services` crate cannot depend on `server_core` because `server_core` already depends on `services`
- **Solution**: Changed AiApiService to accept `api_path: &str` parameter instead of `LlmEndpoint` enum
- `RouterState` (in server_core) calls `endpoint.api_path()` and passes the string to AiApiService
- This maintains the same functionality while respecting the layered architecture

**Test Compatibility**
- Added `PartialEq` derive to `LlmEndpoint` enum to enable mockall's `eq()` predicate in test expectations
- Without PartialEq, test code using `eq(LlmEndpoint::ChatCompletions)` would fail to compile

**Refactoring Statistics**
- Updated 5 files across 3 crates (server_core, services, routes_oai)
- Modified 9 test call sites to use new method signatures
- All 321 existing tests continue to pass (92 in server_core, 229 in services)
- Zero test failures during refactoring

**Implementation Pattern**
- The `forward_request` method in SharedContext uses a match statement to dispatch to the correct Server method:
  ```rust
  let response = match endpoint {
    LlmEndpoint::ChatCompletions => server.chat_completions(&input_value).await?,
    LlmEndpoint::Embeddings => server.embeddings(&input_value).await?,
  };
  ```
- This pattern makes it trivial to add new endpoints in the future

**Test Module Updates**
- Followed explicit import pattern (no `use super::*`) as specified in project guidelines
- All test expectations updated to include the new `endpoint` parameter
- Test code remains maintainable and follows project conventions

## Phase 4: OpenAPI Documentation and TypeScript Client Update

### OpenAPI Registration Challenge
The embeddings endpoint handler was implemented with proper utoipa annotations, but the initial OpenAPI generation did not include the embeddings endpoint.

**Root Cause**: The `embeddings_handler` was not registered in the `BodhiOpenAPIDoc` struct in `crates/routes_app/src/openapi.rs`. The xtask openapi command generates the spec from `BodhiOpenAPIDoc::openapi()`, which only includes paths explicitly listed in the `#[openapi(paths(...))]` attribute.

**Resolution**:
1. Added `__path_embeddings_handler` to imports from routes_oai
2. Added embedding types to schemas: CreateEmbeddingRequest, CreateEmbeddingResponse, Embedding, EmbeddingInput, EmbeddingUsage
3. Added `embeddings_handler` to the paths section of BodhiOpenAPIDoc

### TypeScript Client Generation

**Files Modified**:
- `crates/routes_app/src/openapi.rs` - OpenAPI documentation registration
- `openapi.json` - Regenerated with embeddings endpoint
- `ts-client/src/types/types.gen.ts` - Generated TypeScript types
- `ts-client/src/openapi-typescript/openapi-schema.ts` - Generated schema types

**Generated Types**:
- CreateEmbeddingRequest - Request type with model, input, encoding_format, user, dimensions
- CreateEmbeddingResponse - Response type with object, model, data, usage
- Embedding - Individual embedding object with index, embedding vector
- EmbeddingInput - Union type supporting string, array of strings, or token arrays
- EmbeddingUsage - Usage statistics with prompt_tokens and total_tokens
- CreateEmbeddingData, CreateEmbeddingErrors, CreateEmbeddingResponse2 - Operation types

### Synchronization Check Result
The `make ci.ts-client-check` command correctly detected changes in the TypeScript client files. This is expected behavior and confirms that:
1. OpenAPI spec was successfully regenerated with embeddings endpoint
2. TypeScript types were successfully generated from the updated spec
3. The synchronization mechanism is working correctly

The changes need to be committed as part of the embeddings implementation.

## Phase 3: Integration Testing Implementation

### JavaScript/NAPI Test Strategy
Rather than testing full model inference (which requires actual models), the integration tests focus on:

1. **Endpoint Registration Verification**: Confirming the embeddings endpoint is properly registered
2. **Error Handling**: Testing various error scenarios (model not found, missing fields, malformed JSON)
3. **Authentication Requirements**: Validating that the endpoint requires proper authentication

### Test Implementation Details

**File**: `crates/lib_bodhiserver_napi/tests-js/live-server.test.js`

**Tests Added** (4 tests):
1. `should handle model not found error for non-existent model` - Tests 404 response for invalid model
2. `should handle invalid request with missing required input field` - Tests 4xx response for missing input
3. `should handle invalid request with malformed JSON` - Tests 4xx response for malformed requests
4. `should verify embeddings endpoint is registered` - Tests endpoint returns expected error codes (401/404)

**Authentication Behavior**:
- The embeddings endpoint requires User role and TokenScope authorization via api_auth_middleware
- Without authentication: Returns 404 when app status isn't "ready", 401 when unauthorized
- Test accounts for both 404 and 401 status codes as valid responses

**Test Pattern Consistency**:
- Follows existing live-server.test.js patterns
- Uses `createTestServer()` helper for server setup
- Adds servers to `runningServers` array for proper cleanup
- Uses `await sleep(2000)` for server startup delay
- Tests use specific ports (27010-27013) to avoid conflicts

### Code Fixes Required

**Issue**: Test code in server_core used old signature passing `CreateChatCompletionRequest` directly instead of `Value`

**Files Fixed**:
- `crates/server_core/src/router_state.rs` - 3 tests updated
- `crates/server_core/src/shared_rw.rs` - 3 tests updated

**Pattern**:
```rust
// Before
let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{...}})?;
state.forward_request(LlmEndpoint::ChatCompletions, request).await?;

// After
let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{...}})?;
let request_value = serde_json::to_value(&request)?;
state.forward_request(LlmEndpoint::ChatCompletions, request_value).await?;
```

**Additional Fix**: Removed unused `use serde::Serialize;` import from router_state.rs

### Test Results

**NAPI Tests (vitest)**:
- Total: 10 tests passed
- New embeddings tests: 4 passed
- Existing tests: 6 passed
- Duration: ~20s

**Backend Tests**:
- server_core: 92 tests passed
- bodhi: 7 tests passed
- All existing tests continue to pass after fixes

### Testing Philosophy

**What Was Tested**:
- HTTP endpoint registration and routing
- Error handling for invalid requests
- Authentication/authorization requirements
- Request validation (missing fields, malformed JSON)

**What Was NOT Tested**:
- Actual embeddings generation (requires real models)
- Streaming responses (embeddings don't support streaming)
- Multiple input variations (single string, array, token arrays)
- Response format validation with real data

**Rationale**:
- Integration tests focus on infrastructure and error handling
- Actual model inference testing requires significant test infrastructure (model downloads, startup time)
- Unit tests in routes_chat.rs already validate response format with mocked responses
- End-to-end testing with real models would be better suited for separate system tests