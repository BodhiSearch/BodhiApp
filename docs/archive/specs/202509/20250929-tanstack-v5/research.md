# TanStack Query v5 Migration Research Analysis

## Executive Summary

This document provides a comprehensive analysis of migrating from React Query v3 to TanStack Query v5 and evaluating approaches for enhanced type safety in API calls. The research covers three potential migration paths, their trade-offs, and recommendations for BodhiApp's specific requirements.

## Current State Assessment

### Technology Stack
- **React Query v3.39.3** (legacy version)
- **Axios v1.9.0** for standard HTTP calls with interceptors
- **Native fetch** for Server-Sent Events (SSE) in chat completions
- **@bodhiapp/ts-client** with openapi-typescript for type generation
- **MSW v2.10.5** for API mocking
- **Hardcoded endpoint strings** with manual path construction

### Key Files Analyzed
- `/crates/bodhi/src/hooks/useQuery.ts` - Base query hooks
- `/crates/bodhi/src/hooks/useApiModels.ts` - API model operations
- `/crates/bodhi/src/hooks/useApiTokens.ts` - Token management
- `/crates/bodhi/src/hooks/use-chat-completions.ts` - **SSE-based chat interface**
- `/crates/bodhi/src/lib/apiClient.ts` - Axios configuration
- 11 total hooks using React Query patterns

### Current Pain Points
1. **Hardcoded endpoint strings** throughout hooks (e.g., `'/bodhi/v1/api-models'`)
2. **Manual URL path construction** with string interpolation (`${ENDPOINT_API_MODELS}/${id}`)
3. **No compile-time validation** of paths or parameters
4. **Loose coupling** between TypeScript types and actual API paths
5. **Legacy React Query v3** missing modern type safety features

## Research Findings

### TanStack Query v5 Type Safety Features

#### Major Enhancements
1. **Global Error Type Configuration**
   ```typescript
   declare module '@tanstack/react-query' {
     interface Register {
       defaultError: AxiosError<OpenAiApiError>
     }
   }
   ```

2. **Unified Single Object API**
   - Removed overloads for better TypeScript error messages
   - Consistent API across all hooks
   - Enabled by TypeScript 4.7 improvements

3. **Suspense-Specific Hooks**
   ```typescript
   const { data } = useSuspenseQuery({ ... });
   // data is never undefined (type-level guarantee)
   ```

4. **queryOptions Helper**
   ```typescript
   const queryDef = queryOptions({
     queryKey: ['posts', id],
     queryFn: () => fetchPost(id)
   });
   ```

5. **Enhanced Type Inference**
   - Better autocomplete and error messages
   - Type-safe query keys with associated query functions
   - Global meta type support

### SSE (Server-Sent Events) Compatibility Analysis

#### Current Implementation (use-chat-completions.ts)
```typescript
// Uses native fetch for SSE - CORRECT APPROACH
const response = await fetch(`${baseUrl}${ENDPOINT_OAI_CHAT_COMPLETIONS}`, {
  method: 'POST',
  // ... handles text/event-stream responses
});

if (contentType.includes('text/event-stream')) {
  const reader = response.body?.getReader();
  // Stream processing logic
}
```

#### Key Findings
- **TanStack Query v5**: No native SSE support, but works with any Promise-returning function
- **openapi-fetch**: NO native SSE support - critical limitation
- **openapi-react-query**: Inherits openapi-fetch limitations
- **@hey-api/openapi-ts**: Plugin approach, but no SSE support yet

#### Industry Status
- OpenAPI specification lacks first-class SSE support
- Community discussions ongoing for 5+ years
- Fetch API preferred over axios for SSE in browser environments
- EventSource polyfills required for advanced SSE features

## Three Migration Path Analysis

### Option 1: TanStack Query v5 Only

**Approach:** Upgrade to v5 while maintaining existing axios/fetch architecture

**Pros:**
- ✅ Major type safety improvements through global error/meta types
- ✅ Better TypeScript inference with unified API
- ✅ Suspense-specific hooks with guaranteed non-undefined data
- ✅ queryOptions helper for type-safe query definitions
- ✅ 20% smaller bundle size than v4
- ✅ Works perfectly with existing axios client
- ✅ **Preserves SSE functionality completely**

