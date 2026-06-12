# Files Changed - Quick Reference

## Summary

**Total Files Modified**: 21
- Database: 2 (migrations)
- Backend Service: 6
- Backend API: 3
- TypeScript Client: 2
- Frontend: 8

## Database Layer (2 files)

### `crates/services/migrations/0005_optional_api_keys.up.sql` [NEW]
**Key Changes**:
```sql
ALTER TABLE api_model_alias
  ALTER COLUMN encrypted_api_key DROP NOT NULL,
  ALTER COLUMN salt DROP NOT NULL,
  ALTER COLUMN nonce DROP NOT NULL;
```
**Purpose**: Make encryption columns nullable for optional API key support

### `crates/services/migrations/0005_optional_api_keys.down.sql` [NEW]
**Key Changes**:
```sql
ALTER TABLE api_model_alias
  ALTER COLUMN encrypted_api_key SET NOT NULL,
  ALTER COLUMN salt SET NOT NULL,
  ALTER COLUMN nonce SET NOT NULL;
```
**Purpose**: Rollback migration, restore NOT NULL constraints

## Backend Service Layer (6 files)

### `crates/services/src/db/service.rs`
**Key Changes**:
- `create_api_model_alias(api_key: Option<&str>)` - Changed from `&str` to `Option<&str>`
- `update_api_model_alias(api_key: Option<&str>)` - Changed from `&str` to `Option<&str>`
- Conditional encryption logic: Only encrypt when `api_key.is_some()`
- INSERT/UPDATE with nullable values for `encrypted_api_key`, `salt`, `nonce`

**Lines Modified**: ~50-100 (trait + impl + tests)

### `crates/services/src/ai_api_service.rs`
**Key Changes**:
- `test_prompt(api_key: Option<String>, ...)` - Changed from `&str` to `Option<String>`
- `fetch_models(api_key: Option<String>, ...)` - Changed from `&str` to `Option<String>`
- `get_api_config() -> Result<(ApiAlias, Option<String>)>` - Returns `Option<String>` for API key
- Conditional Authorization header in all methods
- New test: `test_forward_request_without_api_key()`

**Lines Modified**: ~150 (trait + impl + tests)

### `crates/services/src/data_service.rs`
**Key Changes**:
- Support for optional API key lookups
- Graceful handling of None API keys

**Lines Modified**: ~20

### `crates/services/src/test_utils/db.rs`
**Key Changes**:
- Mock expectations updated for `Option<&str>` API key
- Test fixtures for optional API key scenarios

**Lines Modified**: ~30

### `crates/services/src/test_utils/objs.rs`
**Key Changes**:
- Test object builders updated for optional API key
- Helper functions for creating test data without API keys

**Lines Modified**: ~15

### `crates/services/src/resources/en-US/messages.ftl`
**Key Changes**:
- Updated error messages for optional API key scenarios
- Localization strings for new validation messages

**Lines Modified**: ~10

## Backend API Layer (3 files)

### `crates/routes_app/src/api_models_dto.rs`
**Key Changes**:
```rust
// CreateApiModelRequest
pub api_key: Option<String>  // Was: String

// UpdateApiModelRequest
pub api_key: Option<String>  // Was: String

// TestPromptRequest
pub api_key: Option<String>  // Optional
pub id: Option<String>        // Optional

// FetchModelsRequest
pub api_key: Option<String>  // Optional
pub id: Option<String>        // Optional
```
- Removed `validate_fetch_models_credentials()` function
- Updated OpenAPI examples to show optional api_key
- Updated tests for optional validation

**Lines Modified**: ~100

### `crates/routes_app/src/routes_api_models.rs`
**Key Changes**:
```rust
// Three-state credential resolution pattern in handlers
match (&payload.api_key, &payload.id) {
  (Some(key), _) => /* use provided */,
  (None, Some(id)) => /* use stored */,
  (None, None) => /* no credentials */,
}
```
- `create_api_model()`: Pass `Option<&str>` to service
- `update_api_model()`: Pass `Option<&str>` to service
- `test_prompt()`: Implement three-state resolution
- `fetch_models()`: Implement three-state resolution

**Lines Modified**: ~80

### `openapi.json`
**Key Changes**:
- `CreateApiModelRequest.api_key`: `{ type: "string", nullable: true }`
- `UpdateApiModelRequest.api_key`: `{ type: "string", nullable: true }`
- `TestPromptRequest`: Both `api_key` and `id` optional
- `FetchModelsRequest`: Both `api_key` and `id` optional
- Updated examples to show optional fields

