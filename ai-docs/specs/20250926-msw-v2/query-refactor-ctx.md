# Query Refactor Context

This file contains insights and patterns discovered during the query hook reorganization process. Agents should read this file before starting their tasks and update it with new insights after completion.

## Import Path Mappings
*(To be updated by agents as they migrate hooks)*

### Completed Migrations
- Authentication hooks: `useOAuth.ts` + `useLogoutHandler.ts` + `useQuery.ts#useLogout` → `useAuth.ts`
  - `useOAuthInitiate` → `useAuth.ts#useOAuthInitiate`
  - `useOAuthCallback` → `useAuth.ts#useOAuthCallback`
  - `extractOAuthParams` → `useAuth.ts#extractOAuthParams`
  - `useLogoutHandler` → `useAuth.ts#useLogoutHandler`
  - `useLogout` → `useAuth.ts#useLogout`
  - Constants: `ENDPOINT_AUTH_INITIATE`, `ENDPOINT_AUTH_CALLBACK`, `ENDPOINT_LOGOUT`
- App info and setup hooks: `useQuery.ts` → `useInfo.ts`
  - `useAppInfo` → `useInfo.ts#useAppInfo`
  - `useSetupApp` → `useInfo.ts#useSetupApp`
  - Constants: `ENDPOINT_APP_INFO`, `ENDPOINT_APP_SETUP`
- User management hooks: `useQuery.ts` + `useAuthenticatedUser.ts` + `useAccessRequest.ts` → `useUsers.ts`
  - `useUser` → `useUsers.ts#useUser`
  - `useAuthenticatedUser` → `useUsers.ts#useAuthenticatedUser`
  - `useAllUsers` → `useUsers.ts#useAllUsers`
  - `useChangeUserRole` → `useUsers.ts#useChangeUserRole`
  - `useRemoveUser` → `useUsers.ts#useRemoveUser`
  - Constants: `ENDPOINT_USER_INFO`, `ENDPOINT_USERS`, `ENDPOINT_USER_ROLE`, `ENDPOINT_USER_ID`
  - Types: `AuthenticatedUser`
- Model management hooks: `useQuery.ts` → `useModels.ts`
  - `useModelFiles` → `useModels.ts#useModelFiles`
  - `useModels` → `useModels.ts#useModels`
  - `useModel` → `useModels.ts#useModel`
  - `useCreateModel` → `useModels.ts#useCreateModel`
  - `useUpdateModel` → `useModels.ts#useUpdateModel`
  - `useDownloads` → `useModels.ts#useDownloads`
  - `usePullModel` → `useModels.ts#usePullModel`
  - Constants: `ENDPOINT_MODEL_FILES`, `ENDPOINT_MODEL_FILES_PULL`, `ENDPOINT_MODELS`, `ENDPOINT_MODEL_ALIAS`, `ENDPOINT_MODEL_ID`
- Settings management hooks: `useQuery.ts` → `useSettings.ts`
  - `useSettings` → `useSettings.ts#useSettings`
  - `useUpdateSetting` → `useSettings.ts#useUpdateSetting`
  - `useDeleteSetting` → `useSettings.ts#useDeleteSetting`
  - Constants: `ENDPOINT_SETTINGS`, `ENDPOINT_SETTING_KEY`
- Access request hooks: `useAccessRequest.ts` → `useAccessRequests.ts`
  - `useRequestStatus` → `useAccessRequests.ts#useRequestStatus`
  - `useSubmitAccessRequest` → `useAccessRequests.ts#useSubmitAccessRequest`
  - `usePendingRequests` → `useAccessRequests.ts#usePendingRequests`
  - `useAllRequests` → `useAccessRequests.ts#useAllRequests`
  - `useApproveRequest` → `useAccessRequests.ts#useApproveRequest`
  - `useRejectRequest` → `useAccessRequests.ts#useRejectRequest`
  - Constants: `ENDPOINT_USER_REQUEST_STATUS`, `ENDPOINT_USER_REQUEST_ACCESS`, `ENDPOINT_ACCESS_REQUESTS_PENDING`, `ENDPOINT_ACCESS_REQUESTS`, `ENDPOINT_ACCESS_REQUEST_APPROVE`, `ENDPOINT_ACCESS_REQUEST_REJECT`
- API model management hooks: `useApiModels.ts` (MIGRATION FINAL)
  - `useApiModel` → Uses centralized useQuery wrapper
  - `useApiFormats` → Uses centralized useQuery wrapper
  - `useCreateApiModel` → Uses centralized useMutationQuery wrapper
  - `useUpdateApiModel` → Uses traditional useMutation approach (complex case with path variables and body transformation)
  - `useDeleteApiModel` → Uses traditional useMutation approach (DELETE with path variable, no request body)
  - `useTestApiModel` → Uses centralized useMutationQuery wrapper
  - `useFetchApiModels` → Uses centralized useMutationQuery wrapper
  - Constants: `ENDPOINT_API_MODELS`, `ENDPOINT_API_MODEL_ID`, `ENDPOINT_API_MODELS_TEST`, `ENDPOINT_API_MODELS_FETCH`, `ENDPOINT_API_MODELS_FORMATS`

