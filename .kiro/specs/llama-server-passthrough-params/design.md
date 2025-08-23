# Design Document

## Overview

This design document outlines the implementation approach for replacing the structured `GptContextParams` with a string array passthrough system. The change will simplify parameter handling by allowing direct passthrough of llama-server command-line arguments while maintaining the existing `OAIRequestParams` for OpenAI compatibility.

## Architecture

### Current Architecture
```
Alias {
  request_params: OAIRequestParams (struct)
  context_params: GptContextParams (struct)
}
↓
LlamaServerArgs (converts struct fields to CLI args)
↓
llama-server process
```

### New Architecture
```
Alias {
  request_params: OAIRequestParams (struct) - unchanged
  context_params: Vec<String> (string array)
}
↓
LlamaServerArgs (appends string array directly)
↓
llama-server process
```

## Components and Interfaces

### 1. Data Model Changes

#### GptContextParams Replacement
- **Current**: Structured parameters with specific fields (`n_ctx`, `n_parallel`, etc.)
- **New**: `Vec<String>` containing raw command-line arguments
- **Location**: `crates/objs/src/gpt_params.rs`

```rust
// Remove the current GptContextParams struct entirely
// Replace with type alias for clarity
pub type GptContextParams = Vec<String>;
```

#### Alias Structure Update
- **Current**: `context_params: GptContextParams` (struct)
- **New**: `context_params: Vec<String>`
- **Location**: `crates/objs/src/alias.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_builder::Builder, new)]
pub struct Alias {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(default, skip_serializing_if = "is_default")]
  #[builder(default)]
  pub source: AliasSource,
  #[serde(default, skip_serializing_if = "is_default")]
  #[builder(default)]
  pub request_params: OAIRequestParams, // unchanged
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[builder(default)]
  pub context_params: Vec<String>, // changed from GptContextParams
}
```

### 2. Server Process Integration

#### LlamaServerArgs Update
- **Current**: Individual parameter fields with conversion logic
- **New**: Direct append of string array
- **Location**: `crates/llama_server_proc/src/server.rs`

```rust
#[derive(Debug, Clone, Builder)]
pub struct LlamaServerArgs {
  pub model: PathBuf,
  pub alias: String,
  #[builder(default)]
  api_key: Option<String>,
  #[builder(default = "portpicker::pick_unused_port().unwrap_or(8080)")]
  port: u16,
  #[builder(default)]
  host: Option<String>,
  #[builder(default)]
  verbose: bool,
  // Remove individual parameter fields
  #[builder(default)]
  context_params: Vec<String>, // new field
}

impl LlamaServerArgs {
  pub fn to_args(&self) -> Vec<String> {
    let mut args = vec![
      "--alias".to_string(),
      self.alias.clone(),
      "--model".to_string(),
      self.model.to_string_lossy().to_string(),
      "--jinja".to_string(),
    ];

    // Add standard server args (api_key, host, port, etc.)...
    
    // Process and append context parameters
    // Each string in context_params should be a complete argument like "--ctx-size 2048"
    for param in &self.context_params {
      // Split each parameter string into individual arguments
      let param_args: Vec<String> = param
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
      args.extend(param_args);
    }
    
    args
  }
}
```

### 3. API Layer Changes

#### Request/Response Models
- **Location**: `crates/routes_app/src/routes_create.rs`
- **Change**: Update `CreateAliasRequest` to handle `Vec<String>` for context_params

```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAliasRequest {
  alias: Option<String>,
  repo: Repo,
  filename: String,
  snapshot: Option<String>,
  request_params: Option<OAIRequestParams>, // unchanged
  context_params: Option<Vec<String>>, // changed
}
```

### 4. Frontend Changes

#### Schema Updates
- **Location**: `crates/bodhi/src/schemas/alias.ts`
- **Change**: Create separate schemas for form and API interaction

