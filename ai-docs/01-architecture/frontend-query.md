# Frontend Query Architecture

This document serves as the definitive reference for frontend-to-backend API communication patterns in the Bodhi App. It provides comprehensive guidance for implementing API queries, mutations, and data management following established patterns and best practices.

**Purpose**: This documentation is designed for AI coding assistants and human developers to understand and implement consistent API integration patterns across the frontend application.

## Core Architecture Overview

The frontend uses a layered approach for backend communication:

```
┌─────────────────────────────────────────────────────────┐
│                React Components                         │
├─────────────────────────────────────────────────────────┤
│  Custom Hooks (useQuery, useMutation)                   │
├─────────────────────────────────────────────────────────┤
│  API Client (Axios) + Query Client (React Query)        │
├─────────────────────────────────────────────────────────┤
│  Backend API Endpoints                                  │
└─────────────────────────────────────────────────────────┘
```

## API Client Configuration

### Base Client Setup
**Location**: `crates/bodhi/src/lib/apiClient.ts:4-7`

```typescript
const apiClient = axios.create({
  baseURL: '',  // Uses same origin by default
  maxRedirects: 0,
});
```

**Key Characteristics**:
- Empty baseURL defaults to same origin
- No redirects to handle auth flows explicitly
- Basic error logging in response interceptor

### React Query Configuration
**Location**: `crates/bodhi/src/lib/queryClient.ts:3-10`

```typescript
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: false,  // Explicit error handling preferred
    },
  },
});
```

**Design Decisions**:
- No automatic retries - errors handled explicitly
- No window focus refetching - controlled data updates
- Optimized for explicit user-driven data fetching

## Endpoint Structure and Constants

### API Base Paths
**Location**: `crates/bodhi/src/hooks/useQuery.ts:29-44`

```typescript
// Authentication endpoints
export const ENDPOINT_APP_LOGIN = '/app/login';

// Main API base path
export const BODHI_API_BASE = '/bodhi/v1';

// Application endpoints
export const ENDPOINT_APP_INFO = `${BODHI_API_BASE}/info`;
export const ENDPOINT_APP_SETUP = `${BODHI_API_BASE}/setup`;
export const ENDPOINT_USER_INFO = `${BODHI_API_BASE}/user`;
export const ENDPOINT_LOGOUT = `${BODHI_API_BASE}/logout`;
export const ENDPOINT_MODEL_FILES = `${BODHI_API_BASE}/modelfiles`;
export const ENDPOINT_MODEL_FILES_PULL = `${BODHI_API_BASE}/modelfiles/pull`;
export const ENDPOINT_MODELS = `${BODHI_API_BASE}/models`;
export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
export const ENDPOINT_SETTINGS = `${BODHI_API_BASE}/settings`;

// OpenAI-compatible endpoints
export const ENDPOINT_OAI_CHAT_COMPLETIONS = '/v1/chat/completions';
```

**Endpoint Conventions**:
- **Application APIs**: Use `/bodhi/v1` prefix
- **OpenAI-compatible APIs**: Use `/v1` prefix
- **Authentication**: Use `/app` prefix
- **Resource-based routing**: `/models`, `/tokens`, `/settings`

### External API Endpoints
**Location**: `crates/bodhi/src/hooks/useQuery.ts:240-244`

```typescript
export function useFeaturedModels() {
  return useQuery<FeaturedModel[]>(
    'featuredModels',
    'https://api.getbodhi.app/featured-models'
  );
}
```

## Core Query Patterns

### Generic Query Hook
**Location**: `crates/bodhi/src/hooks/useQuery.ts:53-73`

```typescript
export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: Record<string, any>,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params,
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return data;
    },
    options
  );
}
```

**Usage Pattern**:
- Type-safe with generic `<T>`
- Consistent error handling with `AxiosError<ErrorResponse>`
- Standard JSON content type
- Query parameters passed through `params`

### Generic Mutation Hook
**Location**: `crates/bodhi/src/hooks/useQuery.ts:75-106`

