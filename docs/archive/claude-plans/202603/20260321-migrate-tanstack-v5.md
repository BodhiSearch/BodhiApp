# TanStack Query v5 Migration — Complete Archive

## Context

The frontend (`crates/bodhi/src/`) migrated from `react-query@^3.39.3` to `@tanstack/react-query@^5`, reorganized hooks into domain subdirectories, and established comprehensive test infrastructure.

## Commits (in order)

| # | SHA | Description |
|---|-----|-------------|
| 1 | `84ab20af4` | Package upgrade + v5 compatibility (single-object API, `isPending`, array keys, object `invalidateQueries`) |
| 2 | `554b1e793` | Hook reorganization into domain subdirectories + CRUD naming + query key factories |
| 3 | `792812f21` | Review findings: cache invalidation fixes, backwards-compat alias removal, test restoration |
| 4 | `ee6bb81a3` | Split `hooks/access-requests/` into `hooks/apps/` + `hooks/users/` |
| 5 | `5b6fe77a2` | Shared fixture factories (models, mcps, toolsets, tokens) integrated into MSW handlers |
| 6 | `82ebfdcdc` | Test consistency fixes (createWrapper, shared handlers, beforeEach, stream error comments) |
| 7 | `2cff5b8f4` | New hook tests: 69 test cases across 9 files |

---

## Part 1: v5 Package Migration (Commit 1)

### Changes
- `react-query@^3.39.3` → `@tanstack/react-query@^5`
- `useQuery.ts` wrapper: v5 single-object API (`{ queryKey, queryFn, ...options }`)
- `useMutationQuery` wrapper: removed broken auto-invalidation, v5 single-object syntax
- All `invalidateQueries(key)` → `invalidateQueries({ queryKey: key })`
- All string query keys → arrays
- `useMutation` direct call in `use-chat-completions.ts`: two-arg → single-object
- Mutation `isLoading` → `isPending` across ~22 component files
- Removed `skipCacheInvalidation` from all callers (was broken)

### Key Decision
Query `isLoading` kept as-is (v5 still has it with new semantics: `isPending && isFetching`). Only mutation `isLoading` renamed to `isPending`.

---

## Part 2: Hook Reorganization (Commit 2)

### Final Directory Structure
```
hooks/
  constants.ts               # BODHI_API_BASE
  useQuery.ts                # core wrapper (stays flat)
  useQuery.test.ts           # generic query utility tests
  use-toast.ts               # UI utilities (stay flat, kebab-case)
  use-toast-messages.ts
  use-media-query.ts
  use-mobile.tsx
  use-responsive-testid.tsx
  useLocalStorage.ts
  use-browser-detection.ts
  use-extension-detection.ts

  models/
    constants.ts             # modelKeys, modelFileKeys, downloadKeys, apiModelKeys, apiFormatKeys + endpoints
    index.ts
    useModels.ts             # useListModels, useGetModel, useCreateModel, useUpdateModel
    useModels.test.ts
    useModelFiles.ts         # useListModelFiles
    useDownloads.ts          # useListDownloads, usePullModel
    useModelMetadata.ts      # useRefreshAllMetadata, useRefreshSingleMetadata
    useModelCatalog.ts       # useChatModelsCatalog, useEmbeddingModelsCatalog
    useModelsApi.ts          # useGetApiModel, useCreateApiModel, useUpdateApiModel, useDeleteApiModel, ...
    useModelsApi.test.ts

  mcps/
    constants.ts             # mcpKeys, mcpServerKeys, authConfigKeys, oauthTokenKeys + endpoints
    index.ts
    useMcpInstances.ts       # useListMcps, useGetMcp, useCreateMcp, useUpdateMcp, useDeleteMcp
    useMcpInstances.test.ts
    useMcpServers.ts         # useListMcpServers, useGetMcpServer, useCreateMcpServer, useUpdateMcpServer
    useMcpServers.test.ts
    useMcpAuthConfigs.ts     # useListAuthConfigs, useGetAuthConfig, useCreateAuthConfig, useDeleteAuthConfig
    useMcpAuthConfigs.test.ts
    useMcpOAuth.ts           # useDiscoverMcp, useStandaloneDynamicRegister, useOAuthLogin, useOAuthTokenExchange, ...
    useMcpOAuth.test.ts
    useMcpTools.ts           # useFetchMcpTools, useRefreshMcpTools, useExecuteMcpTool
    useMcpTools.test.ts
    useMcpSelection.ts

  toolsets/
    constants.ts             # toolsetKeys, toolsetTypeKeys + endpoints
    index.ts
    useToolsets.ts            # useListToolsets, useGetToolset, useCreateToolset, useUpdateToolset, useDeleteToolset
    useToolsets.test.ts
    useToolsetTypes.ts        # useListToolsetTypes, useEnableToolsetType, useDisableToolsetType
    useToolsetSelection.ts

  tokens/
    constants.ts             # tokenKeys + endpoints
    index.ts
    useTokens.ts             # useListTokens, useCreateToken, useUpdateToken
    useTokens.test.ts

  users/
    constants.ts             # userKeys, accessRequestKeys + endpoints
    index.ts
    useUsers.ts              # useGetUser, useGetAuthenticatedUser, useListUsers, useChangeUserRole, useRemoveUser
    useUsers.test.ts
    useUserAccessRequests.ts # useGetRequestStatus, useSubmitAccessRequest, useListPendingRequests, ...
    useUserAccessRequests.test.ts

  apps/
    constants.ts             # appAccessRequestKeys + endpoints
    index.ts
    useAppAccessRequests.ts  # useGetAppAccessRequestReview, useApproveAppAccessRequest, useDenyAppAccessRequest
    useAppAccessRequests.test.ts

  settings/
    constants.ts             # settingKeys + endpoints
    index.ts
    useSettings.ts           # useListSettings, useUpdateSetting, useDeleteSetting
    useSettings.test.ts

  auth/
    constants.ts
    index.ts
    useAuth.ts
    useAuth.test.tsx

  tenants/
    constants.ts             # tenantKeys + endpoints
    index.ts
    useTenants.ts

  info/
    constants.ts             # appInfoKeys + endpoints
    index.ts
    useInfo.ts               # useGetAppInfo, useSetupApp
    useInfo.test.ts

  chat/
    constants.ts
    index.ts
    useChat.tsx
    useChat.test.tsx
    useChatCompletions.ts
    useChatCompletions.test.tsx
    useChatDb.tsx
    useChatDb.test.tsx
    useChatSettings.tsx
    useChatSettings.test.tsx

  navigation/
    index.ts
    useNavigation.tsx
    useNavigation.test.tsx
```

