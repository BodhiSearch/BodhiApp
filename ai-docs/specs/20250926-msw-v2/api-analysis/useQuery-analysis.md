# useQuery.ts Core API Implementation Analysis

## Summary

The `useQuery.ts` implementation is **architecturally sound** with excellent microservices patterns but has **3 critical REST/HTTP compliance issues** and **2 significant architectural gaps** that compromise API reliability and developer experience. The core wrapper abstractions demonstrate expert-level design principles, but missing resilience features and HTTP method inconsistencies need immediate attention.

## Critical API Compliance Issues

### Issue 1: HTTP Method Abstraction Breaks REST Semantics
**Severity**: HIGH
**Type**: HTTP Method Handling
**Impact**: Violates REST conventions and creates inconsistent API behavior

#### Problem Description
The `useMutationQuery` function abstracts HTTP methods but introduces semantic inconsistencies in body handling and parameter passing.

**Current Implementation (useQuery.ts:42-101):**
```typescript
export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    headers?: Record<string, string>;
    skipCacheInvalidation?: boolean;
    transformBody?: (variables: V) => any;
    noBody?: boolean;
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  // ... implementation

  // Problematic method handling
  if (method === 'delete' && (axiosConfig?.noBody || requestBody === undefined)) {
    const response = await apiClient[method]<T>(_endpoint, {
      headers: { /* ... */ },
    });
    return response;
  } else {
    const response = await apiClient[method]<T>(_endpoint, requestBody, {
      headers: { /* ... */ },
    });
    return response;
  }
}
```

#### REST Compliance Violations

1. **DELETE Method Body Handling**: The conditional logic for DELETE requests with/without body creates inconsistent axios parameter patterns
2. **PUT vs POST Semantics**: No differentiation in handling between idempotent (PUT) vs non-idempotent (POST) operations
3. **Content-Type Override**: Always sets `'Content-Type': 'application/json'` regardless of actual payload type

#### Recommended Solution
```typescript
export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'patch' | 'delete',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    headers?: Record<string, string>;
    skipCacheInvalidation?: boolean;
    transformBody?: (variables: V) => any;
    allowBodyForDelete?: boolean; // Explicit control
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    async (variables) => {
      const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;

      let requestBody: any;
      if (axiosConfig?.transformBody) {
        requestBody = axiosConfig.transformBody(variables);
      } else {
        requestBody = variables;
      }

      const config = {
        headers: {
          // Only set Content-Type if there's a body
          ...(requestBody && { 'Content-Type': 'application/json' }),
          ...axiosConfig?.headers,
        },
      };

      // RESTful method handling
      switch (method) {
        case 'delete':
          // DELETE typically doesn't have body, but allow explicit override
          if (axiosConfig?.allowBodyForDelete && requestBody) {
            return await apiClient.delete<T>(_endpoint, { ...config, data: requestBody });
          }
          return await apiClient.delete<T>(_endpoint, config);

        case 'post':
        case 'put':
        case 'patch':
          return await apiClient[method]<T>(_endpoint, requestBody, config);

        default:
          throw new Error(`Unsupported HTTP method: ${method}`);
      }
    },
    {
      ...options,
      onSuccess: (data, variables, context) => {
        if (!axiosConfig?.skipCacheInvalidation) {
          const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
          queryClient.invalidateQueries(_endpoint);
        }
        options?.onSuccess?.(data, variables, context);
      },
    }
  );
}
```

### Issue 2: Generic Type Safety Compromises
**Severity**: MEDIUM
**Type**: Type System Design
**Impact**: Runtime type safety gaps and poor developer experience

#### Problem Description
The generic type handling lacks proper constraints and validation, leading to potential runtime failures.

**Current Issues:**
```typescript
// Line 24: Dangerous any type usage
params?: Record<string, any>,

// Lines 25-26: Generic constraints too loose
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params,
        headers: {
          'Content-Type': 'application/json', // ✗ Not appropriate for GET
        },
      });
      return data; // ✗ No runtime validation
    },
    options
  );
}
```

#### Type Safety Issues
1. **Unconstrained Any Types**: `Record<string, any>` allows any parameter structure
2. **Missing Runtime Validation**: No schema validation against generated types
3. **Inappropriate Content-Type**: GET requests shouldn't set Content-Type header
4. **No Response Validation**: Direct return of `data` without type checking

