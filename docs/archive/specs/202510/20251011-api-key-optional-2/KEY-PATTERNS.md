# Key Patterns - Reusable Implementation Patterns

This document extracts reusable patterns from the API key optional implementation that can be applied to similar features.

## Pattern 1: Optional Authentication in Services

### Problem
How to make authentication optional in service methods while supporting multiple credential sources?

### Solution
Use `Option<String>` for API credentials with conditional header injection.

### Implementation

**Service Trait**:
```rust
#[async_trait]
pub trait AiApiService: Send + Sync + Debug {
  /// API key is optional - if None, requests without authentication
  async fn fetch_models(
    &self,
    api_key: Option<String>,  // Use String not &str for async + mockall
    base_url: &str,
  ) -> Result<Vec<String>>;
}
```

**Implementation**:
```rust
async fn fetch_models(&self, api_key: Option<String>, base_url: &str) -> Result<Vec<String>> {
  let url = format!("{}/models", base_url);

  // Build request without auth initially
  let mut request = self.client.get(&url);

  // Only add Authorization header if API key present
  if let Some(key) = api_key {
    request = request.header("Authorization", format!("Bearer {}", key));
  }

  let response = request.send().await?;
  // ... handle response
}
```

### Why Option<String> not Option<&str>?
- Mockall with async traits has lifetime issues with borrowed data
- Owned types avoid lifetime complexity
- Performance impact negligible in HTTP context
- Easier to work with in service orchestration

### When to Use
- HTTP client services with optional authentication
- Proxy/forwarding services
- Multi-provider integrations
- Services supporting both public and private APIs

---

## Pattern 2: Three-State Credential Resolution

### Problem
How to handle multiple credential sources with clear precedence?

### Solution
Use tuple matching on `(Option, Option)` for explicit state handling.

### Implementation

**Route Handler**:
```rust
let models = match (&payload.api_key, &payload.id) {
  // Case 1: Provided API key - use it directly (highest precedence)
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

### Benefits
- Explicit precedence (provided > stored > none)
- All states explicitly handled
- Easy to reason about and test
- No nested if-else chains
- Compiler ensures exhaustiveness

### When to Use
- Multiple credential sources (user input, database, config)
- Fallback authentication patterns
- Override scenarios (new value overrides stored)
- API gateway credential routing

---

## Pattern 3: Conditional Field Inclusion (TypeScript)

### Problem
How to conditionally include optional fields in TypeScript objects?

### Solution
Use spread operator with ternary for clean conditional inclusion.

### Implementation

**Basic Pattern**:
```typescript
const request = {
  base_url: data.baseUrl,  // Required field
  ...(data.apiKey ? { api_key: data.apiKey } : {}),  // Conditional
  ...(data.id ? { id: data.id } : {}),               // Conditional
};
```

**With Multiple Conditions**:
```typescript
const testData: TestPromptRequest = {
  base_url: data.baseUrl,
  model: data.model,
  prompt: DEFAULT_PROMPT,

  // Include api_key if provided
  ...(data.apiKey ? { api_key: data.apiKey } : {}),

  // Include id if no api_key and in edit mode
  ...(!data.apiKey && mode === 'edit' && id ? { id } : {}),
};
```

### Benefits
- Clean, readable code
- Fields only present when condition true
- No undefined/null in payload
- TypeScript type safety maintained
- Easy to extend with more conditions

### When to Use
- Optional request fields
- Conditional API payloads
- Form data conversion
- Feature flags in requests

---

## Pattern 4: Checkbox-Controlled Optional Fields (React)

### Problem
How to make a form field optional with UI control?

### Solution
Checkbox controls field visibility and value inclusion.

### Implementation

**Schema**:
```typescript
const formSchema = z.object({
  required_field: z.string().min(1),
  useOptionalField: z.boolean(),        // Checkbox state
  optional_field: z.string().optional(), // Field value
});
```

**Component**:
```tsx
export function OptionalFieldInput({ register, setValue, watch }) {
  const useField = watch('useOptionalField');

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <Checkbox
          id="useOptionalField"
          checked={useField}
          onCheckedChange={(checked) => {
            setValue('useOptionalField', Boolean(checked));
            if (!checked) {
              setValue('optional_field', '');  // Clear when unchecked
            }
          }}
        />
        <Label htmlFor="useOptionalField">Use Optional Field</Label>
      </div>

      {useField && (
        <Input
          placeholder="Enter value"
          {...register('optional_field')}
        />
      )}
    </div>
  );
}
```

**Conversion**:
```typescript
const convertFormToRequest = (form: FormData): RequestDTO => ({
  required_field: form.required_field,
  // Only include if checkbox enabled
  optional_field: form.useOptionalField ? form.optional_field : undefined,
});
```

### Benefits
- Clear user intent (explicit opt-in)
- Prevents accidental empty submissions
- Clean conditional rendering
- Value automatically cleared when disabled
- Easy to test

### When to Use
- Optional credentials (API keys, tokens)
- Feature toggles in forms
- Advanced options
- Conditional configuration

---

## Pattern 5: Nullable Encrypted Database Fields

### Problem
How to handle optional encrypted data in database?

### Solution
Make encryption metadata columns nullable, skip encryption when None.

### Implementation

**Migration**:
```sql
-- Make encryption columns nullable
ALTER TABLE table_name
  ALTER COLUMN encrypted_data DROP NOT NULL,
  ALTER COLUMN salt DROP NOT NULL,
  ALTER COLUMN nonce DROP NOT NULL;