### CRUD Naming Convention
| Pattern | Example |
|---------|---------|
| `useList*` | `useListModels`, `useListMcps`, `useListUsers` |
| `useGet*` | `useGetModel`, `useGetMcp`, `useGetUser` |
| `useCreate*` | `useCreateModel`, `useCreateMcp` |
| `useUpdate*` | `useUpdateModel`, `useUpdateMcp` |
| `useDelete*` | `useDeleteModel`, `useDeleteMcp` |

### Query Key Factory Pattern
```typescript
export const modelKeys = {
  all: ['models'] as const,
  lists: () => [...modelKeys.all, 'list'] as const,
  list: (page, pageSize, sort, sortOrder) => [...modelKeys.lists(), page, pageSize, sort, sortOrder] as const,
  details: () => [...modelKeys.all, 'detail'] as const,
  detail: (id: string) => [...modelKeys.details(), id] as const,
};
```

### Access Request Domain Split
Originally `hooks/access-requests/` contained both user role access requests and 3rd-party OAuth app access requests. Split into:
- `hooks/users/useUserAccessRequests.ts` — user-side: check status, submit request; admin-side: list pending/all, approve, reject
- `hooks/apps/useAppAccessRequests.ts` — 3rd-party OAuth app review, approve, deny

---

## Part 3: Review Findings & Fixes (Commit 3)

### Cache Invalidation Fixes
- `useEnableToolsetType`/`useDisableToolsetType`: added `toolsetTypeKeys.all` invalidation (was only invalidating `toolsetKeys.all`)
- `useUpdateModel`: added `modelKeys.all` invalidation (was only invalidating detail)
- `useRefreshAllMetadata`: added missing invalidation for `modelKeys.all` + `modelFileKeys.all`
- Playground page: fixed `['mcps', id]` → `mcpKeys.detail(id)`
- 4 raw array key locations → key factory references

### Backwards-Compat Alias Removal
Removed 23 deprecated hook name aliases (e.g., `useModels` → only `useListModels`).

### Test Restoration
Restored 13 test files (~4,059 lines) deleted during reorganization commit, with updated import paths and hook names.

---

## Part 4: Test Infrastructure (Commits 4-7)

### Shared Fixture Factories
| File | Factories |
|------|-----------|
| `test-fixtures/models.ts` | `createMockUserAlias`, `createMockApiAlias`, `createMockPaginatedModels`, ... |
| `test-fixtures/mcps.ts` | `createMockMcp`, `createMockMcpServerResponse`, `createMockAuthConfigHeader`, ... |
| `test-fixtures/toolsets.ts` | `createMockToolset`, `createMockToolsetDefinition`, ... |
| `test-fixtures/tokens.ts` | `createMockToken`, `createMockPaginatedTokens`, ... |
| `test-fixtures/apps.ts` | App access request review response fixtures |
| `test-fixtures/access-requests.ts` | User access request fixtures, role constants |

