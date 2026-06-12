# API Key Optional - Technical Architecture

## Overview

This document describes the technical architecture for making API keys optional across BodhiApp's stack, supporting both database-backed configuration (CREATE/UPDATE) and runtime request forwarding.

## Architecture Goals

1. **Flexibility**: Support public APIs (no auth) and private APIs (with auth) seamlessly
2. **Backward Compatibility**: Existing API model configurations continue to work
3. **Type Safety**: Maintain TypeScript type safety across frontend and backend
4. **User Experience**: Simple checkbox UI to control API key requirement
5. **Security**: Maintain encryption for stored API keys when provided

## Part 1: Database-Backed Optional API Key

### Database Layer

**Schema Changes** (`migrations/0005_optional_api_keys.up.sql`):
```sql
-- Make API key related columns nullable
ALTER TABLE api_model_alias
  ALTER COLUMN encrypted_api_key DROP NOT NULL,
  ALTER COLUMN salt DROP NOT NULL,
  ALTER COLUMN nonce DROP NOT NULL;
```

**Rationale**:
- Previously: All three columns required (encryption metadata needed even for empty keys)
- Now: All nullable (no encryption metadata when API key not provided)
- Rollback: `down.sql` restores NOT NULL constraints

### Service Layer

**DbService Trait Update** (`services/src/db/service.rs`):
```rust
async fn create_api_model_alias(
  &self,
  api_model: &ApiModelAlias,
  api_key: Option<&str>,  // Changed from &str to Option<&str>
) -> Result<()>;
```

**Conditional Encryption Logic**:
```rust
let (encrypted_key, salt, nonce) = match api_key {
  Some(key) => {
    let (encrypted, salt, nonce) = self.secret_service.encrypt_data(key)?;
    (Some(encrypted), Some(salt), Some(nonce))
  }
  None => (None, None, None),
};

// INSERT with nullable values
sqlx::query!(
  "INSERT INTO api_model_alias (..., encrypted_api_key, salt, nonce) VALUES (..., $1, $2, $3)",
  encrypted_key,
  salt,
  nonce
)
```

**Key Insight**: Encryption only performed when API key provided, avoiding unnecessary cryptographic operations.

### Backend API Layer

**DTO Updates** (`routes_app/src/api_models_dto.rs`):
```rust
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateApiModelRequest {
  pub api_format: String,
  pub base_url: String,
  pub models: Vec<String>,
  pub prefix: Option<String>,
  pub api_key: Option<String>,  // Now optional
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateApiModelRequest {
  pub api_key: Option<String>,  // Optional - only update if provided
  // ... other fields
}
```

**Route Handler Logic** (`routes_app/src/routes_api_models.rs`):
```rust
// Create handler
let api_key = payload.api_key.as_deref();  // Option<&str>
db_service.create_api_model_alias(&api_model, api_key).await?;

// Update handler
let api_key = payload.api_key.as_deref();  // Option<&str>
db_service.update_api_model_alias(&api_model, api_key).await?;
```

### Frontend Layer

**Schema Updates** (`bodhi/src/schemas/apiModel.ts`):
```typescript
// Form schema with checkbox
export const createApiModelSchema = z.object({
  api_format: z.string().min(1),
  base_url: z.string().url(),
  models: z.array(z.string()),
  prefix: z.string().optional(),
  useApiKey: z.boolean(),           // Checkbox control
  api_key: z.string().optional(),   // Optional value
});

// Conversion - only include if checkbox enabled
export const convertFormToCreateRequest = (form: ApiModelFormData): CreateApiModelRequest => ({
  api_format: form.api_format,
  base_url: form.base_url,
  models: form.models,
  prefix: form.prefix,
  api_key: form.useApiKey ? form.api_key : undefined,  // Conditional inclusion
});
```

**Component Pattern** (`bodhi/src/components/api-models/form/ApiKeyInput.tsx`):
```tsx
<div className="flex items-center gap-2">
  <Checkbox
    id="useApiKey"
    checked={useApiKey}
    onCheckedChange={(checked) => {
      setValue('useApiKey', Boolean(checked));
      if (!checked) {
        setValue('api_key', '');  // Clear value when unchecked
      }
    }}
  />
  <Label htmlFor="useApiKey">Use API Key</Label>
</div>

{useApiKey && (
  <Input
    type="password"
    placeholder="Enter API key"
    {...register('api_key')}
  />
)}
```

**Key Pattern**: Checkbox controls visibility and value inclusion.

## Part 2: Runtime Optional API Key (Forwarding)

### Service Layer Updates

**AiApiService Trait** (`services/src/ai_api_service.rs`):
```rust
#[async_trait]
pub trait AiApiService: Send + Sync + Debug {
  /// Test connectivity - API key optional
  async fn test_prompt(
    &self,
    api_key: Option<String>,  // Changed from &str to Option<String>
    base_url: &str,
    model: &str,
    prompt: &str,
  ) -> Result<String>;

  /// Fetch models - API key optional
  async fn fetch_models(
    &self,
    api_key: Option<String>,  // Changed from &str to Option<String>
    base_url: &str,
  ) -> Result<Vec<String>>;

  /// Forward request - uses get_api_config helper
  async fn forward_request(
    &self,
    api_path: &str,
    id: &str,
    request: Value,
  ) -> Result<Response>;
}
```