```typescript
export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    async (variables) => {
      const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
      const response = await apiClient[method]<T>(_endpoint, variables, {
        headers: { 'Content-Type': 'application/json' },
        validateStatus: (status) => status >= 200 && status < 400,
      });
      return response;
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
        queryClient.invalidateQueries(_endpoint);
        options?.onSuccess?.(data, variables, context);
      },
    }
  );
}
```

**Key Features**:
- Supports dynamic endpoint generation via function
- Automatic cache invalidation on success
- Custom success status validation (200-399)
- Preserves original onSuccess callback

## Pagination Patterns

### Standard Pagination Parameters
**Location**: `crates/bodhi/src/hooks/useQuery.ts:146-163`

```typescript
export function useModelFiles(
  page?: number,
  pageSize?: number,
  sort: string = 'repo',
  sortOrder: string = 'asc'
) {
  return useQuery<PagedApiResponse<ModelFile[]>>(
    ['modelFiles', page?.toString() ?? '-1', pageSize?.toString() ?? '-1', sort, sortOrder],
    ENDPOINT_MODEL_FILES,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}
```

**Pagination Conventions**:
- Parameter names: `page`, `page_size` (NOT `per_page`)
- Sort parameters: `sort`, `sort_order`
- Query keys include all parameters for proper caching
- Optional parameters use `-1` in query key when undefined

### Pagination Response Type
**Location**: `crates/bodhi/src/hooks/useQuery.ts:46-51`

```typescript
type PagedApiResponse<T> = {
  data: T;
  total?: number;
  page?: number;
  page_size?: number;
};
```

**Response Structure**:
- `data`: The actual array of items
- `total`: Total count of items
- `page`: Current page number
- `page_size`: Items per page

## Error Handling Patterns

### Error Response Structure
**Location**: `crates/bodhi/src/types/models.ts:14-23`

```typescript
export interface ErrorResponse {
  error: ApiError;
}

export interface ApiError {
  message: string;
  type: string;
  param?: string;
  code?: string;
}
```

### Error Handling in Hooks
**Location**: `crates/bodhi/src/hooks/useQuery.ts:189-208`

```typescript
export function useCreateModel(options?: {
  onSuccess?: (model: Model) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Model>, AxiosError<ErrorResponse>, AliasFormData> {
  const queryClient = useQueryClient();
  return useMutationQuery<Model, AliasFormData>(ENDPOINT_MODELS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(ENDPOINT_MODELS);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create model';
      options?.onError?.(message);
    },
  });
}
```

**Error Handling Best Practices**:
- Extract user-friendly message from error response
- Provide fallback error messages
- Pass simplified error messages to callbacks
- Maintain error type safety with `AxiosError<ErrorResponse>`

## Specific API Integration Patterns

### Application Info Hook
**Location**: `crates/bodhi/src/hooks/useQuery.ts:108-110`

```typescript
export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);
}
```

### User Authentication Hook
**Location**: `crates/bodhi/src/hooks/useQuery.ts:112-117`

```typescript
export function useUser(options?: { enabled?: boolean }) {
  return useQuery<UserInfo | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}
```

**Pattern**: Conditional fetching with `enabled` option

### Model Management Hooks
**Location**: `crates/bodhi/src/hooks/useQuery.ts:178-187`

```typescript
export function useModel(alias: string) {
  return useQuery<Model>(
    ['model', alias],
    `${ENDPOINT_MODELS}/${alias}`,
    undefined,
    {
      enabled: !!alias,  // Only fetch when alias is provided
    }
  );
}
```

**Pattern**: Resource-specific queries with conditional fetching

### Settings Management
**Location**: `crates/bodhi/src/hooks/useQuery.ts:314-337`

```typescript
export function useUpdateSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Setting>, AxiosError<ErrorResponse>, { key: string; value: string | number | boolean }> {
  const queryClient = useQueryClient();
  return useMutationQuery<Setting, { key: string; value: string | number | boolean }>(
    (vars) => `${ENDPOINT_SETTINGS}/${vars.key}`,  // Dynamic endpoint
    'put',
    {
      onSuccess: () => {
        queryClient.invalidateQueries('settings');
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to update setting';
        options?.onError?.(message);
      },
    }
  );
}
```

