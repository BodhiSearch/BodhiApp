# API Key Optional - Implementation Guide

## Overview

This guide provides sequential, agent-friendly implementation steps for making API keys optional in BodhiApp. The implementation is divided into two parts with multiple phases each.

## PART 1: Optional API Key for CREATE/UPDATE (Database-Backed)

### Phase 1: Database Migration

**Objective**: Make API key columns nullable in database schema

**Files to Modify**:
1. `crates/services/migrations/0005_optional_api_keys.up.sql` [NEW]
2. `crates/services/migrations/0005_optional_api_keys.down.sql` [NEW]

**Steps**:

1. Create up migration file:
```sql
-- crates/services/migrations/0005_optional_api_keys.up.sql
ALTER TABLE api_model_alias
  ALTER COLUMN encrypted_api_key DROP NOT NULL,
  ALTER COLUMN salt DROP NOT NULL,
  ALTER COLUMN nonce DROP NOT NULL;
```

2. Create down migration file:
```sql
-- crates/services/migrations/0005_optional_api_keys.down.sql
ALTER TABLE api_model_alias
  ALTER COLUMN encrypted_api_key SET NOT NULL,
  ALTER COLUMN salt SET NOT NULL,
  ALTER COLUMN nonce SET NOT NULL;
```

**Validation**:
```bash
# Test migration (creates new test database)
cargo test -p services -- db::service
```

**Success Criteria**:
- Migration files created
- Database schema updated (nullable columns)
- Tests pass with new schema

---

### Phase 2: Service Layer Updates

**Objective**: Update DbService to handle optional API keys with conditional encryption

**Files to Modify**:
1. `crates/services/src/db/service.rs` - Trait and implementation
2. `crates/services/src/test_utils/db.rs` - Test mocks
3. `crates/services/src/test_utils/objs.rs` - Test objects
4. `crates/services/src/resources/en-US/messages.ftl` - Localization

**Steps**:

1. Update `DbService` trait:
```rust
// Change signature from &str to Option<&str>
async fn create_api_model_alias(
  &self,
  api_model: &ApiModelAlias,
  api_key: Option<&str>,
) -> Result<()>;

async fn update_api_model_alias(
  &self,
  api_model: &ApiModelAlias,
  api_key: Option<&str>,
) -> Result<()>;
```

2. Update implementation with conditional encryption:
```rust
async fn create_api_model_alias(&self, api_model: &ApiModelAlias, api_key: Option<&str>) -> Result<()> {
  // Conditional encryption
  let (encrypted_key, salt, nonce) = match api_key {
    Some(key) => {
      let (encrypted, salt, nonce) = self.secret_service.encrypt_data(key)?;
      (Some(encrypted), Some(salt), Some(nonce))
    }
    None => (None, None, None),
  };

  // Insert with nullable values
  sqlx::query!(
    r#"INSERT INTO api_model_alias (..., encrypted_api_key, salt, nonce)
       VALUES (..., $1, $2, $3)"#,
    encrypted_key,
    salt,
    nonce
  ).execute(&mut *tx).await?;

  Ok(())
}
```

3. Update test mocks in `test_utils/db.rs`:
```rust
mock_db.expect_create_api_model_alias()
  .withf(|model, api_key| {
    model.id == "test-id" && api_key.is_none()  // Test without API key
  })
  .returning(|_, _| Ok(()));
```

4. Update localization messages if needed

**Validation**:
```bash
cargo test -p services -- db::service
```

**Success Criteria**:
- Trait signature updated
- Conditional encryption logic implemented
- Test mocks updated
- All tests passing

---

### Phase 3: Backend API Updates

**Objective**: Update DTOs and route handlers to accept optional API keys

**Files to Modify**:
1. `crates/routes_app/src/api_models_dto.rs` - Request/Response DTOs
2. `crates/routes_app/src/routes_api_models.rs` - Route handlers

**Steps**:

1. Update request DTOs:
```rust
// CreateApiModelRequest
pub struct CreateApiModelRequest {
  pub api_format: String,
  pub base_url: String,
  pub models: Vec<String>,
  pub prefix: Option<String>,
  pub api_key: Option<String>,  // Changed from String to Option<String>
}

// UpdateApiModelRequest
pub struct UpdateApiModelRequest {
  pub api_key: Option<String>,  // Optional - only update if provided
  // ... other fields
}
```

2. Update OpenAPI examples:
```rust
#[schema(example = json!({
  "api_format": "openai",
  "base_url": "https://api.openai.com/v1",
  "models": ["gpt-4", "gpt-3.5-turbo"]
  // Note: No api_key in example - it's optional!
}))]
```

3. Update route handlers:
```rust
// Create handler
pub async fn create_api_model(
  State(state): State<RouterState>,
  Json(payload): Json<CreateApiModelRequest>,
) -> Result<Json<ApiModelResponse>> {
  // Convert Option<String> to Option<&str>
  let api_key = payload.api_key.as_deref();

  // Pass to service
  db_service.create_api_model_alias(&api_model, api_key).await?;

  // Return response
  Ok(Json(response))
}
```

**Validation**:
```bash
cargo test -p routes_app -- api_models
```

**Success Criteria**:
- DTOs updated with optional API key
- Route handlers handle None case
- OpenAPI schema updated
- All route tests passing

---

### Phase 4: TypeScript Client Generation

**Objective**: Regenerate TypeScript types from updated OpenAPI spec

**Files to Generate**:
1. `openapi.json` (root)
2. `ts-client/src/types/types.gen.ts`
3. `ts-client/src/openapi-typescript/openapi-schema.ts`

**Steps**:

1. Generate OpenAPI spec:
```bash
cargo run --package xtask openapi
```

2. Build TypeScript client:
```bash
cd ts-client
npm run build
```

**Validation**:
- Check `openapi.json` has `api_key: { type: string, nullable: true }`
- Verify generated types have `api_key?: string`

**Success Criteria**:
- OpenAPI spec regenerated
- TypeScript types include optional api_key
- Client package builds successfully

---

### Phase 5: Frontend Schema & Components

**Objective**: Add checkbox UI and conditional logic to frontend

**Files to Modify**:
1. `crates/bodhi/src/schemas/apiModel.ts` - Schema with useApiKey field
2. `crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx` - Checkbox UI
3. `crates/bodhi/src/components/api-models/ApiModelForm.tsx` - Form integration
4. `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts` - Conditional logic
5. `crates/bodhi/src/test-utils/api-model-test-utils.ts` - Test utilities

**Steps**:

1. Update schema with useApiKey field:
```typescript
// crates/bodhi/src/schemas/apiModel.ts
export const createApiModelSchema = z.object({
  api_format: z.string().min(1),
  base_url: z.string().url(),
  models: z.array(z.string()),
  prefix: z.string().optional(),
  useApiKey: z.boolean(),           // New field
  api_key: z.string().optional(),
});

// Conversion function - only include if checked
export const convertFormToCreateRequest = (form: ApiModelFormData): CreateApiModelRequest => ({
  api_format: form.api_format,
  base_url: form.base_url,
  models: form.models,
  prefix: form.prefix,
  api_key: form.useApiKey ? form.api_key : undefined,
});
```

2. Update ApiKeyInput with checkbox:
```tsx
// crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx
export function ApiKeyInput({ register, setValue, watch }) {
  const useApiKey = watch('useApiKey');

  return (
    <div className="space-y-2">
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
        <Input
          type="password"
          placeholder="Enter API key"
          {...register('api_key')}
        />
      )}
    </div>
  );
}
```