```

**Service Logic**:
```rust
async fn store_data(&self, data: Option<&str>) -> Result<()> {
  // Conditional encryption
  let (encrypted, salt, nonce) = match data {
    Some(value) => {
      let (enc, s, n) = self.secret_service.encrypt_data(value)?;
      (Some(enc), Some(s), Some(n))
    }
    None => (None, None, None),  // All NULL if no data
  };

  sqlx::query!(
    "INSERT INTO table_name (encrypted_data, salt, nonce) VALUES ($1, $2, $3)",
    encrypted,
    salt,
    nonce
  )
  .execute(&self.pool)
  .await?;

  Ok(())
}
```

**Retrieval**:
```rust
async fn get_data(&self, id: &str) -> Result<Option<String>> {
  let row = sqlx::query!(
    "SELECT encrypted_data, salt, nonce FROM table_name WHERE id = $1",
    id
  )
  .fetch_one(&self.pool)
  .await?;

  // Decrypt only if all encryption metadata present
  match (row.encrypted_data, row.salt, row.nonce) {
    (Some(encrypted), Some(salt), Some(nonce)) => {
      let decrypted = self.secret_service.decrypt_data(&encrypted, &salt, &nonce)?;
      Ok(Some(decrypted))
    }
    _ => Ok(None),  // Return None if any component missing
  }
}
```

### Benefits
- No wasted encryption operations
- NULL clearly indicates no data (vs encrypted empty string)
- Database integrity maintained
- Migration reversible

### When to Use
- Optional sensitive fields (API keys, tokens, passwords)
- User-configurable credentials
- Feature-gated encrypted data

---

## Pattern 6: Frontend Validation Simplification

### Problem
How to handle validation when backend determines requirements?

### Solution
Remove pre-flight validation, let API respond with requirements.

### Implementation

**Before (Wrong)**:
```typescript
const canFetch = (data: FetchData) => {
  const hasApiKey = Boolean(data.apiKey);
  const hasStoredCreds = mode === 'edit' && Boolean(data.id);

  if (!data.baseUrl) {
    showError("Base URL required");
    return false;
  }

  if (!hasApiKey && !hasStoredCreds) {
    showError("API key or stored credentials required");
    return false;  // Block the API call
  }

  return true;
};
```

**After (Correct)**:
```typescript
const canFetch = (data: FetchData) => {
  // Only validate what we know is absolutely required
  return Boolean(data.baseUrl);
};

