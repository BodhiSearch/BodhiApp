# Remote AI API Integration - Task Breakdown

## Layer 1: Domain Objects Foundation ✅ **COMPLETED**
**Goal: Establish core data structures**

**Files Modified:**
- `crates/objs/src/model_alias.rs` - ModelAlias enum implementation
- `crates/objs/src/api_model_alias.rs` - ApiModelAlias struct
- `crates/objs/src/alias.rs` - Added RemoteApi to AliasSource enum
- `crates/objs/src/lib.rs` - Module exports
- `crates/objs/src/test_utils/objs.rs` - Test utilities

### Task 1.1: Create ModelAlias System ✅ **COMPLETED**
- ✅ Keep existing `Alias` struct unchanged in `crates/objs/src/alias.rs`
- ✅ Create new `ModelAlias` enum in `crates/objs/src/model_alias.rs` with flat variants `User`, `Model`, `Api`
- ✅ Add `RemoteApi` variant to `AliasSource` enum
- ✅ Implement `can_serve(&self, model: &str) -> bool` method on `ModelAlias`
- ✅ Update serialization/deserialization for new enum
- ✅ **Test:** Unit tests for all three variants and serialization

### Task 1.2: Create ApiModelAlias ✅ **COMPLETED**
- ✅ Create `crates/objs/src/api_model_alias.rs`
- ✅ Define `ApiModelAlias` struct with all fields
- ✅ Implement required traits (Debug, Clone, Serialize, Deserialize)
- ✅ **Test:** Unit tests for struct creation and serialization

## Layer 2: Database Layer ✅ **COMPLETED**
**Goal: Persistent storage with encryption**

**Files Modified:**
- `crates/services/migrations/0004_api_models.up.sql` - Database schema
- `crates/services/migrations/0004_api_models.down.sql` - Rollback migration
- `crates/services/src/db/encryption.rs` - AES-GCM encryption service
- `crates/services/src/db/service.rs` - DbService API model methods
- `crates/services/src/db/mod.rs` - Module exports
- `crates/services/src/test_utils/db.rs` - Test utilities
- `crates/services/src/resources/en-US/messages.ftl` - Error messages

### Task 2.1: Database Migration ✅ **COMPLETED**
- ✅ Create migration `0004_api_models.up.sql` and `.down.sql`
- ✅ Define table schema with `alias` as primary key
- ✅ Add indexes for performance
- ✅ **Test:** Migration up/down testing

### Task 2.2: Database Encryption Service ✅ **COMPLETED**
- ✅ Create `crates/services/src/db/encryption.rs` as private module
- ✅ Implement AES-GCM encryption with PBKDF2 key derivation
- ✅ Add salt generation and key masking utilities
- ✅ **Test:** Encryption/decryption round-trip tests with different salts

### Task 2.3: Database Service Extension ✅ **COMPLETED**
- ✅ Extend `DbService` with API model methods
- ✅ Integrate private encryption service
- ✅ Implement CRUD operations for API models
- ✅ **Test:** Database operations with mock encryption service

## Layer 3: Business Services ✅ **COMPLETED**
**Goal: External API integration**

**Files Modified:**
- `crates/services/src/ai_api_service.rs` - OpenAI API client implementation
- `crates/services/src/app_service.rs` - AppService trait extension
- `crates/services/src/lib.rs` - Module exports
- `crates/services/src/test_utils/app.rs` - AppServiceStub updates
- `crates/lib_bodhiserver/src/app_service_builder.rs` - Service registration
- `crates/server_core/src/model_router.rs` - Model routing logic
- `crates/server_core/src/lib.rs` - Module exports

### Task 3.1: AI API Service ✅ **COMPLETED**
- ✅ Create `crates/services/src/ai_api_service.rs`
- ✅ Implement OpenAI API client with reqwest
- ✅ Add test prompt functionality (30 char limit)
- ✅ Add model fetching from OpenAI API
- ✅ Add chat completion forwarding
- ✅ **Test:** Mock HTTP client tests for all operations

### Task 3.2: Model Router Service ✅ **COMPLETED**
- ✅ Create `crates/server_core/src/model_router.rs`
- ✅ Implement model resolution logic
- ✅ **CORRECTED ORDER**: user alias → model → api models resolution order
- ✅ Coordinate with DataService and DbService
- ✅ **Test:** Router decision logic with various scenarios

## Layer 4: HTTP Routes ✅ **COMPLETED**
**Goal: API endpoints for management**