**Why Option<String> instead of Option<&str>?**
- Mockall with async traits has lifetime issues with borrowed data
- Owned `Option<String>` avoids lifetime complexity
- Negligible performance impact (string cloning in HTTP context)

**Helper Method Pattern**:
```rust
/// Get API configuration - API key may be None
async fn get_api_config(&self, id: &str) -> Result<(ApiAlias, Option<String>)> {
  let api_alias = self.db_service.get_api_model_alias(id).await?
    .ok_or_else(|| AiApiServiceError::ModelNotFound(id.to_string()))?;

  // API key lookup - returns None if not configured
  let api_key = self.db_service.get_api_key_for_alias(id).await?;

  Ok((api_alias, api_key))  // Option<String> returned
}
```

**Conditional Authorization Header**:
```rust
async fn test_prompt(
  &self,
  api_key: Option<String>,
  base_url: &str,
  model: &str,
  prompt: &str,
) -> Result<String> {
  // Build request without auth header initially
  let mut request = self.client
    .post(format!("{}/chat/completions", base_url))
    .header("Content-Type", "application/json")
    .json(&request_body);

  // Only add Authorization if API key present
  if let Some(key) = api_key {
    request = request.header("Authorization", format!("Bearer {}", key));
  }

  let response = request.send().await?;
  // ... handle response
}
```

**Pattern Applied to All Methods**:
- `test_prompt`: Conditional header
- `fetch_models`: Conditional header
- `forward_request`: Uses `get_api_config` helper, conditional header

### Route Layer Updates

**Three-State Credential Resolution** (`routes_app/src/routes_api_models.rs`):
```rust
// Test/Fetch handlers use this pattern
let models = match (&payload.api_key, &payload.id) {
  // Case 1: Provided API key - use it directly (takes precedence)
  (Some(key), _) => {
    ai_api_service
      .fetch_models(Some(key.clone()), &payload.base_url)
      .await?
  }

  // Case 2: No API key but has ID - look up stored credentials
  (None, Some(id)) => {
    let stored_key = db_service.get_api_key_for_alias(id).await?;
    let api_model = db_service.get_api_model_alias(id).await?
      .ok_or_else(|| /* error */)?;

    ai_api_service
      .fetch_models(stored_key, &api_model.base_url)
      .await?
  }

  // Case 3: No credentials - proceed without authentication
  (None, None) => {
    ai_api_service
      .fetch_models(None, &payload.base_url)
      .await?
  }
};
```

**Precedence**:
1. Provided API key (highest - allows override in edit mode)
2. Stored credentials via ID (middle - use existing config)
3. No credentials (lowest - try public API)

**Request DTOs** (`routes_app/src/api_models_dto.rs`):
```rust
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct TestPromptRequest {
  /// API key for auth (optional - provide api_key OR id OR neither)
  pub api_key: Option<String>,
  /// API model ID to look up stored credentials (optional)
  pub id: Option<String>,
  /// Base URL (required)
  pub base_url: String,
  pub model: String,
  pub prompt: String,
}

#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct FetchModelsRequest {
  /// API key for auth (optional)
  pub api_key: Option<String>,
  /// API model ID to look up stored credentials (optional)
  pub id: Option<String>,
  /// Base URL (required)
  pub base_url: String,
}
```

**Note**: Removed validation requiring "api_key OR id" - both are now optional.

### Frontend Updates

**Hook Pattern** (`bodhi/src/components/api-models/hooks/useFetchModels.ts`):
```typescript
// Simplified validation - only base_url required
const canFetch = (data: FetchModelsData) => {
  return Boolean(data.baseUrl);
};

const getFetchDisabledReason = (data: FetchModelsData) => {
  if (!data.baseUrl) {
    return 'You need to add base URL to fetch models';
  }
  return '';  // No longer check for API key
};

// Conditional credential inclusion
const fetchModels = async (data: FetchModelsData) => {
  const fetchData: FetchModelsRequest = {
    base_url: data.baseUrl,
    // Spread operator for conditional fields
    ...(data.apiKey ? { api_key: data.apiKey } : {}),
    ...(!data.apiKey && mode === 'edit' && initialData?.id ? { id: initialData.id } : {}),
  };

  // Let API decide if authentication required
  const response = await fetchModelsAPI(fetchData);
  // ... handle response or 401 error
};
```

**Test Connection Hook** (`bodhi/src/components/api-models/hooks/useTestConnection.ts`):
```typescript
// Build credentials based on availability (3-case pattern)
const credentials: { api_key?: string; id?: string } = {};

if (data.apiKey) {
  credentials.api_key = data.apiKey;
} else if (data.id) {
  credentials.id = data.id;
}
// If neither, credentials remains empty

const testData: TestPromptRequest = {
  ...credentials,
  base_url: data.baseUrl,
  model: data.model,
  prompt: DEFAULT_TEST_PROMPT,
};
```

