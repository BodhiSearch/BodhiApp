# Migration Plan: MSW v2 with openapi-msw for api-models Test

## Document Path
`ai-docs/specs/20250926-msw-v2/msw-v2-api-models-migration.md`

## Status
- **Phase 1**: ✅ COMPLETED (See [phase-1.md](./phase-1.md) for detailed findings)
- **Phase 2**: 📋 PLANNED (Updated based on Phase 1 insights)

## Phase 1: Library Setup and Configuration (COMPLETED)

### ✅ 1.1 Dual MSW Installation Strategy
Successfully implemented side-by-side installation:
```json
{
  "devDependencies": {
    "msw": "^1.3.5",              // Keep v1 for existing tests
    "msw2": "npm:msw@^2.7.3",    // v2 alias for new handlers
    "openapi-msw": "^2.0.0",
    "openapi-typescript": "^7.5.0"
  }
}
```

### ✅ 1.2 OpenAPI Type Generation Pipeline
Established working generation workflow:
- `openapi-ts.config.ts` configuration
- `npm run generate:openapi-types` script
- Generated `src/test-utils/generated/openapi-schema.ts`
- Full `paths` and `operations` types available

### ✅ 1.3 Infrastructure Foundation
Created directory structure and basic setup:
```
src/test-utils/
├── msw-v2/
│   ├── setup.ts              # MSW v2 + openapi-msw configuration
│   └── handlers/
│       └── api-models.ts     # Handler skeleton (needs completion)
├── generated/
│   └── openapi-schema.ts     # Generated OpenAPI types
└── (existing MSW v1 files preserved)
```

### 🔍 Key Insights from Phase 1
- **Type Safety is Strict**: openapi-msw enforces exact OpenAPI schema compliance
- **Response Helper Required**: Must use `response(status).json<Type>()` pattern
- **Path Format Matters**: OpenAPI `{param}` vs Express `:param` styling
- **Import Complexity**: MSW alias creates type declaration challenges

## Phase 2: Complete api-models Test Migration to MSW v2 (UPDATED PLAN)

### Phase 2A: Fix Type Infrastructure (CRITICAL)

#### 2A.1 Create Type Declaration File
Create `src/test-utils/msw-v2/msw2.d.ts`:
```typescript
// Fix MSW v2 alias type resolution
declare module 'msw2' {
  export * from 'msw';
}

declare module 'msw2/node' {
  export * from 'msw/node';
}
```

#### 2A.2 Create Comprehensive Type Export Layer
Create `src/test-utils/msw-v2/types.ts`:
```typescript
import type { components, operations } from '../generated/openapi-schema';

// Export all required schema types for handlers
export type AppInfo = components['schemas']['AppInfo'];
export type UserInfo = components['schemas']['UserInfo'];
export type ApiFormatsResponse = components['schemas']['ApiFormatsResponse'];
export type TestPromptResponse = components['schemas']['TestPromptResponse'];
export type FetchModelsResponse = components['schemas']['FetchModelsResponse'];
export type ApiModelResponse = components['schemas']['ApiModelResponse'];
export type CreateApiModelRequest = components['schemas']['CreateApiModelRequest'];
export type OpenAIApiError = components['schemas']['OpenAIApiError'];

// Export operation types for advanced usage
export type TestApiModelOp = operations['testApiModel'];
export type CreateApiModelOp = operations['createApiModel'];
```

#### 2A.3 Fix Handler Implementation with Response Helper Pattern
Update `src/test-utils/msw-v2/handlers/api-models.ts`:
```typescript
import { http } from '../setup';
import type * as Types from '../types';

export function createApiModelHandlers(config: ApiModelHandlerConfig = {}) {
  return [
    // ✅ Correct: Use response helper with exact types
    http.get('/bodhi/v1/info', ({ response }) => {
      return response(200).json<Types.AppInfo>({
        status: 'ready' as const,
        version: '0.1.0'
      });
    }),

    http.get('/bodhi/v1/user', ({ response }) => {
      return response(200).json<Types.UserInfo>({
        email: 'test@example.com',
        role: 'resource_user' as const,
        auth_status: 'logged_in' as const
      });
    }),

    // Handle API format compatibility
    http.get('/bodhi/v1/api-models/api-formats', ({ response }) => {
      return response(200).json<Types.ApiFormatsResponse>({
        data: ['openai' as const] // Map openai-compatible to openai
      });
    }),

    // Type-safe POST handlers
    http.post('/bodhi/v1/api-models/test', ({ response }) => {
      if (config.testConnectionError) {
        return response(200).json<Types.TestPromptResponse>({
          success: false,
          error: config.testConnectionError,
          response: null
        });
      }
      return response(200).json<Types.TestPromptResponse>({
        success: true,
        error: null,
        response: 'Connection successful'
      });
    }),

    // Handle path parameters properly
    http.get('/bodhi/v1/api-models/{id}', ({ params, response }) => {
      return response(200).json<Types.ApiModelResponse>({
        id: params.id as string,
        api_format: 'openai' as const,
        base_url: 'https://api.openai.com/v1',
        api_key_masked: '****123',
        models: ['gpt-3.5-turbo'],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      });
    })
  ];
}
```