**Cons:**
- ❌ Still using hardcoded endpoints
- ❌ No compile-time path validation
- ❌ Manual URL construction remains error-prone

**Migration Effort:** Medium (2-3 days)
**Bundle Impact:** -20% (smaller than current)
**Breaking Changes:** Moderate (codemod available)

### Option 2: TanStack Query v5 + openapi-fetch + openapi-react-query

**Approach:** Full type-safe integration using openapi-typescript ecosystem

**Pros:**
- ✅ Complete type safety from OpenAPI to React
- ✅ Compile-time path validation
- ✅ Auto-completion for endpoints
- ✅ Tiny bundle addition (6kb + 1kb)
- ✅ Maintained by openapi-typescript team

**Cons:**
- ❌ **NO SSE SUPPORT** - Critical limitation for chat interface
- ❌ Requires maintaining two HTTP clients (fetch for regular, native for SSE)
- ❌ Loss of axios interceptor features (auth, logging)
- ❌ More complex testing setup
- ❌ MSW integration requires different patterns

**Migration Effort:** High (5-7 days)
**Bundle Impact:** +7kb
**Risk Level:** High (breaks core chat functionality)

### Option 3: TanStack Query v5 + @hey-api/openapi-ts Plugin

**Approach:** Code generation approach with TanStack Query hooks

**Pros:**
- ✅ Generates TanStack Query hooks directly
- ✅ Supports axios client (@hey-api/client-axios)
- ✅ Type-safe query keys and options
- ✅ Actively maintained (latest update 8 hours ago as of research date)
- ✅ Single source of truth

**Cons:**
- ❌ Still no native SSE support
- ❌ Requires regenerating client code
- ❌ Less mature than other options
- ❌ Additional build step complexity

**Migration Effort:** High (5-7 days)
**Bundle Impact:** Variable (depends on generated code)
**Risk Level:** Medium-High

## Migration Strategy Recommendations

### Recommended: Hybrid Progressive Migration

**Rationale:** Maximizes type safety benefits while preserving critical SSE functionality and existing architecture investments.

#### Phase 1: Upgrade to TanStack Query v5 (Week 1)
**Goal:** Immediate type safety benefits without architectural changes

**Tasks:**
1. Install TanStack Query v5: `npm uninstall react-query && npm install @tanstack/react-query@^5`
2. Apply breaking changes using codemod:
   ```bash
   npx jscodeshift@latest ./src/hooks/ \
     --extensions=ts,tsx \
     --parser=tsx \
     --transform=./node_modules/@tanstack/react-query/build/codemods/src/v5/remove-overloads/remove-overloads.cjs
   ```
3. Manual fixes:
   - Rename `isLoading` → `isPending`
   - Convert to single object API
   - Remove onSuccess/onError callbacks → useEffect
   - Use `throwOnError` instead of `useErrorBoundary`
4. Add global type configuration
5. **Keep existing axios + SSE setup intact**

**Success Criteria:**
- All tests pass
- SSE chat functionality works
- TypeScript compilation successful
- Bundle size reduced

#### Phase 2: Type-Safe Path Construction (Week 2)
**Goal:** Eliminate hardcoded strings without changing HTTP client

**Approach:**
```typescript
// lib/typedPaths.ts
import type { paths } from '@bodhiapp/ts-client';

type PathHelper = {
  [K in keyof paths]: (params?: any) => string
};

export const apiPaths: PathHelper = {
  '/bodhi/v1/api-models/{id}': ({ id }) => `/bodhi/v1/api-models/${id}`,
  '/bodhi/v1/api-models': () => '/bodhi/v1/api-models',
  // Generate for all paths
};

// Usage
const endpoint = apiPaths['/bodhi/v1/api-models/{id}']({ id });
```

**Benefits:**
- Compile-time path validation
- Auto-completion
- Refactoring safety
- No runtime overhead

#### Phase 3: Enhanced Type Safety with queryOptions (Week 3)
**Goal:** Leverage v5's queryOptions for better type inference