**Pattern**: Dynamic endpoint generation for resource-specific operations

## Streaming API Patterns

### Chat Completions with Streaming
**Location**: `crates/bodhi/src/hooks/use-chat-completions.ts:51-188`

```typescript
export function useChatCompletion() {
  const appendMutation = useMutation<void, AxiosError, {
    request: ChatCompletionRequest;
  } & ChatCompletionCallbacks & RequestExts>(
    async ({ request, headers, onDelta, onMessage, onFinish, onError }) => {
      const baseUrl = apiClient.defaults.baseURL ||
        (typeof window !== 'undefined' ? window.location.origin : 'http://localhost');

      const response = await fetch(`${baseUrl}/v1/chat/completions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        let errorData: ErrorResponse | string;
        const contentType = response.headers.get('Content-Type') || '';

        if (contentType.includes('application/json')) {
          errorData = await response.json() as ErrorResponse;
        } else {
          errorData = await response.text();
        }

        onError?.(errorData);
        return;
      }

      const contentType = response.headers.get('Content-Type') || '';

      if (contentType.includes('text/event-stream')) {
        // Handle Server-Sent Events streaming
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();
        let fullContent = '';
        let metadata: MessageMetadata | undefined;

        while (reader) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split('\n').filter(
            line => line.trim() !== '' && line.trim() !== 'data: [DONE]'
          );

          for (const line of lines) {
            try {
              const jsonStr = line.replace(/^data: /, '');
              const json = JSON.parse(jsonStr);

              // Capture metadata from final chunk
              if (json.choices?.[0]?.finish_reason === 'stop' && json.timings) {
                metadata = {
                  model: json.model,
                  usage: json.usage,
                  timings: {
                    prompt_per_second: json.timings?.prompt_per_second,
                    predicted_per_second: json.timings?.predicted_per_second,
                  },
                };
              } else if (json.choices?.[0]?.delta?.content) {
                const content = json.choices[0].delta.content;
                fullContent += content;
                onDelta?.(content);  // Real-time streaming callback
              }
            } catch (e) {
              console.warn('Failed to parse SSE message:', e);
            }
          }
        }

        // Construct final message with metadata
        const finalMessage: Message = {
          role: 'assistant',
          content: fullContent,
        };
        if (metadata) {
          finalMessage.metadata = metadata;
        }
        onFinish?.(finalMessage);
      } else {
        // Handle non-streaming response
        const data: ChatCompletionResponse = await response.json();
        if (data.choices?.[0]?.message) {
          const message = { ...data.choices[0].message };
          if (data.usage) {
            message.metadata = {
              model: data.model,
              usage: data.usage,
              timings: {
                prompt_per_second: data.timings?.prompt_per_second,
                predicted_per_second: data.timings?.predicted_per_second,
              },
            };
          }
          onMessage?.(message);
          onFinish?.(message);
        }
      }
    }
  );

  return {
    append: appendMutation.mutateAsync,
    isLoading: appendMutation.isLoading,
    error: appendMutation.error,
  };
}
```

**Streaming Patterns**:
- Uses native `fetch` API for streaming support
- Handles both streaming (SSE) and non-streaming responses
- Real-time content delivery via `onDelta` callback
- Metadata extraction from final chunks
- Proper error handling for different content types

## API Token Management

### Token-Specific Hooks
**Location**: `crates/bodhi/src/hooks/useApiTokens.ts:41-52`

```typescript
export function useListTokens(
  page: number = 1,
  pageSize: number = 10,
  options?: { enabled?: boolean }
) {
  return useQuery<ListTokensResponse>(
    ['tokens', page.toString(), pageSize.toString()],
    API_TOKENS_ENDPOINT,
    { page, page_size: pageSize },
    options
  );
}
```

**Location**: `crates/bodhi/src/hooks/useApiTokens.ts:54-79`

```typescript
export function useCreateToken(options?: {
  onSuccess?: (response: TokenResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<TokenResponse>, AxiosError<ErrorResponse>, CreateTokenRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<TokenResponse, CreateTokenRequest>(
    API_TOKENS_ENDPOINT,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['tokens']);  // Invalidate list queries
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to generate token';
        options?.onError?.(message);
      },
    }
  );
}
```

**Token Management Patterns**:
- Separate module for token-specific operations
- Consistent pagination and error handling
- Cache invalidation for list queries after mutations
- Type-safe request/response interfaces

## Cache Management Patterns

### Query Key Strategies
**Location**: `crates/bodhi/src/hooks/useQuery.ts:152-159`

```typescript
// Multi-parameter query keys
['modelFiles', page?.toString() ?? '-1', pageSize?.toString() ?? '-1', sort, sortOrder]

