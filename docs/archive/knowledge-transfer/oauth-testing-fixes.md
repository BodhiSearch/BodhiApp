# OAuth Testing Fixes: Comprehensive Analysis and Solutions

## Overview

This document provides a comprehensive analysis of the OAuth testing issues discovered and resolved during frontend testing improvements. The fixes involved critical changes to apiClient configuration, hook consistency patterns, and MSW setup that are essential for reliable frontend testing.

## Problem Discovery

### Initial Issue
OAuth-related tests were failing with "Invalid base URL" errors when using Mock Service Worker (MSW) for API mocking. The issue was specifically affecting pages that use the `AppInitializer` component, which calls `useAppInfo()` immediately upon rendering.

### Root Cause Analysis

#### 1. Empty BaseURL Problem
```typescript
// crates/bodhi/src/lib/apiClient.ts (before fix)
const apiClient = axios.create({
  baseURL: '', // Empty string caused the issue
  maxRedirects: 0,
});
```

**The Problem**: When `baseURL` is empty, axios cannot construct valid URLs from relative endpoint paths like `/bodhi/v1/info`. This happens **before** MSW can intercept the request.

#### 2. Hook Inconsistency
The `useOAuthInitiate` hook was using `useMutation` directly instead of the `useMutationQuery` helper used by other hooks, causing inconsistent behavior and test failures.

#### 3. MSW Interception Failure
MSW requires valid URLs to intercept requests using wildcard patterns. With empty baseURL, the constructed URLs were invalid, preventing MSW from working.

## Solutions Implemented

### 1. ApiClient Test Environment Configuration

**Solution**: Configure baseURL for test environments while preserving production behavior.

```typescript
// crates/bodhi/src/lib/apiClient.ts (after fix)
const isTest = typeof process !== 'undefined' && process.env.NODE_ENV === 'test';
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});
```

**Benefits**:
- ✅ Enables MSW to intercept requests with valid URLs
- ✅ Preserves production behavior (empty baseURL)
- ✅ Fixes AppInitializer component testing
- ✅ No impact on existing functionality

### 2. Hook Consistency Standardization

**Problem**: Inconsistent hook patterns across the codebase.

**Solution**: Standardized all mutation hooks to use `useMutationQuery` helper.

```typescript
// crates/bodhi/src/hooks/useOAuth.ts (after fix)
export function useOAuthInitiate(options?: {
  onSuccess?: (response: AxiosResponse<AuthInitiateResponse>) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AuthInitiateResponse>, AxiosError<ErrorResponse>, void> {
  return useMutationQuery<AuthInitiateResponse, void>(
    ENDPOINT_AUTH_INITIATE,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response),
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth flow';
        options?.onError?.(message);
      },
    },
    {
      validateStatus: (status) => status >= 200 && status < 500, // Accept 401 responses
    }
  );
}
```

### 3. Enhanced useMutationQuery Helper

**Added**: Support for custom axios configuration to handle OAuth-specific requirements.

```typescript
// crates/bodhi/src/hooks/useQuery.ts (enhanced)
export function useMutationQuery<T, V>(
  endpoint: string | ((variables: V) => string),
  method: 'post' | 'put' | 'delete' = 'post',
  options?: UseMutationOptions<AxiosResponse<T>, AxiosError<ErrorResponse>, V>,
  axiosConfig?: {
    validateStatus?: (status: number) => boolean;
    headers?: Record<string, string>;
  }
): UseMutationResult<AxiosResponse<T>, AxiosError<ErrorResponse>, V> {
  // Implementation with custom axios config support
}
```

## Testing Pattern Improvements

### 1. MSW Configuration Standards

**Wildcard Pattern Usage**:
```typescript
// ✅ Correct pattern
server.use(
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
    return res(ctx.json({ status: 'ready' }));
  })
);

// ❌ Incorrect pattern
server.use(
  rest.get('/bodhi/v1/info', (_, res, ctx) => {
    return res(ctx.json({ status: 'ready' }));
  })
);
```

### 2. AppInitializer Testing Requirements

**Critical Pattern**: Pages using `AppInitializer` must mock `ENDPOINT_APP_INFO`.

```typescript
describe('ResourceAdminPage', () => {
  beforeEach(() => {
    // Always required for AppInitializer pages
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );
  });
});
```

### 3. Test Case Design Principles

**Focus on Content Components**:
- ✅ Test `LoginContent`, `ResourceAdminContent`
- ❌ Avoid testing `LoginPage`, `ResourceAdminPage` wrappers

**Separate Scenarios**:
- ✅ Separate success and error test cases
- ❌ Don't merge unrelated scenarios with `unmount()`

## Impact and Results

### Before Fixes
- ❌ OAuth tests failing with "Invalid base URL" errors
- ❌ Inconsistent hook patterns across codebase
- ❌ MSW not intercepting requests properly
- ❌ AppInitializer pages untestable

### After Fixes
- ✅ All OAuth tests passing (13/13 login tests, 7/7 resource-admin tests)
- ✅ Consistent hook patterns using useMutationQuery
- ✅ Reliable MSW interception with wildcard patterns
- ✅ AppInitializer pages fully testable
- ✅ 352/352 total tests passing

## Key Learnings

### 1. Test Environment Configuration is Critical
The most important lesson is that proper test environment configuration in core utilities (like apiClient) is essential for reliable testing. Small configuration issues can cascade into widespread test failures.

### 2. Hook Consistency Prevents Issues
Standardizing on helper functions like `useMutationQuery` prevents inconsistencies that can cause test failures and makes the codebase more maintainable.

### 3. MSW Requires Valid URLs
MSW can only intercept valid HTTP requests. Empty baseURL prevents axios from constructing valid URLs, breaking the entire mocking system.

### 4. Component Architecture Affects Testing
Understanding component architecture (AppInitializer vs non-AppInitializer pages) is crucial for writing effective tests.

## Implementation References

**Key Files Modified**:
- `crates/bodhi/src/lib/apiClient.ts:4-8` - Test environment baseURL configuration
- `crates/bodhi/src/hooks/useQuery.ts:77-113` - Enhanced useMutationQuery with axios config
- `crates/bodhi/src/hooks/useOAuth.ts:33-60` - Standardized OAuth hooks

**Test Files Demonstrating Patterns**:
- `crates/bodhi/src/app/ui/login/page.test.tsx` - OAuth flow testing
- `crates/bodhi/src/app/ui/setup/resource-admin/page.test.tsx` - AppInitializer testing
- `crates/bodhi/src/hooks/useOAuth.test.ts` - Hook testing with MSW

## Future Considerations

### 1. Monitoring Hook Consistency
Ensure all new mutation hooks use the `useMutationQuery` pattern to maintain consistency.

### 2. Test Environment Documentation
Document test environment requirements clearly to prevent similar issues in the future.

### 3. MSW Pattern Enforcement
Consider linting rules or documentation to enforce wildcard pattern usage in MSW handlers.

---

*This analysis documents the comprehensive OAuth testing fixes that resolved critical frontend testing issues and established reliable patterns for future development.*
