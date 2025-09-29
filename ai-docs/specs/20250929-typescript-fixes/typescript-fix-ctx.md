# TypeScript Fix Context & Patterns

This file contains tips, patterns, and troubleshooting guidelines for fixing TypeScript errors in the BodhiApp frontend.

## Error Categories & Solutions

### 1. Import Path Corrections

**Common Incorrect Imports:**
```typescript
// WRONG
import { ENDPOINT_MODELS } from '@/hooks/useQuery';
import { ENDPOINT_APP_INFO } from '@/hooks/useUsers';
import { ENDPOINT_MODEL_FILES } from '@/hooks/useUsers';
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';

// CORRECT
import { ENDPOINT_MODELS } from '@/hooks/useModels';
import { ENDPOINT_APP_INFO } from '@/hooks/useInfo';
import { ENDPOINT_MODEL_FILES } from '@/hooks/useModels';
import { ENDPOINT_SETTINGS } from '@/hooks/useSettings';
```

**Reference Mapping:**
- `ENDPOINT_MODELS` → `@/hooks/useModels`
- `ENDPOINT_MODEL_FILES` → `@/hooks/useModels`
- `ENDPOINT_MODEL_FILES_PULL` → `@/hooks/useModels`
- `ENDPOINT_APP_INFO` → `@/hooks/useInfo`
- `ENDPOINT_USER_INFO` → `@/hooks/useUsers`
- `ENDPOINT_SETTINGS` → `@/hooks/useSettings`

### 2. Mock Data Type Patterns

**Model Object Structure:**
```typescript
// Required properties for model objects in tests
{
  alias: string,
  repo: string,
  filename: string,
  snapshot: string,
  source: "model" | "api",
  request_params?: RequestParams,
  context_params?: string[]
}
```

**Error Status Codes:**
- Allowed error codes: `400 | 401 | 403 | 500`
- **NOT allowed:** `404` (will cause TypeScript errors)

**Setting Metadata Types:**
```typescript
// Number settings need min/max
{ type: "number", min: number, max: number }

// Option settings need options array
{ type: "option", options: string[] }

// Boolean and string are simple
{ type: "boolean" }
{ type: "string" }
```

### 3. Mock Function Parameters

**Common Parameter Patterns:**
```typescript
// Mock delay configuration
mockFunction(response, { delayMs: 100 })  // NOT just number
mockFunction(response, 100)               // WRONG

// Status with location for auth
mockAuthInitiate({ location: 'url' })     // status goes in separate field

// UserResponse type handling
// logged_out: { auth_status: "logged_out" }
// logged_in: { auth_status: "logged_in", user_id, username, role, ... }
```

### 4. Generated Type Alignment

**Key Generated Types:**
- Use types from `@/test-utils/generated/openapi-schema`
- `UserResponse` has discriminated union based on `auth_status`
- `Alias` type has union based on `source` field
- Status codes are strictly typed enums

**Common Type Issues:**
- Object literals must match exact type structure
- Partial types allow optional properties but still require correct types
- Union types require proper discriminant properties

## Troubleshooting Tips

### When Import Errors Occur:
1. Check which constants are actually exported from the target module
2. Look for the correct module that exports the needed constant
3. Update the import statement path

### When Mock Data Fails:
1. Check the generated OpenAPI schema types
2. Ensure all required properties are present
3. Verify property types match exactly
4. For union types, include discriminant properties

### When Function Parameter Errors Occur:
1. Check the function signature in the mock handlers
2. Verify parameter object structure
3. Use proper configuration objects instead of primitive values

## Validation Commands

After each fix:
```bash
# Check TypeScript for specific file
npx tsc --noEmit --project tsconfig.json src/path/to/file.test.tsx

# Check all TypeScript
npm run test:typecheck

# Run tests to ensure functionality
npm test -- src/path/to/file.test.tsx
```

## Agent Insights

*Patterns and solutions discovered during the TypeScript error fixing process by 9 agents*

### Advanced Type Compatibility Patterns

**Literal Type Assertions for Enums:**
```typescript
// When working with union literal types, use 'as const' for proper type inference
object: 'chat.completion' as const,  // Instead of 'chat.completion'
role: 'assistant' as const,          // Instead of 'assistant'
status: 'inactive' as const,         // Instead of 'inactive'
```

**Union Type Property Access:**
```typescript
// For complex union types, use type assertions when properties don't exist on all variants
(result.current.data as any)?.alias  // When Alias union type doesn't allow direct access
```

**Mock Function Parameter Patterns:**
```typescript
// Mock functions expect configuration objects, not primitive values
mockFunction({ delayMs: 100 })      // Correct
mockFunction(100)                   // Incorrect

// Auth functions have specific helpers for different states
mockAuthInitiateUnauthenticated({ location: '...' })  // For 201 status
mockAuthInitiateAlreadyAuthenticated({ location: '...' })  // For 200 status
```

**Null vs Undefined for Optional Fields:**
```typescript
// Optional number fields should use undefined, not null
downloaded_bytes: undefined,  // Correct for optional number
downloaded_bytes: null,       // Incorrect - causes TypeScript error
```

**Template Literal Types:**
```typescript
// Template literals create string types, not enum types
role: `resource_${userRole}` as any,  // Need type assertion for generated strings
```

### MSW v2 Handler Patterns

**Status Code Restrictions:**
- Error handlers have strict status code unions per endpoint
- Not all endpoints support all HTTP error codes
- 404 is commonly restricted, use 400/401/403/500 instead

**Function Signature Constraints:**
- OpenAPI-generated handlers have strict parameter typing
- Cannot override hardcoded behavior with additional properties
- Must use specific helper functions for different scenarios

### Project Architecture Insights

**Import Organization:**
- Endpoint constants are organized by functional domain
- Each hook file exports only constants relevant to its API endpoints
- Import from specific modules, not generic ones

**Type Generation Integration:**
- Generated types from OpenAPI must be used exactly as defined
- No modification of generated type structure in tests
- Use conversion functions between UI forms and API types

**Test Type Safety:**
- Mock objects must align perfectly with generated OpenAPI types
- Discriminated unions require proper discriminant properties
- Type assertions should be used sparingly and only when necessary

### Performance Impact

**Agent-Based Approach Benefits:**
- Parallel processing reduced total fix time significantly
- Systematic grouping prevented conflicts between fixes
- Incremental verification caught issues early
- Knowledge transfer through context file improved later agent efficiency

**Validation Strategy:**
- TypeScript check after each agent group prevented error accumulation
- Functional test verification ensured no breaking changes
- Detailed logging enabled quick issue identification and rollback if needed