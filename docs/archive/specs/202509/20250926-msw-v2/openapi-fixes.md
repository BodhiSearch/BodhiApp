# OpenAPI Schema Optional Fields Fix

**Date**: 2025-09-27
**Status**: Planned
**Context**: TypeScript tests failing due to required fields that should be optional in OpenAPI schema

## Problem Statement

The TypeScript tests are failing because the generated OpenAPI schema marks certain fields as required when they should be optional. This forces test code to include unnecessary boilerplate:

```typescript
const mockModels = [
  {
    source: 'user' as const,
    alias: 'gpt-4',
    repo: 'test/repo',
    filename: 'model.gguf',
    snapshot: 'abc123',
    request_params: {},     // Should be optional
    context_params: [],     // Should be optional
    model_params: {},       // Should be optional
  },
```

## Root Cause

The issue is in `/crates/routes_app/src/api_dto.rs` where `UserAliasResponse` doesn't have the proper utoipa annotations to make these fields optional in the OpenAPI schema.

## How Utoipa Handles Optional Fields

To make fields optional in the generated OpenAPI schema, utoipa requires one of these approaches:

1. **Using `#[serde(default)]` attribute** - Makes the field optional with a default value
2. **Using `Option<T>` type** - Makes the field nullable and optional
3. **Combining both** for fields that can be absent or null

## Current Structure vs. Desired Structure

### Current `UserAliasResponse` (problematic)
```rust
pub struct UserAliasResponse {
    pub model_params: HashMap<String, Value>,     // Always required
    pub request_params: OAIRequestParams,         // Always required
    pub context_params: Vec<String>,              // Always required
}
```

### Source `UserAlias` (correct pattern)
```rust
pub struct UserAlias {
    #[serde(default, skip_serializing_if = "is_default")]
    pub request_params: OAIRequestParams,         // Optional with default
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_params: Vec<String>,              // Optional when empty
}
```

## Implementation Plan

### 1. Update `/crates/routes_app/src/api_dto.rs` - `UserAliasResponse` struct

Add `#[serde(default)]` and optionally `skip_serializing_if` annotations to make fields optional:

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, derive_new::new, ToSchema)]
pub struct UserAliasResponse {
    pub alias: String,
    pub repo: String,
    pub filename: String,
    pub snapshot: String,
    pub source: String,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub model_params: HashMap<String, Value>,

    #[serde(default, skip_serializing_if = "objs::is_default")]
    pub request_params: OAIRequestParams,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_params: Vec<String>,
}
```

### 2. Update the `From<UserAlias>` implementation

Update to handle defaults properly:

```rust
impl From<UserAlias> for UserAliasResponse {
    fn from(alias: UserAlias) -> Self {
        UserAliasResponse {
            repo: alias.repo.to_string(),
            filename: alias.filename,
            snapshot: alias.snapshot,
            alias: alias.alias,
            source: "user".to_string(),
            model_params: HashMap::new(),        // Default empty map
            request_params: alias.request_params, // Already has default
            context_params: alias.context_params, // Already defaults to empty vec
        }
    }
}
```

### 3. Add necessary imports

Add `use objs::is_default;` to the imports if not already present.

## Post-Implementation Steps

1. **Regenerate OpenAPI spec**:
   ```bash
   cargo run --package xtask openapi
   ```

2. **Regenerate TypeScript types**:
   ```bash
   cd ts-client && npm run generate
   ```

3. **The generated TypeScript will have optional fields**:
   ```typescript
   UserAliasResponse: {
       alias: string;
       context_params?: string[];      // Now optional
       filename: string;
       model_params?: {                // Now optional
           [key: string]: unknown;
       };
       repo: string;
       request_params?: OAIRequestParams;  // Now optional
       snapshot: string;
       source: string;
   }
   ```

4. **Test mocks can be simplified** to only include required fields:
   ```typescript
   const mockModels = [
     {
       source: 'user' as const,
       alias: 'gpt-4',
       repo: 'test/repo',
       filename: 'model.gguf',
       snapshot: 'abc123',
       // No need to add empty request_params, context_params, model_params
     },
   ```

## Additional Considerations

### Serialization Optimization

If you want these fields to be truly absent (not sent over the wire when empty), the `skip_serializing_if` predicates ensure that:
- Empty `HashMap` won't be serialized
- Default `OAIRequestParams` won't be serialized
- Empty `Vec` won't be serialized

This matches the behavior of the original `UserAlias` struct and provides a cleaner API.

### Pattern for Other Structs

This same pattern should be applied to other response DTOs that have optional fields:

1. Check if the field should truly be required
2. If optional, add `#[serde(default)]`
3. Consider adding `skip_serializing_if` for cleaner JSON output
4. Ensure utoipa recognizes the optionality through the serde attributes

### Testing Considerations

After implementing these changes:
- Existing API endpoints will continue to work (fields will just be optional)
- Clients that send these fields will continue to work
- Clients that omit these fields will now also work
- The TypeScript types will accurately reflect the API contract

## Related Issues

- Similar patterns may need to be applied to other DTOs in the codebase
- Consider auditing all `ToSchema` structs for proper optional field handling
- May want to create a helper macro for common optional field patterns