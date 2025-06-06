# Backend Integration & State Management

This document details how the frontend integrates with the backend services and manages application state through hooks, utilities, and data flow patterns.

## Architecture Overview

The Bodhi frontend uses a layered architecture for backend integration:

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend Layer                       │
├─────────────────────────────────────────────────────────┤
│  React Components                                       │
│  ├── Pages (Route Components)                           │
│  ├── Feature Components                                 │
│  └── UI Components                                      │
├─────────────────────────────────────────────────────────┤
│  State Management Layer                                 │
│  ├── React Query (Server State)                         │
│  ├── React Hooks (Local State)                          │
│  ├── Context Providers (Global State)                   │
│  └── Local Storage (Persistence)                        │
├─────────────────────────────────────────────────────────┤
│  API Integration Layer                                  │
│  ├── Custom Hooks (useQuery, useMutation)               │
│  ├── API Client (Axios)                                 │
│  ├── Error Handling                                     │
│  └── Request/Response Transformation                    │
├─────────────────────────────────────────────────────────┤
│  Backend Services                                       │
│  ├── REST API Endpoints                                 │
│  ├── WebSocket Connections (Chat Streaming)             │
│  ├── Authentication Service                             │
│  └── File Operations                                    │
└─────────────────────────────────────────────────────────┘
```

## Core Integration Implementation

### API Client Configuration

```typescript
// lib/apiClient.ts - Axios configuration
const apiClient = axios.create({
  baseURL: '',  // Uses same origin
  maxRedirects: 0,
});

apiClient.interceptors.request.use((config) => {
  return config;
});

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('Error:', error.response?.status, error.config?.url);
    return Promise.reject(error);
  }
);
```

### React Query Setup

```typescript
// lib/queryClient.ts - Query client configuration
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: false,  // Disabled for better error handling
    },
  },
});
```

### Core Data Fetching Hooks

#### Generic Query Hook

```typescript
// hooks/useQuery.ts - Generic data fetching
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

#### Mutation Hook with Cache Invalidation

```typescript
// hooks/useQuery.ts - Mutation with automatic cache updates
export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    async (variables) => {
      const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
      return await apiClient[method]<T>(_endpoint, variables, {
        headers: { 'Content-Type': 'application/json' },
        validateStatus: (status) => status >= 200 && status < 400,
      });
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

## State Management Architecture

### Global State Management

The application uses a hybrid approach to state management:

1. **Server State**: React Query for API data
2. **Local State**: React hooks for component state
3. **Global State**: Context providers for shared state
4. **Persistent State**: Local storage for user preferences

### State Categories

#### 1. Server State (React Query)
- **API Data**: Models, users, tokens, chat history
- **Caching**: Automatic background updates and cache invalidation
- **Synchronization**: Real-time data synchronization
- **Error Handling**: Automatic retry and error recovery

```typescript
// Example: Model data fetching
const { data: models, isLoading, error } = useModels();
const { mutate: createModel } = useCreateModel({
  onSuccess: () => {
    toast({ title: 'Model created successfully' });
  }
});
```

### Specific API Integration Hooks

#### Application Info Hook

```typescript
// hooks/useQuery.ts - App info fetching
export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);
}
```

#### User Authentication Hook

```typescript
// hooks/useQuery.ts - User info with conditional fetching
export function useUser(options?: { enabled?: boolean }) {
  return useQuery<UserInfo | null>('user', ENDPOINT_USER_INFO, undefined, {
    retry: false,
    enabled: options?.enabled ?? true,
  });
}
```

#### Model Management Hooks

```typescript
// hooks/useQuery.ts - Model CRUD operations
export function useModels() {
  return useQuery<Model[]>('models', ENDPOINT_MODELS);
}

export function useModel(alias: string) {
  return useQuery<Model>(
    ['model', alias],
    `${ENDPOINT_MODELS}/${alias}`,
    undefined,
    { enabled: !!alias }
  );
}

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