```typescript
export const apiModelQueries = {
  detail: (id: string) =>
    queryOptions({
      queryKey: ['api-models', id],
      queryFn: async () => {
        const { data } = await apiClient.get<ApiModelResponse>(
          apiPaths['/bodhi/v1/api-models/{id}']({ id })
        );
        return data;
      },
    }),
};

// Usage
const { data } = useQuery(apiModelQueries.detail(id));
```

#### Phase 4: Optional Future Enhancement (Month 2+)
**Trigger:** Only if SSE requirements change

- Consider openapi-fetch if SSE is moved to WebSockets
- Evaluate @hey-api plugin if it adds SSE support
- Monitor ecosystem for better solutions

## Technical Implementation Details

### Breaking Changes from React Query v3 → TanStack Query v5

#### Package & Requirements
- Package name: `react-query` → `@tanstack/react-query`
- React requirement: 18.0+ (uses `useSyncExternalStore`)

#### API Changes
1. **Status Renaming**
   ```typescript
   // Before
   const { isLoading } = useQuery(...)

   // After
   const { isPending } = useQuery(...)
   ```

2. **Single Object Signature**
   ```typescript
   // Before
   useQuery(['key'], queryFn, { options })

   // After
   useQuery({ queryKey: ['key'], queryFn, ...options })
   ```

3. **Removed Callbacks**
   ```typescript
   // Before
   useQuery(['key'], queryFn, {
     onSuccess: (data) => { /* side effect */ },
     onError: (error) => { /* side effect */ }
   })

   // After
   const { data, error } = useQuery({ queryKey: ['key'], queryFn })
   useEffect(() => {
     if (data) { /* side effect */ }
   }, [data])
   ```

4. **Error Handling**
   ```typescript
   // Before
   { useErrorBoundary: true }

   // After
   { throwOnError: true }
   ```

### SSE Implementation Strategy

**Critical Decision: Keep existing fetch-based SSE implementation**

```typescript
// Continue using this pattern for SSE endpoints
export function useChatCompletion() {
  const appendMutation = useMutation({
    mutationFn: async ({ request, headers, onDelta, onFinish, onError }) => {
      // Native fetch for SSE - DO NOT CHANGE
      const response = await fetch(`${baseUrl}${ENDPOINT_OAI_CHAT_COMPLETIONS}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers },
        body: JSON.stringify(request),
      });

      // Existing SSE stream processing logic
      if (contentType.includes('text/event-stream')) {
        // Keep this implementation
      }
    }
  });
}
```

### Axios Configuration Preservation

```typescript
// Maintain axios with interceptors for non-SSE endpoints
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});

// Keep existing interceptors for:
// - Authentication token injection
// - Request/response logging
// - Error transformation
apiClient.interceptors.request.use(/* existing logic */);
apiClient.interceptors.response.use(/* existing logic */);
```

### Testing Compatibility

**MSW v2 Integration Remains Unchanged**
```typescript
// Existing MSW setup continues to work with TanStack Query v5
import { typedHttp, http, HttpResponse } from '../test-utils/msw-v2/setup';