## Cross-Layer Coordination

### Data Flow: Create API Model Without Key

```
1. Frontend Form
   - User unchecks "Use API Key" checkbox
   - useApiKey: false, api_key: ''

2. Schema Conversion
   - convertFormToCreateRequest()
   - api_key excluded from request (undefined)

3. Backend DTO
   - CreateApiModelRequest { api_key: None, ... }

4. Route Handler
   - payload.api_key.as_deref() → None

5. Service Layer
   - create_api_model_alias(api_model, None)
   - No encryption performed

6. Database
   - INSERT with NULL for encrypted_api_key, salt, nonce
```

### Data Flow: Fetch Models Without Auth

```
1. Frontend
   - User enters base_url only
   - Clicks "Fetch Models"
   - useApiKey unchecked → api_key: ''

2. Hook Logic
   - fetchModels({ baseUrl, apiKey: undefined, id: undefined })
   - Spread operator: { base_url } (no api_key, no id)

3. Backend DTO
   - FetchModelsRequest { base_url, api_key: None, id: None }

4. Route Handler (3-state match)
   - (None, None) branch
   - Call ai_api_service.fetch_models(None, base_url)

5. Service Layer
   - Build request WITHOUT Authorization header
   - Send GET /models

6. API Response
   - Success (200): Return model list
   - Unauthorized (401): Return error to user
```

### Data Flow: Forward Request with Stored Key

```
1. Frontend
   - User in edit mode (initialData.id exists)
   - Submits chat completion request

2. Route Handler
   - forward_request(api_path, id, request)
   - Calls get_api_config(id)

3. Service Helper
   - get_api_config(id) → (api_alias, Option<String>)
   - Looks up encrypted API key
   - Decrypts if present, returns None if not

4. Forward Logic
   - if let Some(key) = api_key { add Authorization header }
   - POST to remote API

5. Remote API
   - Receives request with or without auth
   - Responds accordingly
```

## Design Patterns

### Pattern 1: Optional Encrypted Fields
**Problem**: How to handle encryption when value is optional?
**Solution**: Make encryption metadata columns nullable, skip encryption when None

### Pattern 2: Three-State Resolution
**Problem**: How to prioritize multiple credential sources?
**Solution**: Match tuple with explicit precedence (provided > stored > none)

### Pattern 3: Conditional Authorization
**Problem**: How to add header only when needed?
**Solution**: Build request first, conditionally add header based on Option

### Pattern 4: Frontend Checkbox Control
**Problem**: How to make field optional in form?
**Solution**: Checkbox controls visibility + inclusion in request

### Pattern 5: Spread Operator for Optional Fields
**Problem**: How to conditionally include fields in TypeScript?
**Solution**: `...(condition ? { field: value } : {})`

## Error Handling

### API Returns 401 Unauthorized
```rust
// Service layer
if status == StatusCode::UNAUTHORIZED {
  return Err(AiApiServiceError::Unauthorized(
    "API requires authentication".to_string()
  ));
}
```

Frontend receives error and shows toast:
```typescript
catch (error) {
  toast({
    title: 'Connection Test Failed',
    description: 'API requires authentication - please add API key',
    variant: 'destructive',
  });
}
```

### Missing Required Fields
```rust
// Database layer
if api_key.is_some() && (salt.is_none() || nonce.is_none()) {
  return Err(DbServiceError::EncryptionError(
    "Cannot encrypt without salt and nonce".to_string()
  ));
}
```

## Security Considerations

1. **Encryption Integrity**: API key still encrypted when provided
2. **No Plaintext Leakage**: Frontend never stores plaintext keys
3. **Masked Display**: Edit mode shows `***` for existing keys
4. **Secure Transmission**: HTTPS required for API key transmission
5. **Authorization Fallback**: Backend validates auth requirements

## Performance Impact

**Negligible**:
- String cloning: Trivial compared to HTTP I/O
- Conditional logic: Branch prediction optimizes hot path
- No additional queries: Same database access pattern
- Frontend: No bundle size increase

## Backward Compatibility

**100% Compatible**:
- Old configs (with API key): Continue to work
- Edit mode: Preserves existing encrypted keys
- Migration: Reversible with down.sql
- API contracts: Optional fields backward compatible

## Testing Strategy

### Unit Tests
- Service layer: Test with/without API key
- Route handlers: Test all three credential states
- Frontend: Test checkbox interactions

### Integration Tests
- End-to-end: Create → Fetch → Test flows
- Public API: Fetch without credentials
- Private API: 401 error handling

### Migration Tests
- Up migration: Make columns nullable
- Down migration: Restore NOT NULL (fails if NULL values exist)

## Future Considerations

1. **Provider Metadata**: Store auth requirements per provider
2. **Auto-Detection**: Probe API to determine if auth required
3. **Credential Rotation**: Update stored keys without re-entering
4. **Multi-Key Support**: Different keys for different operations
5. **OAuth2 Support**: Extend pattern to OAuth flows