// Resource-specific query keys
['model', alias]
['tokens', page.toString(), pageSize.toString()]

// Simple string keys
'appInfo'
'user'
'settings'
```

**Query Key Best Practices**:
- Include all parameters that affect the query result
- Use string conversion for numbers
- Use fallback values (`-1`) for optional parameters
- Hierarchical structure for related queries

### Cache Invalidation Patterns
**Location**: `crates/bodhi/src/hooks/useQuery.ts:96-102`

```typescript
onSuccess: (data, variables, context) => {
  const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
  queryClient.invalidateQueries(_endpoint);  // Invalidate by endpoint
  options?.onSuccess?.(data, variables, context);
},
```

**Location**: `crates/bodhi/src/hooks/useQuery.ts:199-201`

```typescript
onSuccess: (response) => {
  queryClient.invalidateQueries(ENDPOINT_MODELS);  // Invalidate list queries
  options?.onSuccess?.(response.data);
},
```

**Invalidation Strategies**:
- Invalidate related list queries after create/update/delete
- Use endpoint-based invalidation for generic patterns
- Specific query key invalidation for targeted updates

## Testing Patterns

### Test Setup
**Location**: `crates/bodhi/src/tests/wrapper.tsx:10-30`

```typescript
export const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,        // No retries in tests
        refetchOnMount: false,  // Controlled data fetching
      },
    },
  });

  const Wrapper = ({ children }: { children: ReactNode }) => (
    <BrowserRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </BrowserRouter>
  );

  return Wrapper;
};
```

### API Mocking with MSW
**Location**: `crates/bodhi/src/hooks/useQuery.test.ts:67-86`

```typescript
const server = setupServer(
  rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings));
  }),
  rest.put(`*${ENDPOINT_SETTINGS}/:key`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings[0]));
  }),
  rest.delete(`*${ENDPOINT_SETTINGS}/:key`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings[0]));
  }),
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
    return res(ctx.json(mockAppInfo));
  }),
);
```

**Testing Best Practices**:
- Use MSW for API mocking
- Disable retries and automatic refetching in test environment
- Mock all endpoints used in tests
- Test both success and error scenarios

### Test Structure Example
**Location**: `crates/bodhi/src/hooks/useQuery.test.ts:151-161`

```typescript
it('updates setting successfully', async () => {
  const { result } = renderHook(() => useUpdateSetting(), {
    wrapper: createWrapper(),
  });

  await act(async () => {
    await result.current.mutateAsync(updateData);
  });

  expect(result.current.data?.data).toEqual(mockUpdatedSetting);
});
```

## Type Safety Patterns

### Request/Response Types
**Location**: `crates/bodhi/src/types/api.ts:1-21`

```typescript
export interface DownloadRequest {
  id: string;
  repo: string;
  filename: string;
  status: 'pending' | 'completed' | 'error';
  error?: string;
  updated_at: string;
}

export interface ListDownloadsResponse {
  data: DownloadRequest[];
  total: number;
  page: number;
  page_size: number;
}

export interface PullModelRequest {
  repo: string;
  filename: string;
}
```

### Schema Validation
**Location**: `crates/bodhi/src/schemas/alias.ts:37-45`

```typescript
export const createAliasSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  request_params: requestParamsSchema,
  context_params: contextParamsSchema,
});

