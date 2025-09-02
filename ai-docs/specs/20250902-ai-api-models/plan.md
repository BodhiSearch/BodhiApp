# Remote AI API Integration - Implementation Plan

## Overview
Implement OpenAI API provider support with fundamental domain model restructuring, database-backed storage with row-level encryption, and request routing in the chat handler.

## Phase 1: Domain Model Restructuring

### 1.1 Rename and Restructure Alias System
**File: `crates/objs/src/alias.rs`**
- Rename existing `Alias` struct to `LocalModelAlias`
- Create new enum:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Alias {
  User(LocalModelAlias),
  Model(LocalModelAlias),
  Api(ApiModelAlias),
}
```
- Implement `can_serve(&self, model: &str) -> bool` method for model matching

### 1.2 Create ApiModelAlias Structure
**File: `crates/objs/src/api_model_alias.rs`**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiModelAlias {
  pub alias: String,         // Unique identifier (e.g., "openai")
  pub source: AliasSource,    // AliasSource::RemoteApi
  pub provider: String,       // "openai" (lowercase)
  pub base_url: String,       // "https://api.openai.com/v1"
  pub models: Vec<String>,    // Selected models to enable
  pub created_at: DateTime<Utc>,
}
```

### 1.3 Update AliasSource Enum
- Add `RemoteApi` variant to existing `AliasSource` enum

## Phase 2: Database Schema

### 2.1 Primary Key Decision
**Using `alias` as primary key (Recommended)**

**Pros:**
- Simpler schema - one less column
- Natural key that's already unique
- Direct lookups without joins
- Less storage overhead
- Cleaner API - no need to expose internal IDs

**Cons:**
- Cannot rename aliases (immutable by design)
- Slightly larger indexes (string vs integer)
- Foreign key references use strings

**Decision:** Use `alias` as primary key since aliases are immutable by design.

### 2.2 Create Database Migration
**File: `migrations/0004_api_models.up.sql`**
```sql
CREATE TABLE api_models (
  alias TEXT PRIMARY KEY NOT NULL,
  provider TEXT NOT NULL,
  base_url TEXT NOT NULL,
  models TEXT NOT NULL, -- JSON array
  encrypted_api_key BLOB NOT NULL,
  salt BLOB NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_api_models_created_at ON api_models(created_at);
```

## Phase 3: Database Service with Encryption

### 3.1 Private Encryption Service in DbService
**File: `crates/services/src/db/encryption.rs`**
```rust
// Private module inside db service
mod encryption {
  use aes_gcm::{Aes256Gcm, Key, Nonce};
  use pbkdf2::pbkdf2_hmac;
  
  pub(super) struct EncryptionService {
    master_key: Vec<u8>, // From BODHI_ENCRYPTION_KEY
  }
  
  impl EncryptionService {
    pub fn encrypt_with_salt(&self, data: &str, salt: &[u8]) -> Vec<u8>
    pub fn decrypt_with_salt(&self, encrypted: &[u8], salt: &[u8]) -> String
    pub fn mask_api_key(&self, key: &str) -> String // Show first 8 + last 4 chars
    fn derive_key(&self, salt: &[u8]) -> Key<Aes256Gcm>
  }
}
```

### 3.2 Database Service Extension
**File: `crates/services/src/db/service.rs`**
- Initialize internal `EncryptionService` with `BODHI_ENCRYPTION_KEY`
- Add methods:
  - `create_api_model(model: ApiModelAlias, api_key: &str)` 
  - `get_api_model(alias: &str) -> Option<ApiModelAlias>`
  - `update_api_model(alias: &str, model: ApiModelAlias, api_key: Option<&str>)`
  - `delete_api_model(alias: &str)`
  - `list_api_models() -> Vec<ApiModelAlias>`
  - `check_api_model_exists(alias: &str) -> bool`
  - `get_api_key(alias: &str) -> Option<String>` // For AiApiService

## Phase 4: AI API Service

### 4.1 Create AiApiService
**File: `crates/services/src/ai_api_service.rs`**
```rust
pub trait AiApiService: Debug + Send + Sync {
  // Test endpoint - sends prompt (max 30 chars) and returns response
  async fn test_prompt(&self, provider: &str, base_url: &str, api_key: &str, prompt: &str) -> Result<String>;
  
  // Fetch available models from provider
  async fn fetch_models(&self, provider: &str, base_url: &str, api_key: &str) -> Result<Vec<String>>;
  
  // Forward chat completion request using configured alias
  async fn forward_chat_completion(&self, alias: &str, request: CreateChatCompletionRequest) -> Result<Response>;
}
```

### 4.2 Implementation Details
- Use existing SSE forwarding patterns from `crates/server_core/src/fwd_sse.rs`
- Reference `crates/routes_oai/src/routes_chat.rs` for streaming patterns
- Simple HTTP client using reqwest
- Bearer token authentication
- Direct request forwarding (no transformation)
- Coordinate with DbService to get API keys

## Phase 5: Request Routing

### 5.1 Create ModelRouter
**File: `crates/server_core/src/model_router.rs`**
```rust
pub trait ModelRouter: Debug + Send + Sync {
  async fn route_request(&self, model: &str) -> Result<RouteDestination>;
}

pub enum RouteDestination {
  Local(LocalModelAlias),
  Remote(ApiModelAlias),
}

pub struct DefaultModelRouter {
  data_service: Arc<dyn DataService>,
  db_service: Arc<dyn DbService>,
}

impl DefaultModelRouter {
  // Check model against all aliases (local and API)
  // Return appropriate destination
  // Handle conflict resolution based on created_at (API models checked first)
}
```