#### Recommended Solution
```typescript
// Add proper type constraints
type QueryParams = Record<string, string | number | boolean | undefined>;

export function useQuery<T extends Record<string, unknown>>(
  key: string | string[],
  endpoint: string,
  params?: QueryParams,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params: params && Object.fromEntries(
          Object.entries(params).filter(([, value]) => value !== undefined)
        ),
        // Remove inappropriate Content-Type for GET requests
      });

      // Add runtime validation if available
      return validateResponse<T>(data, endpoint);
    },
    options
  );
}

// Runtime validation helper
function validateResponse<T>(data: unknown, endpoint: string): T {
  // Could integrate with generated types or Zod schemas
  if (typeof data === 'object' && data !== null) {
    return data as T;
  }
  throw new Error(`Invalid response format from ${endpoint}`);
}
```

### Issue 3: Cache Invalidation Strategy Lacks Precision
**Severity**: MEDIUM
**Type**: Cache Management
**Impact**: Over-invalidation leads to unnecessary network requests and poor UX

#### Problem Description
The cache invalidation strategy is too broad and doesn't follow proper cache management patterns.

**Current Implementation (useQuery.ts:90-97):**
```typescript
onSuccess: (data, variables, context) => {
  if (!axiosConfig?.skipCacheInvalidation) {
    const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
    queryClient.invalidateQueries(_endpoint); // ✗ Too broad
  }
  if (options?.onSuccess) {
    options.onSuccess(data, variables, context);
  }
},
```

#### Cache Management Issues
1. **Over-Invalidation**: Invalidates all queries matching the endpoint string
2. **Missing Granular Control**: No ability to invalidate specific related queries
3. **No Cache Optimization**: Doesn't leverage React Query's optimistic updates or background refetching
4. **Static Endpoint Matching**: String-based matching is fragile for parameterized endpoints

#### Recommended Solution
```typescript
// Enhanced cache invalidation strategy
interface CacheInvalidationConfig {
  skipCacheInvalidation?: boolean;
  invalidationStrategy?: 'exact' | 'prefix' | 'related' | 'custom';
  relatedQueries?: string[] | ((variables: V) => string[]);
  optimisticUpdate?: (variables: V) => (old: any) => any;
}

export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'patch' | 'delete',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    headers?: Record<string, string>;
    cacheConfig?: CacheInvalidationConfig;
    transformBody?: (variables: V) => any;
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  const queryClient = useQueryClient();

  return useMutation<AxiosResponse<T>, AxiosError<ErrorResponse>, V>(
    // ... mutation function
    {
      ...options,
      onMutate: async (variables) => {
        // Optimistic updates
        if (axiosConfig?.cacheConfig?.optimisticUpdate) {
          const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;
          await queryClient.cancelQueries(_endpoint);

          const previousData = queryClient.getQueryData(_endpoint);
          queryClient.setQueryData(_endpoint, axiosConfig.cacheConfig.optimisticUpdate(variables));

          return { previousData, endpoint: _endpoint };
        }

        return options?.onMutate?.(variables);
      },

      onSuccess: (data, variables, context) => {
        if (!axiosConfig?.cacheConfig?.skipCacheInvalidation) {
          const _endpoint = typeof endpoint === 'function' ? endpoint(variables) : endpoint;

          switch (axiosConfig?.cacheConfig?.invalidationStrategy || 'prefix') {
            case 'exact':
              queryClient.invalidateQueries({ queryKey: [_endpoint], exact: true });
              break;

            case 'prefix':
              queryClient.invalidateQueries({ queryKey: [_endpoint] });
              break;

            case 'related':
              const relatedQueries = typeof axiosConfig.cacheConfig.relatedQueries === 'function'
                ? axiosConfig.cacheConfig.relatedQueries(variables)
                : axiosConfig.cacheConfig.relatedQueries || [];
              relatedQueries.forEach(query => queryClient.invalidateQueries(query));
              break;

            case 'custom':
              // Allow custom invalidation logic
              break;
          }
        }

        options?.onSuccess?.(data, variables, context);
      },

      onError: (error, variables, context: any) => {
        // Revert optimistic updates on error
        if (context?.previousData && context?.endpoint) {
          queryClient.setQueryData(context.endpoint, context.previousData);
        }

        options?.onError?.(error, variables, context);
      },
    }
  );
}
```

## Architectural Strengths