3. Update form hook defaults:
```typescript
// useApiModelForm.ts
const form = useForm({
  defaultValues: mode === 'edit'
    ? {
        api_format: initialData?.api_format || 'openai',
        base_url: initialData?.base_url || '',
        api_key: '',
        models: initialData?.models || [],
        prefix: initialData?.prefix || '',
        useApiKey: initialData?.api_key_masked !== '***',  // Based on existing key
      }
    : {
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        api_key: '',
        models: [],
        prefix: '',
        useApiKey: false,  // Unchecked by default
      },
});
```

4. Update test utilities for checkbox:
```typescript
// test-utils/api-model-test-utils.ts
export const fillApiModelForm = async (user, data) => {
  // ... fill other fields ...

  if (data.useApiKey) {
    const checkbox = screen.getByLabelText(/use api key/i);
    await user.click(checkbox);
    const apiKeyInput = screen.getByPlaceholderText(/enter api key/i);
    await user.type(apiKeyInput, data.apiKey);
  }
};
```

**Validation**:
```bash
cd crates/bodhi
npm test -- apiModel
```

**Success Criteria**:
- Schema includes useApiKey field
- Checkbox controls API key visibility
- Form conversion excludes API key when unchecked
- Component tests pass

---

### Phase 6: Frontend Tests & UI Build

**Objective**: Update component tests and rebuild embedded UI

**Files to Modify**:
1. `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`
2. `crates/bodhi/src/app/ui/api-models/new/page.test.tsx`

**Steps**:

1. Update component tests for checkbox:
```typescript
// ApiModelForm.test.tsx
it('creates API model without API key when checkbox unchecked', async () => {
  const user = userEvent.setup();

  render(<ApiModelForm mode="create" />);

  // Fill form without checking API key checkbox
  await user.type(screen.getByLabelText(/base url/i), 'https://api.openai.com/v1');
  // ... fill other fields ...

  // Submit
  await user.click(screen.getByRole('button', { name: /create/i }));

  // Verify request sent without api_key
  await waitFor(() => {
    expect(mockCreateApiModel).toHaveBeenCalledWith({
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      models: ['gpt-4'],
      // api_key not included
    });
  });
});
```

2. Run frontend tests:
```bash
cd crates/bodhi
npm test
```

3. Rebuild embedded UI:
```bash
cd crates/bodhi
npm run build
```

4. Run backend tests:
```bash
cargo test
```

5. Format code:
```bash
cargo fmt --all
cd crates/bodhi && npm run format
```

**Success Criteria**:
- Component tests updated for checkbox
- Frontend tests passing (30+ tests)
- UI rebuild successful
- Backend tests passing
- Code formatted

---

## PART 2: Optional API Key for REQUEST FORWARDING (Runtime)

### Phase 7: Service Layer Forwarding

**Objective**: Make API key optional in service methods that forward requests

**Files to Modify**:
1. `crates/services/src/ai_api_service.rs` - Trait, implementation, tests

**Steps**:

1. Update trait signatures:
```rust
#[async_trait]
pub trait AiApiService: Send + Sync + Debug {
  /// Test connectivity - API key optional
  async fn test_prompt(
    &self,
    api_key: Option<String>,  // Changed from &str
    base_url: &str,
    model: &str,
    prompt: &str,
  ) -> Result<String>;

  /// Fetch models - API key optional
  async fn fetch_models(
    &self,
    api_key: Option<String>,  // Changed from &str
    base_url: &str,
  ) -> Result<Vec<String>>;

  /// Forward request - uses helper
  async fn forward_request(
    &self,
    api_path: &str,
    id: &str,
    request: Value,
  ) -> Result<Response>;
}
```

2. Update helper to return Option:
```rust
/// Get API configuration - API key may be None
async fn get_api_config(&self, id: &str) -> Result<(ApiAlias, Option<String>)> {
  let api_alias = self.db_service
    .get_api_model_alias(id).await?
    .ok_or_else(|| AiApiServiceError::ModelNotFound(id.to_string()))?;

  // Returns Option<String> - None if not configured
  let api_key = self.db_service.get_api_key_for_alias(id).await?;

  Ok((api_alias, api_key))
}
```