### 5.2 Update Chat Completions Route
**File: `crates/routes_oai/src/routes_chat.rs`**
- Create router instance in handler
- Use router to determine destination:
  - `Local` → Forward to SharedContext (existing behavior)
  - `Remote` → Forward to AiApiService
- Handle both streaming and non-streaming responses
- Maintain existing error handling patterns

## Phase 6: HTTP Routes

### 6.1 API Model Routes
**File: `crates/routes_app/src/routes_api_models.rs`**

Endpoints:
- `POST /bodhi/v1/api-models` - Create new API model configuration
- `POST /bodhi/v1/api-models/{alias}` - Update existing API model
- `GET /bodhi/v1/api-models/{alias}` - Get API model (with masked key)
- `DELETE /bodhi/v1/api-models/{alias}` - Delete API model
- `GET /bodhi/v1/api-models` - List all API models

Test endpoints (without saving):
- `POST /bodhi/v1/api-models/test` - Test prompt (30 char limit)
- `POST /bodhi/v1/api-models/models` - Fetch available models

### 6.2 Request/Response Objects
**File: `crates/routes_app/src/objs.rs`**
```rust
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateApiModelRequest {
  pub alias: String,
  pub provider: String, // "openai"
  pub base_url: String,
  pub api_key: String,
  pub models: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ApiModelResponse {
  pub alias: String,
  pub provider: String,
  pub base_url: String,
  pub api_key_masked: String, // Only first 8 and last 4 chars
  pub models: Vec<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TestPromptRequest {
  pub provider: String,
  pub base_url: String,
  pub api_key: String,
  pub prompt: String, // Max 30 chars enforced
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TestPromptResponse {
  pub response: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FetchModelsRequest {
  pub provider: String,
  pub base_url: String,
  pub api_key: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FetchModelsResponse {
  pub models: Vec<String>,
}
```

### 6.3 Update Models Endpoint
**File: `crates/routes_oai/src/routes_models.rs`**
- Include models from ApiModelAlias configurations
- Mark with appropriate metadata

### 6.4 OpenAPI Documentation
**File: `crates/routes_app/src/openapi.rs`**
- Add new handlers to paths section
- Add new request/response schemas to components
- Ensure proper tagging with API_TAG_MODELS

## Phase 7: Frontend Implementation

### 7.1 UI Routes
- `/ui/api-models` - List API model configurations
- `/ui/api-models/new` - Create new API model

### 7.2 API Model Configuration Form
Components:
- Provider dropdown (only "OpenAI")
- Base URL input (prefilled with `https://api.openai.com/v1`)
- Alias input (auto-generated "openai", editable, validated for uniqueness)
- API key input (password field)
- Test Prompt button (with text input, 30 char limit)
- Fetch Models button
- Model selection checkboxes
- Save button

### 7.3 List View
- Table showing configured API models
- Columns: Alias, Provider, Base URL, Models Count, Created At
- Actions: Edit, Delete
- Never show full API keys

## Phase 8: Testing Strategy

### 8.1 Unit Tests
- Alias enum serialization/deserialization
- API key encryption/decryption with salt
- Model routing logic
- Database operations
- Request validation (30 char limit)

### 8.2 Integration Tests
- Full API model CRUD operations
- API key masking in responses
- Model fetching with mock OpenAI responses
- Request forwarding with mock responses
- Router destination selection

### 8.3 End-to-End Tests
- Complete configuration flow
- Chat completion through remote API
- Streaming responses
- Error scenarios

## Key Design Decisions

1. **Alias as Primary Key**: Using alias as PK since it's immutable and natural
2. **Private Encryption Service**: Keep encryption internal to DbService for cohesion
3. **Route-Level Routing**: Chat handler creates router, not SharedContext dependency
4. **Test Prompt**: Practical API testing with real responses (30 char limit)
5. **OpenAPI First**: All new endpoints documented in OpenAPI spec
6. **SSE Reuse**: Leverage existing streaming infrastructure
7. **Simple Field Name**: Use `models` instead of `selected_models`

## Architecture Flow

1. **Request arrives** at `/v1/chat/completions`
2. **Chat handler creates router** and determines destination
3. **Local destination** → Forward to SharedContext (existing flow)
4. **Remote destination** → Forward to AiApiService
5. **AiApiService** gets API key from DbService and forwards request
6. **Response streaming** uses existing SSE infrastructure

## Out of Scope (For Later)

### Provider Extensions
- Azure OpenAI support
- Anthropic Claude integration  
- Google Gemini integration
- Cohere integration
- Custom OpenAI-compatible endpoints

### Advanced Features
- Request/response transformers for non-OpenAI providers
- Model prefix routing (e.g., "azure/gpt-4")
- Rate limiting and retry logic
- Usage tracking and metrics
- Cost estimation
- OAuth2 authentication flows
- Environment variable references for API keys
- Credential rotation UI
- Bulk model operations

## Success Criteria

- Alias enum properly discriminates between local and API models
- API keys encrypted with row-level salts using BODHI_ENCRYPTION_KEY
- API model configurations persisted in database
- Models can be fetched from OpenAI API
- Test prompts return actual AI responses (30 char limit)
- Chat requests route correctly through ModelRouter in handler
- Streaming responses work seamlessly
- API keys never exposed in GET responses  
- OpenAPI documentation complete and accurate
- Backward compatibility maintained for local aliases
- SharedContext remains focused on local model management