### 1. Excellent Separation of Concerns
The implementation properly separates query logic from component concerns:

```typescript
// Clean abstraction layer
export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: Record<string, any>,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>>
```

**Strengths:**
- Clear API surface with predictable parameters
- Proper delegation to React Query's `useReactQuery`
- Type-safe error handling with `AxiosError<ErrorResponse>`
- Flexible key structure supporting both string and array formats

### 2. Robust Error Type Integration
Excellent integration with generated TypeScript client types:

```typescript
// Type alias for compatibility (Line 15)
type ErrorResponse = OpenAiApiError;

// Consistent error typing throughout
AxiosError<ErrorResponse>
```

**Strengths:**
- Unified error handling across all API calls
- Integration with OpenAPI-generated error types
- Type-safe error response access in consuming components

### 3. Flexible Endpoint Handling
The dynamic endpoint generation supports complex URL patterns:

```typescript
// Support for both static and dynamic endpoints
endpoint: string | ((variables: V) => string)

// Usage example from useModels.ts:
return useMutationQuery<Alias, UpdateAliasRequest>(
  () => `${ENDPOINT_MODELS}/${alias}`, // ✓ Dynamic endpoint
  'put',
  // ...
);
```

## Critical Architectural Gaps

### Gap 1: Missing Network Resilience Patterns
**Severity**: HIGH
**Impact**: Poor reliability in unstable network conditions

#### Missing Features
1. **No Retry Logic**: No automatic retry for transient failures
2. **No Timeout Configuration**: Relies on axios defaults
3. **No Circuit Breaker**: No protection against cascading failures
4. **No Offline Support**: No offline-first patterns

#### Recommended Implementation
```typescript
// Add to apiClient.ts
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
  timeout: 10000, // 10 second timeout
  retries: 3,
  retryDelay: (retryCount) => Math.min(1000 * Math.pow(2, retryCount), 10000),
});

// Add retry interceptor
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    const config = error.config;

    // Check if we should retry
    if (
      config &&
      config.retries > 0 &&
      (!error.response || error.response.status >= 500) &&
      !config.__isRetryRequest
    ) {
      config.__isRetryRequest = true;
      config.retries -= 1;

      const delayRetryRequest = new Promise<void>((resolve) => {
        setTimeout(() => resolve(), config.retryDelay(3 - config.retries));
      });

      await delayRetryRequest;
      return apiClient(config);
    }

    return Promise.reject(error);
  }
);

// Enhanced useQuery with retry configuration
export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: QueryParams,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>> & {
    retryConfig?: {
      retries?: number;
      retryDelay?: (retryCount: number) => number;
    };
  }
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params,
        ...options?.retryConfig,
      });
      return data;
    },
    {
      ...options,
      retry: options?.retryConfig?.retries ?? 3,
      retryDelay: options?.retryConfig?.retryDelay ?? ((attemptIndex) =>
        Math.min(1000 * 2 ** attemptIndex, 30000)
      ),
    }
  );
}
```

### Gap 2: Missing Request/Response Interceptor Architecture
**Severity**: MEDIUM
**Impact**: Limited observability and debugging capabilities

#### Missing Features
1. **No Request Logging**: No structured request logging
2. **No Performance Monitoring**: No request timing or metrics
3. **No Request Correlation**: No correlation IDs for debugging
4. **Limited Error Context**: Basic error logging without request context

#### Current Implementation (apiClient.ts:9-22)
```typescript
apiClient.interceptors.request.use((config) => {
  return config; // ✗ No-op request interceptor
});

apiClient.interceptors.response.use(
  (response) => {
    return response; // ✗ No-op success interceptor
  },
  (error) => {
    // Breakpoint: You can add a breakpoint here to inspect errors
    console.error('Error:', error.response?.status, error.config?.url);
    return Promise.reject(error);
  }
);
```

