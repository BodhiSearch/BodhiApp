# API Integration

This document provides comprehensive guidance for frontend-backend API integration patterns in the Bodhi App, including query hooks, mutation patterns, and error handling.

## Required Documentation References

**MUST READ before API integration changes:**
- `ai-docs/01-architecture/frontend-react.md` - React component patterns and TypeScript conventions
- `ai-docs/01-architecture/rust-backend.md` - Backend service patterns and API design

**FOR DETAILED IMPLEMENTATION:**
- `ai-docs/01-architecture/frontend-query.md` - Complete API integration reference with code examples

## Architecture Overview

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
```typescript
// crates/bodhi/src/lib/apiClient.ts
const apiClient = axios.create({
  baseURL: '',  // Uses same origin by default
  maxRedirects: 0,
});
```

### React Query Configuration
```typescript
// crates/bodhi/src/lib/queryClient.ts
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: false,  // Explicit error handling preferred
    },
  },
});
```

## Endpoint Conventions

### API Base Paths
```typescript
// Authentication endpoints
export const ENDPOINT_APP_LOGIN = '/app/login';

// Main API base path
export const BODHI_API_BASE = '/bodhi/v1';

// Application endpoints
export const ENDPOINT_APP_INFO = `${BODHI_API_BASE}/info`;
export const ENDPOINT_MODELS = `${BODHI_API_BASE}/models`;
export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;

// OpenAI-compatible endpoints
export const ENDPOINT_OAI_CHAT_COMPLETIONS = '/v1/chat/completions';
```

**Endpoint Conventions**:
- **Application APIs**: Use `/bodhi/v1` prefix
- **OpenAI-compatible APIs**: Use `/v1` prefix
- **Authentication**: Use `/app` prefix
- **Resource-based routing**: `/models`, `/tokens`, `/settings`

## Core Query Patterns

### Generic Query Hook
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
        headers: { 'Content-Type': 'application/json' },
      });
      return data;
    },
    options
  );
}
```

### Generic Mutation Hook
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

## Pagination Patterns

### Standard Pagination Parameters
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
```typescript
type PagedApiResponse<T> = {
  data: T;
  total?: number;
  page?: number;
  page_size?: number;
};
```

## Error Handling Patterns

### Error Response Structure
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
```typescript
export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);
}
```

### User Authentication Hook
```typescript
export function useUser(options?: { enabled?: boolean }) {
  return useQuery<UserInfo | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}
```

### Resource-Specific Queries
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

### Dynamic Endpoint Generation
```typescript
export function useUpdateSetting(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}) {
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

## Streaming API Patterns

### Chat Completions with Server-Sent Events
```typescript
export function useChatCompletion() {
  const appendMutation = useMutation<void, AxiosError, {
    request: ChatCompletionRequest;
  } & ChatCompletionCallbacks & RequestExts>(
    async ({ request, headers, onDelta, onMessage, onFinish, onError }) => {
      const response = await fetch(`${baseUrl}/v1/chat/completions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers },
        body: JSON.stringify(request),
      });

      if (contentType.includes('text/event-stream')) {
        // Handle Server-Sent Events streaming
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();
        let fullContent = '';

        while (reader) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          // Process SSE chunks and call onDelta for real-time updates
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

## Component Usage Patterns

### Query Hook Usage
```typescript
export function ModelsList() {
  const { data: models, isLoading, error } = useModels();

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading models</div>;

  return (
    <div>
      {models?.map(model => (
        <div key={model.id}>{model.name}</div>
      ))}
    </div>
  );
}
```

### Mutation Hook Usage
```typescript
export function CreateModelForm() {
  const { mutate: createModel, isLoading } = useCreateModel({
    onSuccess: (model) => {
      toast({
        title: 'Success',
        description: `Model ${model.name} created successfully`,
      });
    },
    onError: (message) => {
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
      });
    },
  });

  const handleSubmit = (data: ModelFormData) => {
    createModel(data);
  };
}
```

## Local Storage Integration

### Type-Safe Local Storage Hook
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

### Usage Example
```typescript
export function ChatSettings() {
  const [settings, setSettings] = useLocalStorage('chat-settings', {
    theme: 'light',
    fontSize: 14,
    autoSave: true,
  });

  const updateTheme = (theme: string) => {
    setSettings(prev => ({ ...prev, theme }));
  };
}
```

## Cache Management

### Query Invalidation
```typescript
// Invalidate specific queries
queryClient.invalidateQueries(['models']);
queryClient.invalidateQueries(['model', alias]);

// Invalidate by endpoint
queryClient.invalidateQueries(ENDPOINT_MODELS);
```

### Cache Keys Strategy
- Use descriptive, hierarchical cache keys
- Include all parameters that affect the query
- Use consistent key patterns across similar queries
- Consider cache invalidation patterns when designing keys

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
5. After the changes run `npm run test` to verify the tests pass
6. Once all the tasks are done, run `npm run format` to format the code

## Related Documentation

- **[Frontend React](frontend-react.md)** - React component patterns and development
- **[Rust Backend](rust-backend.md)** - Backend service patterns and API design
- **[Frontend Query Architecture](frontend-query.md)** - Complete API integration reference
- **[Testing Strategy](testing-strategy.md)** - API testing patterns and mocking

---

*For complete implementation details and code examples, see [Frontend Query Architecture](frontend-query.md). For backend API patterns, see [Rust Backend](rust-backend.md).*