export type AliasFormData = z.infer<typeof createAliasSchema>;
```

**Type Safety Best Practices**:
- Define interfaces for all API request/response types
- Use Zod schemas for form validation
- Leverage TypeScript inference with `z.infer`
- Maintain consistent naming between frontend and backend types

## Local Storage Integration

### Type-Safe Local Storage Hook
**Location**: `crates/bodhi/src/hooks/useLocalStorage.ts` (referenced in chat-db)

```typescript
export function useLocalStorage<T>(
  key: string,
  defaultValue: T
): [T, (value: T | ((val: T) => T)) => void] {
  const [storedValue, setStoredValue] = useState<T>(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : defaultValue;
    } catch (error) {
      console.error(`Error reading localStorage key "${key}":`, error);
      return defaultValue;
    }
  });

  const setValue = (value: T | ((val: T) => T)) => {
    try {
      const valueToStore = value instanceof Function ? value(storedValue) : value;
      setStoredValue(valueToStore);
      window.localStorage.setItem(key, JSON.stringify(valueToStore));
    } catch (error) {
      console.error(`Error setting localStorage key "${key}":`, error);
    }
  };

  return [storedValue, setValue];
}
```

**Local Storage Patterns**:
- Type-safe with generics
- Error handling for JSON parsing
- Functional updates support
- Automatic serialization/deserialization

## Constants and Configuration

### Route Constants
**Location**: `crates/bodhi/src/lib/constants.ts:1-11`

```typescript
export const CURRENT_CHAT_KEY = 'current-chat';
export const ROUTE_CHAT = '/ui/chat';
export const ROUTE_DEFAULT = '/ui/chat';
export const ROUTE_RESOURCE_ADMIN = '/ui/setup/resource-admin';
export const ROUTE_SETUP = '/ui/setup';
export const ROUTE_SETUP_COMPLETE = '/ui/setup/complete';
export const ROUTE_SETUP_DOWNLOAD_MODELS = '/ui/setup/download-models';
export const ROUTE_SETUP_RESOURCE_ADMIN = '/ui/setup/resource-admin';
export const FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED = 'shown-download-models-page';
```

**Constants Organization**:
- Centralized route definitions
- Local storage keys
- Feature flags
- UI-related constants

## Implementation Anomalies

### File Organization Issues
1. **Chat DB Hook Location**: `crates/bodhi/src/hooks/use-chat-db.tsx` - Uses `.tsx` extension despite being primarily a hook file (should be `.ts`)

2. **Mixed Naming Conventions**: Some hooks use kebab-case (`use-chat-completions.ts`) while others use camelCase (`useQuery.ts`)

3. **External API Calls**: Featured models API calls external service (`https://api.getbodhi.app/featured-models`) directly from frontend hook

### Type Definition Inconsistencies
1. **Pagination Response**: Generic `PagedApiResponse<T>` type defined in hooks file rather than types directory

2. **Error Response**: Defined in `types/models.ts` but used across API integration - could be in dedicated API types file

## Best Practices Summary

### Query Implementation
1. **Use generic hooks** for consistent patterns
2. **Include all parameters** in query keys
3. **Handle errors explicitly** with user-friendly messages
4. **Implement conditional fetching** with `enabled` option
5. **Invalidate related queries** after mutations

### Error Handling
1. **Extract meaningful messages** from error responses
2. **Provide fallback messages** for unknown errors
3. **Maintain type safety** with `AxiosError<ErrorResponse>`
4. **Use callback patterns** for component-level error handling

### Performance
1. **Disable automatic retries** for explicit control
2. **Use query key hierarchies** for efficient invalidation
3. **Implement streaming** for real-time data
4. **Cache invalidation strategies** based on data relationships

### Testing
1. **Mock APIs with MSW** for realistic testing
2. **Disable automatic behaviors** in test environment
3. **Test both success and error scenarios**
4. **Use proper test wrappers** with required providers
5. After the changes run `npm run test -- run` to verify the tests pass
6. Once all the tasks are done, run `npm run format` to format the code

This documentation provides the complete reference for implementing consistent, maintainable, and performant API integration patterns in the Bodhi App frontend.