**Lines Modified**: Auto-generated

## TypeScript Client (2 files)

### `ts-client/src/types/types.gen.ts`
**Key Changes**:
```typescript
export interface CreateApiModelRequest {
  api_format: string;
  base_url: string;
  models: string[];
  prefix?: string;
  api_key?: string;  // Optional
}

export interface UpdateApiModelRequest {
  api_key?: string;  // Optional
  // ... other fields
}

export interface TestPromptRequest {
  api_key?: string;  // Optional
  id?: string;       // Optional
  base_url: string;
  model: string;
  prompt: string;
}

export interface FetchModelsRequest {
  api_key?: string;  // Optional
  id?: string;       // Optional
  base_url: string;
}
```

**Lines Modified**: Auto-generated from OpenAPI

### `ts-client/src/openapi-typescript/openapi-schema.ts`
**Key Changes**:
- MSW mock types updated for optional `api_key` fields
- Updated schemas for all affected DTOs

**Lines Modified**: Auto-generated

## Frontend Layer (8 files)

### `crates/bodhi/src/schemas/apiModel.ts`
**Key Changes**:
```typescript
// Schema
export const createApiModelSchema = z.object({
  api_format: z.string().min(1),
  base_url: z.string().url(),
  models: z.array(z.string()),
  prefix: z.string().optional(),
  useApiKey: z.boolean(),        // NEW
  api_key: z.string().optional(),
});

// Conversion - conditional inclusion
export const convertFormToCreateRequest = (form: ApiModelFormData) => ({
  api_format: form.api_format,
  base_url: form.base_url,
  models: form.models,
  prefix: form.prefix,
  api_key: form.useApiKey ? form.api_key : undefined,  // Conditional
});
```

**Lines Modified**: ~80

### `crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx`
**Key Changes**:
```tsx
// Added checkbox UI
<div className="flex items-center gap-2">
  <Checkbox
    id="useApiKey"
    checked={useApiKey}
    onCheckedChange={(checked) => {
      setValue('useApiKey', Boolean(checked));
      if (!checked) setValue('api_key', '');
    }}
  />
  <Label>Use API Key</Label>
</div>

{useApiKey && (
  <Input type="password" {...register('api_key')} />
)}
```

**Lines Modified**: ~40

### `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
**Key Changes**:
- Integrated ApiKeyInput with checkbox
- No significant logic changes (handled by hooks)

**Lines Modified**: ~5

### `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`
**Key Changes**:
```typescript
// Default values
defaultValues: {
  // ...
  useApiKey: mode === 'edit' ? initialData?.api_key_masked !== '***' : false,
  api_key: '',
}

// Conditional operations
handleFetchModels: () => fetchModels({
  apiKey: watchedValues.useApiKey ? watchedValues.api_key : undefined,
  id: !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined,
  baseUrl: watchedValues.base_url,
});
```

**Lines Modified**: ~30

### `crates/bodhi/src/components/api-models/hooks/useFetchModels.ts`
**Key Changes**:
```typescript
// Simplified validation - only base_url required
const canFetch = (data: FetchModelsData) => Boolean(data.baseUrl);

const getFetchDisabledReason = (data: FetchModelsData) => {
  if (!data.baseUrl) return 'You need to add base URL to fetch models';
  return '';  // Don't validate API key
};

// Conditional inclusion
const fetchData: FetchModelsRequest = {
  base_url: data.baseUrl,
  ...(data.apiKey ? { api_key: data.apiKey } : {}),
  ...(!data.apiKey && mode === 'edit' && id ? { id } : {}),
};
```

**Lines Modified**: ~40

### `crates/bodhi/src/components/api-models/hooks/useTestConnection.ts`
**Key Changes**:
```typescript
// Build credentials based on availability (3-case pattern)
const credentials: { api_key?: string; id?: string } = {};

if (data.apiKey) {
  credentials.api_key = data.apiKey;
} else if (data.id) {
  credentials.id = data.id;
}

const testData: TestPromptRequest = {
  ...credentials,
  base_url: data.baseUrl,
  model: data.model,
  prompt: DEFAULT_TEST_PROMPT,
};
```

**Lines Modified**: ~30

### `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`
**Key Changes**:
```typescript
// Updated test: was "shows error without credentials"
it('allows fetch models without credentials (API may return 401)', async () => {
  server.use(...mockFetchApiModels({ models: ['gpt-4'] }));

  await user.click(fetchButton);

  await waitFor(() => {
    expect(mockToast).toHaveBeenCalledWith({
      title: 'Models Fetched Successfully',
      description: 'Found 1 available models',
    });
  });
});

// New tests for checkbox behavior
it('includes api_key when checkbox enabled', async () => { /* ... */ });
it('excludes api_key when checkbox disabled', async () => { /* ... */ });
```