const fetchModels = async (data: FetchData) => {
  try {
    // Always attempt the call - let API decide
    const response = await api.fetchModels({
      base_url: data.baseUrl,
      ...(data.apiKey ? { api_key: data.apiKey } : {}),
    });

    showSuccess(response.models);
  } catch (error) {
    // Handle API errors (401, 403, etc.) with helpful messages
    if (error.status === 401) {
      showError("API requires authentication - please add API key");
    } else {
      showError(error.message);
    }
  }
};
```

### Benefits
- Frontend doesn't make assumptions about API
- Better UX - users can try and see actual error
- Simpler code - no complex validation logic
- More flexible - works with various API configurations

### When to Use
- API authentication requirements unknown
- Multi-provider integrations
- Public/private API support
- Configuration exploration features

---

## Pattern 7: Helper Method for Credential Lookup

### Problem
How to cleanly handle optional credential lookup in services?

### Solution
Helper method returns tuple with `Option<String>` for credentials.

### Implementation

**Helper Method**:
```rust
/// Get configuration with optional credentials
/// Returns (config, api_key) where api_key may be None
async fn get_config(&self, id: &str) -> Result<(Config, Option<String>)> {
  // Get config (required)
  let config = self.db_service
    .get_config(id)
    .await?
    .ok_or_else(|| Error::NotFound(id.to_string()))?;

  // Get API key (optional - None if not configured)
  let api_key = self.db_service
    .get_api_key_for_id(id)
    .await?;  // Returns Option<String>

  Ok((config, api_key))
}
```

**Usage**:
```rust
async fn forward_request(&self, id: &str, request: Value) -> Result<Response> {
  // Get config and optional credentials
  let (config, api_key) = self.get_config(id).await?;

  // Build request with optional auth
  let mut http_request = self.client
    .post(&config.url)
    .json(&request);

  if let Some(key) = api_key {
    http_request = http_request.header("Authorization", format!("Bearer {}", key));
  }

  Ok(http_request.send().await?)
}
```

### Benefits
- Separation of concerns
- Reusable lookup logic
- Clear return type (tuple with Option)
- Easy to test in isolation
- Consistent error handling

### When to Use
- Multi-field lookups with optional components
- Configuration + credentials patterns
- Service orchestration
- Reducing code duplication

---

## Pattern 8: Test Pattern for Optional Values

### Problem
How to test code with optional values comprehensively?

### Solution
Test all states explicitly with clear expectations.

### Implementation

**Rust Tests**:
```rust
#[rstest]
#[case::with_api_key(Some("test-key".to_string()), true)]  // Has auth header
#[case::without_api_key(None, false)]                       // No auth header
#[tokio::test]
async fn test_fetch_models(
  #[case] api_key: Option<String>,
  #[case] expect_auth_header: bool,
) -> Result<()> {
  let mut server = Server::new_async().await;

  let mut mock = server.mock("GET", "/models");

  if expect_auth_header {
    mock = mock.match_header("authorization", mockito::Matcher::Any);
  } else {
    mock = mock.match_header("authorization", mockito::Matcher::Missing);
  }

  let _m = mock.with_status(200).create_async().await;

  let service = create_test_service();
  let result = service.fetch_models(api_key, server.url()).await;

  assert!(result.is_ok());
  Ok(())
}
```

**TypeScript Tests**:
```typescript
describe('Conditional field inclusion', () => {
  it.each([
    { useField: true, apiKey: 'key123', expectInPayload: true },
    { useField: false, apiKey: '', expectInPayload: false },
    { useField: false, apiKey: 'key123', expectInPayload: false },  // Checkbox overrides value
  ])('includes field based on checkbox: %o', async ({ useField, apiKey, expectInPayload }) => {
    const { result } = renderHook(() => useForm());

    act(() => {
      result.current.setValue('useField', useField);
      result.current.setValue('apiKey', apiKey);
    });

    const payload = convertFormToRequest(result.current.getValues());

    if (expectInPayload) {
      expect(payload).toHaveProperty('api_key', apiKey);
    } else {
      expect(payload).not.toHaveProperty('api_key');
    }
  });
});
```

### Benefits
- Comprehensive state coverage
- Explicit expectations
- Parameterized testing reduces duplication
- Clear documentation of behavior

### When to Use
- Testing optional parameters
- Testing conditional logic
- Testing state combinations
- Regression prevention

---

## Pattern 9: OpenAPI Schema for Optional Fields

### Problem
How to properly document optional fields in OpenAPI?

### Solution
Use `nullable: true` or `required: false` with clear examples.

### Implementation

**Rust DTO with ToSchema**:
```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
  "base_url": "https://api.openai.com/v1",
  "models": ["gpt-4"]
  // Note: No api_key in example - it's optional!
}))]
pub struct CreateRequest {
  pub base_url: String,
  pub models: Vec<String>,