Convention: `createMock<Entity>(overrides?: Partial<T>): T` with realistic defaults. Integrated into MSW handlers as single source of truth.

### Test Consistency Fixes
- `useChatCompletions.test.tsx`: replaced inline QueryClient with `createWrapper()`
- `useUsers.test.ts`: replaced inline MSW handlers (`http.get(...)`) with shared handler imports
- `useInfo.test.ts`: changed `beforeAll` → `beforeEach` for handler setup
- Stream error tests: added "Intentional: stream errors silently handled" comments

### New Hook Tests (69 test cases)
| File | Tests |
|------|-------|
| `hooks/mcps/useMcpInstances.test.ts` | 7 |
| `hooks/mcps/useMcpServers.test.ts` | 5 |
| `hooks/mcps/useMcpTools.test.ts` | 9 |
| `hooks/mcps/useMcpOAuth.test.ts` | 9 |
| `hooks/mcps/useMcpAuthConfigs.test.ts` | 7 |
| `hooks/users/useUserAccessRequests.test.ts` | 10 |
| `hooks/apps/useAppAccessRequests.test.ts` | 8 |
| `hooks/models/useModelsApi.test.ts` | 7 |
| `hooks/useQuery.test.ts` | 7 |

### Test Convention
```typescript
import { useHookName } from '@/hooks/<domain>';
import { mockHandler } from '@/test-utils/msw-v2/handlers/<domain>';
import { createMockEntity } from '@/test-fixtures/<domain>';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

describe('useHookName', () => {
  beforeEach(() => { server.use(...mockHandlers()); });

  it('succeeds', async () => {
    const { result } = renderHook(() => useHookName(), { wrapper: createWrapper() });
    await waitFor(() => { expect(result.current.isSuccess).toBe(true); });
  });

  it('mutation calls onSuccess', async () => {
    const onSuccess = vi.fn();
    const { result } = renderHook(() => useHookName({ onSuccess }), { wrapper: createWrapper() });
    await act(async () => { await result.current.mutateAsync(variables); });
    expect(onSuccess).toHaveBeenCalledWith(expectedData);
  });
});
```

### Hooks Intentionally Not Tested (thin query wrappers)
useListMcps, useGetMcp, useListMcpServers, useGetMcpServer, useListAuthConfigs, useGetAuthConfig, useGetOAuthToken, useListModelFiles, useListDownloads, useChatModelsCatalog, useEmbeddingModelsCatalog, useListApiFormats, useFetchApiModels, useTenants, useToolsetTypes queries, useToolsetSelection, useMcpSelection, useLocalStorage, use-media-query, use-mobile, use-responsive-testid, use-toast, use-toast-messages

### Known Skipped Test
`useUsers.test.ts > useListUsers > handles error response` — skipped because `useListUsers` has explicit `retry: 1` that overrides wrapper's `retry: false`; TanStack Query's backoff exceeds `waitFor`'s default timeout. Tracked in `crates/bodhi/TECHDEBT.md`.

---

## Final Stats

| Metric | Before | After |
|--------|--------|-------|
| Package | `react-query@^3.39.3` | `@tanstack/react-query@^5` |
| Hook organization | 15 flat files | 12 domain subdirectories |
| Query key management | Raw strings/arrays | Typed key factories per domain |
| Test fixture infrastructure | Inline mock data | Shared factory functions + unified MSW handlers |
| Hook test coverage | 25% (12/48) | ~50% (21/48), all mutations covered |
| Test count | 886 | 956 |

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Wrapper fate | Keep, update internals | Domain hooks need zero changes during v5 upgrade |
| Auto-invalidation | Remove entirely | Already broken, only ~5 nominal users |
| Query `isLoading` | Keep as-is (v5 semantics) | New semantics more correct for `enabled` queries |
| Mutation `isLoading` | Rename to `isPending` | Required — v5 removes from `UseMutationResult` |
| Key management | Per-domain factories | Type-safe, prevents key typos |
| Hook naming | `useList*/useGet*/useCreate*/useUpdate*/useDelete*` | Explicit, predictable, conventional |
| File naming | camelCase (`useModels.ts`) | Filename matches hook name pattern |
| Access requests split | `hooks/users/` + `hooks/apps/` | User role access ≠ 3rd-party OAuth app access |
| Thin wrapper testing | Skip | Trust useQuery wrapper; only test hooks with logic |
| Fixture factories | By domain, realistic defaults | Single source of truth shared with MSW handlers |
| Stream error handling | Intentional silent handling | Prevents chat UI disruption on partial stream failures |