// OpenAPI-typed handlers continue to work
export const apiModelHandlers = [
  typedHttp.get('/bodhi/v1/api-models/{id}', ({ response, params }) => {
    return response(200).json(mockApiModel);
  })
];
```

## Risk Assessment & Mitigation

### High-Risk Areas
1. **SSE Chat Functionality**
   - **Risk:** Breaking chat completions during migration
   - **Mitigation:** Keep existing fetch implementation unchanged
   - **Testing:** Comprehensive SSE testing in each phase

2. **Authentication Flow**
   - **Risk:** Breaking axios interceptors affects auth
   - **Mitigation:** Preserve axios for non-SSE endpoints
   - **Testing:** Auth integration tests

3. **Type Safety Regression**
   - **Risk:** Losing existing type safety during migration
   - **Mitigation:** Phase-by-phase migration with TypeScript strict mode
   - **Testing:** Intentional compilation errors to verify type safety

### Medium-Risk Areas
1. **Bundle Size Impact**
   - **Risk:** Unexpected bundle size increases
   - **Mitigation:** Bundle analysis in each phase
   - **Monitoring:** Webpack bundle analyzer

2. **Performance Regression**
   - **Risk:** Slower query performance
   - **Mitigation:** Performance benchmarking
   - **Testing:** Load testing with realistic data

### Low-Risk Areas
1. **Developer Experience**
   - **Risk:** Team confusion with new patterns
   - **Mitigation:** Documentation and training
   - **Support:** Pair programming sessions

## Success Metrics

### Phase 1 Success Criteria
- [ ] All existing tests pass
- [ ] SSE chat functionality works identically
- [ ] Bundle size reduced by ~20%
- [ ] TypeScript compilation successful
- [ ] No runtime errors in production

### Phase 2 Success Criteria
- [ ] Compile-time path validation working
- [ ] Intentional typos in paths cause build failures
- [ ] All hardcoded paths eliminated from target hooks
- [ ] IDE auto-completion for API paths
- [ ] No performance regression

### Phase 3 Success Criteria
- [ ] queryOptions pattern adopted for target hooks
- [ ] Type inference improvements visible in IDE
- [ ] Cache invalidation patterns working correctly
- [ ] MSW tests continue to pass
- [ ] Documentation updated for new patterns

## Cost-Benefit Analysis

### Investment
- **Time:** 3-4 weeks of development effort
- **Resources:** 1 senior developer, code review overhead
- **Risk:** Low-medium (phased approach mitigates risk)

### Returns
- **Type Safety:** Compile-time path validation prevents runtime errors
- **Developer Experience:** Better IDE support and error messages
- **Performance:** 20% bundle size reduction
- **Future-Proofing:** Modern query library with active development
- **Maintenance:** Reduced manual endpoint management

### Quantifiable Benefits
- **Bundle Size:** -20% reduction (estimated ~50kb savings)
- **Development Velocity:** Fewer API-related bugs
- **Refactoring Safety:** Compile-time detection of API changes
- **Team Onboarding:** Better typed patterns for new developers

## Alternative Approaches Considered

### Approach: Full openapi-fetch Migration
**Rejected Reason:** No SSE support breaks critical chat functionality
**Future Consideration:** If SSE moved to WebSockets

### Approach: Custom Type-Safe Wrapper
**Rejected Reason:** Significant maintenance overhead vs ecosystem solutions
**Consideration:** Only if ecosystem approaches fail

### Approach: Stay on React Query v3
**Rejected Reason:** Missing type safety improvements and security updates
**Risk:** Technical debt accumulation

## Ecosystem Monitoring

### Libraries to Watch
1. **openapi-fetch**: Monitor for SSE support additions
2. **@hey-api/openapi-ts**: Track TanStack Query plugin maturity
3. **TanStack Query**: Future SSE integration possibilities

### Industry Trends
- OpenAPI specification discussions about SSE support
- React ecosystem moving to React 18+ Suspense patterns
- Increased focus on compile-time type safety

## Conclusion

The recommended hybrid approach balances immediate benefits from TanStack Query v5 with preservation of critical SSE functionality. This strategy provides:

1. **Immediate Value:** Type safety improvements in Week 1
2. **Progressive Enhancement:** Path validation in Week 2
3. **Future Flexibility:** queryOptions patterns in Week 3
4. **Risk Mitigation:** No disruption to core chat functionality

The approach avoids the "perfect solution fallacy" while delivering substantial improvements to developer experience and type safety. The SSE requirement is a hard constraint that eliminates full openapi-fetch adoption, making the hybrid approach the optimal balance of benefits and risks.

## Next Steps

1. **Get stakeholder approval** for the phased migration approach
2. **Schedule Phase 1** implementation (TanStack Query v5 upgrade)
3. **Prepare development environment** with updated dependencies
4. **Create backup branch** before starting migration
5. **Begin with comprehensive test coverage** of existing functionality

---

*Research conducted: September 29, 2025*
*Document version: 1.0*
*Status: Ready for Implementation Planning*