  #[schema(
    nullable = true,
    description = "API key for authentication (optional - omit for public APIs)"
  )]
  pub api_key: Option<String>,
}
```

**Generated OpenAPI**:
```json
{
  "CreateRequest": {
    "type": "object",
    "required": ["base_url", "models"],
    "properties": {
      "base_url": { "type": "string" },
      "models": { "type": "array", "items": { "type": "string" } },
      "api_key": {
        "type": "string",
        "nullable": true,
        "description": "API key for authentication (optional - omit for public APIs)"
      }
    },
    "example": {
      "base_url": "https://api.openai.com/v1",
      "models": ["gpt-4"]
    }
  }
}
```

### Benefits
- Clear API documentation
- Accurate TypeScript generation
- Correct client expectations
- Examples guide usage

### When to Use
- Documenting optional authentication
- Feature flags in APIs
- Backward compatible changes
- Multi-version API support

---

## Pattern 10: Error Message Guidance

### Problem
How to provide helpful errors when optional authentication fails?

### Solution
Context-aware error messages suggesting next steps.

### Implementation

**Service Layer**:
```rust
async fn fetch_models(&self, api_key: Option<String>, base_url: &str) -> Result<Vec<String>> {
  let response = self.send_request(api_key, base_url).await?;

  match response.status() {
    StatusCode::UNAUTHORIZED => Err(AiApiServiceError::Unauthorized(
      if api_key.is_some() {
        "Invalid API key - please check your credentials".to_string()
      } else {
        "API requires authentication - please provide an API key".to_string()
      }
    )),
    StatusCode::FORBIDDEN => Err(AiApiServiceError::Forbidden(
      "Access denied - check API key permissions".to_string()
    )),
    _ => Ok(parse_models(response)?),
  }
}
```

**Frontend**:
```typescript
catch (error) {
  if (error.status === 401) {
    if (formData.useApiKey) {
      toast({
        title: 'Authentication Failed',
        description: 'Invalid API key - please check your credentials',
        variant: 'destructive',
      });
    } else {
      toast({
        title: 'Authentication Required',
        description: 'This API requires an API key. Enable "Use API Key" and try again.',
        variant: 'destructive',
        action: <Button onClick={() => setValue('useApiKey', true)}>Add API Key</Button>
      });
    }
  }
}
```

### Benefits
- Actionable error messages
- Context-aware guidance
- Improved user experience
- Reduced support burden

### When to Use
- Authentication errors
- Configuration errors
- Validation failures
- User-facing APIs

---

## Summary

These patterns provide reusable solutions for:
1. **Optional Authentication** - Service methods with optional credentials
2. **Multi-Source Credentials** - Three-state resolution with precedence
3. **Conditional Inclusion** - TypeScript spread operator pattern
4. **UI Control** - Checkbox-controlled optional fields
5. **Encrypted Storage** - Nullable columns for optional encrypted data
6. **Validation Strategy** - Let backend determine requirements
7. **Helper Methods** - Clean credential lookup patterns
8. **Testing** - Comprehensive optional value testing
9. **Documentation** - OpenAPI schemas for optional fields
10. **Error Handling** - Context-aware error messages

These patterns can be applied to similar features requiring optional authentication, configuration, or data across the stack.
