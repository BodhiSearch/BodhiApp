# Type-Safe MSW Testing Refactor Specification

## Problem Statement

The current MSW (Mock Service Worker) test setup in the frontend has several issues:
1. Mock data is not type-safe against the OpenAPI-generated types from `@bodhiapp/ts-client`
2. When backend API changes occur, tests don't automatically detect breaking changes
3. Inconsistent data structures (e.g., `email` vs `username`, `roles[]` vs `role`)
4. Manual mock data creation is error-prone and doesn't validate against API contracts

## Current State Analysis

### Version Information
- **MSW Version**: 1.3.5 (outdated, v2 is available)
- **OpenAPI Generator**: @hey-api/openapi-ts
- **TypeScript Client**: @bodhiapp/ts-client (file dependency)

### Identified Issues

#### 1. Field Naming Inconsistencies
- **UserInfo**: Still using `email` in some places instead of `username`
- **Role Structure**: Some tests use `roles: ['admin']` array instead of `role: 'resource_admin'`

#### 2. Loose Typing in Mock Handlers
```typescript
// Current approach - no type validation
export interface HandlerOverrides {
  appInfo?: unknown;
  userInfo?: unknown;
  requestStatus?: unknown;
  // ... all using 'unknown'
}
```

#### 3. Manual Mock Data Without Type Checking
```typescript
// Mock data created without type validation
overrides.requestStatus || {
  status: 'pending',
  email: 'user@example.com', // Should be 'username'
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
}
```

## Proposed Solution

### Phase 1: Immediate Fixes (Priority: High)

Fix existing type inconsistencies to unblock current work:

1. **Update Mock Data Fields**
   - Change all `email` to `username` in UserInfo-related mocks
   - Change `roles: Array` to `role: string` pattern
   - Files to update:
     - `src/test-utils/msw-handlers.ts`
     - `src/test-fixtures/access-requests.ts`
     - `src/test-fixtures/users.ts`
     - Various test files in `src/app/ui/`

2. **Update Column Headers**
   - `src/app/ui/users/page.tsx:148` - Change "Email" to "Username"
   - `src/app/ui/access-requests/page.tsx:158` - Change "Email" to "Username"

### Phase 2: Add Type Annotations (Priority: High)

Leverage existing `@bodhiapp/ts-client` types for immediate type safety:

```typescript
// src/test-utils/typed-handlers.ts
import type { 
  UserInfo, 
  UserAccessRequest,
  UserAccessStatusResponse,
  PaginatedUserAccessResponse 
} from '@bodhiapp/ts-client';

export const createTypedHandlers = () => {
  const userInfoHandler = rest.get<never, never, UserInfo>(
    '*/bodhi/v1/info/user',
    (_, res, ctx) => res(ctx.json<UserInfo>({
      logged_in: true,
      username: 'test@example.com',
      role: 'resource_admin'
    }))
  );
  
  return [userInfoHandler, /* ... other handlers */];
};
```

### Phase 3: Upgrade to MSW v2 (Priority: Medium)

Migrate from MSW v1 to v2 for better TypeScript support:

1. **Update Dependencies**
   ```json
   {
     "devDependencies": {
       "msw": "^2.0.0"
     }
   }
   ```

2. **Update Handler Syntax**
   - Change from `rest` to `http` namespace
   - Update response creation syntax
   - Update server setup

### Phase 4: Implement openapi-msw (Priority: Medium)

Add full type-safe mocking with OpenAPI schema integration:

1. **Install openapi-msw**
   ```bash
   npm install --save-dev openapi-msw
   ```

2. **Update TypeScript Generation**
   ```typescript
   // ts-client/openapi-ts.config.ts
   export default defineConfig({
     input: '../openapi.json',
     output: 'src/types',
     plugins: ['@hey-api/typescript'],
     exportType: true, // Export paths type
   });
   ```

3. **Create Type-Safe Handlers**
   ```typescript
   // src/test-utils/openapi-handlers.ts
   import { createOpenApiHttp } from 'openapi-msw';
   import type { paths } from '@bodhiapp/ts-client/types';
   
   const http = createOpenApiHttp<paths>();
   
   export const handlers = [
     http.get('/bodhi/v1/info/user', ({ response }) => {
       return response(200).json({
         logged_in: true,
         username: 'test@example.com',
         role: 'resource_admin'
       });
     }),
   ];
   ```

## Implementation Plan

### Week 1: Immediate Fixes
- [ ] Fix all `email` → `username` migrations
- [ ] Fix all `roles[]` → `role` migrations  
- [ ] Update column headers in data tables
- [ ] Run tests to ensure no regressions

### Week 2: Type Safety
- [ ] Create typed handler factory functions
- [ ] Add TypeScript annotations to existing handlers
- [ ] Update all test fixtures with proper types
- [ ] Validate all mock data against ts-client types

### Week 3: MSW v2 Migration
- [ ] Update MSW to v2
- [ ] Migrate handler syntax
- [ ] Update server setup
- [ ] Fix any breaking changes

### Week 4: OpenAPI-MSW Integration
- [ ] Install and configure openapi-msw
- [ ] Update ts-client generation config
- [ ] Convert handlers to use type-safe approach
- [ ] Remove manual type annotations (now automatic)

## Benefits

1. **Compile-Time Safety**: TypeScript catches API contract violations during development
2. **Auto-Completion**: IDE suggests correct field names and types
3. **Automatic Detection**: Backend API changes immediately surface as TypeScript errors
4. **Reduced Maintenance**: No manual mock updates when API changes
5. **Living Documentation**: Types serve as accurate API documentation

## Testing Strategy

1. **Gradual Migration**: Keep old handlers working while migrating
2. **Parallel Testing**: Run both old and new handlers to verify compatibility
3. **Type Coverage**: Ensure 100% of mock handlers have type annotations
4. **CI Integration**: TypeScript compilation catches issues in CI pipeline

## Success Criteria

- [ ] All mock data validates against OpenAPI-generated types
- [ ] No `unknown` types in test handlers
- [ ] Backend API changes cause immediate TypeScript errors in tests
- [ ] All tests pass with type-safe handlers
- [ ] Developer experience improved with auto-completion

## References

- [OpenAPI-MSW Documentation](https://github.com/christoph-fricke/openapi-msw)
- [MSW v2 Migration Guide](https://mswjs.io/docs/migrations/1.x-to-2.x)
- [Hey-API OpenAPI-TS](https://heyapi.dev/openapi-ts)
- [Type-Safe Mock Backends Article](https://blog.knappi.org/0014-msw-openapi/)

## Notes

- Current MSW version (1.3.5) is functional but lacks optimal TypeScript support
- openapi-msw requires MSW v2, so migration is prerequisite
- Type safety will prevent future issues like email/username confusion
- Investment in type-safe testing infrastructure pays dividends in maintenance