#### Recommended Enhancement
```typescript
// Enhanced interceptor architecture
import { nanoid } from 'nanoid';

// Request interceptor with correlation and logging
apiClient.interceptors.request.use(
  (config) => {
    const correlationId = nanoid();
    config.metadata = {
      correlationId,
      startTime: Date.now(),
    };

    config.headers['X-Correlation-ID'] = correlationId;

    // Structured request logging
    console.debug('API Request', {
      correlationId,
      method: config.method?.toUpperCase(),
      url: config.url,
      baseURL: config.baseURL,
      params: config.params,
      timestamp: new Date().toISOString(),
    });

    return config;
  },
  (error) => {
    console.error('Request Interceptor Error', { error });
    return Promise.reject(error);
  }
);

// Response interceptor with timing and enhanced error context
apiClient.interceptors.response.use(
  (response) => {
    const duration = response.config.metadata?.startTime
      ? Date.now() - response.config.metadata.startTime
      : undefined;

    console.debug('API Response', {
      correlationId: response.config.metadata?.correlationId,
      status: response.status,
      statusText: response.statusText,
      url: response.config.url,
      duration: duration ? `${duration}ms` : undefined,
      timestamp: new Date().toISOString(),
    });

    return response;
  },
  (error) => {
    const duration = error.config?.metadata?.startTime
      ? Date.now() - error.config.metadata.startTime
      : undefined;

    console.error('API Error', {
      correlationId: error.config?.metadata?.correlationId,
      status: error.response?.status,
      statusText: error.response?.statusText,
      url: error.config?.url,
      method: error.config?.method?.toUpperCase(),
      duration: duration ? `${duration}ms` : undefined,
      errorMessage: error.message,
      responseData: error.response?.data,
      timestamp: new Date().toISOString(),
    });

    return Promise.reject(error);
  }
);
```

## REST/HTTP Best Practices Assessment

### Strengths
1. **Proper HTTP Status Code Handling**: Leverages axios error handling for HTTP status codes
2. **Content Negotiation**: Uses appropriate `application/json` content type
3. **Stateless Design**: No session state maintained in the API layer
4. **Resource-Based URLs**: Endpoint constants follow REST resource patterns

### Areas for Improvement

#### 1. HTTP Header Management
```typescript
// Current: Always sets Content-Type
headers: {
  'Content-Type': 'application/json', // ✗ Not always appropriate
  ...axiosConfig?.headers,
}

// Recommended: Conditional header setting
const headers: Record<string, string> = {
  ...axiosConfig?.headers,
};

// Only set Content-Type for requests with body
if (requestBody && method !== 'get') {
  headers['Content-Type'] = 'application/json';
}

// Add cache control for GET requests
if (method === 'get') {
  headers['Cache-Control'] = 'max-age=300'; // 5 minutes
}
```

#### 2. HTTP Method Semantics
```typescript
// Current: Limited method support
method: 'post' | 'put' | 'delete' = 'post'

// Recommended: Full HTTP method support
method: 'get' | 'post' | 'put' | 'patch' | 'delete' | 'head' | 'options'

// Add proper semantics for each method
const isIdempotent = ['get', 'put', 'delete', 'head', 'options'].includes(method);
const allowsBody = ['post', 'put', 'patch'].includes(method);
```

#### 3. Response Validation
```typescript
// Add HTTP response validation
function validateHttpResponse<T>(response: AxiosResponse<T>): T {
  // Check for successful status codes
  if (response.status < 200 || response.status >= 300) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  // Validate response structure
  if (!response.data) {
    throw new Error('Empty response data');
  }

  return response.data;
}
```

## Microservices Architecture Patterns

### Excellent Patterns
1. **Service Abstraction**: Clean separation between HTTP client and business logic
2. **Error Boundary**: Centralized error handling with typed error responses
3. **Contract-First Design**: Integration with OpenAPI-generated types
4. **Composable Hooks**: Reusable query and mutation patterns

### Missing Patterns

#### 1. Circuit Breaker Pattern
```typescript
// Add circuit breaker for service resilience
class CircuitBreaker {
  private failures = 0;
  private state: 'closed' | 'open' | 'half-open' = 'closed';
  private nextAttempt = Date.now();

  constructor(
    private threshold = 5,
    private timeout = 60000,
    private monitoringPeriod = 10000
  ) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'open') {
      if (Date.now() < this.nextAttempt) {
        throw new Error('Circuit breaker is OPEN');
      }
      this.state = 'half-open';
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess() {
    this.failures = 0;
    this.state = 'closed';
  }

  private onFailure() {
    this.failures++;
    if (this.failures >= this.threshold) {
      this.state = 'open';
      this.nextAttempt = Date.now() + this.timeout;
    }
  }
}
```