```typescript
// Form schema - context_params as string for textarea
export const contextParamsFormSchema = z.string().optional();

export const createAliasFormSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  request_params: requestParamsSchema, // unchanged
  context_params: contextParamsFormSchema, // string for form
});

// API schema - context_params as string array
export const contextParamsApiSchema = z.array(z.string()).optional();

export const createAliasApiSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  request_params: requestParamsSchema, // unchanged
  context_params: contextParamsApiSchema, // string array for API
});
```

#### UI Component Updates
- **Location**: `crates/bodhi/src/app/ui/models/AliasForm.tsx`
- **Change**: Replace structured form fields with textarea and convert to API format

```tsx
// Form uses string, converts to array for API
<FormField
  control={form.control}
  name="context_params"
  render={({ field }) => (
    <FormItem>
      <FormLabel>Context Parameters</FormLabel>
      <FormControl>
        <Textarea
          {...field}
          value={field.value || ''}
          onChange={(e) => field.onChange(e.target.value)}
          placeholder="Enter llama-server parameters, one per line:&#10;--ctx-size 2048&#10;--parallel 4"
          rows={6}
        />
      </FormControl>
      <FormMessage />
    </FormItem>
  )}
/>

// Conversion function for form submission
const convertFormToApi = (formData: AliasFormData): AliasApiData => ({
  ...formData,
  context_params: formData.context_params
    ? formData.context_params
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0)
    : undefined
});

// Conversion function for displaying existing data
const convertApiToForm = (apiData: AliasApiData): AliasFormData => ({
  ...apiData,
  context_params: apiData.context_params?.join('\n') || ''
});
```

## Data Models

### Serialization Format

#### Current YAML Format
```yaml
alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
```

#### New YAML Format
```yaml
alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
context_params:
  - "--ctx-size 2048"
  - "--parallel 4"
  - "--n-predict 256"
```

### JSON API Format

#### Request Format
```json
{
  "alias": "tinyllama:instruct",
  "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
  "filename": "tinyllama-1.1b-chat-v0.3.Q2_K.gguf",
  "context_params": [
    "--ctx-size 2048",
    "--parallel 4",
    "--n-predict 256"
  ]
}
```

## Error Handling

### Validation Strategy
- **No parameter validation**: The system will pass parameters directly to llama-server
- **Format validation**: Ensure `context_params` is an array of strings
- **Error propagation**: llama-server errors will be propagated back to the user

### Error Scenarios
1. **Invalid parameter format**: Return 400 Bad Request if `context_params` is not a string array
2. **llama-server startup failure**: Return 500 Internal Server Error with llama-server error message
3. **Conflicting parameters**: Let llama-server handle (last occurrence wins)

## Testing Strategy

### Unit Tests
1. **Alias serialization/deserialization** with new format
2. **LlamaServerArgs.to_args()** with context parameters
3. **API request/response** handling
4. **Frontend form** parameter parsing

### Integration Tests
1. **End-to-end alias creation** with context parameters
2. **Server startup** with custom parameters
3. **API roundtrip** tests
4. **Frontend form submission** tests

### Migration Tests
1. **Error handling** for old format data
2. **Clear error messages** for invalid formats

## Implementation Phases

### Phase 1: Backend Data Model Changes
- Update `GptContextParams` to `Vec<String>`
- Update `Alias` struct
- Update serialization/deserialization
- Add comprehensive unit tests

### Phase 2: Server Integration
- Update `LlamaServerArgs` structure
- Modify `to_args()` method
- Update server startup logic
- Add integration tests

### Phase 3: API Layer Updates
- Update request/response models
- Modify API handlers
- Update API tests
- Test end-to-end flows

### Phase 4: Frontend Changes
- Update TypeScript schemas
- Replace form components
- Update UI tests
- Test user workflows

### Phase 5: Documentation and Cleanup
- Update API documentation
- Remove unused code
- Update user documentation
- Final integration testing