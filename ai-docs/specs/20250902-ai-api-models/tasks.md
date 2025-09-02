# Remote AI API Integration - Task Breakdown

## Layer 1: Domain Objects Foundation ✅ **COMPLETED**
**Goal: Establish core data structures**

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

## Layer 3: Business Services 
**Goal: External API integration**

### Task 3.1: AI API Service ✅ **COMPLETED**
- ✅ Create `crates/services/src/ai_api_service.rs`
- ✅ Implement OpenAI API client with reqwest
- ✅ Add test prompt functionality (30 char limit)
- ✅ Add model fetching from OpenAI API
- ✅ Add chat completion forwarding
- ✅ **Test:** Mock HTTP client tests for all operations

### Task 3.2: Model Router Service 🔄 **IN PROGRESS**
- 🔄 Create `crates/server_core/src/model_router.rs`
- 🔄 Implement model resolution logic
- **⚠️ UPDATED REQUIREMENT**: Handle conflict resolution with **CORRECTED ORDER**: user alias → model → api models (NOT api models first)
- 🔄 Coordinate with DataService and DbService
- 🔄 **Test:** Router decision logic with various scenarios

## Layer 4: HTTP Routes
**Goal: API endpoints for management**

### Task 4.1: API Model Management Routes
- Create `crates/routes_app/src/routes_api_models.rs`
- Implement CRUD endpoints for API models
- Add test endpoints (test prompt, fetch models)
- Create request/response objects in `objs.rs`
- **Test:** HTTP endpoint tests with mock services

### Task 4.2: OpenAPI Documentation
- Update `crates/routes_app/src/openapi.rs`
- Add all new endpoints to OpenAPI spec
- Add request/response schemas
- Update route composition
- **Test:** Verify OpenAPI spec generation

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