**Files Modified:**
- `crates/routes_app/src/api_models_dto.rs` - Request/response DTOs with validation
- `crates/routes_app/src/routes_api_models.rs` - HTTP route handlers implementation
- `crates/routes_app/src/lib.rs` - Module exports and integration
- `crates/routes_app/src/openapi.rs` - OpenAPI documentation updates
- `crates/routes_all/src/routes.rs` - Route composition and middleware integration
- `crates/objs/src/api_tags.rs` - Added API_TAG_API_MODELS constant
- `openapi.json` - Generated OpenAPI specification
- `ts-client/src/types/types.gen.ts` - Generated TypeScript types

### Task 4.1: API Model Management Routes ✅ **COMPLETED**

#### 4.1.1: Create Request/Response DTOs (`crates/routes_app/src/api_models_dto.rs`) ✅ **COMPLETED**
- ✅ Create `CreateApiModelRequest` struct with validation
  - ✅ `alias: String` (unique identifier)
  - ✅ `provider: String` (e.g., "openai", "anthropic")
  - ✅ `base_url: String` (API endpoint URL)
  - ✅ `api_key: String` (authentication key)
  - ✅ `models: Vec<String>` (available models list)
- ✅ Create `UpdateApiModelRequest` for partial updates
- ✅ Create `TestPromptRequest` with 30-char limit validation
- ✅ Create response objects: `ApiModelResponse`, `TestPromptResponse`, `FetchModelsResponse`
- ✅ Add API key masking utility (show first 3, last 6 chars)
- ✅ Add `PaginatedApiModelResponse` for list endpoints
- ✅ **Test:** Serialization/deserialization, validation rules

#### 4.1.2: Implement HTTP Route Handlers (`crates/routes_app/src/routes_api_models.rs`) ✅ **COMPLETED**
**Endpoints:**
- ✅ `GET /bodhi/v1/api-models` - List all configurations (paginated)
- ✅ `GET /bodhi/v1/api-models/{alias}` - Get specific configuration
- ✅ `POST /bodhi/v1/api-models` - Create new configuration
- ✅ `PUT /bodhi/v1/api-models/{alias}` - Update existing configuration  
- ✅ `DELETE /bodhi/v1/api-models/{alias}` - Delete configuration
- ✅ `POST /bodhi/v1/api-models/test` - Test connectivity with prompt (corrected endpoint)
- ✅ `POST /bodhi/v1/api-models/fetch-models` - Fetch available models (corrected endpoint)

**Updated AiApiService Integration:**
- ✅ Use new interface: `test_prompt(api_key, base_url, model, prompt)`
- ✅ Use new interface: `fetch_models(api_key, base_url)`
- ✅ Handle parameter extraction from database for test endpoints

**Implementation Pattern:**
```rust
pub async fn create_api_model_handler(
    State(state): State<Arc<dyn RouterState>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateApiModelRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiModelResponse>), ApiError> {
    // 1. Validate input (base_url format, alias uniqueness)
    // 2. Create ApiModelAlias from request
    // 3. Use DbService to save with encrypted API key
    // 4. Return response with masked API key
}

pub async fn test_api_model_handler(
    State(state): State<Arc<dyn RouterState>>,
    Path(alias): Path<String>,
    WithRejection(Json(payload), _): WithRejection<Json<TestPromptRequest>, ApiError>,
) -> Result<Json<TestPromptResponse>, ApiError> {
    // 1. Get API model config from database (alias, base_url, models)
    // 2. Get decrypted API key from database
    // 3. Use AiApiService.test_prompt(api_key, base_url, model, prompt)
    // 4. Return success/failure response
}
```

- ✅ **Test:** Use TestDbService with real database records, not mocks

#### 4.1.3: Update Module Exports ✅ **COMPLETED**
- ✅ Update `crates/routes_app/src/lib.rs` to export new DTOs (implemented in api_models_dto.rs)
- ✅ Update `crates/routes_app/src/lib.rs` to include new module
- ✅ Add `API_TAG_API_MODELS` constant to `crates/objs/src/lib.rs`

### Task 4.2: OpenAPI Documentation ✅ **COMPLETED**

#### 4.2.1: Update OpenAPI Schema (`crates/routes_app/src/openapi.rs`) ✅ **COMPLETED**
- ✅ Add endpoint constant: `make_ui_endpoint!(ENDPOINT_API_MODELS, "api-models")`
- ✅ Add new tag: `(name = API_TAG_API_MODELS, description = "Remote AI API model configuration")`
- ✅ Add all DTOs to components.schemas section
- ✅ Add all path handlers to openapi paths section
- ✅ Update imports to include new handlers