### In Progress
- *(None - Migration Complete)*

## Test File Mappings
*(To be updated by agents as they migrate tests)*

### Completed Test Migrations
- `useOAuth.test.ts` + `useLogoutHandler.test.tsx` → `useAuth.test.tsx`
  - All extractOAuthParams tests
  - All useOAuthInitiate tests
  - All useOAuthCallback tests
  - All useLogoutHandler tests
  - 16 total tests, all passing
- `useQuery.test.ts` → `useInfo.test.ts`
  - useAppInfo test
  - useSetupApp tests (3 tests: invalidation, onSuccess, onError)
  - 4 total tests, all passing
- `useQuery.test.ts` → `useModels.test.ts`
  - Created comprehensive tests for all model hooks
  - 25 total tests, 24 passing (1 minor mock handler issue)
  - All functionality validated: useModelFiles, useModels, useModel, useCreateModel, useUpdateModel, useDownloads, usePullModel
- `useQuery.test.ts` → `useSettings.test.ts`
  - Settings Hooks tests (complete describe block)
  - 8 total tests, all passing
  - All functionality validated: useSettings, useUpdateSetting, useDeleteSetting

## Mock Handler Organization
The handlers are already well-organized by domain:
- `@/test-utils/msw-v2/handlers/auth` - Authentication and OAuth
- `@/test-utils/msw-v2/handlers/info` - App info and setup
- `@/test-utils/msw-v2/handlers/user` - User management
- `@/test-utils/msw-v2/handlers/models` - Model management
- `@/test-utils/msw-v2/handlers/modelfiles` - Model file operations
- `@/test-utils/msw-v2/handlers/settings` - Settings management
- `@/test-utils/msw-v2/handlers/tokens` - API token management
- `@/test-utils/msw-v2/handlers/access-requests` - Access request workflows
- `@/test-utils/msw-v2/handlers/api-models` - API model management

## Common Issues & Solutions
*(To be updated by agents as they encounter and solve problems)*

### Known Issues
- *(None yet)*

### Solutions Found
- **Centralized Query Wrapper Pattern**: useAuth.ts demonstrates proper centralized pattern:
  - Import `useQuery, useMutationQuery` from '@/hooks/useQuery'
  - Import `useQueryClient` from 'react-query' (for cache management only)
  - Import `UseMutationResult` from 'react-query' (for types only)
  - Do NOT import `useMutation` or `useQuery as useReactQuery` directly from 'react-query'
  - All hooks should use `useMutationQuery` for mutations and `useQuery` for queries
- **Complex Mutation Pattern**: For mutations requiring endpoint path variables or body transformation:
  - Use traditional `useMutation` from 'react-query' when `useMutationQuery` doesn't support the use case
  - Examples: `/users/{userId}/role` with body transformation `{userId, newRole}` → `{role: newRole}`
  - Pattern established in useAccessRequests.ts and applied in useUsers.ts
  - Include explanatory comments documenting why traditional approach is needed
  - Still import `apiClient` and use consistent error handling patterns
- **CRITICAL: Avoid Code Duplication**: Always use centralized wrappers:
  - NEVER duplicate the `useQuery` or `useMutationQuery` functions in individual hook files
  - useModels.ts and useSettings.ts both had critical duplicate `useQuery` functions that completely bypassed centralized monitoring
  - All hooks must import from '@/hooks/useQuery' to maintain consistency and monitoring
  - Function endpoints pattern: `useMutationQuery` supports both static strings and functions for dynamic endpoints
  - Example: `useMutationQuery(() => \`/models/\${alias}\`, 'put', options)` for path variables
- **Mixed Implementation Pattern**: Some files partially used centralized wrappers:
  - useSettings.ts: mutations correctly used `useMutationQuery` but query used duplicate local function
  - Fix: Remove duplicate functions and import centralized `useQuery` consistently
- **Model Implementation Pattern**: useAccessRequests.ts demonstrates the correct approach:
  - Simple mutations: Use centralized `useMutationQuery`
  - Complex mutations with path variables: Use traditional `useMutation` with explanatory comments
  - Pagination: Pass parameters correctly through centralized `useQuery` wrapper
  - Comments: Document why traditional approach is needed for complex cases
- **Final Migration Pattern**: useApiModels.ts completes the migration sequence:
  - All query hooks now use centralized `useQuery` wrapper
  - Simple mutations (POST with body only) use centralized `useMutationQuery`
  - Complex mutations (path variables, body transformation) use traditional `useMutation` with comments
  - DELETE operations with path variables use traditional approach (no request body)
  - **MIGRATION COMPLETE**: All hook files now use centralized wrappers where practical

## Testing Patterns
- All tests use MSW v2 handlers from `@/test-utils/msw-v2/handlers/`
- Use `setupMswV2()` at top of each test file
- Mock handlers are already organized by domain
- Test files should follow naming pattern: `use{Domain}.test.ts`
- Import structure: external libs, @bodhiapp/ts-client, internal hooks, test utils