### Phase 2B: Migrate Test File Structure

#### 2B.1 Update Test Imports
Transform `src/app/ui/api-models/new/page.test.tsx`:
```typescript
// Remove MSW v1 imports
// import { rest } from 'msw';
// import { setupServer } from 'msw/node';

// Add MSW v2 imports
import { setupServer } from 'msw2/node';
import { createApiModelHandlers } from '@/test-utils/msw-v2/handlers/api-models';

// Keep existing test utilities unchanged
import { createWrapper } from '@/tests/wrapper';
import { /* existing imports */ } from '@/test-utils/api-model-test-utils';
```

#### 2B.2 Update Server Setup Pattern
```typescript
// Create server with MSW v2
const server = setupServer();

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// In tests, use new type-safe handlers
server.use(...createApiModelHandlers(createTestHandlers.openaiHappyPath()));
```

### Phase 2C: Handle Schema Compatibility Issues

#### 2C.1 API Format Enum Mapping
Address OpenAPI schema vs UI expectations:
- Schema: `"openai" | "placeholder"`
- UI expects: `"openai" | "openai-compatible"`
- **Solution**: Map both to `"openai"` in handlers, document the decision

#### 2C.2 Update Test Data Configuration
Update `src/test-utils/api-model-test-data.ts`:
```typescript
import type * as Types from './msw-v2/types';

// Replace HandlerOverrides with typed configuration
export interface ApiModelHandlerConfig {
  testConnectionError?: string;
  fetchModelsError?: boolean;
  createError?: string;
  availableModels?: string[];
  createdModel?: Types.ApiModelResponse;
  existingModel?: Types.ApiModelResponse;
  appInfo?: Types.AppInfo;
  userInfo?: Types.UserInfo;
}

// Update test scenarios to use proper types
export const createTestHandlers = {
  openaiHappyPath: (): ApiModelHandlerConfig => ({
    availableModels: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview'],
    createdModel: {
      id: 'test-openai-model',
      api_format: 'openai' as const,
      base_url: 'https://api.openai.com/v1',
      api_key_masked: '****123',
      models: ['gpt-4'],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }
  })
  // ... other scenarios with proper typing
};
```

## Phase 3: Testing and Validation

### 3.1 Individual Test Verification
```bash
# Run only the migrated api-models test
npm test -- api-models/new/page.test.tsx
```

Expected outcomes:
- All tests pass with MSW v2
- Type safety catches any mock/API mismatches
- No runtime errors from handler migration

### 3.2 Full Test Suite Verification
```bash
# Run complete test suite
npm test
```

Expected outcomes:
- Other tests using MSW v1 continue working
- No interference between v1 and v2
- All tests pass

## File Structure After Migration

```
src/test-utils/
├── msw-handlers.ts                   # Existing v1 (unchanged for other tests)
├── api-model-test-utils.ts          # Test utilities (unchanged)
├── api-model-test-data.ts           # Updated with typed data
├── generated/
│   └── openapi-schema.ts            # Generated paths type
└── msw-v2/
    ├── setup.ts                      # MSW v2 server setup
    └── handlers/
        └── api-models.ts             # Type-safe v2 handlers
```

## Benefits Achieved

1. **Type Safety**: Full TypeScript support for request/response types
2. **Contract Validation**: Compile-time errors for API mismatches
3. **Better DX**: Auto-complete for paths, params, and responses
4. **No Disruption**: Other tests continue using MSW v1
5. **Foundation for Migration**: Patterns established for migrating other tests

## Success Criteria

✅ api-models test runs successfully with MSW v2
✅ Full type safety for all mocked endpoints
✅ No regression in other tests
✅ Clear migration pattern established for remaining tests