#### 4.2.2: Route Composition Integration (`crates/routes_all/src/routes.rs`) ✅ **COMPLETED**
- ✅ Import all new handlers and endpoint constants
- ✅ Add routes to `power_user_apis` layer (requires PowerUser role)
- ✅ Apply proper middleware for authentication and authorization
- ✅ Maintain consistent error handling with existing patterns

**Authorization Requirements:**
- ✅ All API model management endpoints require PowerUser role or higher
- ✅ Test endpoints also require PowerUser role (to prevent API key abuse)
- ✅ Use same middleware pattern as existing model management endpoints

### Task 4.3: Comprehensive Testing Strategy ✅ **COMPLETED**

#### 4.3.1: Unit Tests ✅ **COMPLETED**
- ✅ Test request/response serialization with various payloads
- ✅ Test API key masking logic (ensure only safe characters exposed)
- ✅ Test validation rules for all request types
- ✅ Test error scenarios (invalid URLs, empty fields, etc.)

#### 4.3.2: Integration Tests with TestDbService ✅ **COMPLETED**
```rust
#[rstest]
#[awt]
#[tokio::test]
async fn test_create_api_model_handler(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
) -> anyhow::Result<()> {
    // 1. Set up test data in real database
    // 2. Create RouterState with TestDbService
    // 3. Call handler with test request
    // 4. Verify database state and response
    // 5. Test edge cases (duplicate alias, invalid data)
    Ok(())
}
```

- ✅ Use real TestDbService (not mocks) for database operations
- ✅ Test full CRUD flow with actual encryption/decryption
- ✅ Test authorization scenarios (implemented in routes_all middleware)
- ✅ Test pagination and sorting functionality
- ✅ Test error propagation from services to HTTP responses

#### 4.3.3: AiApiService Integration Tests ✅ **COMPLETED**
```rust
#[rstest]
#[awt]
#[tokio::test]
async fn test_api_model_test_prompt_handler(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
) -> anyhow::Result<()> {
    // 1. Insert API model record with encrypted key
    // 2. Create mock AiApiService or use test implementation
    // 3. Test successful prompt test
    // 4. Test API errors (unauthorized, rate limit, etc.)
    // 5. Verify proper error translation to HTTP responses
    Ok(())
}
```

- ✅ Test integration with updated AiApiService interface
- ✅ Test parameter passing (api_key, base_url, model, prompt)
- ✅ Test error handling for API failures
- ✅ Test streaming response handling where applicable

**Security Testing:**
- ✅ Verify API keys are never exposed in responses
- ✅ Verify proper encryption/decryption of stored keys  
- ✅ Test authorization boundaries (User vs PowerUser access)
- ✅ Test input validation prevents injection attacks

## Layer 5: Request Routing Integration
**Goal: Integrate routing into chat flow**

### Task 5.1: Update Chat Completions Route
- Modify `crates/routes_oai/src/routes_chat.rs`
- Add router creation in handler
- Implement routing decision logic
- Handle both local and remote destinations
- Maintain existing error handling
- **Test:** End-to-end routing tests

### Task 5.2: Update Models Endpoint
- Modify `crates/routes_oai/src/routes_models.rs`
- Include API models in model listing
- Add appropriate metadata
- **Test:** Models endpoint returns both local and remote models

## Layer 6: Frontend Implementation
**Goal: User interface for management**

### Task 6.1: API Model Management UI
- Create React components for API model management
- Implement create/list/edit forms
- Add API key masking in display
- Add test prompt and model fetching UI
- **Test:** Component tests with mock API calls

### Task 6.2: UI Route Integration
- Add new routes to Next.js routing
- Integrate with existing navigation
- Add proper error handling and loading states
- **Test:** Integration tests for complete UI flow

## Layer 7: Integration Testing
**Goal: Comprehensive system testing**

### Task 7.1: End-to-End Testing
- Test complete configuration flow
- Test chat completions through remote API
- Test streaming responses
- Test error scenarios
- **Test:** Full system integration tests

### Task 7.2: Performance and Security Testing
- Test encryption performance
- Test API key security (never exposed)
- Test concurrent request handling
- Test database migration performance
- **Test:** Performance benchmarks and security audit

## Review Points

Each layer completion requires:
1. **Unit tests passing** for the layer
2. **Integration tests** with downstream layers
3. **Code review** for architecture compliance
4. **Documentation updates** where needed