export function useUpdateModel(alias: string, options?: {
  onSuccess?: (model: Model) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<Model>, AxiosError<ErrorResponse>, AliasFormData> {
  const queryClient = useQueryClient();
  return useMutationQuery<Model, AliasFormData>(`${ENDPOINT_MODELS}/${alias}`, 'put', {
    onSuccess: (response) => {
      queryClient.invalidateQueries(['model', alias]);
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to update model';
      options?.onError?.(message);
    },
  });
}
```

## API Integration Patterns

### Request Handling
1. **Authentication**
   - Token management
   - Session handling
   - Refresh token logic

2. **Error Handling**
   - Consistent error responses
   - Retry logic
   - User feedback

3. **Data Caching**
   - Local storage
   - Memory cache
   - Cache invalidation

### Real-time Features

#### Chat Streaming Implementation

The chat system uses Server-Sent Events for real-time streaming:

```typescript
// hooks/use-chat-completions.ts - Core streaming implementation
export function useChatCompletion() {
  const appendMutation = useMutation<void, AxiosError, {
    request: ChatCompletionRequest;
  } & ChatCompletionCallbacks & RequestExts>(
    async ({ request, headers, onDelta, onMessage, onFinish, onError }) => {
      const baseUrl = apiClient.defaults.baseURL || window.location.origin;

      const response = await fetch(`${baseUrl}/v1/chat/completions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = await response.json() as ErrorResponse;
        onError?.(errorData);
        return;
      }

      const contentType = response.headers.get('Content-Type') || '';

      if (contentType.includes('text/event-stream')) {
        // Handle streaming response via SSE
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();
        let fullContent = '';

        while (true) {
          const { done, value } = await reader!.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split('\n').filter(line => line.trim() !== '');

          for (const line of lines) {
            const jsonStr = line.replace(/^data: /, '');
            const json = JSON.parse(jsonStr);

            if (json.choices?.[0]?.delta?.content) {
              const content = json.choices[0].delta.content;
              fullContent += content;
              onDelta?.(content);  // Real-time content streaming
            }
          }
        }

        onFinish?.({ role: 'assistant', content: fullContent });
      } else {
        // Handle non-streaming response
        const data: ChatCompletionResponse = await response.json();
        const message = data.choices?.[0]?.message;
        if (message) {
          onMessage?.(message);
          onFinish?.(message);
        }
      }
    }
  );

  return {
    mutate: appendMutation.mutate,
    isLoading: appendMutation.isLoading,
    error: appendMutation.error,
  };
}
```

## State Management Patterns

### Local Storage Integration

```typescript
// hooks/useLocalStorage.ts - Type-safe local storage
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

### Navigation State Management

```typescript
// hooks/use-navigation.ts - Navigation state
export function useNavigation() {
  const [currentItem, setCurrentItem] = useLocalStorage<string>('navigation-current-item', '');

  return {
    currentItem,
    setCurrentItem,
  };
}
```

## Data Flow Architecture

### Request Flow
```
React Component
    ↓ (user action)
Custom Hook (useQuery/useMutation)
    ↓ (API call)
API Client (axios)
    ↓ (HTTP request)
Backend API Endpoint
    ↓ (business logic)
Service Layer
    ↓ (data access)
Database/External Services
```

### Response Flow
```
Database/External Services
    ↓ (data)
Service Layer
    ↓ (processed data)
Backend API Endpoint
    ↓ (JSON response)
API Client (axios)
    ↓ (parsed response)
Custom Hook (React Query cache)
    ↓ (state update)
React Component Re-render
```

## Error Handling Implementation

### API Error Types and Responses

```typescript
// types/api.ts - Error response structure
export interface ErrorResponse {
  error: {
    message: string;
    code?: string;
    details?: Record<string, any>;
  };
}

// Error handling in hooks
export function useCreateModel(options?: {
  onSuccess?: (model: Model) => void;
  onError?: (message: string) => void;
}) {
  return useMutationQuery<Model, AliasFormData>(ENDPOINT_MODELS, 'post', {
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create model';
      options?.onError?.(message);
    },
  });
}
```

### Toast Notification Integration

```typescript
// hooks/use-toast.ts - User feedback system
export function useToast() {
  const { toast } = useToast();

  const showSuccess = (message: string) => {
    toast({
      title: "Success",
      description: message,
      variant: "default",
    });
  };

  const showError = (message: string) => {
    toast({
      title: "Error",
      description: message,
      variant: "destructive",
    });
  };

  return { showSuccess, showError };
}
```

## Performance Optimizations

### React Query Configuration

```typescript
// lib/queryClient.ts - Optimized query client
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,  // Prevent unnecessary refetches
      retry: false,                 // Handle errors explicitly
      staleTime: 5 * 60 * 1000,    // 5 minutes
      cacheTime: 10 * 60 * 1000,   // 10 minutes
    },
    mutations: {
      retry: false,  // Don't retry mutations automatically
    },
  },
});
```

### Optimistic Updates

```typescript
// Example: Optimistic model updates
export function useUpdateModel(alias: string) {
  const queryClient = useQueryClient();

  return useMutationQuery<Model, AliasFormData>(`${ENDPOINT_MODELS}/${alias}`, 'put', {
    onMutate: async (newData) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries(['model', alias]);

      // Snapshot previous value
      const previousModel = queryClient.getQueryData(['model', alias]);

      // Optimistically update
      queryClient.setQueryData(['model', alias], newData);

      return { previousModel };
    },
    onError: (err, newData, context) => {
      // Rollback on error
      if (context?.previousModel) {
        queryClient.setQueryData(['model', alias], context.previousModel);
      }
    },
    onSettled: () => {
      // Always refetch after error or success
      queryClient.invalidateQueries(['model', alias]);
    },
  });
}
```

## Related Documentation

- **[Authentication](authentication.md)** - Security implementation details
- **[Frontend Architecture](frontend-architecture.md)** - React component architecture
- **[App Overview](app-overview.md)** - High-level system architecture

## Future Improvements

1. **API Layer**
   - API client generation
   - Type safety
   - Documentation

2. **State Management**
   - Global state solution
   - State persistence
   - Performance optimization

3. **Error Handling**
   - Error boundary implementation
   - Logging system
   - Error analytics
