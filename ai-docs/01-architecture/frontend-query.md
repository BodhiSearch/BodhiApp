# Frontend Query Architecture & Patterns

## Overview

The BodhiApp frontend implements a sophisticated query architecture built on React Query v3.39.3, providing a unified approach to data fetching, caching, and state management. This document provides comprehensive guidelines for understanding, implementing, and extending the query patterns used throughout the application.

## Core Architecture

### Technology Foundation
- **React Query v3.39.3** - Data fetching and caching library (NOT TanStack Query)
- **Axios** - HTTP client with interceptors and error handling
- **TypeScript** - Strong typing for all query operations
- **MSW** - Mock Service Worker for testing API interactions

### Key Design Principles
1. **Unified Query Interface** - Single `useQuery` and `useMutationQuery` abstractions
2. **Type Safety** - Comprehensive TypeScript coverage for all operations
3. **Automatic Cache Management** - Intelligent invalidation and refetching
4. **Error Handling** - Consistent error patterns across all queries
5. **Testing First** - MSW-based testing for all query operations

## Core Query Hooks

### Base Query Hook: `useQuery<T>`

**Location**: `crates/bodhi/src/hooks/useQuery.ts:55-75`

```typescript
function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: Record<string, any>,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>>
```

**Key Features**:
- Generic type parameter for response data
- Flexible query key structure (string or array)
- Optional query parameters
- Automatic JSON content-type headers
- Consistent error typing with `AxiosError<ErrorResponse>`

**Usage Pattern**:
```typescript
// Simple query
const appInfo = useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);

// Parameterized query
const models = useQuery<PagedApiResponse<Model[]>>(
  ['models', page.toString(), pageSize.toString(), sort, sortOrder],
  ENDPOINT_MODELS,
  { page, page_size: pageSize, sort, sort_order: sortOrder }
);
```

### Base Mutation Hook: `useMutationQuery<T, V>`

**Location**: `crates/bodhi/src/hooks/useQuery.ts:77-113`

```typescript
function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    validateStatus?: (status: number) => boolean;
    headers?: Record<string, string>;
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V>
```

**Key Features**:
- Dynamic endpoint generation via function parameter
- Configurable HTTP methods (POST, PUT, DELETE)
- Automatic query invalidation on success
- Custom status validation for special cases (e.g., OAuth 401 responses)
- Extensible headers and axios configuration

## Query Key Patterns

### Hierarchical Key Structure
Query keys follow a hierarchical pattern for optimal cache management:

```typescript
// Entity-based keys
'appInfo'                    // Singleton entities
'user'                       // User information
'settings'                   // Application settings

// Collection-based keys with parameters
['models', page, pageSize, sort, sortOrder]     // Paginated collections
['tokens', page, pageSize]                      // API tokens
['downloads', page, pageSize]                   // Download status

// Individual entity keys
['model', alias]             // Specific model by alias
['chat', chatId]            // Specific chat by ID
```

### Cache Invalidation Strategy
- **Specific invalidation**: Target exact query keys for precise updates
- **Pattern invalidation**: Use partial keys to invalidate related queries
- **Global invalidation**: Clear all queries on logout/major state changes

## Specialized Query Patterns

### Authentication Queries

**OAuth Flow Implementation** (`useOAuth.ts`):
```typescript
// "Dumb" frontend pattern - send all params to backend
export function useOAuthInitiate(): UseMutationResult<...> {
  return useMutationQuery<AuthInitiateResponse, void>(
    ENDPOINT_AUTH_INITIATE,
    'post',
    { /* callbacks */ },
    {
      // Accept both success and auth-required responses
      validateStatus: (status) => (status >= 200 && status < 400) || status === 401,
    }
  );
}
```

**Key Principles**:
- Frontend sends all query parameters without validation
- Backend handles all OAuth logic and validation
- Custom status validation for auth flows
- Pages handle redirects, not hooks

### Paginated Data Queries

**Pattern**: Multi-parameter query keys with automatic cache segmentation
```typescript
export function useModels(page: number, pageSize: number, sort: string, sortOrder: string) {
  return useQuery<PagedApiResponse<Model[]>>(
    ['models', page.toString(), pageSize.toString(), sort, sortOrder],
    ENDPOINT_MODELS,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}
```