**Lines Modified**: ~100

### `crates/bodhi/src/test-utils/api-model-test-utils.ts`
**Key Changes**:
```typescript
// Helper for filling form with checkbox
export const fillApiModelForm = async (user, data) => {
  // ... fill other fields ...

  if (data.useApiKey) {
    const checkbox = screen.getByLabelText(/use api key/i);
    await user.click(checkbox);
    const input = screen.getByPlaceholderText(/enter api key/i);
    await user.type(input, data.apiKey);
  }
};
```

**Lines Modified**: ~20

### `crates/bodhi/src/app/ui/api-models/new/page.test.tsx`
**Key Changes**:
- Updated integration test for new checkbox UI
- Test complete create flow with/without API key

**Lines Modified**: ~15

## Other Modified Files

### `getbodhi.app/public/releases.json`
**Key Changes**:
- Updated release metadata (unrelated to this feature)

**Lines Modified**: ~10

## File Change Summary by Type

### New Files (2)
- `crates/services/migrations/0005_optional_api_keys.up.sql`
- `crates/services/migrations/0005_optional_api_keys.down.sql`

### Backend Modified (9)
- `crates/services/src/db/service.rs`
- `crates/services/src/ai_api_service.rs`
- `crates/services/src/data_service.rs`
- `crates/services/src/test_utils/db.rs`
- `crates/services/src/test_utils/objs.rs`
- `crates/services/src/resources/en-US/messages.ftl`
- `crates/routes_app/src/api_models_dto.rs`
- `crates/routes_app/src/routes_api_models.rs`
- `openapi.json`

### TypeScript Client Generated (2)
- `ts-client/src/types/types.gen.ts`
- `ts-client/src/openapi-typescript/openapi-schema.ts`

### Frontend Modified (8)
- `crates/bodhi/src/schemas/apiModel.ts`
- `crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx`
- `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
- `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`
- `crates/bodhi/src/components/api-models/hooks/useFetchModels.ts`
- `crates/bodhi/src/components/api-models/hooks/useTestConnection.ts`
- `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`
- `crates/bodhi/src/app/ui/api-models/new/page.test.tsx`

### Test Utilities (1)
- `crates/bodhi/src/test-utils/api-model-test-utils.ts`

## Key Patterns Across Files

### Pattern 1: Option<T> for Optional Values
**Rust**: `Option<&str>`, `Option<String>`
**TypeScript**: `field?: string`

### Pattern 2: Conditional Inclusion
**Rust**: `match api_key { Some(key) => ..., None => ... }`
**TypeScript**: `...(condition ? { field: value } : {})`

### Pattern 3: Three-State Resolution
```rust
match (&provided, &stored) {
  (Some(p), _) => /* use provided */,
  (None, Some(s)) => /* use stored */,
  (None, None) => /* no credentials */,
}
```

### Pattern 4: Checkbox Control
```tsx
<Checkbox checked={useField} onChange={setUseField} />
{useField && <Input {...register('field')} />}
```

### Pattern 5: Conditional Encryption
```rust
let (encrypted, salt, nonce) = match value {
  Some(v) => (Some(encrypt(v)), Some(salt), Some(nonce)),
  None => (None, None, None),
};
```

## Implementation Checklist

- [x] Database migration files created
- [x] Service layer traits updated
- [x] Conditional encryption implemented
- [x] Backend DTOs updated
- [x] Route handlers with three-state resolution
- [x] OpenAPI spec regenerated
- [x] TypeScript types regenerated
- [x] Frontend schema with useApiKey field
- [x] Checkbox UI component
- [x] Frontend hooks updated
- [x] Component tests updated
- [x] All tests passing
- [x] Code formatted
- [x] UI rebuilt

## Quick Command Reference

```bash
# Run migrations (automatic in tests)
cargo test -p services -- db::service

# Run service tests
cargo test -p services ai_api_service

# Run route tests
cargo test -p routes_app api_models

# Regenerate OpenAPI
cargo run --package xtask openapi

# Rebuild TypeScript client
cd ts-client && npm run build

# Run frontend tests
cd crates/bodhi && npm test

# Rebuild UI
cd crates/bodhi && npm run build

# Format code
cargo fmt --all
cd crates/bodhi && npm run format

# Run all tests
cargo test && cd crates/bodhi && npm test
```