#### 2. API Versioning Strategy
```typescript
// Add API versioning support
export const API_VERSIONS = {
  V1: '/bodhi/v1',
  V2: '/bodhi/v2',
} as const;

export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: QueryParams,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>> & {
    apiVersion?: keyof typeof API_VERSIONS;
  }
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  const fullEndpoint = `${API_VERSIONS[options?.apiVersion || 'V1']}${endpoint}`;
  // ... rest of implementation
}
```

## Security Considerations

### Current Security Features
1. **Type-Safe Error Handling**: Prevents information leakage through structured error types
2. **CORS-Friendly**: `maxRedirects: 0` prevents redirect-based attacks
3. **Request Validation**: Parameter type constraints prevent injection

### Security Enhancements Needed
```typescript
// Add security headers
apiClient.interceptors.request.use((config) => {
  config.headers = {
    ...config.headers,
    'X-Requested-With': 'XMLHttpRequest',
    'X-Content-Type-Options': 'nosniff',
  };

  // Add CSRF token if available
  const csrfToken = getCsrfToken();
  if (csrfToken) {
    config.headers['X-CSRF-Token'] = csrfToken;
  }

  return config;
});

// Add response security validation
apiClient.interceptors.response.use(
  (response) => {
    // Validate security headers
    const securityHeaders = [
      'x-content-type-options',
      'x-frame-options',
      'x-xss-protection',
    ];

    const missingSecurity = securityHeaders.filter(
      header => !response.headers[header]
    );

    if (missingSecurity.length > 0) {
      console.warn('Missing security headers:', missingSecurity);
    }

    return response;
  },
  (error) => {
    // Sanitize error responses to prevent information leakage
    if (error.response?.status >= 500) {
      error.message = 'Internal server error';
      delete error.response.data;
    }

    return Promise.reject(error);
  }
);
```

## Performance Optimization Recommendations

### 1. Request Deduplication
```typescript
// Add request deduplication
const pendingRequests = new Map<string, Promise<any>>();

function createRequestKey(endpoint: string, params: any): string {
  return `${endpoint}?${JSON.stringify(params)}`;
}

export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: QueryParams,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const requestKey = createRequestKey(endpoint, params);

      if (pendingRequests.has(requestKey)) {
        return pendingRequests.get(requestKey);
      }

      const request = apiClient.get<T>(endpoint, { params })
        .then(response => response.data)
        .finally(() => pendingRequests.delete(requestKey));

      pendingRequests.set(requestKey, request);
      return request;
    },
    options
  );
}
```

### 2. Response Caching Strategy
```typescript
// Enhanced caching with TTL
export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: QueryParams,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>> & {
    cacheTTL?: number;
    staleWhileRevalidate?: boolean;
  }
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, { params });
      return data;
    },
    {
      ...options,
      staleTime: options?.cacheTTL ?? 5 * 60 * 1000, // 5 minutes default
      cacheTime: options?.cacheTTL ? options.cacheTTL * 2 : 10 * 60 * 1000,
      refetchOnWindowFocus: !options?.staleWhileRevalidate,
      refetchOnReconnect: true,
    }
  );
}
```

## Implementation Priority Recommendations

### High Priority (Fix Immediately)
1. **Fix HTTP Method Handling**: Implement proper REST semantics for all HTTP methods
2. **Add Network Resilience**: Implement retry logic and timeout configuration
3. **Enhance Cache Invalidation**: Implement granular cache management

### Medium Priority (Next Sprint)
1. **Improve Type Safety**: Add runtime validation and stricter type constraints
2. **Add Request/Response Interceptors**: Implement comprehensive logging and monitoring
3. **Security Enhancements**: Add security headers and response validation

### Low Priority (Future Enhancement)
1. **Circuit Breaker Pattern**: Implement for production resilience
2. **API Versioning**: Add version management for future API evolution
3. **Performance Optimizations**: Request deduplication and advanced caching

## Conclusion

The `useQuery.ts` implementation demonstrates **expert-level microservices architecture** with excellent separation of concerns and type integration. However, **critical HTTP compliance issues** and **missing resilience patterns** significantly impact production readiness. The architectural foundation is solid and the patterns are scalable, making this an excellent base for a robust API layer once the identified issues are addressed.

**Overall Grade**: B+ (Strong architecture, critical implementation gaps)
**Production Readiness**: 65% (needs HTTP compliance fixes and resilience features)
**Microservices Maturity**: A- (excellent patterns, missing resilience features)