**Benefits**:
- Each page/sort combination cached independently
- Automatic cache invalidation on mutations
- Type-safe pagination parameters

### Streaming Data Queries

**Chat Completions** (`use-chat-completions.ts`):
- Uses native `fetch` API for Server-Sent Events (SSE)
- Bypasses React Query for streaming operations
- Integrates with React Query for final message storage

```typescript
// Streaming implementation outside React Query
const response = await fetch(`${baseUrl}/v1/chat/completions`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', ...headers },
  body: JSON.stringify(request),
});

// Process SSE stream with callbacks
if (contentType.includes('text/event-stream')) {
  // Handle streaming response with onDelta callbacks
}
```

### Local Storage Integration

**Pattern**: Hybrid server/local state management
```typescript
// Local storage hook with React Query integration
export function useChatDB() {
  const [chats, setChats] = useState<Chat[]>(() => {
    // Initialize from localStorage
  });
  
  const createOrUpdateChat = useCallback(async (chat: Chat) => {
    // Update local state and localStorage
    // No server synchronization required
  }, []);
}
```

## Error Handling Patterns

### Consistent Error Types
All queries use standardized error handling:

```typescript
interface ErrorResponse {
  error: ApiError;
}

interface ApiError {
  message: string;
  type: string;
  param?: string;
  code?: string;
}
```

### Error Processing in Mutations
```typescript
onError: (error: AxiosError<ErrorResponse>) => {
  const message = 
    error?.response?.data?.error?.message || 'Default error message';
  const code = error?.response?.data?.error?.code;
  options?.onError?.(message, code);
}
```

### Custom Error Handling
- **OAuth flows**: Accept 401 as valid response for auth initiation
- **Chat completions**: Handle both JSON and text error responses
- **Settings**: Provide specific error messages for validation failures

## Testing Patterns

### MSW Integration
All query hooks are tested using Mock Service Worker:

```typescript
const server = setupServer(
  rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings));
  }),
  rest.put(`*${ENDPOINT_SETTINGS}/:key`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings[0]));
  })
);
```

### Test Wrapper Pattern
```typescript
const { result } = renderHook(() => useSettings(), {
  wrapper: createWrapper(), // Provides QueryClient context
});
```

### Cache Invalidation Testing
```typescript
// Verify query invalidation after mutations
await waitFor(() => {
  expect(settingsResult.current.dataUpdatedAt).toBeGreaterThan(
    initialDataUpdatedAt
  );
});
```

## Performance Optimization

### Query Configuration
- **Retry policies**: Disabled for user queries (`retry: false`)
- **Stale time**: Configured per query type
- **Cache time**: Optimized for user experience
- **Background refetching**: Enabled for critical data

### Bundle Optimization
- **Tree shaking**: Individual hook exports
- **Code splitting**: Lazy loading for complex queries
- **Type optimization**: Minimal runtime overhead

## Implementation Guidelines

### Creating New Query Hooks

1. **Define types** in appropriate `/types` files
2. **Create endpoint constants** in `useQuery.ts`
3. **Implement base query** using `useQuery` or `useMutationQuery`
4. **Add error handling** with consistent patterns
5. **Write comprehensive tests** with MSW mocking
6. **Document query key patterns** for cache management

### Naming Conventions
- **Query hooks**: `use[Entity]` (e.g., `useSettings`, `useModels`)
- **Mutation hooks**: `use[Action][Entity]` (e.g., `useCreateModel`, `useUpdateSetting`)
- **Endpoints**: `ENDPOINT_[RESOURCE]` (e.g., `ENDPOINT_MODELS`)
- **Query keys**: Hierarchical strings/arrays matching entity structure

### Cache Management
- **Invalidate specific keys** after mutations
- **Use query key patterns** for related data invalidation
- **Consider cache dependencies** when designing key structures
- **Test invalidation behavior** in all mutation tests

This query architecture provides a robust, type-safe, and testable foundation for all frontend data operations in the BodhiApp.