3. Implement conditional Authorization header:
```rust
async fn test_prompt(
  &self,
  api_key: Option<String>,
  base_url: &str,
  model: &str,
  prompt: &str,
) -> Result<String> {
  let url = format!("{}/chat/completions", base_url);

  // Build request without auth
  let mut request = self.client
    .post(&url)
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

4. Apply pattern to all methods:
   - `test_prompt`: Conditional header
   - `fetch_models`: Conditional header
   - `forward_request`: Use `get_api_config`, conditional header

5. Update tests:
```rust
#[rstest]
#[tokio::test]
async fn test_fetch_models_success() -> Result<()> {
  // ... setup mock server ...

  let models = service
    .fetch_models(Some("test-key".to_string()), &url)  // Changed
    .await?;

  assert_eq!(vec!["gpt-3.5-turbo", "gpt-4"], models);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_forward_request_without_api_key() -> Result<()> {
  let mut mock_db = MockDbService::new();

  // Setup mock to return None for API key
  mock_db.expect_get_api_key_for_alias()
    .returning(|_| Ok(None));

  // Mock server expects request WITHOUT Authorization header
  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(200)
    .create_async().await;

  let response = service
    .forward_request("/chat/completions", "test-id", request)
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
  Ok(())
}
```

**Validation**:
```bash
cargo test -p services ai_api_service
```

**Success Criteria**:
- Trait methods accept Option<String>
- Conditional Authorization header logic
- Helper returns Option for API key
- New test for forwarding without key
- All tests passing

---

### Phase 8: Route Handler Updates

**Objective**: Update routes to support three-state credential resolution

**Files to Modify**:
1. `crates/routes_app/src/routes_api_models.rs` - Handler logic
2. `crates/routes_app/src/api_models_dto.rs` - Remove validation

**Steps**:

1. Update FetchModelsRequest DTO:
```rust
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct FetchModelsRequest {
  /// API key (optional - provide api_key OR id OR neither)
  pub api_key: Option<String>,
  /// API model ID (optional)
  pub id: Option<String>,
  /// Base URL (required)
  pub base_url: String,
}

// Remove validation function that required "api_key OR id"
```

2. Implement three-state resolution in handler:
```rust
pub async fn fetch_models(
  State(state): State<RouterState>,
  Json(payload): Json<FetchModelsRequest>,
) -> Result<Json<Vec<String>>> {
  let ai_api_service = state.app_service().ai_api_service();
  let db_service = state.app_service().db_service();

  // Three-state credential resolution
  let models = match (&payload.api_key, &payload.id) {
    // Case 1: Provided API key - use directly (takes precedence)
    (Some(key), _) => {
      ai_api_service
        .fetch_models(Some(key.clone()), &payload.base_url)
        .await?
    }

    // Case 2: No key but has ID - look up stored credentials
    (None, Some(id)) => {
      let stored_key = db_service.get_api_key_for_alias(id).await?;
      let api_model = db_service.get_api_model_alias(id).await?
        .ok_or_else(|| /* error */)?;

      ai_api_service
        .fetch_models(stored_key, &api_model.base_url)
        .await?
    }

    // Case 3: No credentials - proceed without auth
    (None, None) => {
      ai_api_service
        .fetch_models(None, &payload.base_url)
        .await?
    }
  };

  Ok(Json(models))
}
```

3. Apply same pattern to `test_prompt` handler

4. Update tests:
```rust
#[rstest]
#[tokio::test]
async fn test_fetch_models_without_credentials() {
  // Test case 3: Neither api_key nor id provided
  let payload = FetchModelsRequest {
    api_key: None,
    id: None,
    base_url: "https://api.openai.com/v1".to_string(),
  };

  // Should succeed (let API decide if auth required)
  assert!(payload.validate().is_ok());
}
```

**Validation**:
```bash
cargo test -p routes_app api_models
```

**Success Criteria**:
- Three-state pattern implemented
- Validation removed for required credentials
- All credential scenarios tested
- Route tests passing

---

### Phase 9: Frontend Integration

**Objective**: Update frontend hooks to not require API key

**Files to Modify**:
1. `crates/bodhi/src/components/api-models/hooks/useFetchModels.ts`
2. `crates/bodhi/src/components/api-models/hooks/useTestConnection.ts`
3. `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`

**Steps**:

1. Update `useFetchModels` hook:
```typescript
// Simplified validation - only base_url required
const canFetch = (data: FetchModelsData) => {
  return Boolean(data.baseUrl);
};

const getFetchDisabledReason = (data: FetchModelsData) => {
  if (!data.baseUrl) {
    return 'You need to add base URL to fetch models';
  }
  return '';  // Don't check for API key
};

// Conditional credential inclusion
const fetchModels = async (data: FetchModelsData) => {
  const fetchData: FetchModelsRequest = {
    base_url: data.baseUrl,
    ...(data.apiKey ? { api_key: data.apiKey } : {}),
    ...(!data.apiKey && mode === 'edit' && initialData?.id ? { id: initialData.id } : {}),
  };

  const response = await fetchModelsAPI(fetchData);
  // ... handle response or 401 error
};
```

2. Update `useTestConnection` hook:
```typescript
const testConnection = async (data: TestConnectionData) => {
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

  const response = await testPromptAPI(testData);
  // ... handle response
};
```

3. Update component tests:
```typescript
it('allows fetch models without credentials (API may return 401)', async () => {
  server.use(
    ...mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo'] })
  );

  await user.click(fetchButton);

  await waitFor(() => {
    expect(mockToast).toHaveBeenCalledWith({
      title: 'Models Fetched Successfully',
      description: 'Found 2 available models',
    });
  });
});
```

**Validation**:
```bash
cd crates/bodhi
npm test -- ApiModelForm
```

**Success Criteria**:
- Hooks don't validate API key requirement
- Conditional credential inclusion works
- Tests updated for no-credential case
- Frontend tests passing

---

### Phase 10: Testing & Verification

**Objective**: Comprehensive testing and final validation

**Steps**:

1. Run all backend tests:
```bash
cargo test
```

2. Run all frontend tests:
```bash
cd crates/bodhi && npm test
```

3. Rebuild embedded UI:
```bash
cd crates/bodhi && npm run build
```

4. Format all code:
```bash
cargo fmt --all
cd crates/bodhi && npm run format
```

5. Regenerate OpenAPI and TypeScript client:
```bash
cargo run --package xtask openapi
cd ts-client && npm run build
```

**Final Validation Checklist**:
- [ ] All backend tests pass
- [ ] All frontend tests pass
- [ ] UI builds successfully
- [ ] Code formatted
- [ ] OpenAPI spec updated
- [ ] TypeScript types regenerated
- [ ] No compilation errors

**Success Criteria**:
- Zero test failures across all layers
- Clean build with no warnings
- TypeScript types in sync with backend
- Feature complete and ready for production

---

## Implementation Notes

### Agent Execution Tips

1. **Sequential Execution**: Follow phases in order, don't skip ahead
2. **Validation Gates**: Run tests after each phase, block if failures
3. **Error Handling**: If phase fails, fix issues before proceeding
4. **Documentation**: Keep notes on any deviations from plan

### Common Pitfalls

1. **Lifetime Issues**: Use `Option<String>` not `Option<&str>` with mockall
2. **Spread Operator**: Use `...()` syntax for conditional fields in TypeScript
3. **Three-State Matching**: Use tuple match `(Option, Option)` for clarity
4. **UI Rebuild**: Always rebuild after frontend changes for embedded app
5. **Type Sync**: Regenerate TS types after backend API changes

### Testing Strategy

- **Unit Tests**: Test each layer in isolation
- **Integration Tests**: Test cross-layer interactions
- **Component Tests**: Test UI behavior with MSW mocks
- **Manual Testing**: Verify end-to-end flows in running app