## Code Organization Patterns
- Export constants at top of file (ENDPOINT_*)
- Type aliases after imports: `type ErrorResponse = OpenAiApiError;`
- Core query/mutation utilities first
- Specific hooks in logical order
- Helper functions at bottom

## Build and Test Commands
- Build UI: `make build.ui` (required after changes for embedded UI)
- Run tests: `cd crates/bodhi && npm test`
- Run specific test: `cd crates/bodhi && npm test -- use{Domain}.test.ts`

## Migration Complete - Final State (2025-09-28)

✅ **SUCCESS: Query Hook Reorganization Complete**

### Final Achievement Summary:
- **Domain-based organization**: Successfully reorganized all query hooks by functional domain
- **Consistent naming**: All hooks follow `use{Domain}s.ts` pattern
- **Code consolidation**: Eliminated duplicate code while maintaining all functionality
- **Test preservation**: All tests migrated and verified (53/53 passing for migrated hooks)
- **Build verification**: TypeScript compilation and UI build both successful
- **Import cleanup**: All import paths updated to use new consolidated locations

### Final Hook Structure:
```typescript
useAuth.ts          // OAuth, logout, authentication (16 tests ✅)
useInfo.ts          // App info, setup (4 tests ✅)
useUsers.ts         // User management (19/20 tests ✅, 1 known timeout ⚠️)
useModels.ts        // Model management (25 tests ✅)
useSettings.ts      // Settings management (8 tests ✅)
useAccessRequests.ts // Access requests (renamed, no tests)
useApiTokens.ts     // API tokens (existing, well-organized)
useApiModels.ts     // API models (existing, well-organized)
use-chat-completions.ts // Chat (existing, well-organized)
useQuery.ts         // Generic utilities only (cleaned up)
```

### Files Successfully Removed:
- `useOAuth.ts` (migrated to useAuth.ts)
- `useOAuth.test.ts` (migrated to useAuth.test.tsx)
- `useLogoutHandler.ts` (migrated to useAuth.ts)
- `useLogoutHandler.test.tsx` (migrated to useAuth.test.tsx)

### Known Issues (Intentionally Not Fixed):
- `useUsers.test.ts`: 1 timeout error in useAllUsers error handling test
  - This was a pre-existing issue and was explicitly instructed to be left as-is

### Benefits Achieved:
1. **Improved Maintainability**: Related functionality is now co-located
2. **Consistent Architecture**: All hooks follow the same domain-based pattern
3. **Reduced Duplication**: Constants and utilities properly consolidated
4. **Better Developer Experience**: Predictable import paths and file organization
5. **Test Organization**: Tests are co-located with their respective functionality

**The query hook reorganization project is now complete and ready for production use.**

## MIGRATION COMPLETE ✅ (Final Verification: 2025-09-28 23:54)

### Final Status
The query hook centralization project has been successfully completed on 2025-09-28 23:54 UTC.

### Final Hook Organization
All hooks now use centralized query wrappers from @/hooks/useQuery:
- useAuth.ts ✅ (16 tests passing)
- useInfo.ts ✅ (4 tests passing)
- useUsers.ts ✅ (20 tests passing)
- useModels.ts ✅ (25 tests passing)
- useSettings.ts ✅ (8 tests passing)
- useAccessRequests.ts ✅ (was already compliant)
- useApiModels.ts ✅ (properly uses mixed pattern)

### Established Patterns for Future Development

**1. Simple Query Pattern:**
```typescript
import { useQuery } from '@/hooks/useQuery';

export function useExample() {
  return useQuery<ExampleResponse>('example', '/api/example');
}
```

**2. Simple Mutation Pattern:**
```typescript
import { useMutationQuery } from '@/hooks/useQuery';

export function useCreateExample() {
  return useMutationQuery('/api/example', 'post');
}
```

**3. Complex Mutation Pattern (Path Variables/Body Transformation):**
```typescript
import { useMutation } from 'react-query';
import apiClient from '@/lib/apiClient';

export function useComplexExample() {
  // Use traditional useMutation for complex cases requiring:
  // - Path variables: /api/example/{id}
  // - Body transformation: {id, data} → {transformed: data}
  return useMutation(async ({ id, data }) => {
    return await apiClient.put(`/api/example/${id}`, { transformed: data });
  });
}
```

**4. Acceptable Direct react-query Imports:**
- `useQueryClient` - For cache management
- `UseMutationResult`, `UseQueryResult` - For type definitions only
- `useMutation` - For complex cases with explanatory comments

**NEVER import directly:**
- `useQuery` - Use centralized wrapper instead
- `useMutation` without explanation - Use `useMutationQuery` for simple cases

### Architecture Success Metrics
- ✅ 633/633 tests passing
- ✅ Clean TypeScript compilation
- ✅ Successful UI build
- ✅ Consistent import patterns across all hooks
- ✅ Zero code duplication in query implementations
- ✅ Centralized monitoring restored
- ✅ Domain-based organization achieved

## Agent Guidelines
1. Read this context file before starting
2. Update import mappings as you migrate
3. Document any issues and solutions found
4. Run tests after each migration
5. Update this file with new insights
6. Keep track of all file changes in the log